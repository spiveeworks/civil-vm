extern crate sulphate_lib;
extern crate totem_cell;

pub mod algorithm;
pub mod data;
pub mod event;
pub mod item;
pub mod game;
pub mod parser;
pub mod table;

pub mod prelude {
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

    pub fn extract<T>(vals: &mut Dict<T>, names: &Dict<String>) -> Dict<T> {
        let mut result = Dict::with_capacity(names.len());
        for (new, old) in names {
            let name = new.clone();
            let val = vals
                .remove(old)
                .expect("Term not available for new state");
            result.insert(name, val);
        }
        result
    }
}
