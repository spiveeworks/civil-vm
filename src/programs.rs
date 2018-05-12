use data;
use game;

pub enum TableTerm {
    Action(&'static Action, usize),
}

pub struct Table {
    terms: Vec<TableTerm>,
}

type Action = [Statement];

pub enum Statement {
    Debug(String),
    ExecTable(usize),
}


pub fn execute(
    game: &mut game::Game,
    mut code: &Action,
    mut pc: usize,
    mut vars: Vec<data::Field>
) {
    loop {
        match code[pc] {
            Statement::Debug(ref to_print) => {
                println!("Debug: {}", to_print);
            },
            Statement::ExecTable(table_term) => {
                /*
                let table = vars.pop().unwrap().table();
                let term = table[table_term];
                let (new_code, new_pc) = term.action();
                code = new_code;
                pc = new_pc;
                */
            },
        }
    }
}
