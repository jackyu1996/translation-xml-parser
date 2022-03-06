use roxmltree::Document;

#[derive(Debug)]
pub struct TmxFile {
    pub path: String,
    pub tus: Vec<TU>,
    raw_content: String,
}

#[derive(Debug, Default)]
pub struct TU {
    pub tuid: String,
    pub tuvs: Vec<TUV>,
}

#[derive(Debug, Default)]
pub struct TUV {
    pub language: String,
    pub seg: String,
}

impl TmxFile {
    pub fn new(path: String) -> TmxFile {
        let content = crate::read_file(&path);

        return TmxFile {
            path: path.to_string(),
            tus: Vec::new(),
            raw_content: content,
        };
    }

    pub fn parse(&mut self) {
        let mut cur_tu = TU::default();
        let mut cur_tuv = TUV::default();

        let doc = Document::parse(&self.raw_content).unwrap();

        for node in doc.descendants() {
            match node.tag_name().name() {
                "tu" => {
                    if cur_tu.tuvs.len() != 0 {
                        self.tus.push(cur_tu);
                        cur_tu = TU::default();
                    }
                    cur_tu.tuid = node.attribute("tuid").unwrap().to_string();
                }
                "tuv" => {
                    if !cur_tuv.seg.is_empty() {
                        cur_tu.tuvs.push(cur_tuv);
                        cur_tuv = TUV::default();
                    }
                    cur_tuv.language = node
                        .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                        .unwrap()
                        .to_string();
                }
                "seg" => {
                    cur_tuv.seg = crate::get_children_text(node).concat();
                }
                _ => (),
            }
        }
    }
}

mod tests {
    #[test]
    fn it_works() {
        let mut t = crate::tmx::TmxFile::new("./tests/CITIC.tmx".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
