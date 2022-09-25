use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};

pub struct TbxFile {
    pub path: String,
    pub term_entries: Vec<TermEntry>,
    raw_content: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TermEntry {
    #[serde(rename = "$value")]
    pub lang_sets: Vec<LangSet>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LangSet {
    #[serde(rename = "$primitive=xml:lang")]
    pub language: String,
    #[serde(rename = "$value")]
    pub tig: Tig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tig {
    #[serde(rename = "$value")]
    pub term: Term,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Term {
    #[serde(rename = "$value")]
    pub term: String,
}

impl TbxFile {
    pub fn new(path: String) -> TbxFile {
        let raw_content = crate::read_xml(&path);

        return TbxFile {
            path,
            term_entries: Vec::new(),
            raw_content,
        };
    }

    pub fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_term_entry = TermEntry::default();
        let mut cur_lang_set = LangSet::default();
        let mut cur_tig = Tig::default();
        let mut cur_term = Term::default();

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"langSet" => {
                        cur_lang_set.language = crate::get_attribute("xml:lang", &e, &reader);
                    }
                    b"term" => {
                        cur_term = Term {
                            term: reader.read_text(e.name()).unwrap().into_owned(),
                        };
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"termEntry" => {
                        if cur_term_entry.lang_sets.len() != 0 {
                            self.term_entries.push(cur_term_entry);
                        }
                        cur_term_entry = TermEntry::default();
                    }
                    b"langSet" => {
                        if cur_lang_set.tig.term.term != "" {
                            cur_term_entry.lang_sets.push(cur_lang_set);
                        }
                        cur_lang_set = LangSet::default();
                    }
                    b"tig" => {
                        if cur_tig.term.term != "" {
                            cur_lang_set.tig = cur_tig;
                        }
                        cur_tig = Tig::default();
                    }
                    b"term" => {
                        if cur_term.term != "" {
                            cur_tig.term = cur_term;
                        }
                        cur_term = Term::default();
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

mod tests {
    #[test]
    fn dummy_for_debug() {
        let mut t = crate::tbx::TbxFile::new("./tests/lancom.tbx".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
