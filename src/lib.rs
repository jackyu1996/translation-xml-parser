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
    fn parse_segment(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Vec<Box<SegNode>> {
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
                                content: SegNode::parse_segment(&mut recurse_reader, buf),
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
