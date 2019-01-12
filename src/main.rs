extern crate flop;

use flop::prelude::*;

use flop::event::EventQueue;
use flop::instance::FlopInstance;
use flop::load_types::get_types;

fn main() {
    let now = Time::try_from(0.0).unwrap();
    // only do this once ok?
    let totem = unsafe { Totem::new() };
    let event_queue = EventQueue::new(now);
    let types = get_types("./data");

    let mut game = FlopInstance { totem, event_queue, types };

    FlopInstance::run(
        &mut game,

        "Root".into(),
        "init".into(),
    );
}
