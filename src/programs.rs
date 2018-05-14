use prelude::*;

use data;
use game;

pub enum TableTerm {
    Action(Action),
}

impl TableTerm {
    pub fn action(self: &Self) -> &Action {
        match *self {
            TableTerm::Action(ref result) => result,
            // _ => panic!("Expected action"),
        }
    }
}

pub struct Table {
    terms: Dict<TableTerm>,
}

type Action = Vec<Statement>;

pub enum Statement {
    Debug(String),
    ExecEntity {
        ent_name: String,
        action_name: String,
        args: Vec<String>,
    },
    State {
        name: String,
        terms: Dict<String>,
        wait: Option<f64>,
    },
}

fn get_action<'a>(
    types: &'a Dict<data::EntityType>,

    entity_type_name: &String,
    table_name: &String,
    action_name: &String,
) -> &'a Action {
    let entity_type = &types[entity_type_name];
    let table = &entity_type[table_name];
    table.terms[action_name].action()
}

pub fn execute(
    totem: &mut Totem,
    game: &mut game::Game,
    types: &Dict<data::EntityType>,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    mut vars: Dict<data::Field>
) {
    // current continuation
    let mut cc = Some((entity, table_name, action_name));

    loop {
        let (entity, table_name, action_name) =
            cc.take().unwrap();

        let entity_type= {
            let entity = entity.borrow(totem);
            entity.type_name.clone()
        };

        let code = get_action(types, &entity_type, &table_name, &action_name);
        let mut pc = 0;

        while cc.is_none() {
            match code[pc] {
                Statement::Debug(ref to_print) => {
                    println!("Debug: {}", to_print);
                },
                Statement::ExecEntity {
                    ref ent_name,
                    ref action_name,
                    ref args,
                } => {
                    {
                        let entity_ref = vars[ent_name].entity();

                        let entity = Strong::clone(&entity_ref.data);
                        let table_name = entity_ref.table.clone();
                        let action_name = action_name.clone();

                        cc = Some((entity, table_name, action_name));
                    }

                    vars.retain(|k, _| args.contains(k));
                },
                Statement::State {
                    ref name,
                    ref terms,
                    wait,
                } => {
                    let data = extract(&mut vars, terms);
                    let event = {
                        if let Some(time) = wait {
                            unimplemented!();
                        } else {
                            None
                        }
                    };

                    let state_name = name.clone();

                    let entity = entity.borrow_mut(totem);

                    entity.table_name = table_name;
                    entity.action_name = action_name;
                    entity.state_name = state_name;
                    entity.data = data;
                    entity.event = event;

                    break;
                },
            }

            pc += 1;
        }
    }
}

fn extract<T>(vals: &mut Dict<T>, names: &Dict<String>) -> Dict<T> {
    let mut result = Dict::with_capacity(names.len());
    for (new, old) in names {
        let name = new.clone();
        let val = vals
            .remove(old)
            .expect("Term not available for new state");
        result.insert(name, val);
    }
    result
}

