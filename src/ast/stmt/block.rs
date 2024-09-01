use crate::ast::{Resolver, Stmt};
use crate::{Environment, Error, Scopes};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Rc<dyn Stmt>>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut statements = String::new();
        statements.push_str("<b>{");
        for stmt in &self.statements {
            statements.push_str(&format!("{}\n", stmt));
        }
        write!(f, "{}}}", statements)
    }
}

impl Stmt for Block {
    fn interpret(&self) -> Result<(), Error> {
        unsafe {
            let previous = Rc::new(RefCell::new(crate::ENVIRONMENT.borrow().clone()));

            crate::ENVIRONMENT = Lazy::new(|| Rc::new(RefCell::new(Environment::new(None))));
            crate::ENVIRONMENT
                .borrow_mut()
                .set_enclosing(Some(previous.clone()));
            for statement in &self.statements {
                statement.interpret()?;
            }
            crate::ENVIRONMENT.borrow_mut().from(previous);
            Ok(())
        }
    }
}

impl Resolver for Block {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        scopes.begin_scope();
        for statement in self.statements.clone() {
            statement.resolve(scopes)?;
        }
        scopes.end_scope();
        Ok(())
    }
}

impl Block {
    pub fn excute(&self) -> Result<(), Error> {
        for statement in &self.statements {
            statement.interpret()?;
        }
        Ok(())
    }
}
