#![allow(unused)]
#![warn(unused_imports)]

use super::code_table::CodeTable;
use code_database::{AbsolutePath, FileTable, FileTrait, TransactionId};
use crate::ir::ast::Module;
use crate::parser::parse;
use failure::Error;
use std::collections::btree_map::Entry as BTreeEntry;
use std::collections::BTreeMap;

#[derive(Debug)]
crate struct Entry {
    module: Module<'static>,
    last_revision: usize,
}

crate struct AstTable {
    index: BTreeMap<AbsolutePath, Entry>,
}

impl AstTable {
    crate fn new() -> AstTable {
        AstTable {
            index: BTreeMap::new(),
        }
    }

    crate fn get_revision(
        &self,
        code_table: &mut CodeTable,
        file_table: &mut FileTable<impl FileTrait>,
        key: &AbsolutePath,
    ) -> Option<usize> {
        code_table.get_revision(file_table, key)
    }

    crate fn get(
        &mut self,
        file_table: &mut FileTable<impl FileTrait>,
        code_table: &mut CodeTable,
        key: &AbsolutePath,
        transaction: TransactionId,
    ) -> Result<Option<&Module<'static>>, Error> {
        self.refresh_cache(key, file_table, code_table, transaction)?;

        match self.index.get(key) {
            None => Ok(None),
            Some(entry) => Ok(Some(&entry.module)),
        }
    }

    fn refresh_cache(
        &mut self,
        key: &AbsolutePath,
        file_table: &mut FileTable<impl FileTrait>,
        code_table: &mut CodeTable,
        transaction: TransactionId,
    ) -> Result<(), Error> {
        match self.index.entry(key.clone()) {
            BTreeEntry::Vacant(vacant) => {
                fill_cache(
                    BTreeEntry::Vacant(vacant),
                    file_table,
                    code_table,
                    transaction,
                )?;
            }
            BTreeEntry::Occupied(occupied) => {
                let last_revision = {
                    let entry = occupied.get();
                    entry.last_revision
                };

                if !code_table.is_valid(file_table, key, last_revision) {
                    fill_cache(
                        BTreeEntry::Occupied(occupied),
                        file_table,
                        code_table,
                        transaction,
                    )?;
                }
            }
        }

        Ok(())
    }
}

fn fill_cache(
    entry: BTreeEntry<AbsolutePath, Entry>,
    file_table: &mut FileTable<impl FileTrait>,
    code_table: &mut CodeTable,
    transaction: TransactionId,
) -> Result<(), Error> {
    println!("Filling cache {:?}", entry);

    let key = entry.key().clone();
    let file = match code_table.get(file_table, entry.key(), transaction)? {
        None => return Ok(()),
        Some(file) => file,
    }.clone();

    let src = file.src().to_string();
    let parsed = parse(&src)?;
    let parsed = parsed.into_owned();

    let new_entry = Entry {
        module: parsed,
        last_revision: 0,
    };

    println!("Inserting into {:?}", entry);

    match entry {
        BTreeEntry::Occupied(mut occupied) => {
            occupied.insert(new_entry);
        }
        BTreeEntry::Vacant(vacant) => {
            vacant.insert(new_entry);
        }
    };

    Ok(())
}