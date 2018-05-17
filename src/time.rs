use sulphate_lib::event_queue;

use programs;

pub type EventQueue = event_queue::EventQueue<programs::Event>;

