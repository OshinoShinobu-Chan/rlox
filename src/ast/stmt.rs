//! statement in AST
use crate::ast::Resolver;
use crate::error::Error;

pub mod class;
pub use class::Class;
pub use class::Instance;
pub mod expression;
pub use expression::Expression;
pub mod print;
pub use print::Print;
pub mod vardecl;
pub use vardecl::VarDecl;
pub mod block;
pub use block::Block;
pub mod if_expr;
pub use if_expr::IfExpr;
pub mod while_expr;
pub use while_expr::WhileExpr;
pub mod function;
pub use function::Function;
pub mod return_expr;
pub use return_expr::ReturnExpr;

pub trait Stmt: std::fmt::Display + std::fmt::Debug + Resolver {
    fn interpret(&self) -> Result<(), Error>;
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}
