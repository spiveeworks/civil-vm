use prelude::*;

use algorithm;
use data;

pub enum TableTerm {
    Initializer(algorithm::Algorithm),
}

impl TableTerm {
    pub fn algorithm(self: &Self) -> &algorithm::Algorithm {
        match *self {
            TableTerm::Initializer(ref result) => result,
            // _ => panic!("Expected action"),
        }
    }
}

pub struct Table {
    pub terms: Dict<TableTerm>,
}

pub type Signature = Dict<SignatureTerm>;

pub enum SignatureTerm {
    Initializer(Vec<data::FieldType>),
}
