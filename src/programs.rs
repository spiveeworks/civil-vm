use data;
use game;

pub enum TableTerm {
    Action(&'static Action),
}

pub struct Table {
    terms: Vec<TableTerm>,
}

type Action = [Statement];

pub enum Statement {
    Debug(String),
}


pub fn execute(
    game: &mut game::Game,
    code: &Action,
    pc: usize,
    mut vars: Vec<data::Field>) {
    loop {
        match code[pc] {
            Statement::Debug(ref to_print) => {
                println!("Debug: {}", to_print);
            },
        }
    }
}
