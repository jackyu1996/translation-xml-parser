pub mod tbx;
pub mod tmx;
pub mod xliff;

use roxmltree::Node;
use std::fs::File;
use std::io::Read;

pub fn get_children_text<'a>(node: Node<'a, '_>) -> Vec<&'a str> {
    return node
        .children()
        .filter(|n| n.is_text())
        .map(|n| n.text().expect("Cannot read segment text!")) // Which segment should be clarified.
        .collect::<Vec<&str>>();
}

pub fn read_file(path: &str) -> String {
    let mut file = File::open(path).expect("Cannot read input file!");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Cannot read file to a string!");

    return content;
}
