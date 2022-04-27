use roxmltree::Document;
use std::io::prelude::*;
use zip;

#[derive(Debug)]
pub struct XliffFile {
    pub path: String,
    pub xfiles: Vec<XFile>,
    raw_content: String,
}

#[derive(Debug, Default)]
pub struct XFile {
    pub src_language: String,
    pub tgt_language: String,
    pub trans_units: Vec<TransUnit>,
}

#[derive(Debug, Default)]
pub struct TransUnit {
    pub id: String,
    pub source: String,
    pub target: String,
}

impl XliffFile {
    pub fn new(path: String) -> XliffFile {
        let content = crate::read_file(&path);

        return XliffFile {
            path: path,
            xfiles: Vec::new(),
            raw_content: content,
        };
    }

    pub fn parse(&mut self) {
        let mut cur_xfile = XFile::default();
        let mut cur_trans_unit = TransUnit::default();

        let doc = Document::parse(&self.raw_content).expect("Failed to parse Xliff File!");

        for file in doc.descendants().filter(|n| n.tag_name().name() == "file") {
            cur_xfile.src_language = file.attribute("source-language").unwrap().to_string();
            cur_xfile.tgt_language = file.attribute("target-language").unwrap().to_string();

            for unit in file
                .descendants()
                .filter(|n| n.tag_name().name() == "trans-unit")
            {
                cur_trans_unit.id = unit.attribute("id").unwrap().to_string();
                for value in unit.children() {
                    match value.tag_name().name() {
                        "source" => {
                            cur_trans_unit.source = crate::get_children_text(value).concat()
                        }
                        "target" => {
                            cur_trans_unit.target = crate::get_children_text(value).concat()
                        }
                        _ => (),
                    }
                }
                cur_xfile.trans_units.push(cur_trans_unit);
                cur_trans_unit = TransUnit::default();
            }
            self.xfiles.push(cur_xfile);
            cur_xfile = XFile::default();
        }
    }

    pub fn read_from_xlz(path: String) -> XliffFile {
        let xlzfile = std::fs::File::open(&path).expect("Cannot open xlz file!");

        let mut archive = zip::ZipArchive::new(xlzfile).expect("Invalid xlz file!");

        let mut file = match archive.by_name("content.xlf") {
            Ok(file) => file,
            Err(_) => {
                panic!("content.xlf not found in xlz file")
            }
        };

        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        return XliffFile {
            path: path,
            xfiles: Vec::new(),
            raw_content: contents,
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_for_debug() {
        let mut t = crate::xliff::XliffFile::new("./tests/sul.txlf".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
