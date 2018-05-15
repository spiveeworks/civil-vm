use prelude::*;

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
    use sulphate_lib::event_queue::Simulation;
    pub fn invoke_next(self: &mut Self) {
        self.invoke_next();
    }
}

impl sulphate::GeneralEvent<game::Game> for programs::Event {
    fn invoke(self: Self, game: &mut game::Game) {
        self.invoke(game);
    }
}
