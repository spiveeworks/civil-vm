use std::mem;

use prelude::*;

use data;
use game;
use time;

// for the parser >:P
pub enum Item {
    Action(Action),
    Table(Vec<String>),
}

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
    pub terms: Dict<TableTerm>,
}

pub type Action = Vec<Statement>;

pub enum Statement {
    Debug(String),
    ExecEntity {
        ent_name: String,
        action_name: String,
        args: Dict<String>,
    },
    State {
        name: String,
        terms: Dict<String>,
        wait: Option<Time>,
    },
    CancelWait,
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

fn continue_action(
    totem: &mut Totem,
    event_queue: &mut time::EventQueue,
    types: &Dict<data::EntityType>,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,
    pc: usize,
) {
    let vars = {
        let entity = entity.borrow_mut(totem);
        entity.event = None;

        mem::replace(&mut entity.data, Dict::new())
    };
    execute_action(
        totem,
        event_queue,
        types,

        entity,
        table_name,
        action_name,

        vars,
        pc,
        false,
    );
}

fn execute_reaction(
    totem: &mut Totem,
    event_queue: &mut time::EventQueue,
    types: &Dict<data::EntityType>,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    vars: Dict<data::Field>,
) {
    execute_action(
        totem,
        event_queue,
        types,

        entity,
        table_name,
        action_name,

        vars,
        0,
        true,
    );
}

pub fn execute_action(
    totem: &mut Totem,
    event_queue: &mut time::EventQueue,
    types: &Dict<data::EntityType>,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    mut vars: Dict<data::Field>,
    mut pc: usize,
    mut has_state: bool,
) {
    // current continuation
    let mut cc = None;

    let type_name = {
        let entity = entity.borrow(totem);
        entity.type_name.clone()
    };

    let code = get_action(types, &type_name, &table_name, &action_name);

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

                vars = extract(&mut vars, args);
            },
            Statement::State {
                ref name,
                ref terms,
                wait,
            } => {
                if has_state {
                    panic!("Tried to overwrite state without cancelling");
                }

                let state_name = name.clone();

                update_state(
                    totem,
                    event_queue,

                    &entity,
                    &mut vars,
                    terms,

                    table_name,
                    action_name,
                    state_name,

                    pc,
                    wait,
                );

                return;
            },
            Statement::CancelWait => {
                let entity = entity.borrow_mut(totem);
                let event = entity.event.take();
                if let Some(data::EventHandle(ref time, id)) = event {
                    event_queue.cancel_event(time, id);
                }

                entity.data = Dict::new();

                has_state = false;
            },
        }

        pc += 1;
    }

    if has_state {
        if pc != code.len() {
            println!("Warning: Code after external call will not be executed
without creating a new entity state");
        }
    } else if let Statement::State {
        ref name,
        ref terms,
        wait,
    } = code[pc] {
        let state_name = name.clone();

        update_state(
            totem,
            event_queue,

            &entity,
            &mut vars,
            terms,

            table_name,
            action_name,
            state_name,

            pc,
            wait,
        );
    } else {
        panic!("Tried to exit without resetting state");
    }

    if let Some((entity, table_name, action_name)) = cc {
        execute_reaction(
            totem,
            event_queue,
            types,
            entity,
            table_name,
            action_name,
            vars
        );
    }
}

fn update_state(
    totem: &mut Totem,
    event_queue: &mut time::EventQueue,

    entity: &data::Entity,
    vars: &mut data::Data,
    terms: &Dict<String>,

    table_name: String,
    action_name: String,
    state_name: String,

    pc: usize,
    wait: Option<Time>,
) {
    let data = extract(vars, terms);
    let event = {
        if let Some(time) = wait {
            let entity = Strong::clone(entity);
            let event = Event { entity, table_name, action_name, pc };

            let absolute_time = event_queue.now() + time;
            let id = event_queue.enqueue_absolute(event, absolute_time);

            Some(data::EventHandle(absolute_time, id))
        } else {
            None
        }
    };

    let entity = entity.borrow_mut(totem);

    entity.state_name = state_name;
    entity.data = data;
    entity.event = event;
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


pub struct Event {
    pub entity: data::Entity,

    pub table_name: String,
    pub action_name: String,
    pub pc: usize,
}

impl Event {
    pub fn invoke(self: Self, game: &mut game::Game) {
        continue_action(
            &mut game.totem,
            &mut game.event_queue,
            &mut game.types,
            self.entity,

            self.table_name,
            self.action_name,
            self.pc,
        );
    }
}
