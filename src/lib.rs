pub mod tbx;
pub mod tmx;
pub mod xliff;
pub mod xlsx;

use quick_xml::escape::unescape;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
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
    pub id: String,
    pub content: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SelfClosingNode {
    pub node_type: String,
    pub id: String,
}

impl<'a> FromIterator<&'a SegNode> for String {
    fn from_iter<I: IntoIterator<Item = &'a SegNode>>(iter: I) -> Self {
        let mut s = String::new();

        for n in iter {
            match n {
                SegNode::Text(content) => s.push_str(&content),
                SegNode::OpenOrClose(node) => {
                    s.push_str(&unescape(&node.content).unwrap().to_owned())
                }
                SegNode::SelfClosing(..) => s.push_str(""),
            }
        }

        return s;
    }
}

impl SegNode {
    fn parse_segment(reader: &mut Reader<&[u8]>, buffer: &mut Vec<u8>) -> Vec<SegNode> {
        let mut nodes = Vec::new();

        let mut cur_selfclosing = SelfClosingNode::default();
        let mut cur_openorclose = OpenOrCloseNode::default();

        loop {
            buffer.clear();
            match reader.read_event_into(buffer) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    // is there a way to determine closeness of tag by quick_xml
                    b"bpt" | b"ept" | b"ph" => {
                        cur_openorclose.node_type =
                            String::from_utf8_lossy(e.name().as_ref()).to_string();
                        // consider getting all attributes
                        cur_openorclose.id = crate::get_attribute(&reader, &e, "id");
                        cur_openorclose.content = reader.read_text(e.name()).unwrap().into_owned();
                        nodes.push(SegNode::OpenOrClose(cur_openorclose));
                        cur_openorclose = OpenOrCloseNode::default();
                    }
                    b"bx" | b"ex" | b"g" | b"x" => {
                        cur_selfclosing.node_type =
                            String::from_utf8_lossy(e.name().as_ref()).to_string();
                        cur_selfclosing.id = crate::get_attribute(&reader, &e, "id");
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
        }
    }
}

pub fn get_attribute(reader: &Reader<&[u8]>, start: &BytesStart, attribute_name: &str) -> String {
    return start
        .try_get_attribute(attribute_name)
        .unwrap()
        .unwrap()
        .decode_and_unescape_value(reader)
        .unwrap()
        .into_owned();
}

pub fn read_xml(path: &str) -> String {
    let mut file = File::open(path).expect("Cannot read input file!");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Cannot read file to a string!");

    return content;
}
