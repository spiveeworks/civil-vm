use prelude::*;

use sulphate_lib::event_queue;

use algorithm;
use data;
use event;
use item;

pub struct FlopInstance {
    pub totem: Totem,
    pub event_queue: event::EventQueue,
    pub types: Dict<item::EntityType>,
    pub root: data::EntityRef,
}

// purely for the Simulate trait, do not use
impl AsMut<event::EventQueue> for FlopInstance {
    fn as_mut(self: &mut Self) -> &mut event::EventQueue {
        &mut self.event_queue
    }
}

impl FlopInstance {
    pub fn invoke_next(self: &mut Self) {
        event_queue::Simulation::invoke_next(self);
    }

    pub fn run(
        mut totem: Totem,
        mut event_queue: event::EventQueue,
        types: Dict<item::EntityType>,

        root_type: String,
        root_table: String,
        init: String,
    ) {
        let root = algorithm::execute_init(
            &mut totem,
            &mut event_queue,
            &types,

            root_type,
            root_table,
            init,
            Vec::new(),
        );

        let mut game = FlopInstance { totem, event_queue, types, root };

        while !game.event_queue.is_empty() {
            game.invoke_next();
        }
        println!("Nothing happened.");
    }
}
