use super::types::InferType;
use super::{Annotated, Expression, TypeEnv};
use crate::infer::unify::UnifyTable;
use crate::infer::{Constraint, Constraints};
use crate::ir::pos::SpannedItem;
use crate::ir::resolved;
use crate::Type;

#[derive(Debug, Clone)]
crate struct Block {
    crate expressions: Vec<Annotated<Expression>>,
}

impl Block {
    crate fn from(
        block: resolved::Block,
        vars: &mut UnifyTable,
        env: &TypeEnv,
    ) -> Annotated<Block> {
        let expressions = block
            .expressions
            .into_iter()
            .map(|e| e.annotate(vars, &env))
            .collect();

        Annotated {
            item: Block { expressions },
            ty: vars.fresh(),
        }
    }
}

impl Block {
    crate fn last_ty(&self) -> InferType {
        match self.expressions.last() {
            None => InferType::Resolved(Type::Void.synthetic("void")),
            Some(e) => e.ty.clone(),
        }
    }
}

impl Annotated<Block> {
    crate fn constraints(&self) -> Constraints {
        let Annotated { ty, item } = self;

        let mut constraints = Constraints::empty() + Constraint(item.last_ty(), ty.clone());

        for expression in &item.expressions {
            constraints += expression.constraints();
        }

        constraints
    }
}
