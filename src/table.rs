use prelude::*;

use algorithm;
use data;

pub enum TableTerm {
    Action(algorithm::Algorithm),
    Constructor(algorithm::Algorithm),
}

impl TableTerm {
    pub fn algorithm(self: &Self) -> &algorithm::Algorithm {
        match *self {
            TableTerm::Action(ref result) => result,
            TableTerm::Constructor(ref result) => result,
        }
    }
}

pub struct Table {
    pub terms: Dict<TableTerm>,
}

pub type Signature = Dict<SignatureTerm>;

pub enum SignatureTerm {
    Initializer(Vec<data::FieldType>),
    Action(Vec<data::FieldType>),
}
