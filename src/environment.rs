use crate::error::Error;
use crate::syntax::Value;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Box<Value>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn set_enclosing(&mut self, enclosing: Option<Rc<RefCell<Environment>>>) {
        self.enclosing = enclosing;
    }

    pub fn get_enclosing(&self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.clone()
    }

    pub fn from(&mut self, other: Rc<RefCell<Self>>) {
        self.values = other.borrow().values.clone();
        self.enclosing = other.borrow().enclosing.clone();
    }

    pub fn define(&mut self, name: String, value: Box<Value>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: Token) -> Result<Box<Value>, Error> {
        match self.values.get(&token.lexeme) {
            Some(value) => Ok(value.clone()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    enclosing.borrow().get(token)
                } else {
                    Err(Error::new(
                        token.line,
                        token.lexeme,
                        "Undefined variable".to_string(),
                    ))
                }
            }
        }
    }

    pub fn get_at(&self, distance: usize, token: Token) -> Result<Box<Value>, Error> {
        if let Some(value) = self.ancestor(distance).borrow().values.get(&token.lexeme) {
            Ok(value.clone())
        } else {
            Err(Error::new(
                token.line,
                token.lexeme,
                "Undefined variable".to_string(),
            ))
        }
    }

    pub fn assign(&mut self, token: Token, value: Box<Value>) -> Result<(), Error> {
        match self.values.get(&token.lexeme) {
            Some(_) => {
                self.values.insert(token.lexeme, value);
                Ok(())
            }
            None => {
                if let Some(enclosing) = &mut self.enclosing {
                    enclosing.borrow_mut().assign(token, value)
                } else {
                    Err(Error::new(
                        token.line,
                        token.lexeme,
                        "Undefined variable".to_string(),
                    ))
                }
            }
        }
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        token: Token,
        value: Box<Value>,
    ) -> Result<(), Error> {
        if let Some(_) = self.ancestor(distance).borrow().values.get(&token.lexeme) {
            self.ancestor(distance)
                .borrow_mut()
                .values
                .insert(token.lexeme, value);
            Ok(())
        } else {
            Err(Error::new(
                token.line,
                token.lexeme,
                "Undefined variable".to_string(),
            ))
        }
    }

    pub fn ancestor(&self, distance: usize) -> Rc<RefCell<Self>> {
        let mut env = self.clone();
        for _ in 0..distance {
            let e = env.enclosing.clone();
            if let Some(e) = e {
                env = e.borrow().clone();
            } else {
                return Rc::new(RefCell::new(Environment::new(None)));
            }
        }
        Rc::new(RefCell::new(env))
    }

    pub fn get_global(&self) -> Rc<RefCell<Self>> {
        let mut env = self.clone();
        while let Some(enclosing) = env.enclosing.clone() {
            env = enclosing.borrow().clone();
        }
        Rc::new(RefCell::new(env))
    }

    pub fn look_up_variable(
        &self,
        name: Token,
        expr: Rc<dyn crate::syntax::Expr>,
    ) -> Result<Box<Value>, Error> {
        let distance =
            unsafe { crate::LOCALS.get(&(Rc::as_ptr(&expr) as *const dyn crate::syntax::Expr)) };
        if let Some(distance) = distance {
            self.get_at(*distance, name)
        } else {
            self.get_global().borrow().get(name)
        }
    }
}
