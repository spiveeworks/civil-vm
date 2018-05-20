use prelude::*;

use sulphate_lib::event_queue;

use algorithm;
use data;
use event;
use item;

pub struct Game {
    pub totem: Totem,
    pub event_queue: event::EventQueue,
    pub types: Dict<item::EntityType>,
    pub root: data::Entity,
}

// purely for the Simulate trait, do not use
impl AsMut<event::EventQueue> for Game {
    fn as_mut(self: &mut Self) -> &mut event::EventQueue {
        &mut self.event_queue
    }
}

impl Game {
    pub fn invoke_next(self: &mut Self) {
        event_queue::Simulation::invoke_next(self);
    }

    pub fn run(self: &mut Self) {
        algorithm::execute_init(
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
