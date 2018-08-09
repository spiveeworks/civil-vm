use std::mem;

use prelude::*;

use ast;
use data;
use event;
use item;

use instance::FlopParts;

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
    DebugNums(Vec<Expression>),
    // TODO self.method() and Type.initializer stuff
    Evaluate {
        // multiple expressions all so that x, y = y, x is possible :P
        expressions: Vec<Expression>,
        results: Vec<String>,
    },
    State(Expression),
    Wait(Expression),
    Return(Vec<Expression>),
    Branch {
        condition: Expression,
        break_offset: usize,
    },
    PatternBranch {
        data: Expression,
        arms: Dict<(Vec<String>, usize)>,
        default_offset: usize,
    },
    Continue(usize),
    Jump(usize),
}

#[derive(Clone)]
pub enum Expression {
    Var(String),
    InitObject {
        type_name: String,
        table_name: String,
        init_name: String,
        args: Vec<Expression>,
    },
    ExecObject {
        object_name: String,
        action_name: String,
        args: Vec<Expression>,
    },
    InitSet,
    ExternCall {
        function_name: String,
        args: Vec<Expression>,
    },
    VirtualizeObject {
        interface_name: String,
        object: Box<Expression>,
    },
    SelfObject,
    SelfData,

    Data {
        name: String,
        fields: Dict<Expression>,
    },

    Const(f64),
    Comparison(Box<Expression>, Vec<(ast::CompareOp, Expression)>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
}

impl Algorithm {
    fn bind_args(self: &Self, args: Vec<data::Field>) -> data::Data {
        self.param_list
            .iter()
            .cloned()
            .zip(args)
            .collect()
    }
}

/// Executes a constructor on a fresh object
pub fn execute_ctor<G: Flop>(
    game: &mut G,
    type_name: String,
    init_name: String,
    args: Vec<data::Field>,
) -> data::Object {
    let tref = data::ObjectData::new(type_name);

    execute_algorithm(
        game,
        Strong::clone(&tref),
        init_name,

        ExecType::Ctor(args),
    );

    tref
}


pub enum ExecType {
    Ctor(Vec<data::Field>),
    Fun(Vec<data::Field>),
    Resume(usize),
}

pub fn execute_algorithm<G: Flop>(
    game: &mut G,

    object: Strong<data::ObjectData>,
    algorithm_name: String,

    input: ExecType,
) -> Vec<data::Field> {
    let mut result = None;
    let mut pc = 0;
    let mut has_state = false;
    let mut vars;

    let type_name = {
        let object = object.borrow(game.totem());
        object.type_name.clone()
    };


    let alg = &*item::get_algorithm(
        game.types(),
        &type_name,
        &algorithm_name
    );

    if let ExecType::Fun(_) = input { has_state = true; }
    match input {
        ExecType::Fun(args) | ExecType::Ctor(args) => {
            vars = alg.bind_args(args);
        },
        ExecType::Resume(pc_) => {
            pc = pc_;
            vars = {
                let object = object.borrow_mut(game.totem());
                object.event.take().unwrap();

                mem::replace(&mut object.data, Dict::new())
            };
        },
    }

    let code = &alg.steps;
    while pc < code.len() {
        if let Statement::Wait(time) = &code[pc] {
            let time_ = evaluate_expression(
                game,
                time,
                &mut vars,
                &object,
            ).num();
            let time = Time::try_from(time_)
                .expect("Num Error");
            let (totem, _, event_queue) = game.parts();
            wait(
                totem,
                event_queue,

                &object,

                algorithm_name,
                pc,

                time,
            );

            break;
        } else if result.is_some() {
            break;
        } else { match code[pc] {
            Statement::Wait(_) => {
                unreachable!();
            },
            Statement::Debug(ref to_print) => {
                println!("Debug: {}", to_print);
            },
            Statement::DebugNums(ref exprs) => {
                let mut result = evaluate_expressions(
                    game,
                    exprs,
                    &mut vars,
                    &object,
                ).into_iter();
                print!("Debug: {}", result.next().unwrap().num());
                for x in result {
                    print!(", {}", x.num());
                }
                println!("");
            },
            Statement::Evaluate {
                ref results,
                ref expressions,
            } => {
                let result_vals = evaluate_expressions(
                    game,
                    expressions,
                    &mut vars,
                    &object,
                );
                for (name, val) in results.iter().zip(result_vals) {
                    vars.insert(name.clone(), val);
                }
            },
            Statement::State(ref state) => {
                let (state_name, data) = evaluate_expression(
                    game,
                    state,
                    &mut vars,
                    &object,
                ).unwrap_data();

                let object = object.borrow_mut(game.totem());

                let event = object.event.take();
                if let Some(event::EventHandle(ref time, id)) = event {
                    game.event_queue().cancel_event(time, id);
                }

                object.state_name = state_name;
                object.data = data;

                has_state = true;
            }

            Statement::Return(ref vals) => {
                let vals = evaluate_expressions(
                    game,
                    vals,
                    &mut vars,
                    &object,
                );
                result = Some(vals);
            },

            Statement::Branch {
                ref condition,
                break_offset,
            } => {
                let mut condition = evaluate_expression(
                    game,
                    condition,
                    &mut vars,
                    &object,
                ).bool();
                if !condition {
                    pc += break_offset;
                    continue;
                }
            },
            Statement::PatternBranch {
                ref data,
                ref arms,
                default_offset,
            } => {
                let (name, mut field_vals) = evaluate_expression(
                    game,
                    data,
                    &mut vars,
                    &object,
                ).unwrap_data();
                let mut offset = default_offset;
                if let Some((fields, this_offset)) = arms.get(&name) {
                    for field in fields {
                        let val = field_vals.remove(field)
                            .expect("field not found for match arm");
                        vars.insert(field.clone(), val);
                    }
                    offset = *this_offset;
                }
                pc += offset;
                continue;
            },
            Statement::Continue(break_offset) => {
                pc -= break_offset;
                continue;
            },
            Statement::Jump(break_offset) => {
                pc += break_offset;
                continue;
            },
        }}

        pc += 1;
    }

    if !has_state {
        panic!("Tried to exit without resetting state");
    }

    result.unwrap_or_else(|| Vec::new())
}

fn wait(
    totem: &mut Totem,
    event_queue: &mut event::EventQueue,

    object_: &data::Object,

    action_name: String,
    mut pc: usize,

    time: Time,
) {
    let object = Strong::clone(object_);
    pc += 1;

    let event = event::Event { object, action_name, pc };

    let absolute_time = event_queue.now() + time;
    let id = event_queue.enqueue_absolute(event, absolute_time);

    let object = object_.borrow_mut(totem);

    object.event = Some(event::EventHandle(absolute_time, id));
}

fn evaluate_expression<G: Flop>(
    game: &mut G,
    expression: &Expression,
    vars: &mut data::Data,
    object: &data::Object,
) -> data::Field {
    let mut result = Vec::new();

    evaluate_expression_into(
        game,
        expression,
        vars,
        object,
        &mut result,
    );

    assert!(result.len() == 1, "Expected single result");
    result.pop().unwrap()
}

fn evaluate_expressions<G: Flop>(
    game: &mut G,
    expressions: &Vec<Expression>,
    vars: &mut data::Data,
    object: &data::Object,
) -> Vec<data::Field> {
    let mut result = Vec::new();

    for expression in expressions {
        evaluate_expression_into(
            game,
            expression,
            vars,
            object,
            &mut result,
        );
    }

    result
}

fn evaluate_expression_into<G: Flop>(
    game: &mut G,
    expression: &Expression,
    vars: &mut data::Data,
    object: &data::Object,
    result: &mut Vec<data::Field>,
) {
    use self::Expression::*;
    match *expression {
        Var(ref name) => {
            let val = vars[name].clone();
            result.push(val);
        },
        InitObject {
            ref type_name,
            ref table_name,
            ref init_name,
            ref args,
        } => {
            let args = evaluate_expressions(
                game,
                args,
                vars,
                object,
            );
            let alg_name = item::get_algorithm_name(
                game.types(),
                type_name,
                table_name,
                init_name,
            ).clone();
            let tref = execute_ctor(
                game,
                type_name.clone(),
                alg_name,
                args
            );

            let vref = data::ObjectRef {
                data: tref,
                table: table_name.clone(),
            };
            let result_term = data::Field::VRef(vref);
            result.push(result_term);
        },
        ExecObject {
            ref object_name,
            ref action_name,
            ref args,
        } => {
            let mut args = evaluate_expressions(
                game,
                args,
                vars,
                &object,
            );

            if !vars.contains_key(object_name) {
                let type_name = object_name;

                let tref = execute_ctor(
                    game,
                    type_name.clone(),
                    action_name.clone(),
                    args
                );

                let result_term = data::Field::TRef(tref);
                result.push(result_term);
                return;
            }
            use data::Field::*;
            if let Set(x) = vars.get_mut(object_name).unwrap() {
                if action_name == "add" {
                    assert!(args.len() == 1, "Set.add expects one arg");
                    let key = args.pop().unwrap().unwrap_vref();
                    x.insert(data::ObjectKey(key), ());
                } else if action_name == "remove" {
                    assert!(args.len() == 1, "Set.remove expects one arg");
                    let key = args.pop().unwrap().unwrap_vref();
                    x.remove(&data::ObjectKey(key));
                } else if action_name == "next" {
                    assert!(args.len() == 0, "Set.next expects no args");
                    let (val, ()) = data::set_pop(x)
                        .expect("Cannot remove from empty set");
                    result.push(data::Field::VRef(val));
                } else if action_name == "not_empty" {
                    // TODO !set.is_empty()
                    assert!(args.len() == 0, "Set.not_empty expects no args");
                    result.push(data::Field::from_bool(!x.is_empty()));
                }
                // return so that we can continue in a scope where vars still
                // exists
                return;
            }


            let tref;
            let alg_name;
            match vars[object_name].clone() {
                TRef(tref_) => {
                    tref = tref_;
                    alg_name = action_name.clone();
                },
                VRef(vref) => {
                    tref = vref.data;

                    let (totem, types, _) = game.parts();
                    let type_name = &tref.borrow(totem).type_name;
                    let interface_name = vref.table;
                    alg_name = item::get_algorithm_name(
                        types,
                        type_name,
                        &interface_name,
                        action_name,
                    ).clone();
                },
                _ => {
                    panic!("Method called on simple data");
                },
            }

            let result_vals = execute_algorithm(
                game,
                tref,
                alg_name,
                ExecType::Fun(args),
            );

            result.extend(result_vals);
        },
        InitSet => {
            result.push(data::Field::Set(data::ObjectSet::new()));
        },

        ExternCall { ref function_name, ref args } => {
            let args = evaluate_expressions(
                game,
                args,
                vars,
                object,
            );

            let results = game.extern_call(function_name, args);

            result.extend(results);
        },

        VirtualizeObject { ref interface_name, object: ref tref_expr } => {
            let mut tref = evaluate_expression(
                game,
                tref_expr,
                vars,
                &object,
            );
            let table = interface_name.clone();
            let data = tref.unwrap_tref();
            let vref = data::ObjectRef { table, data };
            result.push(data::Field::VRef(vref));
        },

        SelfObject => {
            result.push(data::Field::TRef(Strong::clone(object)));
        },
        SelfData => {
            let obj = object.borrow(&game.totem());
            let name = obj.state_name.clone();
            let data = obj.data.clone();
            result.push(data::Field::Data(name, data));
        },

        Data { ref name, ref fields } => {
            // TODO make this another kind of eval function?
            // might make errors even worse
            let mut eval = |expr|
                evaluate_expression(
                    game,
                    expr,
                    vars,
                    object,
                );
            let data = fields
                 .iter()
                 .map(|(fname, val)| (fname.clone(), eval(val)))
                 .collect();
            result.push(data::Field::Data(name.clone(), data));
        },

        Const(x) => {
            result.push(data::Field::Num(x));
        },
        Comparison(ref x, ref ys) => {
            let mut x = evaluate_expression(
                game,
                &**x,
                vars,
                object,
            ).num();
            for (ref op, ref y) in ys {
                let y = evaluate_expression(
                    game,
                    y,
                    vars,
                    object,
                ).num();
                use ast::CompareOp::*;
                let succeeded = match op {
                    Equals => x == y,
                    NEquals => x != y,
                    LessEq => x <= y,
                    GreaterEq => x >= y,
                    Less => x < y,
                    Greater => x > y,
                };
                if !succeeded {
                    result.push(data::Field::from_bool(false));
                    return;
                }
                x = y;
            }
            result.push(data::Field::from_bool(true));
        },
        Add(ref x, ref y) => {
            let mut x = evaluate_expression(
                game,
                &**x,
                vars,
                object,
            ).num();
            let mut y = evaluate_expression(
                game,
                &**y,
                vars,
                object,
            ).num();
            result.push(data::Field::Num(x + y));
        },
        Sub(ref x, ref y) => {
            let mut x = evaluate_expression(
                game,
                &**x,
                vars,
                object,
            ).num();
            let mut y = evaluate_expression(
                game,
                &**y,
                vars,
                object,
            ).num();

            result.push(data::Field::Num(x - y));
        },
        Mul(ref x, ref y) => {
            let mut x = evaluate_expression(
                game,
                &**x,
                vars,
                object,
            ).num();
            let mut y = evaluate_expression(
                game,
                &**y,
                vars,
                object,
            ).num();

            result.push(data::Field::Num(x * y));
        },
        Div(ref x, ref y) => {
            let mut x = evaluate_expression(
                game,
                &**x,
                vars,
                object,
            ).num();
            let mut y = evaluate_expression(
                game,
                &**y,
                vars,
                object,
            ).num();

            result.push(data::Field::Num(x / y));
        },
        Pow(ref x, ref y) => {
            let mut x = evaluate_expression(
                game,
                &**x,
                vars,
                object,
            ).num();
            let mut y = evaluate_expression(
                game,
                &**y,
                vars,
                object,
            ).num();

            result.push(data::Field::Num(x.powf(y)));
        },
    }
}
