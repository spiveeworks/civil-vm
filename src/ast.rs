use prelude::*;

use runtime;

pub struct Algorithm {
    pub param_list: Vec<String>,
    pub steps: Vec<Statement>,
}

pub type TablePath = runtime::TablePath;

#[derive(Clone)]
pub enum Statement {
    Debug(String),
    DebugNums(Vec<Expression>),
    GotoAlg {
        table: TablePath,
        alg_name: String,
        args: Vec<Expression>,
    },
    Assign {
        expressions: Vec<Expression>,
        results: Vec<String>,
    },
    State {
        name: String,
        terms: Dict<String>,
    },
    Wait(Expression),
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
        Debug(msg) => runtime::Statement::Debug(msg),
        DebugNums(nums) => runtime::Statement::DebugNums(
            convert_expressions(nums)
        ),
        GotoAlg {
            table,
            alg_name,
            args,
        } => runtime::Statement::GotoAlg {
            table,
            alg_name,
            args: convert_expressions(args),
        },
        Assign {
            expressions,
            results,
        } => runtime::Statement::Assign {
            expressions: convert_expressions(expressions),
            results,
        },
        State {
            name,
            terms,
        } => runtime::Statement::State {
            name,
            terms,
        },
        Wait(expr) => runtime::Statement::Wait(convert_expression(expr)),
        CancelWait => runtime::Statement::CancelWait,
        SetAdd {
            set_name,
            to_add,
        } => runtime::Statement::SetAdd {
            set_name,
            to_add: convert_expression(to_add),
        },
        SetRemove {
            set_name,
            to_remove,
        } => runtime::Statement::SetRemove {
            set_name,
            to_remove: convert_expression(to_remove),
        },
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
        InitObject {
            type_name,
            table_name,
            init_name,
            args,
        } => runtime::Expression::InitObject {
            type_name,
            table_name,
            init_name,
            args: convert_expressions(args),
        },
        ExecObject {
            object_name,
            action_name,
            args,
        } => runtime::Expression::ExecObject {
            object_name,
            action_name,
            args: convert_expressions(args),
        },
        InitSet => runtime::Expression::InitSet,
        ExternCall {
            function_name,
            args,
        } => runtime::Expression::ExternCall {
            function_name,
            args: convert_expressions(args),
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
