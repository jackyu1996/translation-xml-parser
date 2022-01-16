pub mod tbx;
pub mod tmx;
pub mod xliff;

use quick_xml::events::BytesStart;

pub fn get_attribute(e: &BytesStart, attr_key: &str) -> String {
    let attr_value = e
        .attributes()
        .find(|a| a.as_ref().unwrap().key == attr_key.as_bytes())
        .unwrap()
        .unwrap()
        .value
        .into_owned();
    return String::from_utf8(attr_value).unwrap();
}
