use std::{fs::File, io::BufReader};

use calamine::{open_workbook, Reader, Xlsx};
use quick_xml::Reader as XML_Reader;

use crate::xliff::TransUnit;
use crate::SegNode;

pub struct TranslationXlsx {
    pub path: String,
    pub trans_units: Vec<TransUnit>,
    xlsx: Xlsx<BufReader<File>>,
}

impl TranslationXlsx {
    pub fn new(path: &str) -> TranslationXlsx {
        let workbook: Xlsx<_> = open_workbook(path).expect("Cannot open xlsx file");

        return TranslationXlsx {
            path: path.to_owned(),
            trans_units: Vec::new(),
            xlsx: workbook,
        };
    }

    pub fn parse(&mut self) {
        let all_worksheets = self.xlsx.worksheets();
        let first_sheet = all_worksheets.get(0).unwrap();

        let mut cur_trans_unit;

        println!("Checking {} sheet: {}", self.path, &first_sheet.0);

        let mut trans_unit_rows = first_sheet.1.rows();

        let _ = &trans_unit_rows.next(); // We skip the first row as headers

        let mut buffer = Vec::new();

        for r in trans_unit_rows {
            let mut source_reader = XML_Reader::from_str(r.get(1).unwrap().get_string().unwrap());
            let mut target_reader = XML_Reader::from_str(r.get(2).unwrap().get_string().unwrap());

            let id = r.get(0).unwrap().get_string().unwrap().to_string();
            let source = SegNode::parse_segment(&mut source_reader, &mut buffer);
            let target = SegNode::parse_segment(&mut target_reader, &mut buffer);

            cur_trans_unit = TransUnit { id, source, target };
            if cur_trans_unit.id != "" {
                self.trans_units.push(cur_trans_unit)
            }
        }
    }
}
