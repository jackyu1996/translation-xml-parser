use crate::{
    search_in_transunits, GetMeta, MatchResult, MetaInfo, SearchInFile, SearchString, SegNode,
};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::io::prelude::*;
use zip;

#[derive(Debug, Clone)]
pub struct XliffFile {
    pub path: String,
    pub xfiles: Vec<XFile>,
    raw_content: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct XFile {
    pub src_language: String,
    pub tgt_language: String,
    pub trans_units: Vec<TransUnit>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TransUnit {
    pub id: String,
    pub sn: u16,
    pub translate: String,
    pub source: Vec<Box<SegNode>>,
    pub target: Vec<Box<SegNode>>,
}

impl XliffFile {
    pub fn new(path: &str) -> XliffFile {
        let content = crate::read_to_string(path);

        let mut xliff_file = XliffFile {
            path: path.to_owned(),
            xfiles: Vec::new(),
            raw_content: content,
        };
        xliff_file.parse();
        return xliff_file;
    }

    pub fn new_zipped(path: &str, inner_xliff_name: &str) -> XliffFile {
        let zipped_file = std::fs::File::open(&path).expect("Cannot open zipped file!");

        let mut archive = zip::ZipArchive::new(zipped_file).expect("Invalid zip file!");

        let mut file = match archive.by_name(inner_xliff_name) {
            Ok(file) => file,
            Err(_) => {
                panic!("{} not found in zipped file", inner_xliff_name)
            }
        };

        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Failed to read into a string");

        let mut xliff_file = XliffFile {
            path: path.to_owned(),
            xfiles: Vec::new(),
            raw_content: contents,
        };
        xliff_file.parse();
        return xliff_file;
    }

    fn parse(&mut self) {
        let mut buf = Vec::new();
        let mut sn = 0;

        let mut cur_xfile = XFile::default();
        let mut cur_trans_unit = TransUnit::default();
        let mut cur_source: Vec<Box<SegNode>>;
        let mut cur_target: Vec<Box<SegNode>>;

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"file" => {
                        cur_xfile.src_language = crate::get_attributes(&reader, &e)
                            .get("source-language")
                            .expect("source-language attribute not found")
                            .to_owned()
                            .to_lowercase();
                        cur_xfile.tgt_language = crate::get_attributes(&reader, &e)
                            .get("target-language")
                            .expect("target-language attribute not found")
                            .to_owned()
                            .to_lowercase();
                    }
                    b"trans-unit" => {
                        cur_trans_unit.id = crate::get_attributes(&reader, &e)
                            .get("id")
                            .expect("id attribute not found")
                            .to_owned();
                        sn += 1;
                        cur_trans_unit.sn = sn;
                        cur_trans_unit.translate = crate::get_attributes(&reader, &e)
                            .get("translate")
                            .unwrap_or(&"yes".to_string())
                            .to_owned()
                    }
                    b"source" => {
                        cur_source = SegNode::parse_inline(&mut reader, &mut buf);
                        if cur_source.len() != 0 {
                            cur_trans_unit.source = cur_source;
                        }
                    }
                    b"target" => {
                        cur_target = SegNode::parse_inline(&mut reader, &mut buf);
                        if cur_target.len() != 0 {
                            cur_trans_unit.target = cur_target;
                        }
                    }
                    b"alt-trans" => {
                        reader.read_to_end(e.name()).unwrap();
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
}

impl SearchInFile for XliffFile {
    fn search_in_file(
        &self,
        include_tags: bool,
        matcher: &Box<dyn SearchString>,
    ) -> Vec<MatchResult> {
        let mut match_results = Vec::new();

        for file in &self.xfiles {
            search_in_transunits(&file.trans_units, include_tags, matcher, &mut match_results)
        }

        return match_results;
    }
}

impl GetMeta for XliffFile {
    fn get_meta(&self) -> MetaInfo {
        let mut languages = HashMap::new();

        for f in &self.xfiles {
            let src_key = f.src_language.as_str();
            let tgt_key = f.tgt_language.as_str();
            let src_acc_len = languages.get(src_key).unwrap_or(&0).to_owned();
            let tgt_acc_len = languages.get(tgt_key).unwrap_or(&0).to_owned();

            let len = f.trans_units.len();
            languages.insert(src_key, len + src_acc_len);
            languages.insert(tgt_key, len + tgt_acc_len); // how about non-translated?
        }

        return MetaInfo { languages };
    }

    fn get_filename(&self) -> String {
        return self.path.to_owned();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_for_debug() {
        let t = crate::xliff::XliffFile::new(&"./tests/approval.sdlxliff");
        dbg!(t.xfiles);
        assert!(1 != 2);
    }
}
