use prelude::*;

use sulphate_lib::event_queue;

use programs;
use time;

pub struct Game {
    future: time::EventQueue,
    root: Strong<()>,
}

impl AsMut<time::EventQueue> for Game {
    fn as_mut(self: &mut Self) -> &mut time::EventQueue {
        &mut self.future
    }
}

impl Game {
    pub fn invoke_next(self: &mut Self) {
        use sulphate_lib::event_queue::Simulation;
        self.invoke_next();
    }
}

impl event_queue::GeneralEvent<Game> for programs::Event {
    fn invoke(self: Self, game: &mut Game) {
        self.invoke(game);
    }
}
