use std::hash::{Hash, Hasher};

use prelude::*;

use event;
use item;

pub enum FieldType {
    Num,
    Ref(item::TableIdent),
}

// we could use a union
#[derive(Clone)]
pub enum Field {
    Num(f64),
    Entity(EntityRef),
    // Data(DataTerm),
    // Weak(WeakRef),
    // List(???),
    Set(EntitySet),
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

    pub fn unwrap_entity(self: Self) -> EntityRef {
        match self {
            Field::Entity(result) => result,
            _ => panic!("Expected entity"),
        }
    }

    pub fn set(self: &mut Self) -> &mut EntitySet {
        match *self {
            Field::Set(ref mut result) => result,
            _ => panic!("Expected set"),
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

#[derive(Clone)]
pub struct EntityRef {
    pub table: String,
    pub data: Strong<EntityData>,
}


#[derive(Clone)]
pub struct EntityKey(pub EntityRef);

impl EntityKey {
    fn as_usize(self: &Self) -> usize {
        let as_ref = &*self.0.data;
        as_ref as *const Cell<EntityData> as usize
    }
}

impl Hash for EntityKey {
    fn hash<H: Hasher>(self: &Self, state: &mut H) {
        self.as_usize().hash(state);
    }
}

impl PartialEq for EntityKey {
    fn eq(self: &Self, other: &Self) -> bool {
        self.as_usize() == other.as_usize()
    }
}

impl Eq for EntityKey {
}

pub type EntitySet = ::std::collections::HashMap<EntityKey, ()>;

