use prelude::*;

use sulphate_lib::event_queue;

use data;
use programs;
use time;

pub struct Game {
    pub totem: Totem,
    pub event_queue: time::EventQueue,
    pub types: Dict<data::EntityType>,
    pub root: data::Entity,
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

    pub fn run(self: &mut Self) {
        programs::execute_init(
            &mut self.totem,
            &mut self.event_queue,
            &mut self.types,
            Strong::clone(&self.root),
        );

        while !self.event_queue.is_empty() {
            self.invoke_next();
        }
        println!("Nothing happened.");
    }
}

impl event_queue::GeneralEvent<Game> for programs::Event {
    fn invoke(self: Self, game: &mut Game) {
        programs::Event::invoke(self, game);
    }
}
