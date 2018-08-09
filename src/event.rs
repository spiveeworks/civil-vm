use sulphate_lib::event_queue;

use prelude::*;

use data;
use runtime;


pub type EventQueue = event_queue::EventQueue<Event>;

pub struct EventHandle(pub Time, pub usize);

pub struct Event {
    pub object: data::Object,

    pub action_name: String,
    pub pc: usize,
}

impl Event {
    pub fn invoke<G: Flop>(self: Self, game: &mut G) {
        runtime::execute_algorithm(
            game,
            self.object,
            self.action_name,
            runtime::ExecType::Resume(self.pc),
        );
    }
}

impl<G: Flop> event_queue::GeneralEvent<G> for Event {
    fn invoke(self: Self, game: &mut G) {
        Event::invoke(self, game);
    }
}
