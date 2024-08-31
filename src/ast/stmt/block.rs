use crate::ast::{Resolver, Stmt};
use crate::{Environment, Error, Scopes, Token};
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
            let previous = Rc::new(RefCell::new(crate::ENVIRONMENT.clone()));

            crate::ENVIRONMENT = Lazy::new(|| Environment::new(None));
            crate::ENVIRONMENT.set_enclosing(Some(previous.clone()));
            for statement in &self.statements {
                if let Err(e) = statement.interpret() {
                    let mut ret_val = None;
                    if e.message == "return" {
                        ret_val = Some(crate::ENVIRONMENT.get(Token {
                            token_type: crate::token_type::TokenType::Identifier(
                                "return".to_string(),
                            ),
                            lexeme: "return".to_string(),
                            line: 0,
                        })?);
                    }
                    crate::ENVIRONMENT.from(crate::ENVIRONMENT.get_enclosing().unwrap());
                    if let Some(value) = ret_val {
                        crate::ENVIRONMENT.define("return".to_string(), value);
                    }
                    return Err(e);
                }
            }
            crate::ENVIRONMENT.from(crate::ENVIRONMENT.get_enclosing().unwrap());
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
