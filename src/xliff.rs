use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;

pub struct XliffFile {
    pub path: String,
    pub src_language: String,
    pub tgt_language: String,
    pub trans_units: Vec<TransUnit>,
    reader: Reader<BufReader<File>>,
}

#[derive(Debug)]
pub struct TransUnit {
    pub source: String,
    pub target: String,
    pub notes: Vec<String>,
}

impl XliffFile {
    pub fn new(path: String) -> XliffFile {
        let read_result = Reader::from_file(&path);

        let file_reader = match read_result {
            Ok(file) => file,
            Err(e) => panic!("Failed to open file: {:?}", e),
        };

        return XliffFile {
            path: path,
            src_language: String::new(),
            tgt_language: String::new(),
            trans_units: Vec::new(),
            reader: file_reader,
        };
    }

    pub fn parse(&mut self) {
        let mut trans_unit = TransUnit {
            source: String::new(),
            target: String::new(),
            notes: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match self.reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    //b"file" => {
                    //let attrs: Vec<String> = e
                    //.attributes()
                    //.map(|a| {
                    //String::from_utf8(
                    //a.unwrap().value.into_owned(),
                    //)
                    //.unwrap()
                    //})
                    //.collect();
                    //println!("{:?}", attrs)
                    //}
                    b"trans-unit" => {
                        if !trans_unit.source.is_empty() && !trans_unit.target.is_empty() {
                            self.trans_units.push(trans_unit);
                        }

                        trans_unit = TransUnit {
                            source: String::new(),
                            target: String::new(),
                            notes: Vec::new(),
                        };
                    }
                    b"source" => {
                        trans_unit.source =
                            self.reader.read_text(e.name(), &mut Vec::new()).unwrap();
                    }
                    b"target" => {
                        trans_unit.target =
                            self.reader.read_text(e.name(), &mut Vec::new()).unwrap();
                    }
                    b"note" => {
                        let note = self.reader.read_text(e.name(), &mut Vec::new()).unwrap();
                        trans_unit.notes.push(note);
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
