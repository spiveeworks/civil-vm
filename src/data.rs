use prelude::*;
use programs;

// we could use a union
pub enum Field {
    Num(f64),
    Data(Data),
    Entity(EntityRef),
    // Weak(WeakRef),
    // List(???),
}

pub struct Data {
    variant: usize,
    fields: Dict<Field>,
}

pub struct EntityData {
    event: Option<()>,
    action: String,
    state: String,
    data: Data,
}

pub struct EntityRef {
    table: String,
    data: Strong<EntityData>,
}

