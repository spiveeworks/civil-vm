use prelude::*;

use data;
use runtime;

pub enum TableTerm {
    Function(runtime::Algorithm),
    Constructor(runtime::Algorithm),
}

impl TableTerm {
    pub fn algorithm(self: &Self) -> &runtime::Algorithm {
        match *self {
            TableTerm::Function(ref result) => result,
            TableTerm::Constructor(ref result) => result,
        }
    }
}

pub struct Table {
    pub terms: Dict<TableTerm>,
}

pub type Signature = Dict<SignatureTerm>;

pub enum SignatureTerm {
    Constructor(Vec<data::FieldType>),
    Function(Vec<data::FieldType>),
}
