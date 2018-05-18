extern crate sulphate_lib;
extern crate totem_cell;

pub mod data;
pub mod game;
pub mod parser;
pub mod programs;
pub mod time;

pub mod prelude {
    pub use sulphate_lib::Time;

    pub use totem_cell::Totem;
    pub type Cell<T> = ::totem_cell::TotemCell<T>;

    pub type Strong<T> = ::std::rc::Rc<Cell<T>>;
    pub type Weak<T> = ::std::rc::Weak<Cell<T>>;


    pub fn strong<T>(val: T) -> Strong<T> where T: 'static {
        ::std::rc::Rc::new(Cell::new(val))
    }

    pub type Dict<T> = ::std::collections::HashMap<String, T>;
}
