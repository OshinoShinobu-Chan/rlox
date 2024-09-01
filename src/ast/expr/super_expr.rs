use crate::ast::{Expr, Resolver, Value};
use crate::{Token, TokenType};
use std::rc::Rc;

#[derive(Debug)]
pub struct SuperExpr {
    pub keyword: Token,
    pub method: Token,
}

impl Resolver for SuperExpr {
    fn resolve(
        self: std::rc::Rc<Self>,
        scopes: &mut crate::Scopes,
    ) -> Result<(), crate::error::Error> {
        if scopes.get_current_class().is_none() {
            return Err(crate::Error::report(
                self.keyword.clone(),
                "Can't use 'super' outside of a class".to_string(),
            ));
        } else if scopes.get_current_class() == Some(crate::ClassType::Class) {
            return Err(crate::Error::report(
                self.keyword.clone(),
                "Can't use 'super' in a class with no superclass".to_string(),
            ));
        }
        scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.keyword);
        Ok(())
    }
}

impl Expr for SuperExpr {
    fn eval(&self) -> Result<Box<crate::ast::Value>, crate::error::Error> {
        unsafe {
            let distance = crate::LOCALS.get(&(self as *const dyn Expr)).unwrap();
            let super_class = crate::ENVIRONMENT
                .borrow()
                .get_at(*distance, self.keyword.clone())?;
            let this = crate::ENVIRONMENT.borrow().get_at(
                distance - 1,
                Token {
                    token_type: TokenType::This,
                    lexeme: "this".to_string(),
                    line: 0,
                },
            )?;
            if let Value::Instance(this) = this.as_ref().clone() {
                let method = super_class.get_method(&self.method.lexeme);
                if let Some(mut method) = method {
                    method.bind(this);
                    Ok(method)
                } else {
                    Err(crate::Error::new(
                        self.method.line,
                        self.method.lexeme.clone(),
                        format!("Undefined property '{}'", self.method.lexeme),
                    ))
                }
            } else {
                unreachable!()
            }
        }
    }
}

impl std::fmt::Display for SuperExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<super {}>", self.method.lexeme)
    }
}
