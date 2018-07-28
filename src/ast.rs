use prelude::*;

use runtime;

pub struct Algorithm {
    pub param_list: Vec<String>,
    pub steps: Vec<Statement>,
}

pub type TablePath = runtime::TablePath;

#[derive(Clone)]
pub enum Statement {
    Bang,
    Evaluate {
        expressions: Vec<Expression>,
        results: Vec<String>,
    },
    // self = Data {} once data expressions exist?
    State {
        name: String,
        terms: Dict<String>,
    },
    // TODO just overwrite state instead of explicitly cancelling?
    CancelWait,
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
    Method {
        names: Vec<String>,
        args: Vec<Expression>,
    },

    Const(f64),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
}

pub fn convert_algorithm(alg: Algorithm) -> runtime::Algorithm {
    let Algorithm { param_list, steps } = alg;
    let steps = steps.into_iter().map(convert_statement).collect();
    runtime::Algorithm { param_list, steps }
}

fn convert_statement(step: Statement) -> runtime::Statement {
    use self::Statement::*;
    match step {
        Bang => runtime::Statement::Debug("BANG".into()),
        Evaluate { mut expressions, results } => {
            if results.len() == 0 {
                if expressions.len() == 1 {
                    use self::Expression::Method;
                    if let Method { names, args } = &mut expressions[0] {
                        let result = convert_simple_statement(names, args);
                        if let Some(result) = result {
                            return result;
                        }
                    }
                } else {
                    panic!("Separate statements with semicolon not comma");
                }
            }

            runtime::Statement::Evaluate {
                expressions: convert_expressions(expressions),
                results,
            }
        },
        State {
            name,
            terms,
        } => runtime::Statement::State {
            name,
            terms,
        },
        CancelWait => runtime::Statement::CancelWait,
        SetIterate {
            var_name,
            set_name,
            break_offset,
        } => runtime::Statement::SetIterate {
            var_name,
            set_name,
            break_offset,
        },
        Continue => runtime::Statement::Continue,
    }
}

// if this returns `None` then it didn't modify the inputs
fn convert_simple_statement(names: &mut Vec<String>, args: &mut Vec<Expression>)
    -> Option<runtime::Statement>
{
    if names.len() == 1 {
        if names[0] == "print" {
            let args = ::std::mem::replace(args, Vec::new());
            return Some(runtime::Statement::DebugNums(
                convert_expressions(args)
            ));
        } else if names[0] == "wait" {
            assert!(args.len() == 1, "wait expects 1 argument");
            let arg = args.pop().unwrap();
            return Some(runtime::Statement::Wait(convert_expression(arg)));
        }
    } else if names.len() == 2 {
        if names[1] == "add" {
            assert!(args.len() == 1, "Set.add expects 1 argument");
            let arg = args.pop().unwrap();
            return Some(runtime::Statement::SetAdd {
                set_name: names[0].clone(),
                to_add: convert_expression(arg),
            });
        } else if names[1] == "remove" {
            assert!(args.len() == 1, "Set.remove expects 1 argument");
            let arg = args.pop().unwrap();
            return Some(runtime::Statement::SetAdd {
                set_name: names[0].clone(),
                to_add: convert_expression(arg),
            });
        }
    }
    None
}

fn convert_expressions(vals: Vec<Expression>) -> Vec<runtime::Expression> {
    vals.into_iter()
        .map(convert_expression)
        .collect()
}

fn convert_expression(val: Expression) -> runtime::Expression {
    use self::Expression::*;
    match val {
        MoveVar(name) => runtime::Expression::MoveVar(name),
        CloneVar(name) => runtime::Expression::CloneVar(name),
        Method {
            names,
            args,
        } => {
            let args = convert_expressions(args);
            if names.len() == 2 {
                if names[0] == "Set" && names[1] == "new" {
                    assert!(args.len() == 0, "Set.new expects no arguments");
                    runtime::Expression::InitSet
                } else if names[0] == "game" {
                    runtime::Expression::ExternCall {
                        function_name: names[1].clone(),
                        args,
                    }
                } else {
                    runtime::Expression::ExecObject {
                        object_name: names[0].clone(),
                        action_name: names[1].clone(),
                        args,
                    }
                }
            } else if names.len() == 3 {
                runtime::Expression::InitObject {
                    type_name: names[0].clone(),
                    table_name: names[1].clone(),
                    init_name: names[2].clone(),
                    args,
                }
            } else {
                panic!("Too much stuff");
            }
        },

        Const(f64) => runtime::Expression::Const(f64),
        Add(l, r) => runtime::Expression::Add(box_convert(l), box_convert(r)),
        Sub(l, r) => runtime::Expression::Sub(box_convert(l), box_convert(r)),
        Mul(l, r) => runtime::Expression::Mul(box_convert(l), box_convert(r)),
        Div(l, r) => runtime::Expression::Div(box_convert(l), box_convert(r)),
        Pow(l, r) => runtime::Expression::Pow(box_convert(l), box_convert(r)),
    }
}

fn box_convert(val: Box<Expression>) -> Box<runtime::Expression> {
    Box::new(convert_expression(*val))
}
