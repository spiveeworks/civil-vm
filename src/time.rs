use sulphate_lib as sulphate;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time;

pub type EventQueue = sulphate::EventQueue<programs::Event, Time>;

