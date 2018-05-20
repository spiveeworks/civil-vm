use prelude::*;

use algorithm;

pub enum TableTerm {
    Initializer(algorithm::Initializer),
}

impl TableTerm {
    pub fn initializer(self: &Self) -> &algorithm::Initializer {
        match *self {
            TableTerm::Initializer(ref result) => result,
            // _ => panic!("Expected action"),
        }
    }
}

pub struct Table {
    pub terms: Dict<TableTerm>,
}

