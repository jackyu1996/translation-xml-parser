use crate::{extract_text, GetMeta, MatchResult, MetaInfo, SearchInFile, SearchString, SegNode};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct TbxFile {
    pub path: String,
    pub term_entries: Vec<TermEntry>,
    raw_content: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TermEntry {
    #[serde(rename = "$value")]
    pub lang_sets: Vec<LangSet>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LangSet {
    #[serde(rename = "$primitive=xml:lang")]
    pub language: String,
    #[serde(rename = "$value")]
    pub tig: Tig,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Tig {
    #[serde(rename = "$value")]
    pub term: Vec<Box<SegNode>>,
}

impl TbxFile {
    pub fn new(path: &str) -> TbxFile {
        let raw_content = crate::read_to_string(path);

        let mut tbx_file = TbxFile {
            path: path.to_string(),
            term_entries: Vec::new(),
            raw_content,
        };
        tbx_file.parse();
        return tbx_file;
    }

    fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_term_entry = TermEntry::default();
        let mut cur_lang_set = LangSet::default();
        let mut cur_tig = Tig::default();

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"langSet" => {
                        cur_lang_set.language = crate::get_attributes(&reader, &e)
                            .get("xml:lang")
                            .unwrap_or(&"".to_string())
                            .to_owned()
                            .to_lowercase();
                    }
                    b"term" => {
                        cur_tig = Tig {
                            term: SegNode::parse_inline(&mut reader, &mut buf),
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
                    b"term" => {
                        if cur_tig.term.len() != 0 {
                            cur_lang_set.tig = cur_tig;
                        }
                        cur_tig = Tig::default();
                    }
                    b"langSet" => {
                        if cur_tig.term.len() != 0 {
                            cur_lang_set.tig = cur_tig;
                            cur_term_entry.lang_sets.push(cur_lang_set);
                        }
                        cur_tig = Tig::default();
                        cur_lang_set = LangSet::default();
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

impl GetMeta for TbxFile {
    fn get_meta(&self) -> MetaInfo {
        let mut languages = HashMap::new();

        for ls in self
            .term_entries
            .iter()
            .map(|te| &te.lang_sets)
            .flatten()
            .collect::<Vec<_>>()
        {
            let cur_lang = ls.language.as_str();
            let acc_len = languages.get(cur_lang).unwrap_or(&0).to_owned();
            languages.insert(cur_lang, acc_len + 1);
        }

        return MetaInfo { languages };
    }

    fn get_filename(&self) -> String {
        return self.path.to_owned();
    }
}

impl SearchInFile for TbxFile {
    fn search_in_file(
        &self,
        include_tags: bool,
        matcher: &Box<dyn SearchString>,
    ) -> Vec<MatchResult> {
        let mut match_results = Vec::new();

        for te in &self.term_entries {
            for ls in &te.lang_sets {
                let cur_term = extract_text(&ls.tig.term, include_tags);
                if let Some(match_result) = matcher.match_string(&cur_term) {
                    match_results.push(MatchResult {
                        text: cur_term,
                        matched: match_result,
                        extra: te
                            .lang_sets
                            .iter()
                            .filter(|l| l.language != ls.language)
                            .map(|u| u.tig.term.iter().collect::<String>())
                            .collect::<Vec<String>>(),
                    })
                }
            }
        }
        return match_results;
    }
}

mod tests {
    #[test]
    fn dummy_for_debug() {
        let t = crate::tbx::TbxFile::new(&"./tests/lancom.tbx");
        dbg!(&t.term_entries);
        assert!(1 != 2);
    }
}
