use std::{fs::File, io::BufReader};

use calamine::{open_workbook, Reader, Xlsx};

use crate::xliff::TransUnit;

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

        for r in trans_unit_rows {
            cur_trans_unit = TransUnit {
                id: r.get(0).unwrap().get_string().unwrap().to_string(),
                source: r.get(1).unwrap().get_string().unwrap().to_string(),
                target: r.get(2).unwrap().get_string().unwrap().to_string(),
            };
            if cur_trans_unit.id != "" {
                self.trans_units.push(cur_trans_unit)
            }
        }
    }
}
