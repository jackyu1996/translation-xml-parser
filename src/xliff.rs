use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;

use crate::get_attribute;

pub struct XliffFile {
    pub path: String,
    pub src_language: String,
    pub tgt_language: String,
    pub xfiles: Vec<XFile>,
    reader: Reader<BufReader<File>>,
}

#[derive(Debug, Default)]
pub struct XFile {
    pub trans_units: Vec<TransUnit>,
}

#[derive(Debug, Default)]
pub struct TransUnit {
    pub id: String,
    pub source: String,
    pub target: String,
    pub notes: Vec<String>,
}

impl XliffFile {
    pub fn new(path: String) -> XliffFile {
        let read_result = Reader::from_file(&path);

        let file_reader = match read_result {
            Ok(file) => file,
            Err(e) => panic!("Failed to open file: {:?}", e),
        };

        return XliffFile {
            path: path,
            src_language: String::new(),
            tgt_language: String::new(),
            xfiles: Vec::new(),
            reader: file_reader,
        };
    }

    pub fn parse(&mut self) {
        let mut cur_xfile = XFile::default();
        let mut cur_trans_unit = TransUnit::default();
        let mut cur_src_or_tgt = Vec::new();
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"file" => {
                        self.src_language = get_attribute(e, "source-language");
                        self.tgt_language = get_attribute(e, "target-language");

                        if cur_xfile.trans_units.len() != 0 {
                            self.xfiles.push(cur_xfile);
                        }

                        cur_xfile = XFile::default();
                    }
                    b"trans-unit" => {
                        if !cur_trans_unit.id.is_empty()
                            && !cur_trans_unit.source.is_empty()
                            && !cur_trans_unit.target.is_empty()
                        {
                            cur_xfile.trans_units.push(cur_trans_unit);
                        }

                        cur_trans_unit = TransUnit {
                            id: get_attribute(e, "id"),
                            source: String::new(),
                            target: String::new(),
                            notes: Vec::new(),
                        };
                    }
                    b"source" => {
                        if !cur_trans_unit.source.is_empty() {
                            cur_trans_unit.source = cur_src_or_tgt.to_owned().into_iter().collect();
                        };
                        cur_src_or_tgt.clear();
                        cur_src_or_tgt
                            .push(self.reader.read_text(e.name(), &mut Vec::new()).unwrap());
                    }
                    b"bx" | b"ex" | b"bpt" | b"ept" | b"mrk" => {
                        cur_src_or_tgt
                            .push(self.reader.read_text(e.name(), &mut Vec::new()).unwrap());
                    }
                    b"target" => {
                        if !cur_trans_unit.target.is_empty() {
                            cur_trans_unit.target = cur_src_or_tgt.to_owned().into_iter().collect();
                        };
                        cur_src_or_tgt.clear();
                        cur_src_or_tgt
                            .push(self.reader.read_text(e.name(), &mut Vec::new()).unwrap());
                    }
                    b"note" => {
                        let note = self.reader.read_text(e.name(), &mut Vec::new()).unwrap();
                        cur_trans_unit.notes.push(note);
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                Err(e) => panic!(
                    "Error at position {}: {:?}",
                    self.reader.buffer_position(),
                    e
                ),
                _ => (),
            }
            buf.clear();
        }
    }
}
