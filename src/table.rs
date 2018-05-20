use prelude::*;

use algorithm;


pub enum TableTerm {
    Action(algorithm::Action),
}

impl TableTerm {
    pub fn action(self: &Self) -> &algorithm::Action {
        match *self {
            TableTerm::Action(ref result) => result,
            // _ => panic!("Expected action"),
        }
    }
}

pub struct Table {
    pub terms: Dict<TableTerm>,
}

