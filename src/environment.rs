use crate::{Error, Token, Value};
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

    pub fn assign(target: Rc<RefCell<Self>>, token: Token, value: Box<Value>) -> Result<(), Error> {
        let mut target = target.borrow_mut();
        let v = target.values.get(&token.lexeme);
        match v {
            Some(_) => {
                target.values.insert(token.lexeme, value);
                Ok(())
            }
            None => {
                if let Some(enclosing) = target.enclosing.clone() {
                    Self::assign(enclosing, token, value)
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

    pub fn assign_array(
        target: Rc<RefCell<Self>>,
        token: Token,
        indeces: Vec<usize>,
        value: Box<Value>,
    ) -> Result<(), Error> {
        let mut target = target.borrow_mut();
        let v = target.values.get_mut(&token.lexeme);
        match v {
            Some(v) => {
                if let Value::Array(array) = v.as_mut() {
                    let mut current = array;
                    for index in indeces.iter().take(indeces.len() - 1) {
                        if let Value::Array(array) = current[*index].as_mut() {
                            current = array;
                        } else {
                            return Err(Error::new(
                                token.line,
                                token.lexeme,
                                "Not an array".to_string(),
                            ));
                        }
                    }
                    current[indeces[indeces.len() - 1]] = value;
                    Ok(())
                } else {
                    Err(Error::new(
                        token.line,
                        token.lexeme,
                        "Not an array".to_string(),
                    ))
                }
            }
            None => {
                if let Some(enclosing) = target.enclosing.clone() {
                    Self::assign_array(enclosing, token, indeces, value)
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
        target: Rc<RefCell<Self>>,
        distance: usize,
        token: Token,
        value: Box<Value>,
    ) -> Result<(), Error> {
        let ancestor_ = Self::ancestor_mut(target.clone(), distance);
        let mut ancestor = ancestor_.borrow_mut();
        if ancestor.values.get(&token.lexeme).is_some() {
            ancestor.values.insert(token.lexeme, value);
            Ok(())
        } else {
            Err(Error::new(
                token.line,
                token.lexeme,
                "Undefined variable".to_string(),
            ))
        }
    }

    pub fn assign_array_at(
        target: Rc<RefCell<Self>>,
        distance: usize,
        token: Token,
        indeces: Vec<usize>,
        value: Box<Value>,
    ) -> Result<(), Error> {
        let ancestor = Self::ancestor_mut(target.clone(), distance);
        let mut target = ancestor.borrow_mut();
        let v = target.values.get_mut(&token.lexeme);
        match v {
            Some(v) => {
                if let Value::Array(array) = v.as_mut() {
                    let mut current = array;
                    for index in indeces.iter().take(indeces.len() - 1) {
                        if let Value::Array(array) = current[*index].as_mut() {
                            current = array;
                        } else {
                            return Err(Error::new(
                                token.line,
                                token.lexeme,
                                "Not an array".to_string(),
                            ));
                        }
                    }
                    current[indeces[indeces.len() - 1]] = value;
                    Ok(())
                } else {
                    Err(Error::new(
                        token.line,
                        token.lexeme,
                        "Not an array".to_string(),
                    ))
                }
            }
            None => Err(Error::new(
                token.line,
                token.lexeme,
                "Undefined variable".to_string(),
            )),
        }
    }

    pub fn ancestor_mut(target: Rc<RefCell<Self>>, distance: usize) -> Rc<RefCell<Self>> {
        let mut env = target.clone();
        for _ in 0..distance {
            let e = env.clone().borrow().enclosing.clone();
            if let Some(e) = e {
                env = e.clone();
            } else {
                return Rc::new(RefCell::new(Environment::new(None)));
            }
        }
        env
    }

    pub fn ancestor(&self, distance: usize) -> Rc<RefCell<Self>> {
        let mut env = Rc::new(RefCell::new(self.clone()));
        for _ in 0..distance {
            let e = env.borrow().enclosing.clone();
            if let Some(e) = e {
                env = e.clone();
            } else {
                return Rc::new(RefCell::new(Environment::new(None)));
            }
        }
        env
    }

    pub fn get_global_mut(target: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        let mut env = target.clone();
        loop {
            let tmp = env.borrow().enclosing.clone();
            if let Some(tmp) = tmp {
                env = tmp;
            } else {
                break;
            }
        }
        env
    }

    pub fn get_global(&self) -> Rc<RefCell<Self>> {
        let mut result = Rc::new(RefCell::new(self.clone()));
        loop {
            let tmp = result.borrow().enclosing.clone();
            if let Some(tmp) = tmp {
                result = tmp;
            } else {
                break;
            }
        }
        result
    }

    pub fn look_up_variable(
        &self,
        name: Token,
        expr: *const dyn crate::ast::Expr,
    ) -> Result<Box<Value>, Error> {
        let distance = unsafe { crate::LOCALS.get(&expr) };
        if let Some(distance) = distance {
            self.get_at(*distance, name)
        } else {
            self.get_global().borrow().get(name)
        }
    }
}
