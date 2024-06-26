use crate::{
    search_in_transunits, GetMeta, MatchResult, MetaInfo, SearchInFile, SearchString, SegNode,
};
use std::{collections::HashMap, fs::File, io::BufReader, process::exit};

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
        let workbook: Xlsx<_> = match open_workbook(path) {
            Ok(workbook) => workbook,
            Err(e) => {
                eprintln!("Failed to open xlsx file {} {:?}", path, e);
                exit(1);
            }
        };

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

        if let Some(first_sheet) = all_worksheets.get(0) {
            let mut sn = 0;

            let mut cur_trans_unit;

            println!("Checking {} sheet: {}", self.path, &first_sheet.0);

            let mut trans_unit_rows = first_sheet.1.rows();

            let header = trans_unit_rows.next().unwrap_or_default();
            self.src_language = header[1]
                .get_string()
                .unwrap_or_default()
                .to_string()
                .to_lowercase();
            self.tgt_language = header[2]
                .get_string()
                .unwrap_or_default()
                .to_string()
                .to_lowercase();

            let mut buffer = Vec::new();

            for r in trans_unit_rows {
                let id = r
                    .get(0)
                    .unwrap_or(&calamine::DataType::Empty {})
                    .to_string();
                let source_value = r
                    .get(1)
                    .unwrap_or(&calamine::DataType::Empty {})
                    .to_string();
                let target_value = r
                    .get(2)
                    .unwrap_or(&calamine::DataType::Empty {})
                    .to_string();

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
        } else {
            eprintln!("First worksheet not found!");
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
