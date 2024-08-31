use crate::ast::stmt::Function;
use crate::ast::{Resolver, Stmt};
use crate::{Error, FunctionType, Scopes, Token, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Rc<Function>>,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name.lexeme)
    }
}

impl Resolver for Class {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        let enclosing_class = scopes.get_current_class();
        scopes.set_current_class(Some(crate::ClassType::Class));

        scopes.declare(self.name.clone())?;
        scopes.define(self.name.clone());
        scopes.begin_scope();
        if let Some(scope) = scopes.peek_mut() {
            scope.insert("this".to_string(), true);
        }
        for method in &self.methods {
            if method.name.lexeme == "init" {
                crate::ast::stmt::function::resolve_function(
                    method.clone(),
                    scopes,
                    FunctionType::Initializer,
                )?;
            } else {
                crate::ast::stmt::function::resolve_function(
                    method.clone(),
                    scopes,
                    FunctionType::Method,
                )?;
            }
        }
        scopes.end_scope();

        scopes.set_current_class(enclosing_class);
        Ok(())
    }
}

impl Stmt for Class {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        unsafe {
            crate::ENVIRONMENT.define(self.name.lexeme.clone(), Box::new(Value::Nil));
            let mut methods = HashMap::new();
            for method in self.methods.clone() {
                if method.name.lexeme == "init" {
                    (*(Rc::as_ptr(&method) as *mut Function)).is_initializer = true;
                }
                methods.insert(
                    method.name.lexeme.clone(),
                    Box::new(Value::Fun(method.clone(), None)),
                );
            }
            let class = Box::new(Value::Class {
                class: Rc::new(self.clone()),
                methods,
            });
            crate::ENVIRONMENT.assign(self.name.clone(), class)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    pub fields: HashMap<String, Box<Value>>,
    pub methods: HashMap<String, Box<Value>>,
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<instance of {}>", self.class.name.lexeme)
    }
}

impl Instance {
    pub fn get(&self, name: &str, this: Rc<RefCell<Self>>) -> Option<Box<Value>> {
        if self.fields.contains_key(name) {
            self.fields.get(name).cloned()
        } else if self.methods.contains_key(name) {
            let mut method = self.methods.get(name).cloned().unwrap();
            method.bind(this);
            Some(method)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Box<Value>) {
        self.fields.insert(name.to_string(), value);
    }
}
