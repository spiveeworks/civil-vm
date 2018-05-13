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

impl Field {
    pub fn num(self: &Self) -> f64 {
        match *self {
            Field::Num(result) => result,
            _ => panic!("Expected number"),
        }
    }

    pub fn data(self: &Self) -> &Data {
        match *self {
            Field::Data(ref result) => result,
            _ => panic!("Expected data"),
        }
    }

    pub fn entity(self: &Self) -> &EntityRef {
        match *self {
            Field::Entity(ref result) => result,
            _ => panic!("Expected entity"),
        }
    }
}

pub struct Data {
    variant: usize,
    fields: Dict<Field>,
}

pub type EntityType = Dict<programs::Table>;

pub struct EntityData {
    pub type_name: String,
    event: Option<()>,
    action: String,
    state: String,
    data: Data,
}

pub struct EntityRef {
    pub table: String,
    pub data: Strong<EntityData>,
}

