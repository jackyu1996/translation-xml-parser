use crate::{extract_text, GetMeta, MatchResult, MetaInfo, SearchInFile, SearchString, SegNode};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TmxFile {
    pub path: String,
    pub tus: Vec<TU>,
    raw_content: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TU {
    pub tuid: String,
    pub tuvs: Vec<TUV>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TUV {
    pub language: String,
    pub seg: Vec<Box<SegNode>>,
}

impl TmxFile {
    pub fn new(path: &str) -> TmxFile {
        let content = crate::read_to_string(path);

        let mut tmx_file = TmxFile {
            path: path.to_string(),
            tus: Vec::new(),
            raw_content: content,
        };
        tmx_file.parse();
        return tmx_file;
    }

    fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_tu = TU::default();
        let mut cur_tuv = TUV::default();
        let mut cur_seg: Vec<Box<SegNode>>;

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"tu" => {
                        cur_tu.tuid = crate::get_attributes(&reader, &e)
                            .get("tuid")
                            .unwrap_or(&"".to_string())
                            .to_owned()
                    }
                    b"tuv" => {
                        cur_tuv.language = crate::get_attributes(&reader, &e)
                            .get("xml:lang")
                            .unwrap_or(&"".to_string())
                            .to_owned()
                            .to_lowercase()
                    }

                    b"seg" => {
                        cur_seg = SegNode::parse_inline(&mut reader, &mut buf);
                        if cur_seg.len() != 0 {
                            cur_tuv.seg = cur_seg;
                        }
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"tuv" => {
                        if cur_tuv.seg.len() != 0 {
                            cur_tu.tuvs.push(cur_tuv)
                        }

                        cur_tuv = TUV::default();
                    }
                    b"tu" => {
                        if cur_tu.tuvs.len() != 0 {
                            self.tus.push(cur_tu);
                        }

                        cur_tu = TU::default();
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

impl GetMeta for TmxFile {
    fn get_meta(&self) -> MetaInfo {
        let mut languages = HashMap::new();

        for tuv in self
            .tus
            .iter()
            .map(|tu| &tu.tuvs)
            .flatten()
            .collect::<Vec<_>>()
        {
            let cur_lang = tuv.language.as_str();
            let acc_len = languages.get(cur_lang).unwrap_or(&0).to_owned();
            languages.insert(cur_lang, acc_len + 1);
        }

        return MetaInfo { languages };
    }

    fn get_filename(&self) -> String {
        return self.path.to_owned();
    }
}

impl SearchInFile for TmxFile {
    fn search_in_file(
        &self,
        include_tags: bool,
        matcher: &Box<dyn SearchString>,
    ) -> Vec<MatchResult> {
        let mut match_results = Vec::new();

        for tu in &self.tus {
            for tuv in &tu.tuvs {
                let cur_tuv = extract_text(&tuv.seg, include_tags);
                if let Some(match_result) = matcher.match_string(&cur_tuv) {
                    match_results.push(MatchResult {
                        text: cur_tuv,
                        matched: match_result,
                        extra: tu
                            .tuvs
                            .iter()
                            .filter(|v| v.language != tuv.language)
                            .map(|v| v.seg.iter().collect::<String>())
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
        let t = crate::tmx::TmxFile::new(&"./tests/CITIC.tmx");
        dbg!(t.tus);
        assert!(1 != 2);
    }
}
