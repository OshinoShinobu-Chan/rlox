use crate::ast::stmt::function::Builtin;
use crate::ast::Value;

pub static CLOCK: Builtin = Builtin {
    arity: 0,
    call: |_: Vec<Box<Value>>| {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(Box::new(Value::Number(time)))
    },
};

pub static STR: Builtin = Builtin {
    arity: 1,
    call: |args: Vec<Box<Value>>| Ok(Box::new(Value::String(args[0].to_string()))),
};

pub static LEN: Builtin = Builtin {
    arity: 1,
    call: |args| {
        if let Value::Array(array) = &*args[0] {
            Ok(Box::new(Value::Number(array.len() as f64)))
        } else if let Value::String(array) = &*args[0] {
            Ok(Box::new(Value::Number(array.len() as f64)))
        } else {
            Err(crate::error::Error::new(
                0,
                "".to_string(),
                "Argument must be an array".to_string(),
            ))
        }
    },
};

pub static NUM: Builtin = Builtin {
    arity: 1,
    call: |args| {
        if let Value::String(s) = &*args[0] {
            match s.parse::<f64>() {
                Ok(n) => Ok(Box::new(Value::Number(n))),
                Err(_) => Err(crate::error::Error::new(
                    0,
                    "".to_string(),
                    "Argument must be a number".to_string(),
                )),
            }
        } else if let Value::Number(n) = &*args[0] {
            Ok(Box::new(Value::Number(*n)))
        } else {
            Err(crate::error::Error::new(
                0,
                "".to_string(),
                "Argument must be a string".to_string(),
            ))
        }
    },
};

pub static INPUT: Builtin = Builtin {
    arity: 0,
    call: |_: Vec<Box<Value>>| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        Ok(Box::new(Value::String(input.trim().to_string())))
    },
};
