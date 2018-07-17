extern crate flop;

use std::fs;

use flop::prelude::*;

use flop::instance::FlopInstance;
use flop::item::ObjectType;
use flop::parser::TypeParser;
use flop::event::EventQueue;

fn get_types() -> Dict<ObjectType> {
    let parser = TypeParser::new();
    let paths = fs::read_dir("./data")
        .expect("Failed to open \"data\" directory");

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

fn main() {
    let now = Time::try_from(0.0).unwrap();
    // only do this once ok?
    let totem = unsafe { Totem::new() };
    let event_queue = EventQueue::new(now);
    let types = get_types();

    let mut game = FlopInstance { totem, event_queue, types };

    FlopInstance::run(
        &mut game,

        "Root".into(),
        "Root".into(),
        "init".into(),
    );
}
