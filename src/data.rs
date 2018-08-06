use std::hash::{Hash, Hasher};

use prelude::*;

use event;
use item;

pub enum FieldType {
    Num,
    VRef(item::TableIdent),
    TRef(String),
}

// we could use a union
#[derive(Clone)]
pub enum Field {
    Num(f64),
    VRef(ObjectRef),
    TRef(Object),
    Data(String, Data),
    // Weak(WeakRef),
    // List(???),
    Set(ObjectSet),
}

impl Field {
    pub fn from_bool(val: bool) -> Self {
        Field::Data(if val { "True" } else { "False" }.into(), Dict::new())
    }
    pub fn num(self: &Self) -> f64 {
        match *self {
            Field::Num(result) => result,
            _ => panic!("Expected number"),
        }
    }

    pub fn tref(self: &Self) -> &Object {
        match *self {
            Field::TRef(ref result) => result,
            _ => panic!("Expected object"),
        }
    }

    pub fn vref(self: &Self) -> &ObjectRef {
        match *self {
            Field::VRef(ref result) => result,
            _ => panic!("Expected object"),
        }
    }

    pub fn unwrap_tref(self: Self) -> Object {
        match self {
            Field::TRef(result) => result,
            _ => panic!("Expected object"),
        }
    }

    pub fn unwrap_vref(self: Self) -> ObjectRef {
        match self {
            Field::VRef(result) => result,
            _ => panic!("Expected object"),
        }
    }

    pub fn unwrap_data(self: Self) -> (String, Data) {
        match self {
            Field::Data(name, data) => (name, data),
            _ => panic!("Expected data"),
        }
    }

    pub fn bool(self: &Self) -> bool {
        match *self {
            Field::Data(ref name, ref data) => {
                if data.len() != 0 {
                    panic!("Bool can not have fields");
                }
                if name == "True" {
                    return true;
                }
                if name == "False" {
                    return false;
                }
                panic!("Bool is either True or False");
            },
            _ => panic!("Expected data (bool)"),
        }
    }

    pub fn unwrap_set(self: Self) -> ObjectSet {
        match self {
            Field::Set(result) => result,
            _ => panic!("Expected set"),
        }
    }

    pub fn set(self: &mut Self) -> &mut ObjectSet {
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

pub type Object = Strong<ObjectData>;

pub struct ObjectData {
    // for cancelling the current wait timer
    pub event: Option<event::EventHandle>,

    pub type_name: String,
    // purely for saving to file
    pub state_name: String,

    pub data: Data,
}

impl ObjectData {
    pub fn new(type_name: String) -> Object {
        let data = Dict::new();
        let state_name = "EMPTY".into();
        let event = None;
        let object = ObjectData { event, type_name, state_name, data };
        strong(object)
    }
}

#[derive(Clone)]
pub struct ObjectRef {
    pub table: String,
    pub data: Object,
}


#[derive(Clone)]
pub struct ObjectKey(pub ObjectRef);

impl ObjectKey {
    fn as_usize(self: &Self) -> usize {
        let as_ref = &*self.0.data;
        as_ref as *const Cell<ObjectData> as usize
    }
}

impl Hash for ObjectKey {
    fn hash<H: Hasher>(self: &Self, state: &mut H) {
        self.as_usize().hash(state);
    }
}

impl PartialEq for ObjectKey {
    fn eq(self: &Self, other: &Self) -> bool {
        self.as_usize() == other.as_usize()
    }
}

impl Eq for ObjectKey {
}

pub type ObjectSet = ::std::collections::HashMap<ObjectKey, ()>;

