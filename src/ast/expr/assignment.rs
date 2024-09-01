use crate::ast::{Expr, Resolver};
use crate::{Environment, Error, Token, Value, ENVIRONMENT};
use rlox_macro::Expr;
use std::rc::Rc;

#[derive(Expr, Debug)]
pub struct Assignment {
    pub name: Token,
    pub value: Rc<dyn Expr>,
}

impl Expr for Assignment {
    fn eval(&self) -> Result<Box<Value>, Error> {
        let value = self.value.eval()?;
        let distance = unsafe { crate::LOCALS.get(&(self as *const dyn Expr)) };
        if let Some(distance) = distance {
            unsafe {
                Environment::assign_at(
                    crate::ENVIRONMENT.clone(),
                    *distance,
                    self.name.clone(),
                    value.clone(),
                )?;
            }
        } else {
            unsafe {
                Environment::assign(
                    Environment::get_global_mut(ENVIRONMENT.clone()),
                    self.name.clone(),
                    value.clone(),
                )?;
            }
        }
        Ok(value)
    }
}

impl Resolver for Assignment {
    fn resolve(self: Rc<Self>, scopes: &mut crate::Scopes) -> Result<(), Error> {
        self.value.clone().resolve(scopes)?;
        scopes.resolve_local(Rc::as_ptr(&self) as *const dyn Expr, &self.name);
        Ok(())
    }
}
