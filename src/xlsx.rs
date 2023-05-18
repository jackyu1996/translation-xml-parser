use crate::{
    search_in_transunits, GetMeta, MatchResult, MetaInfo, SearchInFile, SearchString, SegNode,
};
use std::{collections::HashMap, fs::File, io::BufReader};

use calamine::{open_workbook, Reader, Xlsx};
use quick_xml::Reader as XML_Reader;

use crate::xliff::TransUnit;

pub struct TranslationXlsx {
    pub path: String,
    pub trans_units: Vec<TransUnit>,
    pub src_language: String,
    pub tgt_language: String,
    xlsx: Xlsx<BufReader<File>>,
}

impl TranslationXlsx {
    pub fn new(path: &str) -> TranslationXlsx {
        let workbook: Xlsx<_> = open_workbook(path).expect("Cannot open xlsx file");

        let mut translation_xlsx = TranslationXlsx {
            path: path.to_owned(),
            trans_units: Vec::new(),
            xlsx: workbook,
            src_language: "".to_string(),
            tgt_language: "".to_string(),
        };
        translation_xlsx.parse();
        return translation_xlsx;
    }

    fn parse(&mut self) {
        let all_worksheets = self.xlsx.worksheets();
        let first_sheet = all_worksheets.get(0).unwrap();
        let mut sn = 0;

        let mut cur_trans_unit;

        println!("Checking {} sheet: {}", self.path, &first_sheet.0);

        let mut trans_unit_rows = first_sheet.1.rows();

        let header = trans_unit_rows.next().unwrap();
        self.src_language = header[1]
            .get_string()
            .unwrap()
            .to_string()
            .to_lowercase();
        self.tgt_language = header[2]
            .get_string()
            .unwrap()
            .to_string()
            .to_lowercase();

        let mut buffer = Vec::new();

        for r in trans_unit_rows {
            let id = r.get(0).unwrap().to_string();
            let source_value = r.get(1).unwrap().to_string();
            let target_value = r.get(2).unwrap().to_string();

            let mut source_reader = XML_Reader::from_str(&source_value);
            let mut target_reader = XML_Reader::from_str(&target_value);

            let source = SegNode::parse_inline(&mut source_reader, &mut buffer);
            let target = SegNode::parse_inline(&mut target_reader, &mut buffer);
            sn += 1;

            cur_trans_unit = TransUnit {
                id,
                sn,
                source,
                target,
                translate: "yes".to_string(),
            };
            if cur_trans_unit.id != "" {
                self.trans_units.push(cur_trans_unit)
            }
        }
    }
}

impl SearchInFile for TranslationXlsx {
    fn search_in_file(
        &self,
        include_tags: bool,
        matcher: &Box<dyn SearchString>,
    ) -> Vec<MatchResult> {
        let mut match_results = Vec::new();

        search_in_transunits(&self.trans_units, include_tags, matcher, &mut match_results);

        return match_results;
    }
}

impl GetMeta for TranslationXlsx {
    fn get_meta(&self) -> MetaInfo {
        return MetaInfo {
            languages: HashMap::from([
                (self.src_language.as_str(), self.trans_units.len()),
                (self.tgt_language.as_str(), self.trans_units.len()),
            ]),
        };
    }

    fn get_filename(&self) -> String {
        return self.path.to_owned();
    }
}
