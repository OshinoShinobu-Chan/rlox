use crate::ast::{Expr, Resolver};
use crate::Token;
use std::rc::Rc;

#[derive(Debug)]
pub struct ArrayExpr {
    pub name: Rc<dyn Expr>,
    pub bracket: Token,
    pub index: Rc<dyn Expr>,
}

impl std::fmt::Display for ArrayExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<a>{}[{}]", self.name, self.index)
    }
}

impl Resolver for ArrayExpr {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), crate::error::Error> {
        self.name.clone().resolve(scopes)?;
        self.index.clone().resolve(scopes)
    }
}

impl Expr for ArrayExpr {
    fn eval(&self) -> Result<Box<crate::ast::Value>, crate::error::Error> {
        let array = self.name.eval()?;
        let index = self.index.eval()?;
        if let crate::ast::Value::Number(index) = *index {
            if let crate::ast::Value::Array(array) = *array {
                if index.is_nan() || index.is_infinite() || index < 0.0 {
                    return Err(crate::error::Error::new(
                        self.bracket.line,
                        self.bracket.lexeme.clone(),
                        "Index must be a non-negative integer".to_string(),
                    ));
                }
                if index as usize >= array.len() {
                    return Err(crate::error::Error::new(
                        self.bracket.line,
                        self.bracket.lexeme.clone(),
                        "Index out of bounds".to_string(),
                    ));
                }
                Ok(array[index as usize].clone())
            } else {
                Err(crate::error::Error::new(
                    self.bracket.line,
                    self.bracket.lexeme.clone(),
                    "Can't index non-array value".to_string(),
                ))
            }
        } else {
            Err(crate::error::Error::new(
                self.bracket.line,
                self.bracket.lexeme.clone(),
                "Index must be a number".to_string(),
            ))
        }
    }
}
