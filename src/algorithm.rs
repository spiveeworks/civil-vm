use std::mem;

use prelude::*;

use data;
use event;
use item;

pub struct Algorithm {
    pub param_list: Vec<String>,
    pub steps: Vec<Statement>,
}

pub enum Statement {
    Debug(String),
    ExecEntity {
        ent_name: String,
        action_name: String,
        args: Dict<String>,
    },
    Assign {
        // multiple expressions all so that x, y = y, x is possible :P
        expressions: Vec<Expression>,
        results: Vec<String>,
    },
    State {
        name: String,
        terms: Dict<String>,
        wait: Option<Time>,
    },
    CancelWait,
}

pub enum Expression {
    MoveVar(String),
    CloneVar(String),
    InitEntity {
        type_name: String,
        table_name: String,
        init_name: String,
        args: Vec<Expression>,
    },
}

pub fn execute_init(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    type_name: String,
    table_name: String,
    init_name: String,
    args: Vec<data::Field>,
) -> data::Entity {
    let args = item::get_algorithm(
        types,
        &type_name,
        &table_name,
        &init_name,
    )   .param_list
        .iter()
        .cloned()
        .zip(args)
        .collect();
    let result = data::EntityData::new(type_name);
    execute_reaction(
        totem,
        event_queue,
        types,
        Strong::clone(&result),

        table_name,
        init_name,
        args
    );
    result
}

pub fn continue_action(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

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
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

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
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

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

    let code = &item::get_algorithm(
        types,
        &type_name,
        &table_name,
        &action_name
    ).steps;

    while pc < code.len() && cc.is_none() {
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
            Statement::Assign {
                ref results,
                ref expressions,
            } => {
                unimplemented!();
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
                if let Some(event::EventHandle(ref time, id)) = event {
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
    } else if let Some(&Statement::State {
        ref name,
        ref terms,
        wait,
    }) = code.get(pc) {
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
    event_queue: &mut event::EventQueue,

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
            let event = event::Event { entity, table_name, action_name, pc };

            let absolute_time = event_queue.now() + time;
            let id = event_queue.enqueue_absolute(event, absolute_time);

            Some(event::EventHandle(absolute_time, id))
        } else {
            None
        }
    };

    let entity = entity.borrow_mut(totem);

    entity.state_name = state_name;
    entity.data = data;
    entity.event = event;
}

fn evaluate_expressions(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    expressions: &Vec<Expression>,
    vars: &mut data::Data,
) -> Vec<data::Field> {
    let mut result = Vec::new();

    use self::Expression::*;
    for expression in expressions {
        match *expression {
            MoveVar(ref name) => {
                let val = vars.remove(name).expect("Variable not in scope");
                result.push(val);
            },
            CloneVar(ref name) => {
                let val = vars[name].clone();
                result.push(val);
            },
            InitEntity {
                ref type_name,
                ref table_name,
                ref init_name,
                ref args,
            } => {
                let args = evaluate_expressions(
                    totem,
                    event_queue,
                    types,

                    expressions,
                    vars,
                );
                let result_val = execute_init(
                    totem,
                    event_queue,
                    types,

                    type_name.clone(),
                    table_name.clone(),
                    init_name.clone(),
                    args
                );

                let table = table_name.clone();
                let result_ref = data::EntityRef { data: result_val, table };
                let result_term = data::Field::Entity(result_ref);
                result.push(result_term);
            },
        }
    }

    result
}
