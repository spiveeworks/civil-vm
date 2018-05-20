use prelude::*;

use event;


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

pub type Entity = Strong<EntityData>;

pub struct EntityData {
    // for cancelling the current wait timer
    pub event: Option<event::EventHandle>,

    pub type_name: String,
    // purely for saving to file
    pub state_name: String,

    pub data: Data,
}

impl EntityData {
    pub fn new(type_name: String) -> Entity {
        let data = Dict::new();
        let state_name = "EMPTY".into();
        let event = None;
        let entity = EntityData { event, type_name, state_name, data };
        strong(entity)
    }
}

pub struct EntityRef {
    pub table: String,
    pub data: Strong<EntityData>,
}

