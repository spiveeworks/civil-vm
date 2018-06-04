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
    },
    Wait(Time),
    CancelWait,
    SetAdd {
        set_name: String,
        to_add: Expression,
    },
    SetRemove {
        set_name: String,
        to_remove: Expression,
    },
    SetIterate {
        set_name: String,
        var_name: String,
        break_line: usize,
    },
    Continue,
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
) -> data::EntityRef {
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

    let table = table_name.clone();
    let data = data::EntityData::new(type_name);

    execute_action(
        totem,
        event_queue,
        types,

        Strong::clone(&data),
        table_name,
        init_name,

        Some(args),
        0,
        false,
    );

    data::EntityRef { table, data }
}

pub fn execute_action(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    vars: Option<data::Data>,  // None to use entity's state
    pc: usize,
    mut has_state: bool,
) {
    let vars = {
        if let Some(vars) = vars {
            vars
        } else {
            has_state = false;

            let entity = entity.borrow_mut(totem);
            entity.event = None;

            mem::replace(&mut entity.data, Dict::new())
        }
    };

    let result = execute_algorithm(
        totem,
        event_queue,
        types,

        entity,
        table_name,
        action_name,

        vars,
        pc,
        has_state,
    );

    if let AlgorithmResult::ExternContinuation {
        entity,
        table_name,
        action_name,
        vars,
    } = result {
        execute_action(
            totem,
            event_queue,
            types,

            entity,
            table_name,
            action_name,

            Some(vars),
            0,
            true,
        );
    }
}

pub enum AlgorithmResult {
    ExternContinuation {
        entity: data::Entity,
        table_name: String,
        action_name: String,
        vars: Dict<data::Field>,
    },
    ContinueLoop {
        vars: Dict<data::Field>,
    },
    ReturnVals(Vec<data::Field>),
}

pub fn execute_algorithm(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    mut vars: Dict<data::Field>,
    mut pc: usize,
    mut has_state: bool,
) -> AlgorithmResult {
    let mut result = None;

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

    while pc < code.len() && result.is_none() {
        match code[pc] {
            Statement::Debug(ref to_print) => {
                println!("Debug: {}", to_print);
            },
            Statement::ExecEntity {
                ref ent_name,
                action_name: ref new_action_name,
                ref args,
            } => {
                pc += 1;
                if let Some(&Statement::Wait(time)) = code.get(pc) {
                    wait(
                        totem,
                        event_queue,

                        &entity,

                        table_name,
                        action_name,
                        pc,

                        time,
                    );
                }

                let new_entity = vars[ent_name].entity().clone();

                let new_vars = extract(&mut vars, args);

                result = Some(AlgorithmResult::ExternContinuation {
                    entity: new_entity.data,
                    table_name: new_entity.table,
                    action_name: new_action_name.clone(),
                    vars: new_vars,
                });

                break;
            },
            Statement::Assign {
                ref results,
                ref expressions,
            } => {
                let result_vals = evaluate_expressions(
                    totem,
                    event_queue,
                    types,

                    expressions,
                    &mut vars,
                );
                for (name, val) in results.iter().zip(result_vals) {
                    vars.insert(name.clone(), val);
                }
            },
            Statement::State {
                ref name,
                ref terms,
            } => {
                if has_state {
                    panic!("Tried to overwrite state without cancelling");
                }

                let entity = entity.borrow_mut(totem);

                entity.state_name = name.clone();
                entity.data = extract(&mut vars, terms);

                has_state = true;
            }

            Statement::Wait(time) => {
                wait(
                    totem,
                    event_queue,

                    &entity,

                    table_name,
                    action_name,
                    pc,

                    time,
                );

                break;
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

            Statement::SetAdd { ref set_name, ref to_add } => {
                let mut vals = evaluate_expression(
                    totem,
                    event_queue,
                    types,

                    to_add,
                    &mut vars,
                );
                let ent = vals.remove(0);
                let key = data::EntityKey(ent.unwrap_entity());
                vars.get_mut(set_name)
                    .expect("Set not found")
                    .set()
                    .insert(key, ());
            },
            Statement::SetRemove { ref set_name, ref to_remove } => {
                let mut vals = evaluate_expression(
                    totem,
                    event_queue,
                    types,

                    to_remove,
                    &mut vars,
                );
                let ent = vals.remove(0);
                let key = data::EntityKey(ent.unwrap_entity());
                vars.get_mut(set_name)
                    .expect("Set not found")
                    .set()
                    .remove(&key);
            },
            Statement::SetIterate {
                ref set_name,
                ref var_name,
                break_line,
            } => {
                let set = vars.remove(set_name).expect("no set").unwrap_set();
                for (ent, ()) in &set {
                    let val = data::Field::Entity(ent.0.clone());
                    vars.insert(var_name.clone(), val);

                    let result = execute_algorithm(
                        totem,
                        event_queue,
                        types,

                        entity.clone(),
                        table_name.clone(),
                        action_name.clone(),

                        vars,
                        pc + 1,
                        has_state,
                    );

                    vars = {
                        if let AlgorithmResult::ContinueLoop {
                            vars
                        } = result {
                            vars
                        } else {
                            panic!("Loop leakage");
                        }
                    };
                }

                vars.insert(set_name.clone(), data::Field::Set(set));

                pc = break_line;
            },
            Statement::Continue => {
                result = Some(AlgorithmResult::ContinueLoop {
                    vars
                });
                break;
            },
        }

        pc += 1;
    }

    if has_state {
        if pc != code.len() {
            println!("Warning: Code after external call will not be executed
without creating a new entity state");
        }
    } else {
        panic!("Tried to exit without resetting state");
    }

    result.unwrap_or_else(|| AlgorithmResult::ReturnVals(Vec::new()))
}

fn wait(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,

    entity_: &data::Entity,

    table_name: String,
    action_name: String,
    mut pc: usize,

    time: Time,
) {
    let entity = Strong::clone(entity_);
    pc += 1;

    let event = event::Event { entity, table_name, action_name, pc };

    let absolute_time = event_queue.now() + time;
    let id = event_queue.enqueue_absolute(event, absolute_time);

    let entity = entity_.borrow_mut(totem);

    entity.event = Some(event::EventHandle(absolute_time, id));
}

fn evaluate_expression(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    expression: &Expression,
    vars: &mut data::Data,
) -> Vec<data::Field> {
    let mut result = Vec::new();

    evaluate_expression_into(
        totem,
        event_queue, 
        types,

        expression,
        vars,
        &mut result,
    );

    result
}
fn evaluate_expressions(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    expressions: &Vec<Expression>,
    vars: &mut data::Data,
) -> Vec<data::Field> {
    let mut result = Vec::new();

    for expression in expressions {
        evaluate_expression_into(
            totem,
            event_queue, 
            types,

            expression,
            vars,
            &mut result,
        );
    }

    result
}

fn evaluate_expression_into(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,
    types: &Dict<item::EntityType>,

    expression: &Expression,
    vars: &mut data::Data,
    result: &mut Vec<data::Field>,
) {
    use self::Expression::*;
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

                args,
                vars,
            );
            let result_ref = execute_init(
                totem,
                event_queue,
                types,

                type_name.clone(),
                table_name.clone(),
                init_name.clone(),
                args
            );

            let result_term = data::Field::Entity(result_ref);
            result.push(result_term);
        },
    }
}
