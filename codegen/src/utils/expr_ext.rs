use syntax::ast::Expr;
use syntax::ast::ExprKind::*;

pub trait ExprExt {
    fn is_location(&self) -> bool;
}

impl ExprExt for Expr {
    fn is_location(&self) -> bool {
        match self.node {
            Path(..) => true,
            Cast(ref expr, _) => expr.is_location(),
            Field(ref expr, _) => expr.is_location(),
            TupField(ref expr, _) => expr.is_location(),
            Index(ref expr, _) => expr.is_location(),
            _ => false
        }
    }
}
