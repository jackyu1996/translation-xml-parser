use roxmltree::Document;

#[derive(Debug)]
pub struct TbxFile {
    pub path: String,
    pub term_entries: Vec<TermEntry>,
    raw_content: String,
}

#[derive(Debug, Default)]
pub struct TermEntry {
    pub lang_sets: Vec<LangSet>,
}

#[derive(Debug, Default)]
pub struct LangSet {
    pub language: String,
    pub tigs: Vec<Tig>,
}

#[derive(Debug, Default)]
pub struct Tig {
    pub term: String,
    pub description: String,
}

impl TbxFile {
    pub fn new(path: String) -> TbxFile {
        let content = crate::read_file(&path);

        return TbxFile {
            path: path,
            term_entries: Vec::new(),
            raw_content: content,
        };
    }

    pub fn parse(&mut self) {
        let mut cur_term_entry = TermEntry::default();
        let mut cur_lang_set = LangSet::default();
        let mut cur_tig = Tig::default();

        let doc = Document::parse(&self.raw_content).unwrap();

        for node in doc.descendants() {
            match node.tag_name().name() {
                "termEntry" => {
                    if cur_term_entry.lang_sets.len() != 0 {
                        self.term_entries.push(cur_term_entry);
                        cur_term_entry = TermEntry::default();
                    }
                }
                "langSet" => {
                    if cur_lang_set.tigs.len() != 0 {
                        cur_term_entry.lang_sets.push(cur_lang_set);
                        cur_lang_set = LangSet::default();
                    }
                    cur_lang_set.language = node
                        .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                        .unwrap()
                        .to_string();
                }
                "tig" => {
                    if !cur_tig.term.is_empty() {
                        cur_lang_set.tigs.push(cur_tig);
                        cur_tig = Tig::default();
                    }
                }
                "term" => {
                    cur_tig.term = crate::get_children_text(node).concat();
                }
                "descrip" => {
                    cur_tig.description = crate::get_children_text(node).concat();
                }
                _ => (),
            }
        }
    }
}

mod tests {
    #[test]
    fn it_works() {
        let mut t = crate::tbx::TbxFile::new("./tests/lancom.tbx".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
