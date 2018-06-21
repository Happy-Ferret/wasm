use crate::compile::math::MathOperator;
use crate::ir::{FunctionModifiers, Spanned, Type};
use nan_preserving_float::F64;
use std::collections::HashMap;
use std::fmt;

#[derive(PartialEq, Clone, new)]
pub struct RawIdentifier<'input> {
    pub name: &'input str,
}

pub type Identifier<'input> = Spanned<RawIdentifier<'input>>;

impl Spanned<RawIdentifier<'input>> {
    crate fn as_ref(&self) -> Spanned<&'input str> {
        Spanned {
            node: self.node.name,
            span: self.span,
        }
    }
}

pub fn ident<'input>(name: &'input str) -> RawIdentifier<'input> {
    RawIdentifier::new(name)
}

impl fmt::Debug for RawIdentifier<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(PartialEq, Clone, new)]
pub struct Parameter<'input> {
    pub name: Identifier<'input>,
    pub ty: Type,
}

impl fmt::Debug for Parameter<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.name, self.ty)
    }
}

#[derive(PartialEq, Clone)]
pub struct Parameters<'input> {
    pub list: Vec<Parameter<'input>>,
}

impl Parameters<'input> {
    crate fn iter(&self) -> impl Iterator<Item = (Spanned<&str>, &Type)> {
        self.list.iter().map(|arg| (arg.name.as_ref(), &arg.ty))
    }
}

impl fmt::Debug for Parameters<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.list.iter()).finish()
    }
}

impl Parameters<'input> {
    pub fn new(list: Vec<Parameter<'input>>) -> Parameters<'input> {
        Parameters { list }
    }

    crate fn empty() -> Parameters<'input> {
        Parameters { list: vec![] }
    }

    crate fn from_parser(
        arg: Parameter<'input>,
        rest: Vec<Parameter<'input>>,
    ) -> Parameters<'input> {
        let mut list = vec![arg];
        list.extend(rest);
        Parameters { list }
    }
}

#[derive(PartialEq, Clone)]
pub struct Function<'input> {
    pub name: Identifier<'input>,
    pub args: Parameters<'input>,
    pub ret: Type,
    pub body: Block<'input>,
    pub modifiers: FunctionModifiers,
    crate mappings: HashMap<String, u32>,
}

impl Function<'input> {
    pub fn new(
        name: Identifier<'input>,
        args: Parameters<'input>,
        ret: Type,
        body: Block<'input>,
    ) -> Function<'input> {
        let mappings = function_mappings(&args);

        Function {
            name,
            args,
            ret,
            body,
            modifiers: FunctionModifiers::new(),
            mappings,
        }
    }

    pub fn exported(mut self) -> Function<'input> {
        self.modifiers.export = true;
        self
    }
}

fn function_mappings(args: &Parameters<'input>) -> HashMap<String, u32> {
    let mut map = HashMap::new();

    for (i, (name, _)) in args.iter().enumerate() {
        map.insert(name.node.to_string(), i as u32);
    }

    map
}

impl fmt::Debug for Function<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entry(&"name", &self.name)
            .entry(&"args", &self.args)
            .entry(&"return", &self.ret)
            .entry(&"body", &self.body)
            .entry(&"export", &self.modifiers.export)
            .finish()
    }
}

#[derive(PartialEq, Clone, Debug, new)]
pub struct Module<'input> {
    pub funcs: Vec<Function<'input>>,
}

#[derive(PartialEq, Clone, new)]
pub struct Block<'input> {
    pub expressions: Vec<Expression<'input>>,
}

impl fmt::Debug for Block<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.expressions.iter()).finish()
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ConstExpression {
    Integer(i64),
    Float(F64),
    Bool(bool),
}

impl fmt::Debug for ConstExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstExpression::Integer(int) => write!(f, "{:?}", *int),
            ConstExpression::Float(float) => write!(f, "{:?}", *float),
            ConstExpression::Bool(boolean) => write!(f, "{:?}", *boolean),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Expression<'input> {
    Const(ConstExpression),
    VariableAccess(Identifier<'input>),
    Binary(MathOperator, Box<BinaryExpression<'input>>),
}

impl fmt::Debug for Expression<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value: &dyn fmt::Debug = match self {
            Expression::Const(constant) => constant,
            Expression::VariableAccess(id) => id,
            Expression::Binary(op, box BinaryExpression { lhs, rhs }) => {
                return write!(f, "{:?} {:?} {:?}", lhs, op, rhs);
            }
        };

        write!(f, "{:?}", value)
    }
}

#[derive(PartialEq, Clone, new)]
pub struct BinaryExpression<'input> {
    pub lhs: Expression<'input>,
    pub rhs: Expression<'input>,
}

impl fmt::Debug for BinaryExpression<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} + {:?}", self.lhs, self.rhs)
    }
}
