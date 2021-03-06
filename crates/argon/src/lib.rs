#![feature(crate_visibility_modifier)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![deny(rust_2018_idioms)]
#![deny(unused_must_use)]
#![allow(unused_extern_crates)]

#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

pub mod compilation;
pub mod compile;
pub mod database;
pub mod debuggable;
crate mod infer;
pub mod ir;
pub mod lexer;
pub mod parser;

#[allow(warnings)]
pub mod grammar;

pub use self::compilation::Compilation;
pub use self::compile::*;
pub use self::grammar::ModuleParser;
pub use self::ir::*;
pub use self::parser::ParseError;

crate use self::infer::unify::UnifyTable;

#[cfg(test)]
crate fn init_logger() {
    #![allow(unused_must_use)]
    pretty_env_logger::try_init();
}
