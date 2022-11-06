pub mod tbx;
pub mod tmx;
pub mod xliff;
pub mod xlsx;

use quick_xml::escape::unescape;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub enum SegNode {
    Text(String),
    OpenOrClose(OpenOrCloseNode),
    SelfClosing(SelfClosingNode),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OpenOrCloseNode {
    pub node_type: String,
    pub attributes: HashMap<String, String>,
    pub content: Vec<SegNode>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SelfClosingNode {
    pub node_type: String,
    pub attributes: HashMap<String, String>,
}

impl<'a> FromIterator<&'a SegNode> for String {
    fn from_iter<I: IntoIterator<Item = &'a SegNode>>(iter: I) -> Self {
        let mut s = String::new();

        for n in iter {
            match n {
                SegNode::Text(content) => s.push_str(&content),
                SegNode::OpenOrClose(node) => s.push_str(
                    &unescape(&node.content.iter().collect::<String>())
                        .unwrap()
                        .to_owned(),
                ),
                SegNode::SelfClosing(..) => s.push_str(""),
            }
        }

        return s;
    }
}

impl SegNode {
    fn parse_segment(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Vec<SegNode> {
        let mut nodes = Vec::new();

        let mut cur_selfclosing = SelfClosingNode::default();
        let mut cur_openorclose = OpenOrCloseNode::default();

        loop {
            match reader.read_event_into(buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"bpt" | b"ept" | b"ph" | b"g" | b"mrk" => {
                        cur_openorclose.node_type =
                            String::from_utf8_lossy(e.name().as_ref()).to_string();
                        cur_openorclose.attributes = crate::get_attributes(&reader, &e);
                        let recurse_content = reader.read_text(e.name()).unwrap().into_owned();
                        if recurse_content.contains(['<', '>']) {
                            let mut recurse_reader = Reader::from_str(&recurse_content);
                            cur_openorclose.content =
                                SegNode::parse_segment(&mut recurse_reader, buf);
                        } else {
                            cur_openorclose.content = vec![SegNode::Text(recurse_content)];
                        }
                        nodes.push(SegNode::OpenOrClose(cur_openorclose));
                        cur_openorclose = OpenOrCloseNode::default();
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) => match e.name().as_ref() {
                    b"bx" | b"ex" | b"x" => {
                        cur_selfclosing.node_type =
                            String::from_utf8_lossy(e.name().as_ref()).to_string();
                        cur_selfclosing.attributes = crate::get_attributes(&reader, &e);
                        nodes.push(SegNode::SelfClosing(cur_selfclosing));
                        cur_selfclosing = SelfClosingNode::default();
                    }
                    _ => (),
                },
                Ok(Event::Text(e)) => nodes.push(SegNode::Text(e.unescape().unwrap().into_owned())),
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

pub fn read_xml(path: &str) -> String {
    let mut file = File::open(path).expect("Cannot read input file!");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Cannot read file to a string!");

    return content;
}
