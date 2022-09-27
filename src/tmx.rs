use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct TmxFile {
    pub path: String,
    pub tus: Vec<TU>,
    raw_content: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TU {
    pub tuid: String,
    pub tuvs: Vec<TUV>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TUV {
    pub language: String,
    pub seg: String,
}

impl TmxFile {
    pub fn new(path: String) -> TmxFile {
        let content = crate::read_xml(&path);

        return TmxFile {
            path: path.to_string(),
            tus: Vec::new(),
            raw_content: content,
        };
    }

    pub fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_tu = TU::default();
        let mut cur_tuv;

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"tu" => cur_tu.tuid = crate::get_attribute("tuid", &e, &reader),
                    b"tuv" => {
                        cur_tuv = TUV {
                            language: crate::get_attribute("xml:lang", &e, &reader),
                            seg: reader.read_text(e.name()).unwrap().into_owned(),
                        };
                        if cur_tuv.seg != "" {
                            cur_tu.tuvs.push(cur_tuv)
                        }
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"tu" => {
                        if cur_tu.tuvs.len() != 0 {
                            self.tus.push(cur_tu);
                        }

                        cur_tu = TU::default();
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                _ => (),
            }
            buf.clear()
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
