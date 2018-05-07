pub enum Field {
    Integer(i64),  // for numbers with conservation, and C-enums, and indeces
    Real(f64),  // for any other kind of number
    Data(Data),  // substructures
    Ref(Ref),
}

pub struct Data {
    var: usize,  // variant of the enum
    fields: Vec<Field>,  // could also be a hashmap: String -> Field
}

pub struct Ref {
    table: ::programs::Table,
    data: ::std::rc::Rc<Data>, // totem cell?
}

