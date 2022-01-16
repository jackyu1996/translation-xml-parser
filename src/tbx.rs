use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;

pub struct TbxFile {
    pub path: String,
    pub term_entries: Vec<TermEntry>,
    reader: Reader<BufReader<File>>,
}

#[derive(Debug)]
pub struct TermEntry {
    pub lang_sets: Vec<LangSet>,
}

#[derive(Debug)]
pub struct LangSet {
    pub language: String,
    pub tigs: Vec<Tig>,
}

#[derive(Debug)]
pub struct Tig {
    pub term: String,
}

impl TbxFile {
    pub fn new(path: String) -> TbxFile {
        let read_result = Reader::from_file(&path);

        let file_reader = match read_result {
            Ok(file) => file,
            Err(e) => panic!("Failed to open file: {:?}", e),
        };

        return TbxFile {
            path: path,
            term_entries: Vec::new(),
            reader: file_reader,
        };
    }

    pub fn parse(&mut self) {
        let mut term_entry = TermEntry {
            lang_sets: Vec::new(),
        };
        let mut lang_set = LangSet {
            language: String::new(),
            tigs: Vec::new(),
        };
        let mut tig = Tig {
            term: String::new(),
        };

        let mut buf = Vec::new();

        loop {
            match self.reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"termEntry" => {
                        if term_entry.lang_sets.len() != 0 {
                            self.term_entries.push(term_entry);
                        };

                        term_entry = TermEntry {
                            lang_sets: Vec::new(),
                        };
                    }
                    b"langSet" => {
                        if lang_set.tigs.len() != 0 {
                            term_entry.lang_sets.push(lang_set);
                        }

                        lang_set = LangSet {
                            language: String::new(),
                            tigs: Vec::new(),
                        }
                    }
                    b"tig" => {
                        if !tig.term.is_empty() {
                            lang_set.tigs.push(tig);
                        }

                        tig = Tig {
                            term: String::new(),
                        }
                    }
                    b"term" => {
                        let term = self.reader.read_text(e.name(), &mut Vec::new()).unwrap();
                        tig.term = term;
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
