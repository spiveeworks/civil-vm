use sulphate_lib::event_queue;

use programs;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time;

pub type EventQueue = event_queue::EventQueue<programs::Event, Time>;

