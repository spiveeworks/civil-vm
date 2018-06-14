use prelude::*;

use algorithm;
use table;

pub type EntityType = Dict<table::Table>;

pub type TableIdent = (String, String);

pub enum Item {
    TableTerm(table::TableTerm),
    TableInstance {
        signature: String,
        implementors: Dict<String>,
    },
    TableSignature(table::Signature),
}

pub fn link(items: Vec<(String, Item)>) -> EntityType {
    let mut table_defs = Vec::new();
    let mut algs = Dict::new();
    for (name, item) in items {
        use self::Item::*;
        match item {
            TableInstance { signature, implementors } => {
                drop(signature);  // not useful until type checking exists
                table_defs.push((name, implementors));
            },
            TableTerm(term) => {
                algs.insert(name, term);
            },
            TableSignature(_) => (),
        }
    }

    let mut tables = Dict::new();
    for (name, table_def) in table_defs {
        let mut table_terms = Dict::new();
        for (method, implementor) in table_def {
            let alg = algs.remove(&implementor)
                .expect("Undefined action");
            let table_term = alg;
            table_terms.insert(method, table_term);
        }
        let table = table::Table {
            terms: table_terms,
        };
        tables.insert(name, table);
    }
    tables
}

pub fn get_algorithm<'a>(
    types: &'a Dict<EntityType>,

    entity_type_name: &String,
    table_name: &String,
    algorithm_name: &String,
) -> &'a algorithm::Algorithm {
    let entity_type = &types[entity_type_name];
    let table = &entity_type[table_name];
    table.terms[algorithm_name].algorithm()
}

