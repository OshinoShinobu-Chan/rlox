use crate::ast::expr::VarExpr;
use crate::ast::stmt::Function;
use crate::ast::{Expr, Resolver, Stmt};
use crate::{Error, FunctionType, Scopes, Token, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Rc<Function>>,
    pub super_class: Option<Rc<VarExpr>>,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let super_class = self.super_class.clone().map(|x| x.name.lexeme.clone());
        write!(
            f,
            "<class {}, super class {:?}>",
            self.name.lexeme, super_class
        )
    }
}

impl Resolver for Class {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error> {
        let enclosing_class = scopes.get_current_class();
        scopes.set_current_class(Some(crate::ClassType::Class));

        scopes.declare(self.name.clone())?;
        scopes.define(self.name.clone());
        if let Some(super_class) = self.super_class.clone() {
            if super_class.name.lexeme == self.name.lexeme {
                return Err(Error::report(
                    super_class.name.clone(),
                    "A class can't inherit from itself".to_string(),
                ));
            }
            super_class.resolve(scopes)?;
            scopes.set_current_class(Some(crate::ClassType::SubClass));
        }
        if self.super_class.is_some() {
            scopes.begin_scope();
            if let Some(scope) = scopes.peek_mut() {
                scope.insert("super".to_string(), true);
            }
        }

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
        if self.super_class.is_some() {
            scopes.end_scope();
        }
        scopes.set_current_class(enclosing_class);
        Ok(())
    }
}

impl Stmt for Class {
    fn interpret(&self) -> Result<(), crate::error::Error> {
        unsafe {
            crate::ENVIRONMENT.define(self.name.lexeme.clone(), Box::new(Value::Nil));
            let mut super_class = None;
            if let Some(super_cls) = self.super_class.clone() {
                super_class = Some(super_cls.eval()?);
                if !super_class.as_ref().unwrap().is_class() {
                    return Err(crate::Error::new(
                        self.name.line,
                        self.name.lexeme.clone(),
                        "Superclass must be a class".to_string(),
                    ));
                }
            }
            let mut methods = HashMap::new();
            for method in self.methods.clone() {
                if method.name.lexeme == "init" {
                    (*(Rc::as_ptr(&method) as *mut Function)).is_initializer = true;
                }
                methods.insert(
                    method.name.lexeme.clone(),
                    Box::new(Value::Fun(method.clone(), None, super_class.clone())),
                );
            }
            let class = Box::new(Value::Class {
                class: self.name.lexeme.clone(),
                methods,
                super_class,
            });
            crate::ENVIRONMENT.assign(self.name.clone(), class)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: String,
    pub fields: HashMap<String, Box<Value>>,
    pub methods: HashMap<String, Box<Value>>,
    pub super_class: Option<Box<Value>>,
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<instance of {}>", self.class)
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
        } else if let Some(super_class) = self.super_class.clone() {
            let mut method = super_class.get_method(name);
            method.as_mut().map(|x| x.bind(this));
            method
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Box<Value>) {
        self.fields.insert(name.to_string(), value);
    }
}
