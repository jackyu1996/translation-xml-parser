use roxmltree::Document;

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

        let doc = Document::parse(&self.raw_content).unwrap();

        for node in doc.descendants() {
            match node.tag_name().name() {
                "file" => {
                    if cur_xfile.trans_units.len() != 0 {
                        self.xfiles.push(cur_xfile);
                        cur_xfile = XFile::default();
                    }
                    cur_xfile.src_language = node.attribute("source-language").unwrap().to_string();
                    cur_xfile.tgt_language = node.attribute("target-language").unwrap().to_string();
                }
                "trans-unit" => {
                    if cur_trans_unit.id != ""
                        && cur_trans_unit.source != ""
                        && cur_trans_unit.target != ""
                    {
                        cur_xfile.trans_units.push(cur_trans_unit);
                        cur_trans_unit = TransUnit::default();
                    }
                    cur_trans_unit.id = node.attribute("id").unwrap().to_string()
                }
                "source" => {
                    if node.parent().unwrap().tag_name().name() == "trans-unit" {
                        cur_trans_unit.source = crate::get_children_text(node).concat();
                    }
                }
                "target" => {
                    if node.parent().unwrap().tag_name().name() == "trans-unit" {
                        cur_trans_unit.target = crate::get_children_text(node).concat();
                    }
                }
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let mut t = crate::xliff::XliffFile::new("./tests/sul.txlf".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
