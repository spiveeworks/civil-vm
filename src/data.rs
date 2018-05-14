use prelude::*;
use programs;

// we could use a union
pub enum Field {
    Num(f64),
    Entity(EntityRef),
    // Data(DataTerm),
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

    pub fn entity(self: &Self) -> &EntityRef {
        match *self {
            Field::Entity(ref result) => result,
            _ => panic!("Expected entity"),
        }
    }
}

/*
struct DataTerm {
    variant: String,
    terms: Data,
}
*/

pub type Data = Dict<Field>;

pub type EntityType = Dict<programs::Table>;

pub struct EntityData {
    // for cancelling the current wait timer
    event: Option<()>,

    // path to current code execution point
    pub type_name: String,
    table_name: String,
    action_name: String,
    state_name: String,

    data: Data,
}

pub struct EntityRef {
    pub table: String,
    pub data: Strong<EntityData>,
}

