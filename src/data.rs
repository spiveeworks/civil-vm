use prelude::*;
use programs;

// we could use a union
pub enum Field {
    Integer(i64),  // for numbers with conservation, and C-enums, and indeces
    Real(f64),  // for any other kind of number
    Data(Data),  // substructures
    Ref(Ref),
    // Weak(WeakRef),
    // List(???),
}

pub struct Data {
    var: usize,  // variant
    fields: Vec<Field>,  // could also be a hashmap: String -> Field
}

pub struct Ref {
    table: &'static programs::Table,
    data: Strong<Data>,
}

