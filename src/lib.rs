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

pub struct MetaInfo<'a> {
    languages: HashMap<&'a str, usize>,
}

impl<'a> fmt::Display for MetaInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for k in self.languages.keys() {
            write!(f, "{} with {} entries\n", k, self.languages.get(k).unwrap()).unwrap()
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
        write!(f, "{}: \"{}\"\n", self.matched, self.text).unwrap();
        write!(f, "Extra info:\n").unwrap();
        for e in &self.extra {
            write!(f, "\t\"{}\"\n", e).unwrap();
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SearchString for Regex {
    fn match_string<'a>(&'a self, text: &'a str) -> Option<String> {
        if let Some(first_match) = self.find(text).unwrap() {
            return Some(first_match.as_str().to_string());
        } else {
            return None;
        }
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
                extra: vec![target],
            });
        } else if let Some(match_result) = matcher.match_string(&target) {
            match_results.push(MatchResult {
                text: target,
                matched: match_result,
                extra: vec![source],
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
    match path.extension().unwrap().to_str().unwrap() {
        "xliff" | "xlf" | "txlf" | "sdlxliff" => {
            Box::new(xliff::XliffFile::new(path.to_str().unwrap()))
        }
        "xlz" => Box::new(xliff::XliffFile::new_xlz(path.to_str().unwrap())),
        "tmx" => Box::new(tmx::TmxFile::new(path.to_str().unwrap())),
        "tbx" => Box::new(tbx::TbxFile::new(path.to_str().unwrap())),
        "xlsx" => Box::new(xlsx::TranslationXlsx::new(path.to_str().unwrap())),
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
                SegNode::OpenOrCloseNode { content, .. } => s.push_str(
                    &unescape(&content.iter().collect::<String>())
                        .unwrap()
                        .to_owned(),
                ),
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
                        let recurse_content = reader.read_text(e.name()).unwrap().into_owned();
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
                    nodes.push(Box::new(SegNode::Text(e.unescape().unwrap().into_owned())))
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    // note: you have to add end tag here to break
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
    let mut file = File::open(path).expect("Cannot read input file!");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Cannot read file to a string!");

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
        attributes.insert(
            String::from_utf8_lossy(i.as_ref().unwrap().key.into_inner()).into_owned(),
            i.as_ref()
                .unwrap()
                .decode_and_unescape_value(reader)
                .expect("Failed to decode attribute value")
                .into_owned(),
        );
    }
    return attributes;
}
