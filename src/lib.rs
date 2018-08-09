extern crate sulphate_lib;
extern crate totem_cell;

pub mod ast;
pub mod data;
pub mod event;
pub mod instance;
pub mod interface;
pub mod item;
pub mod load_types;
pub mod parser;
pub mod runtime;

pub mod prelude {
    pub use instance::Flop;

    // Units (just Time)
    pub use sulphate_lib::Time;


    // Cell
    pub use totem_cell::Totem;
    pub type Cell<T> = ::totem_cell::TotemCell<T>;


    // Rc
    pub type Strong<T> = ::std::rc::Rc<Cell<T>>;
    pub type Weak<T> = ::std::rc::Weak<Cell<T>>;

    pub fn strong<T>(val: T) -> Strong<T> where T: 'static {
        ::std::rc::Rc::new(Cell::new(val))
    }


    // HashMap
    pub type Dict<T> = ::std::collections::HashMap<String, T>;

    pub fn extract<T: Clone>(vals: &Dict<T>, names: &Dict<String>) -> Dict<T> {
        let mut result = Dict::with_capacity(names.len());
        for (new, old) in names {
            let name = new.clone();
            let val = vals[old].clone();
            result.insert(name, val);
        }
        result
    }
}
