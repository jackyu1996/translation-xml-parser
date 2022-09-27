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
    pub source: String,
    pub target: String,
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

    pub fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_xfile = XFile::default();
        let mut cur_trans_unit = TransUnit::default();
        let mut cur_source;
        let mut cur_target;

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"file" => {
                        cur_xfile.src_language =
                            crate::get_attribute("source-language", &e, &reader);
                        cur_xfile.tgt_language =
                            crate::get_attribute("target-language", &e, &reader)
                    }
                    b"trans-unit" => cur_trans_unit.id = crate::get_attribute("id", &e, &reader),
                    b"source" => {
                        cur_source = reader.read_text(e.name()).unwrap().into_owned();
                        if cur_source != "" {
                            cur_trans_unit.source = cur_source;
                        }
                    }
                    b"target" => {
                        cur_target = reader.read_text(e.name()).unwrap().into_owned();
                        if cur_target != "" {
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
                        if cur_trans_unit.source != "" && cur_trans_unit.target != "" {
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
