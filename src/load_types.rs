use std::fs;

use prelude::*;

use item;
use parser;

pub fn get_types(dirpath: &str) -> Dict<item::ObjectType> {
    let parser = parser::TypeParser::new();
    let paths = fs::read_dir(dirpath)
        .expect("Failed to open directory");

    let mut types = Dict::new();
    for path in paths {
        use std::io::Read;

        let path = path.expect("IO error getting paths").path();
        let path_str = path.to_str().expect("invalid path name");
        let file_name = path.file_stem().unwrap().to_str().unwrap().into();
        let mut file = fs::File::open(path_str).expect("Failed to open file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read file");

        let parsed = parser.parse(&content).unwrap();
        types.insert(file_name, parsed);
    }

    types
}

