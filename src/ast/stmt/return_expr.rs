use crate::ast::{Expr, Resolver, Stmt};
use crate::{Error, Scopes, Token};
use std::rc::Rc;

#[derive(Debug)]
pub struct ReturnExpr {
    pub keyword: Token,
    pub value: Option<Rc<dyn Expr>>,
}

impl std::fmt::Display for ReturnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match &self.value {
            Some(expr) => format!("{}", expr),
            None => "Nil".to_string(),
        };
        write!(f, "<re>({} {})", self.keyword, value)
    }
}

impl Stmt for ReturnExpr {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        if let Some(expr) = &self.value {
            let value = expr.eval()?;
            unsafe {
                crate::ENVIRONMENT.define("return".to_string(), value);
            }
        } else {
            unsafe {
                crate::ENVIRONMENT.define("return".to_string(), Box::new(crate::Value::Nil));
            }
        }
        Err(Error {
            line: 0,
            loc: "".to_string(),
            message: "return".to_string(),
        })
    }
}

impl Resolver for ReturnExpr {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        if scopes.get_current_function().is_none() {
            return Err(Error::report(
                self.keyword.clone(),
                "Can't return from top-level code".to_string(),
            ));
        } else if let Some(crate::FunctionType::Initializer) = scopes.get_current_function() {
            if self.value.is_some() {
                return Err(Error::report(
                    self.keyword.clone(),
                    "Can't return a value from an initializer".to_string(),
                ));
            }
        }
        if let Some(expr) = &self.value {
            expr.clone().resolve(scopes)?;
        }
        Ok(())
    }
}
