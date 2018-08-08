use sulphate_lib::event_queue;

use prelude::*;

use data;
use runtime;


pub type EventQueue = event_queue::EventQueue<Event>;

pub struct EventHandle(pub Time, pub usize);

pub struct Event {
    pub object: data::Object,

    pub table_name: String,
    pub action_name: String,
    pub pc: usize,
}

impl Event {
    pub fn invoke<G: Flop>(self: Self, game: &mut G) {
        runtime::resume_algorithm(
            game,

            self.object,
            self.table_name,
            self.action_name,
            self.pc,
        );
    }
}

impl<G: Flop> event_queue::GeneralEvent<G> for Event {
    fn invoke(self: Self, game: &mut G) {
        Event::invoke(self, game);
    }
}
