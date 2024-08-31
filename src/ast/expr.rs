//! expressions in AST
use crate::ast::Resolver;
use crate::error::Error;
use crate::Value;

pub mod literal;
pub use literal::Literal;
pub mod operator;
pub mod unary;
pub use unary::Unary;
pub mod binary;
pub use binary::Binary;
pub mod grouping;
pub use grouping::Grouping;
pub mod varexpr;
pub use varexpr::VarExpr;
pub mod assignment;
pub use assignment::Assignment;
pub mod logic;
pub use logic::Logic;
pub mod call;
pub use call::Call;
pub mod get;
pub use get::Get;
pub mod set;
pub use set::Set;
pub mod this;
pub use this::This;

pub trait Expr: std::fmt::Display + std::fmt::Debug + Resolver {
    fn eval(&self) -> Result<Box<Value>, Error>;
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}
