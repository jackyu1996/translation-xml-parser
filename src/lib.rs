pub mod tbx;
pub mod tmx;
pub mod xliff;
pub mod xlsx;

use quick_xml::events::BytesStart;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io::Read;

pub fn get_attribute(reader: &Reader<&[u8]>, start: &BytesStart, attribute_name: &str) -> String {
    return start
        .try_get_attribute(attribute_name)
        .unwrap()
        .unwrap()
        .decode_and_unescape_value(reader)
        .unwrap()
        .into_owned();
}

pub fn read_xml(path: &str) -> String {
    let mut file = File::open(path).expect("Cannot read input file!");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Cannot read file to a string!");

    return content;
}
