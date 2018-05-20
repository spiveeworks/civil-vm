use prelude::*;

use algorithm;
use table;

pub type EntityType = Dict<table::Table>;

// for the parser >:P
pub enum Item {
    Action(algorithm::Action),
    Table(Vec<String>),
}

pub fn link(items: Vec<(String, Item)>) -> EntityType {
    let mut table_defs = Vec::new();
    let mut actions = Dict::new();
    for (name, item) in items {
        use self::Item::*;
        match item {
            Table(table_def) => {
                table_defs.push((name, table_def));
            },
            Action(action) => {
                actions.insert(name, action);
            },
        }
    }

    let mut tables = Dict::new();
    for (name, table_def) in table_defs {
        let mut table_terms = Dict::new();
        for name in table_def {
            let action = actions.remove(&name)
                .expect("Undefined action");
            let table_term = table::TableTerm::Action(action);
            table_terms.insert(name, table_term);
        }
        let table = table::Table {
            terms: table_terms,
        };
        tables.insert(name, table);
    }
    tables
}
