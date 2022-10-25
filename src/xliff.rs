use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};

use std::io::prelude::*;
use zip;

#[derive(Debug)]
pub struct XliffFile {
    pub path: String,
    pub xfiles: Vec<XFile>,
    raw_content: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct XFile {
    pub src_language: String,
    pub tgt_language: String,
    pub trans_units: Vec<TransUnit>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransUnit {
    pub id: String,
    pub source: Vec<SegNode>,
    pub target: Vec<SegNode>,
}

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

impl FromIterator<SegNode> for String {
    fn from_iter<I: IntoIterator<Item = SegNode>>(iter: I) -> Self {
        let mut s = String::new();

        for n in iter {
            match n {
                SegNode::Text(content) => s.push_str(&content),
                SegNode::OpenOrClose(node) => s.push_str(&node.content),
                SegNode::SelfClosing(..) => s.push_str(""),
            }
        }

        return s;
    }
}

impl XliffFile {
    pub fn new(path: &str) -> XliffFile {
        let content = crate::read_xml(path);

        return XliffFile {
            path: path.to_owned(),
            xfiles: Vec::new(),
            raw_content: content,
        };
    }

    fn parse_segment(reader: &mut Reader<&[u8]>, buffer: &mut Vec<u8>) -> Vec<SegNode> {
        let mut nodes = Vec::new();

        let mut cur_selfclosing = SelfClosingNode::default();
        let mut cur_openorclose = OpenOrCloseNode::default();

        loop {
            buffer.clear();
            match reader.read_event_into(buffer) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"bpt" | b"ept" | b"ph" => {
                        cur_openorclose.node_type =
                            String::from_utf8_lossy(e.name().as_ref()).to_string();
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
                    b"source" | b"target" => {
                        return nodes;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_xfile = XFile::default();
        let mut cur_trans_unit = TransUnit::default();
        let mut cur_source: Vec<SegNode>;
        let mut cur_target: Vec<SegNode>;

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"file" => {
                        cur_xfile.src_language =
                            crate::get_attribute(&reader, &e, "source-language");
                        cur_xfile.tgt_language =
                            crate::get_attribute(&reader, &e, "target-language")
                    }
                    b"trans-unit" => cur_trans_unit.id = crate::get_attribute(&reader, &e, "id"),
                    b"source" => {
                        cur_source = XliffFile::parse_segment(&mut reader, &mut buf);
                        if cur_source.len() != 0 {
                            cur_trans_unit.source = cur_source;
                        }
                    }
                    b"target" => {
                        cur_target = XliffFile::parse_segment(&mut reader, &mut buf);
                        if cur_target.len() != 0 {
                            cur_trans_unit.target = cur_target;
                        }
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"file" => {
                        if cur_xfile.trans_units.len() != 0 {
                            self.xfiles.push(cur_xfile);
                        }
                        cur_xfile = XFile::default();
                    }
                    b"trans-unit" => {
                        if cur_trans_unit.source.len() != 0 && cur_trans_unit.target.len() != 0 {
                            cur_xfile.trans_units.push(cur_trans_unit)
                        }
                        cur_trans_unit = TransUnit::default();
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                _ => (),
            }
            buf.clear()
        }
    }

    pub fn read_from_xlz(path: &str) -> XliffFile {
        let xlzfile = std::fs::File::open(&path).expect("Cannot open xlz file!");

        let mut archive = zip::ZipArchive::new(xlzfile).expect("Invalid xlz file!");

        let mut file = match archive.by_name("content.xlf") {
            Ok(file) => file,
            Err(_) => {
                panic!("content.xlf not found in xlz file")
            }
        };

        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Failed to read into a string");

        return XliffFile {
            path: path.to_owned(),
            xfiles: Vec::new(),
            raw_content: contents,
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_for_debug() {
        let mut t = crate::xliff::XliffFile::new(&"./tests/sul.txlf");
        t.parse();
        assert!(1 != 2);
    }
}
