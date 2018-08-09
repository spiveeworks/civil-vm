use prelude::*;

use interface;
use runtime;

pub type ObjectType = Dict<interface::Interface>;

pub type InterfaceIdent = (String, String);

pub enum Item {
    Function(runtime::Algorithm),
    Constructor(runtime::Algorithm),
    Interface {
        role_name: String,
        implementors: Dict<String>,
    },
    Role(interface::Role),
}

pub fn link(items: Vec<(String, Item)>) -> ObjectType {
    let mut table_defs = Vec::new();
    let mut algs = Dict::new();
    let mut roles = Dict::new();
    for (name, item) in items {
        use self::Item::*;
        match item {
            Interface { role_name, implementors } => {
                drop(role_name);  // will be useful for static analysis though
                table_defs.push((name, implementors));
            },
            Function(term) => {
                algs.insert(name, term);
            },
            Constructor(term) => {
                algs.insert(name, term);
            },
            Role(role) => {
                roles.insert(name, role);
            },
        }
    }

    let mut tables = Dict::new();
    for (name, table_def) in table_defs {
        let mut algorithms = Dict::new();
        for (method, implementor) in table_def {
            let it = algs.remove(&implementor)
                .expect("Function not defined");
            algorithms.insert(method, it);
        }
        let table = interface::Interface { algorithms };
        tables.insert(name, table);
    }
    tables
}

pub fn get_algorithm<'a>(
    types: &'a Dict<ObjectType>,

    object_type_name: &String,
    table_name: &String,
    runtime_name: &String,
) -> &'a runtime::Algorithm {
    let object_type = &types[object_type_name];
    let table = &object_type[table_name];
    &table.algorithms[runtime_name]
}

