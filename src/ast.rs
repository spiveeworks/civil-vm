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
    State(Expression),
    // TODO just overwrite state instead of explicitly cancelling?
    CancelWait,
    WhileLoop {
        condition: Expression,
        block: Vec<Statement>,
    },
    Branch {
        if_branches: Vec<(Expression, Vec<Statement>)>,
        else_branch: Vec<Statement>,
    },
    Match {
        data: Expression,
        arms: Vec<(String, Vec<String>, Vec<Statement>)>,
        def: Vec<Statement>,
    },
}

#[derive(Clone)]
pub enum Expression {
    Var(String),
    Method {
        names: Vec<String>,
        args: Vec<Expression>,
    },
    SelfObject,
    SelfData,
    Data {
        name: String,
        fields: Vec<(String, Expression)>,
    },

    Const(f64),
    Comparison(Box<Expression>, Vec<(CompareOp, Expression)>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
}

#[derive(Clone)]
pub enum CompareOp {
    Equals,
    NEquals,
    LessEq,
    GreaterEq,
    Greater,
    Less,
}

pub fn convert_algorithm(alg: Algorithm) -> runtime::Algorithm {
    let param_list = alg.param_list;
    let steps = convert_statements(alg.steps);
    runtime::Algorithm { param_list, steps }
}

fn convert_statements(steps: Vec<Statement>) -> Vec<runtime::Statement> {
    let mut result = Vec::new();
    for x in steps.into_iter() {
        convert_statement(x, &mut result);
    }
    result
}

fn convert_statement(step: Statement, result: &mut Vec<runtime::Statement>) {
    use self::Statement::*;
    let converted = match step {
        Bang => runtime::Statement::Debug("BANG".into()),
        Evaluate { mut expressions, results } => {
            if results.len() == 0 {
                if expressions.len() == 1 {
                    use self::Expression::Method;
                    if let Method { names, args } = &mut expressions[0] {
                        let converted = convert_simple_statement(names, args);
                        if let Some(converted) = converted {
                            result.push(converted);
                            return;
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
        State(state) => runtime::Statement::State(convert_expression(state)),
        CancelWait => runtime::Statement::CancelWait,
        WhileLoop {
            condition,
            block,
        } => {
            // could extend, insert, push to avoid unnecessary heap allocs
            // might not be faster tho
            let condition = convert_expression(condition);
            let block = convert_statements(block);
            let block_len = block.len();
            let break_offset = block_len + 2;
            result.push(runtime::Statement::Branch {
                condition,
                break_offset,
            });
            result.extend(block);
            result.push(runtime::Statement::Continue(block_len + 1));
            return;
        },
        Branch { mut if_branches, else_branch } => {
            let mut rest = convert_statements(else_branch);
            let mut blocks = Vec::with_capacity(if_branches.len() + 1);
            for (cond, block) in if_branches {
                let condition = convert_expression(cond);
                let mut block = convert_statements(block);

                // we could also store the conditions and put a noop into the
                // blocks... but its going to branch when it drops so we might
                // as well branch to check anyway
                let statement = runtime::Statement::Branch {
                    condition,
                    break_offset: 0,
                };
                block.insert(0, statement);

                blocks.push(block);
            }
            blocks.push(rest);
            let (mut codes, offsets) = link_blocks(blocks, 0);
            for i in 0..offsets.len()-2 {
                if let runtime::Statement::Branch {
                    ref mut break_offset,
                    ..
                } = &mut codes[offsets[i]] {
                    *break_offset = offsets[i+1] - offsets[i];
                } else {
                    unreachable!();
                }
            }
            result.extend(codes);
            return;
        },
        Match { data, arms, def } => {
            let arms_len = arms.len();
            let mut blocks = Vec::with_capacity(arms_len);
            let mut new_arms = Dict::with_capacity(arms_len);
            for (i, (variant, fields, block)) in arms.into_iter().enumerate() {
                let block = convert_statements(block);
                blocks.push(block);
                new_arms.insert(variant, (fields, i));
            }
            let def = convert_statements(def);
            blocks.push(def);

            let (codes, offsets) = link_blocks(blocks, 1);
            for (_, (_, ref mut i)) in &mut new_arms {
                let index = *i;
                *i = offsets[index];
            }

            // offsets = [branch, branch, ..., default, continue]
            // so the number of branches is the index of the default
            let default_offset = offsets[arms_len];

            let data = convert_expression(data);
            let statement = runtime::Statement::PatternBranch {
                data,
                arms: new_arms,
                default_offset,
            };
            result.push(statement);
            result.extend(codes);
            return;
        },
    };
    result.push(converted);
}

fn link_blocks(
    blocks: Vec<Vec<runtime::Statement>>,
    initial_offset: usize,
) -> (Vec<runtime::Statement>, Vec<usize>) {
    let mut offset = initial_offset;
    let mut offsets = Vec::with_capacity(blocks.len() + 1);
    for block in &blocks {
        offsets.push(offset);
        offset += block.len() + 1;
    }
    offsets.push(offset);
    let mut result = Vec::with_capacity(offset - initial_offset);
    for (i, block) in blocks.into_iter().enumerate() {
        let next_offset = offsets[i+1];
        result.extend(block);
        result.push(runtime::Statement::Jump(offset - next_offset + 1));
    }
    (result, offsets)
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
        Var(name) => runtime::Expression::Var(name),
        Method {
            names,
            args,
        } => {
            let args = convert_expressions(args);
            if names.len() == 1 {
                assert!(args.len() == 1,
                    "Object virtualization expects 1 argument");
                runtime::Expression::VirtualizeObject {
                    interface_name: names[0].clone(),
                    object: Box::new({args}.pop().unwrap()),
                }
            } else if names.len() == 2 {
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
        SelfObject => runtime::Expression::SelfObject,
        SelfData => runtime::Expression::SelfData,
        Data { name, fields } => {
            let fields = fields
                .into_iter()
                .map(|(name, val)| (name, convert_expression(val)))
                .collect();
            runtime::Expression::Data { name, fields }
        },

        Const(f64) => runtime::Expression::Const(f64),
        Comparison(l, r) => runtime::Expression::Comparison(
            box_convert(l),
            r.into_iter()
             .map(|(c, r)|(c, convert_expression(r)))
             .collect(),
        ),
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
