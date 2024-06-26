pub mod tbx;
pub mod tmx;
pub mod xliff;
pub mod xlsx;

use fancy_regex::Regex;
use quick_xml::escape::unescape;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

pub struct MetaInfo<'a> {
    languages: HashMap<&'a str, usize>,
}

impl<'a> fmt::Display for MetaInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for k in self.languages.keys() {
            write!(
                f,
                "{} with {} entries\n",
                k,
                self.languages.get(k).unwrap_or(&0)
            )
            .unwrap_or_default()
        }
        Ok(())
    }
}

pub struct MatchResult {
    pub text: String,
    pub matched: String,
    pub extra: Vec<String>,
}

impl fmt::Display for MatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: \"{}\"\n", self.matched, self.text).unwrap_or_default();
        write!(f, "Extra info:\n").unwrap_or_default();
        for e in &self.extra {
            write!(f, "\t\"{}\"\n", e).unwrap_or_default();
        }
        Ok(())
    }
}

pub trait GetMeta {
    fn get_filename(&self) -> String;
    fn get_meta(&self) -> MetaInfo;
}

pub trait SearchString {
    fn match_string<'a>(&'a self, text: &'a str) -> Option<String>;
    fn count_matches<'a>(&'a self, text: &'a str) -> usize;
    fn as_any(&self) -> &dyn Any;
}

impl SearchString for String {
    fn match_string<'a>(&'a self, text: &'a str) -> Option<String> {
        if let Some(_) = text.find(self) {
            return Some(self.to_string());
        } else {
            return None;
        }
    }

    fn count_matches<'a>(&'a self, text: &'a str) -> usize {
        return text.matches(self).count();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SearchString for Regex {
    fn match_string<'a>(&'a self, text: &'a str) -> Option<String> {
        if let Some(first_match) = self.find(text).unwrap_or_default() {
            return Some(first_match.as_str().to_string());
        } else {
            return None;
        }
    }

    fn count_matches<'a>(&'a self, text: &'a str) -> usize {
        return self.find_iter(text).count();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn search_in_transunits(
    trans_units: &Vec<xliff::TransUnit>,
    include_tags: bool,
    matcher: &Box<dyn SearchString>,
    match_results: &mut Vec<MatchResult>,
) {
    for tu in trans_units {
        let source = extract_text(&tu.source, include_tags);
        let target = extract_text(&tu.target, include_tags);
        if let Some(match_result) = matcher.match_string(&source) {
            match_results.push(MatchResult {
                text: source,
                matched: match_result,
                extra: vec![tu.id.clone(), target],
            });
        } else if let Some(match_result) = matcher.match_string(&target) {
            match_results.push(MatchResult {
                text: target,
                matched: match_result,
                extra: vec![tu.id.clone(), source],
            })
        }
    }
}

pub trait SearchInFile {
    fn search_in_file(
        &self,
        include_tags: bool,
        matcher: &Box<dyn SearchString>,
    ) -> Vec<MatchResult>;
}

pub trait IsTranslationXML: GetMeta + SearchInFile {
    fn as_any(&self) -> &dyn Any;
}

impl IsTranslationXML for xliff::XliffFile {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IsTranslationXML for tmx::TmxFile {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IsTranslationXML for tbx::TbxFile {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IsTranslationXML for xlsx::TranslationXlsx {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn read_file_with_parser(path: &PathBuf) -> Box<dyn IsTranslationXML> {
    match path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        "xliff" | "xlf" | "txlf" | "sdlxliff" | "mxliff" | "mqxliff" => {
            Box::new(xliff::XliffFile::new(path.to_str().unwrap_or_default()))
        }
        "xlz" => Box::new(xliff::XliffFile::new_zipped(
            path.to_str().unwrap_or_default(),
            "content.xlf",
        )),
        "mqxlz" => Box::new(xliff::XliffFile::new_zipped(
            path.to_str().unwrap_or_default(),
            "document.mqxliff",
        )),
        "tmx" => Box::new(tmx::TmxFile::new(path.to_str().unwrap_or_default())),
        "tbx" => Box::new(tbx::TbxFile::new(path.to_str().unwrap_or_default())),
        "xlsx" => Box::new(xlsx::TranslationXlsx::new(
            path.to_str().unwrap_or_default(),
        )),
        _ => panic!("Unsupported file type"), // use a cli-compliant way to panic
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SegNode {
    Text(String),
    OpenOrCloseNode {
        node_type: String,
        attributes: HashMap<String, String>,
        content: Vec<Box<SegNode>>,
    },
    SelfClosingNode {
        node_type: String,
        attributes: HashMap<String, String>,
    },
}

pub fn is_text_node(node: &&Box<SegNode>) -> bool {
    let allowed_inline_text_tags = ["g", "mrk"];

    let allow_tag = match node.as_ref() {
        SegNode::Text(..) => true,
        SegNode::OpenOrCloseNode { node_type, .. } => {
            allowed_inline_text_tags.contains(&node_type.as_str()) && true
        }
        SegNode::SelfClosingNode { .. } => false,
    };

    return allow_tag;
}

impl<'a> FromIterator<&'a Box<SegNode>> for String {
    fn from_iter<I: IntoIterator<Item = &'a Box<SegNode>>>(iter: I) -> Self {
        let mut s = String::new();

        for n in iter {
            match n.as_ref() {
                SegNode::Text(content) => s.push_str(&Box::new(content)),
                SegNode::OpenOrCloseNode { content, .. } => {
                    let mut node_text = String::default();

                    let content = content.iter().collect::<String>();
                    match unescape(&content) {
                        Ok(text) => node_text = text.to_string(),
                        Err(e) => eprintln!("Error escaping text '{}': {:?}", &content, e),
                    }

                    s.push_str(&node_text)
                }
                SegNode::SelfClosingNode { .. } => s.push_str(""),
            }
        }

        return s;
    }
}

impl SegNode {
    fn parse_inline(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Vec<Box<SegNode>> {
        let mut nodes = Vec::new();

        loop {
            match reader.read_event_into(buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"bpt" | b"ept" | b"ph" | b"g" | b"mrk" => {
                        let node_type = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        let attributes = crate::get_attributes(&reader, &e);
                        let recurse_content =
                            reader.read_text(e.name()).unwrap_or_default().into_owned();
                        if recurse_content.contains(['<', '>']) {
                            let mut recurse_reader = Reader::from_str(&recurse_content);
                            nodes.push(Box::new(SegNode::OpenOrCloseNode {
                                node_type,
                                attributes,
                                content: SegNode::parse_inline(&mut recurse_reader, buf),
                            }))
                        } else {
                            nodes.push(Box::new(SegNode::OpenOrCloseNode {
                                node_type,
                                attributes,
                                content: vec![Box::new(SegNode::Text(recurse_content))],
                            }))
                        }
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) => match e.name().as_ref() {
                    b"bx" | b"ex" | b"x" => {
                        let node_type = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        let attributes = crate::get_attributes(&reader, &e);
                        nodes.push(Box::new(SegNode::SelfClosingNode {
                            node_type,
                            attributes,
                        }));
                    }
                    _ => (),
                },
                Ok(Event::Text(e)) => {
                    let mut node_text = String::default();

                    match e.unescape() {
                        Ok(text) => node_text = text.to_string(),
                        Err(e) => eprintln!("Error escaping text '{}': {:?}", &node_text, e),
                    }

                    nodes.push(Box::new(SegNode::Text(node_text)))
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    // NOTE: you have to add end tag here to break
                    b"source" | b"target" | b"seg" | b"term" => {
                        return nodes;
                    }
                    _ => (),
                },
                Ok(Event::Eof) => return nodes,
                _ => (),
            }
            buf.clear();
        }
    }
}

pub fn read_to_string(path: &str) -> String {
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open {}: {:?}", path, e);
            exit(1);
        }
    };

    let mut content = String::new();

    match file.read_to_string(&mut content) {
        Ok(..) => (),
        Err(..) => {
            eprintln!("Failed to read {} into a string", path);
            exit(4);
        }
    }

    return content;
}

pub fn extract_text(segs: &Vec<Box<SegNode>>, include_tags: bool) -> String {
    return segs
        .iter()
        .filter(|n| include_tags || is_text_node(n))
        .collect::<String>();
}

pub fn get_attributes(reader: &Reader<&[u8]>, start: &BytesStart) -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    for i in start.attributes() {
        let attribute = match i.as_ref() {
            Ok(attr) => attr,
            Err(e) => {
                eprintln!(
                    "Failed to reference attribute at {} {:?}",
                    reader.buffer_position(),
                    e
                );
                exit(2);
            }
        };
        attributes.insert(
            String::from_utf8_lossy(attribute.key.into_inner()).into_owned(),
            attribute
                .decode_and_unescape_value(reader)
                .unwrap_or_default()
                .into_owned(),
        );
    }
    return attributes;
}
