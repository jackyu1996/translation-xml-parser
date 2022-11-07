use crate::SegNode;
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
    pub seg: Vec<SegNode>,
}

impl TmxFile {
    pub fn new(path: &str) -> TmxFile {
        let content = crate::read_xml(path);

        let mut tmx_file = TmxFile {
            path: path.to_string(),
            tus: Vec::new(),
            raw_content: content,
        };
        tmx_file.parse();
        return tmx_file;
    }

    fn parse(&mut self) {
        let mut buf = Vec::new();

        let mut cur_tu = TU::default();
        let mut cur_tuv = TUV::default();
        let mut cur_seg: Vec<SegNode>;

        let mut reader = Reader::from_str(&self.raw_content);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"tu" => {
                        cur_tu.tuid = crate::get_attributes(&reader, &e)
                            .get("tuid")
                            .unwrap_or(&"".to_string())
                            .to_owned()
                    }
                    b"tuv" => {
                        cur_tuv.language = crate::get_attributes(&reader, &e)
                            .get("xml:lang")
                            .unwrap()
                            .to_owned()
                    }

                    b"seg" => {
                        cur_seg = SegNode::parse_segment(&mut reader, &mut buf);
                        if cur_seg.len() != 0 {
                            cur_tuv.seg = cur_seg;
                        }
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"tuv" => {
                        if cur_tuv.seg.len() != 0 {
                            cur_tu.tuvs.push(cur_tuv)
                        }

                        cur_tuv = TUV::default();
                    }
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
        let t = crate::tmx::TmxFile::new(&"./tests/CITIC.tmx");
        dbg!(t.tus);
        assert!(1 != 2);
    }
}
