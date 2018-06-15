use std::mem;

use prelude::*;

use data;
use event;
use item;

pub struct Algorithm {
    pub param_list: Vec<String>,
    pub steps: Vec<Statement>,
}

#[derive(Clone)]
pub enum TablePath {
    Virtual(String),
    Static(String, String),
}

#[derive(Clone)]
pub enum Statement {
    Debug(String),
    GotoAlg {
        table: TablePath,
        alg_name: String,
        args: Vec<Expression>,
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
        var_name: String,
        set_name: String,
        break_offset: usize,
    },
    Continue,
}

#[derive(Clone)]
pub enum Expression {
    MoveVar(String),
    CloneVar(String),
    InitEntity {
        type_name: String,
        table_name: String,
        init_name: String,
        args: Vec<Expression>,
    },
    ExecEntity {
        entity_name: String,
        action_name: String,
        args: Vec<Expression>,
    },
    InitSet,
    ExternCall {
        function_name: String,
        args: Vec<Expression>,
    },
}

fn bind_args(
    types: &Dict<item::EntityType>,

    type_name: &String,
    table_name: &String,
    init_name: &String,

    args: Vec<data::Field>,
) -> data::Data {
    item::get_algorithm(
        types,
        type_name,
        table_name,
        init_name,
    )   .param_list
        .iter()
        .cloned()
        .zip(args)
        .collect()
}

pub fn execute_init<G: Flop>(
    game: &mut G,
    type_name: String,
    table_name: String,
    init_name: String,
    args: Vec<data::Field>,
) -> data::EntityRef {
    let args = bind_args(
        game.types(),

        &type_name,
        &table_name,
        &init_name,

        args,
    );

    let table = table_name.clone();
    let data = data::EntityData::new(type_name);

    execute_action(
        game,
        Strong::clone(&data),
        table_name,
        init_name,

        Some(args),
        0,
        false,
    );

    data::EntityRef { table, data }
}

pub fn execute_action<G: Flop>(
    game: &mut G,
    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    vars: Option<data::Data>,  // None to use entity's state
    pc: usize,
    mut has_state: bool,
) -> Vec<data::Field> {
    let vars = {
        if let Some(vars) = vars {
            vars
        } else {
            has_state = false;

            let entity = entity.borrow_mut(game.totem());
            entity.event = None;

            mem::replace(&mut entity.data, Dict::new())
        }
    };

    let result = execute_algorithm(
        game,
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
        has_state,
    } = result {
        execute_action(
            game,
            entity,
            table_name,
            action_name,

            Some(vars),
            0,
            has_state,
        )
    } else if let AlgorithmResult::ReturnVals(vals) = result {
        vals
    } else {
        Vec::new()
    }
}

pub enum AlgorithmResult {
    ExternContinuation {
        entity: data::Entity,
        table_name: String,
        action_name: String,
        vars: Dict<data::Field>,
        has_state: bool,
    },
    ContinueLoop {
        vars: Dict<data::Field>,
    },
    ReturnVals(Vec<data::Field>),
}

pub fn execute_algorithm<G: Flop>(
    game: &mut G,

    entity: Strong<data::EntityData>,
    table_name: String,
    action_name: String,

    mut vars: Dict<data::Field>,
    mut pc: usize,
    mut has_state: bool,
) -> AlgorithmResult {
    let mut result = None;

    let type_name = {
        let entity = entity.borrow(game.totem());
        entity.type_name.clone()
    };

    let code = item::get_algorithm(
        game.types(),
        &type_name,
        &table_name,
        &action_name
    ).steps.clone();

    while pc < code.len() && result.is_none() {
        match code[pc] {
            Statement::Debug(ref to_print) => {
                println!("Debug: {}", to_print);
            },
            Statement::GotoAlg {
                ref table,
                alg_name: ref new_action_name,
                ref args,
            } => {
                pc += 1;
                if let Some(&Statement::Wait(time)) = code.get(pc) {
                    let (totem, _, event_queue) = game.parts();
                    wait(
                        totem,
                        event_queue,

                        &entity,

                        table_name.clone(),
                        action_name,
                        pc,

                        time,
                    );
                }

                let (new_entity, new_table_name, is_initalizer) = match table {
                    TablePath::Virtual(ref ent_name) => {
                        let ent_ref = vars[ent_name].entity().clone();
                        (ent_ref.data, ent_ref.table, false)
                    },
                    TablePath::Static(ref type_name, ref table_name) => {
                        let ent = data::EntityData::new(type_name.clone());
                        (ent, table_name.clone(), true)
                    },
                };

                let vals = evaluate_expressions(
                    game,
                    args,
                    &mut vars,
                );

                let (totem, types, _) = game.parts();
                let new_vars = bind_args(
                    types,

                    &new_entity.borrow(totem).type_name,
                    &new_table_name,
                    &new_action_name,

                    vals,
                );

                result = Some(AlgorithmResult::ExternContinuation {
                    entity: new_entity,
                    table_name: new_table_name,
                    action_name: new_action_name.clone(),
                    vars: new_vars,
                    has_state: !is_initalizer,
                });

                break;
            },
            Statement::Assign {
                ref results,
                ref expressions,
            } => {
                let result_vals = evaluate_expressions(
                    game,
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

                let entity = entity.borrow_mut(game.totem());

                entity.state_name = name.clone();
                entity.data = extract(&mut vars, terms);

                has_state = true;
            }

            Statement::Wait(time) => {
                let (totem, _, event_queue) = game.parts();
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
                let entity = entity.borrow_mut(game.totem());
                let event = entity.event.take();
                if let Some(event::EventHandle(ref time, id)) = event {
                    game.event_queue().cancel_event(time, id);
                }

                entity.data = Dict::new();

                has_state = false;
            },

            Statement::SetAdd { ref set_name, ref to_add } => {
                let mut vals = evaluate_expression(
                    game,
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
                    game,
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
                break_offset,
            } => {
                let break_line = pc + break_offset;

                let set = vars.remove(set_name).expect("no set").unwrap_set();
                for (ent, ()) in &set {
                    let val = data::Field::Entity(ent.0.clone());
                    vars.insert(var_name.clone(), val);

                    let result = execute_algorithm(
                        game,

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

    if !has_state {
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

fn evaluate_expression<G: Flop>(
    game: &mut G,
    expression: &Expression,
    vars: &mut data::Data,
) -> Vec<data::Field> {
    let mut result = Vec::new();

    evaluate_expression_into(
        game,
        expression,
        vars,
        &mut result,
    );

    result
}

fn evaluate_expressions<G: Flop>(
    game: &mut G,
    expressions: &Vec<Expression>,
    vars: &mut data::Data,
) -> Vec<data::Field> {
    let mut result = Vec::new();

    for expression in expressions {
        evaluate_expression_into(
            game,
            expression,
            vars,
            &mut result,
        );
    }

    result
}

fn evaluate_expression_into<G: Flop>(
    game: &mut G,
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
                game,
                args,
                vars,
            );
            let result_ref = execute_init(
                game,
                type_name.clone(),
                table_name.clone(),
                init_name.clone(),
                args
            );

            let result_term = data::Field::Entity(result_ref);
            result.push(result_term);
        },
        ExecEntity {
            ref entity_name,
            ref action_name,
            ref args,
        } => {
            let entity = vars[entity_name].clone().unwrap_entity();
            let args = evaluate_expressions(
                game,
                args,
                vars,
            );
            let args = {
                let type_name = &entity.data.borrow(game.totem()).type_name;
                bind_args(
                    game.types(),

                    type_name,
                    &entity.table,
                    action_name,

                    args,
                )
            };
            let result_vals = execute_action(
                game,
                entity.data,
                entity.table,
                action_name.clone(),
                Some(args),
                0,
                true,
            );

            result.extend(result_vals);
        },
        InitSet => {
            result.push(data::Field::Set(data::EntitySet::new()));
        },

        ExternCall { ref function_name, ref args } => {
            let args = evaluate_expressions(
                game,
                args,
                vars,
            );

            let results = game.extern_call(function_name, args);

            result.extend(results);
        },
    }
}
