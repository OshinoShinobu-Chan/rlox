use crate::ast::{Expr, Resolver};
use crate::{Error, Scopes, Token};
use std::rc::Rc;

pub struct This {
    pub keyword: Token,
}

impl Resolver for This {
    fn resolve(self: std::rc::Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        if scopes.get_current_class().is_none() {
            Err(Error::new(
                self.keyword.line,
                self.keyword.lexeme.clone(),
                "Cannot use 'this' outside of a class".to_string(),
            ))
        } else {
            scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.keyword);
            Ok(())
        }
    }
}

impl std::fmt::Display for This {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<this>")
    }
}

impl std::fmt::Debug for This {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<this>")
    }
}

impl Expr for This {
    fn eval(&self) -> Result<Box<crate::ast::value::Value>, crate::error::Error> {
        unsafe {
            crate::ENVIRONMENT
                .borrow()
                .look_up_variable(self.keyword.clone(), self)
        }
    }
}
