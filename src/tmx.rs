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

        let doc = Document::parse(&self.raw_content).expect("Failed to parse tmx file");

        for node in doc.descendants().filter(|n| n.tag_name().name() == "tu") {
            cur_tu.tuid = node
                .attribute("tuid")
                .expect("No tuid attribute found")
                .to_string();
            for tuv in node.children().filter(|n| n.tag_name().name() == "tuv") {
                cur_tuv.language = tuv
                    .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                    .expect("No lang attribute found")
                    .to_string();
                for seg in tuv.children().filter(|n| n.tag_name().name() == "seg") {
                    cur_tuv.seg = crate::get_children_text(seg).concat();
                }
                cur_tu.tuvs.push(cur_tuv);
                cur_tuv = TUV::default();
            }
            self.tus.push(cur_tu);
            cur_tu = TU::default();
        }
    }
}

mod tests {
    #[test]
    fn dummy_for_debug() {
        let mut t = crate::tmx::TmxFile::new("./tests/CITIC.tmx".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
