use parity_wasm::elements;
use wasmi;
use wasm::Token;

crate mod coerce;

pub type ParseError<'input> = lalrpop_util::ParseError<usize, Token<'input>, &'static str>;

pub fn invoke(
    module: &elements::Module,
    name: &str,
    args: &[wasmi::RuntimeValue],
) -> Option<wasmi::RuntimeValue> {
    let module = wasmi::Module::from_parity_wasm_module(module.clone()).unwrap();
    let main = wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default())
        .unwrap()
        .run_start(&mut wasmi::NopExternals)
        .unwrap();

    main.invoke_export(name, args, &mut wasmi::NopExternals)
        .unwrap()
}

#[allow(unused_macros)]
macro_rules! runtime_values {
    ($($e:expr),*) => {
        vec![
            $(
                $crate::coerce::AsRuntimeValue::as_runtime_value(&$e)
            ),*
        ]
    }
}

#[allow(unused_macros)]
macro_rules! return_type {
    (void) => { None };
    ($e:expr) => { Some($crate::coerce::AsRuntimeValue::as_runtime_value(&$e)) }
}

macro_rules! syntax {
    ($mod_name:ident { module $syntax:expr; invoke $name:ident ($($args:expr),*) = $expected:tt }) => {
        #[allow(unused)]
        mod $mod_name {
            use wasm::test_helpers::*;

            use crate::invoke;
            use wasm::{ast, compile_module, parser};
            use wasm::ir::Type;
            use nan_preserving_float::{F32, F64};

            fn module() -> ast::Module<'static> {
                println!("{}", $syntax);
                parser::parse($syntax).unwrap()
            }

            #[test]
            fn test_compile() {
                let module = compile_module(&module());
                let value = invoke(&module.unwrap(), stringify!($name), &runtime_values!($($args),*));

                assert_eq!(value, return_type!($expected));
            }
        }
    };

    ($mod_name:ident { module $syntax:expr; parse as |$module_builder:ident| $parse:expr; invoke $name:ident ($($args:expr),*) = $expected:tt }) => {
        #[allow(unused)]
        mod $mod_name {
            use wasm::test_helpers::*;

            use crate::invoke;
            use wasm::{ast, compile_module, parser};
            use wasm::ir::Type;
            use nan_preserving_float::{F32, F64};

            fn module() -> ast::Module<'static> {
                println!("{}", $syntax);
                parser::parse($syntax).unwrap()
            }

            #[test]
            fn test_parse() {
                let expected: Vec<ast::Module<'static>> = 
                    AstBuilder::new().module(|$module_builder| $parse).done();

                assert_eq!(vec![module()], expected);
            }

            #[test]
            fn test_compile() {
                let module = compile_module(&module());
                let value = invoke(&module.unwrap(), stringify!($name), &runtime_values!($($args),*));

                assert_eq!(value, return_type!($expected));
            }
        }
    }
}

macro_rules! compile_error {
    ($mod_name:ident { module $syntax:expr; error at $pos:expr }) => {
        #[allow(unused)]
        mod $mod_name {
            use wasm::{ast, compile_module, parser};
            use wasm::ir::{Type};
            use nan_preserving_float::{F32, F64};
            use $crate::ParseError;

            fn module() -> Result<ast::Module<'static>, ParseError<'static>> {
                parser::parse($syntax)
            }

            #[test]
            fn test() {
                let location = match module() {
                    Ok(module) => panic!("Expected parse error"),
                    Err(err) => parser::location(err)
                };

                assert_eq!(location, parser::ErrorLocation::Byte($pos));
            }
        }
    }
}

syntax!(noop_function {
    module "export def noop () {}";

    // parse as |module| {
    //     module.function("noop", |f| {
    //         f.exported()
    //     })
    // };

    invoke noop() = void
});

syntax!(identity_function {
    module "export def id(x: i32) -> i32 { x }";

    // parse as |module| {
    //     module.function("id", |f| {
    //         f.exported();
    //         f.param("x", Type::I32);
    //         f.returning(Type::I32);
    //         f.expression(|e| e.variable("x"))
    //     })
    // };

    invoke id(10) = 10
});

compile_error!(missing_type {
    module "export def plus(x: i32, y) -> i32 { x + y }";

    error at 25
});

macro_rules! binary_operator {
    ($name:ident | $lhs:tt $operator:tt $rhs:tt = $result:tt) => {
        macro_rules! for_type {
            ($ty:ident) => {
                mod $ty {
                    syntax!(vars {
                        module concat!("export def ", stringify!($name), "(x: ", stringify!($ty), ", y: ", stringify!($ty), ") -> ", stringify!($ty), " { x ", stringify!($operator), " y }");

                        invoke $name(($lhs as $ty), ($rhs as $ty)) = { ($lhs as $ty) $operator ($rhs as $ty) }
                    });

                    syntax!(const_right {
                        module concat!("export def ", stringify!($name), "(x: ", stringify!($ty), ") -> ", stringify!($ty), " { x ", stringify!($operator), " ", stringify!($rhs), " }");

                        invoke $name(($lhs as $ty)) = { ($lhs as $ty) $operator ($rhs as $ty) }
                    });
                }
            }
        }

        mod $name {
            for_type!(i32);
            for_type!(i64);
            for_type!(u32);
            for_type!(u64);
            for_type!(f32);
            for_type!(f64);
        }
    }
}

binary_operator!(add | 100 + 50 = 150);
binary_operator!(sub | 100 - 50 = 50);
binary_operator!(mul | 10 * 11 = 110);
binary_operator!(div | 100 / 5 = 20);

syntax!(plus_function_i32 {
    module "export def plus(x: i32, y: i32) -> i32 { x + y }";

    // parse as |module| {
    //     module.function("plus", |f| {
    //         f.exported();
    //         f.param("x", Type::I32);
    //         f.param("y", Type::I32);
    //         f.returning(Type::I32);
    //         f.expression(|e| e.variable("x") + e.variable("y"));
    //     })
    // };

    invoke plus(20, 33) = 53
});

syntax!(plus_function_f32 {
    module "export def plus(x: f32, y: f32) -> f32 { x + y }";

    // parse as |module| {
    //     module.function("plus", |f| {
    //         f.exported();
    //         f.param("x", Type::F32);
    //         f.param("y", Type::F32);
    //         f.returning(Type::F32);
    //         f.expression(|e| e.variable("x") + e.variable("y"));
    //     })
    // };

    invoke plus(20.0f32, 33.0f32) = 53.0f32
});


syntax!(plus_function_i64 {
    module "export def plus(x: i64, y: i64) -> i64 { x + y }";

    // parse as |module| {
    //     module.function("plus", |f| {
    //         f.exported();
    //         f.param("x", Type::I64);
    //         f.param("y", Type::I64);
    //         f.returning(Type::I64);
    //         f.expression(|e| e.variable("x") + e.variable("y"));
    //     })
    // };

    invoke plus(20i64, 33i64) = 53i64
});

syntax!(plus_function_f64 {
    module "export def plus(x: f64, y: f64) -> f64 { x + y }";

    // parse as |module| {
    //     module.function("plus", |f| {
    //         f.exported();
    //         f.param("x", Type::F64);
    //         f.param("y", Type::F64);
    //         f.returning(Type::F64);
    //         f.expression(|e| e.variable("x") + e.variable("y"));
    //     })
    // };

    invoke plus(20.0f64, 33.0f64) = 53.0f64
});


syntax!(plus_const_left {
    module "export def plus(x: i32) -> i32 { x + 1 }";

    // parse as |module| {
    //     module.function("plus", |f| {
    //         f.exported();
    //         f.param("x", Type::I32);
    //         f.returning(Type::I32);
    //         f.expression(|e| e.variable("x") + e.i32(1));
    //     })
    // };

    invoke plus(20) = 21
});

/*
syntax!(plus_const_right {
    module "export def plus(x: i32) -> i32 { 1 + x }";

    // parse as |module| {
    //     module.function("plus", |f| {
    //         f.exported();
    //         f.param("x", Type::I32);
    //         f.returning(Type::I32);
    //         f.expression(|e| e.i32(1) + e.variable("x"));
    //     })
    // };

    invoke plus(20) = 21
});
*/

syntax!(minus_i32 {
    module "export def minus(x: i32, y: i32) -> i32 { x - y }";

    invoke minus(30i32, 10i32) = 20i32
});

syntax!(minus_i64 {
    module "export def minus(x: i64, y: i64) -> i64 { x - y }";

    invoke minus(30i64, 10i64) = 20i64
});


syntax!(minus_f32 {
    module "export def minus(x: f32, y: f32) -> f32 { x - y }";

    invoke minus(30f32, 10f32) = 20f32
});

syntax!(minus_f64 {
    module "export def minus(x: f64, y: f64) -> f64 { x - y }";

    invoke minus(30f64, 10f64) = 20f64
});