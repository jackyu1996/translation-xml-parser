use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;

pub struct TmxFile {
    pub path: String,
    pub tus: Vec<TU>,
    reader: Reader<BufReader<File>>,
}

#[derive(Debug)]
pub struct TU {
    pub tuvs: Vec<TUV>,
}

#[derive(Debug)]
pub struct TUV {
    pub src_language: String,
    pub tgt_language: String,
    pub seg: String,
}

impl TmxFile {
    pub fn new(path: String) -> TmxFile {
        let read_result = Reader::from_file(&path);

        let file_reader = match read_result {
            Ok(file) => file,
            Err(e) => panic!("Failed to open file: {:?}", e),
        };

        return TmxFile {
            path: path,
            tus: Vec::new(),
            reader: file_reader,
        };
    }

    pub fn parse(&mut self) {
        let mut tu = TU { tuvs: Vec::new() };

        let mut tuv = TUV {
            src_language: String::new(),
            tgt_language: String::new(),
            seg: String::new(),
        };

        let mut buf = Vec::new();

        loop {
            match self.reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"tu" => {
                        if tu.tuvs.len() != 0 {
                            self.tus.push(tu);
                        }
                        tu = TU { tuvs: Vec::new() }
                    }
                    b"tuv" => {
                        if !tuv.src_language.is_empty()
                            && !tuv.tgt_language.is_empty()
                            && !tuv.seg.is_empty()
                        {
                            tu.tuvs.push(tuv);
                        }

                        tuv = TUV {
                            src_language: String::new(),
                            tgt_language: String::new(),
                            seg: String::new(),
                        };
                    }
                    b"seg" => {
                        tuv.seg = self.reader.read_text(e.name(), &mut Vec::new()).unwrap();
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                Err(e) => panic!(
                    "Error at position {}: {:?}",
                    self.reader.buffer_position(),
                    e
                ),
                _ => (),
            }
            buf.clear();
        }
    }
}
