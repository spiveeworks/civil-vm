use prelude::*;

use sulphate_lib::event_queue;

use data;
use programs;
use time;

pub struct Game {
    pub totem: Totem,
    pub event_queue: time::EventQueue,
    pub types: Dict<data::EntityType>,
    _root: data::Entity,
}

// purely for the Simulate trait, do not use
impl AsMut<time::EventQueue> for Game {
    fn as_mut(self: &mut Self) -> &mut time::EventQueue {
        &mut self.event_queue
    }
}

impl Game {
    pub fn invoke_next(self: &mut Self) {
        event_queue::Simulation::invoke_next(self);
    }
}

impl event_queue::GeneralEvent<Game> for programs::Event {
    fn invoke(self: Self, game: &mut Game) {
        programs::Event::invoke(self, game);
    }
}
