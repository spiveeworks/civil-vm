use prelude::*;

use data;
use runtime;


pub struct Interface {
    pub algorithms: Dict<runtime::Algorithm>,
}

pub type Role = Dict<RoleTerm>;

pub enum RoleTerm {
    Constructor(Vec<data::FieldType>),
    Function(Vec<data::FieldType>),
}
