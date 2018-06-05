use sulphate_lib::event_queue;

use prelude::*;

use algorithm;
use data;
use game;


pub type EventQueue = event_queue::EventQueue<Event>;

pub struct EventHandle(pub Time, pub usize);

pub struct Event {
    pub entity: data::Entity,

    pub table_name: String,
    pub action_name: String,
    pub pc: usize,
}

impl Event {
    pub fn invoke(self: Self, game: &mut game::Game) {
        algorithm::execute_action(
            &mut game.totem,
            &mut game.event_queue,
            &mut game.types,

            self.entity,
            self.table_name,
            self.action_name,

            None,
            self.pc,
            true,
        );
    }
}

impl event_queue::GeneralEvent<game::Game> for Event {
    fn invoke(self: Self, game: &mut game::Game) {
        Event::invoke(self, game);
    }
}
