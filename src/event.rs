use sulphate_lib::event_queue;

use prelude::*;

use algorithm;
use data;


pub type EventQueue = event_queue::EventQueue<Event>;

pub struct EventHandle(pub Time, pub usize);

pub struct Event {
    pub entity: data::Entity,

    pub table_name: String,
    pub action_name: String,
    pub pc: usize,
}

impl Event {
    pub fn invoke<G: Flop>(self: Self, game: &mut G) {
        algorithm::execute_action(
            game,

            self.entity,
            self.table_name,
            self.action_name,

            None,
            self.pc,
            true,
        );
    }
}

impl<G: Flop> event_queue::GeneralEvent<G> for Event {
    fn invoke(self: Self, game: &mut G) {
        Event::invoke(self, game);
    }
}
