use prelude::*;

use ast;
use runtime;

//
// AST
//

pub enum Item {
    Function(ast::Algorithm),
    Constructor(ast::Algorithm),
    Interface {
        type_name: Option<String>,
        role_name: String,
        implementors: Dict<String>,
    },
    Role(Role),
}

pub enum FieldType {
    Num,
    VRef { type_name: String, interface_name: String },
    TRef(String),
}

pub type Role = Dict<RoleTerm>;

pub enum RoleTerm {
    Constructor(Vec<FieldType>),
    Function(Vec<FieldType>),
}

//
// runtime
//

pub struct Interface {
    pub implementors: Dict<String>,
}

pub struct ObjectType {
    interfaces: Dict<Interface>,
    algorithms: Dict<runtime::Algorithm>,
}

pub fn collect(items: Vec<(String, Item)>) -> ObjectType {
    let mut interfaces = Dict::new();
    let mut algorithms = Dict::new();
    let mut roles = Dict::new();

    for (name, item) in items {
        match item {
            Item::Interface { type_name, role_name, implementors } => {
                // these will be useful for static analysis one day
                drop(type_name);
                drop(role_name);
                interfaces.insert(name, Interface { implementors });
            },
            Item::Function(alg) | Item::Constructor(alg) => {
                let alg = ast::convert_algorithm(alg);
                algorithms.insert(name, alg);
            },
            Item::Role(role) => {
                roles.insert(name, role);
            },
        }
    }

    ObjectType { interfaces, algorithms }
}

pub fn get_algorithm_name<'a>(
    types: &'a Dict<ObjectType>,

    object_type_name: &String,
    interface_name: &String,
    method_name: &String,
) -> &'a String {
    let object_type = &types[object_type_name];
    let interface = &object_type.interfaces[interface_name];
    &interface.implementors[method_name]
}

pub fn get_algorithm<'a>(
    types: &Dict<ObjectType>,

    object_type_name: &String,
    alg_name: &String,
) -> &'static runtime::Algorithm {
    let object_type = &types[object_type_name];
    let result: *const runtime::Algorithm = &object_type.algorithms[alg_name];

    // safe because we dont edit code at runtime
    // way better than cloning
    // if you get a segfault, try cloning code again i guess?
    // should probably stop FlopInstance from exposing these mutably
    unsafe { &*result }
}

