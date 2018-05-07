extern crate totem_cell;

pub mod data;
pub mod programs;

pub mod prelude {
    pub use totem_cell::Totem;
    pub type Cell<T> = ::totem_cell::TotemCell<T>;

    pub type Strong<T> = ::std::rc::Rc<Cell<T>>;
    pub type Weak<T> = ::std::rc::Weak<Cell<T>>;


    pub fn strong<T>(val: T) -> Strong<T> where T: 'static {
        ::std::rc::Rc::new(Cell::new(val))
    }


    pub fn heap<T: 'static>(val: T) -> &'static T {
        let as_box = Box::new(val);
        let as_ptr = Box::into_raw(as_box);
        unsafe { &*as_ptr }
    }
}
