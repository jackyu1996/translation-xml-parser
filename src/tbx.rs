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
    pub terms: Vec<Term>,
}

#[derive(Debug, Default)]
pub struct Term {
    pub term: String,
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
        let mut cur_term = Term::default();

        let doc = Document::parse(&self.raw_content).unwrap();

        for node in doc.descendants().filter(|n| n.tag_name().name() == "body") {
            for te in node.children() {
                for ls in te.children().filter(|n| n.tag_name().name() == "langSet") {
                    cur_lang_set.language = ls
                        .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                        .unwrap()
                        .to_string();
                    for term in ls.descendants().filter(|n| n.tag_name().name() == "term") {
                        cur_term.term = crate::get_children_text(term).concat();
                        cur_lang_set.terms.push(cur_term);
                        cur_term = Term::default();
                    }
                    cur_term_entry.lang_sets.push(cur_lang_set);
                    cur_lang_set = LangSet::default();
                }
                self.term_entries.push(cur_term_entry);
                cur_term_entry = TermEntry::default();
            }
        }
    }
}

mod tests {
    #[test]
    fn dummy_for_debug() {
        let mut t = crate::tbx::TbxFile::new("./tests/lancom.tbx".to_string());
        t.parse();
        assert!(1 != 2);
    }
}
