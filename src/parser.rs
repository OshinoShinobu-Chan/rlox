//! Parser
use std::rc::Rc;

use crate::ast::expr::*;
use crate::ast::stmt::*;
use crate::error::Error;
use crate::token::Token;
use crate::token_type::TokenType;

macro_rules! binary_loop {
    ($name: ident, $left: ident, $right: ident, $new_struct: ident, $op: path, $($operator: path),* ) => {
        fn $name(&mut self) -> Result<Rc<dyn Expr>, Error> {
            let mut left = self.$left()?;
            while self.is_match(vec![$op, $($operator),*]) {
                let operator = self.previous();
                let right = self.$right()?;
                left = Rc::new($new_struct {
                    left,
                    operator,
                    right,
                });
            }
            Ok(left)
        }
    };
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    /// Create a new parser from a list of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Rc<dyn Stmt>>, Error> {
        let mut statements = Vec::new();
        while !self.is_end() {
            statements.push(self.declaration()?);
        }

        return Ok(statements);
    }

    pub fn declaration(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let result;
        if self.is_match(vec![TokenType::Var]) {
            result = self.var_declaration();
        } else if self.is_match(vec![TokenType::Fun]) {
            result = self.function("function");
        } else if self.is_match(vec![TokenType::Class]) {
            result = self.class_declaration();
        } else {
            result = self.statement();
        }
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    pub fn class_declaration(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let name = self.consume(TokenType::Identifier("".to_string()), "Expect class name")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_end() {
            let fun = self.function("method")?;
            let function = unsafe { Rc::from_raw(Rc::into_raw(fun) as *const Function) };
            methods.push(function);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body")?;
        Ok(Rc::new(Class { name, methods }))
    }

    pub fn function(&mut self, kind: &str) -> Result<Rc<dyn Stmt>, Error> {
        let name = self.consume(
            TokenType::Identifier("".to_string()),
            ("Expect ".to_string() + kind + " name, but find '" + &self.peek().lexeme + "'")
                .as_str(),
        )?;
        self.consume(
            TokenType::LeftParen,
            ("Expect '(' after ".to_string() + kind + " name").as_str(),
        )?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            while {
                if params.len() >= 255 {
                    return Err(Error::report(
                        self.peek(),
                        "Can't have more than 255 parameters".to_string(),
                    ));
                }
                params.push(self.consume(
                    TokenType::Identifier("".to_string()),
                    "Expect parameter name",
                )?);
                self.is_match(vec![TokenType::Comma])
            } {}
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
        self.consume(
            TokenType::LeftBrace,
            ("Expect '{' before ".to_string() + kind + " body").as_str(),
        )?;
        let body = self.block()?;
        if body.type_name() == std::any::type_name::<Block>() {
            unsafe {
                let body_ptr = Rc::from_raw(Rc::into_raw(body) as *mut Block);
                Ok(Rc::new(Function {
                    name,
                    params,
                    body: body_ptr,
                    is_initializer: false,
                }))
            }
        } else {
            Err(Error::report(
                self.peek(),
                "Expect function body".to_string(),
            ))
        }
    }

    pub fn var_declaration(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let name = self.consume(
            TokenType::Identifier("".to_string()),
            "Expect variable name",
        )?;
        let initializer;
        if self.is_match(vec![TokenType::Equal]) {
            initializer = self.expression()?;
        } else {
            initializer = Rc::new(Literal {
                value: Token {
                    token_type: TokenType::Nil,
                    lexeme: "".to_string(),
                    line: 0,
                },
            });
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Rc::new(VarDecl {
            name,
            initializer: Some(initializer),
        }))
    }

    pub fn statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        if self.is_match(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.is_match(vec![TokenType::While]) {
            self.while_statement()
        } else if self.is_match(vec![TokenType::LeftBrace]) {
            self.block()
        } else if self.is_match(vec![TokenType::If]) {
            self.if_statement()
        } else if self.is_match(vec![TokenType::For]) {
            self.for_statement()
        } else if self.is_match(vec![TokenType::Return]) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }

    pub fn return_statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Rc::new(ReturnExpr { keyword, value }))
    }

    pub fn for_statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'")?;
        let mut initializer = None;
        if self.is_match(vec![TokenType::Semicolon]) {
            self.consume(TokenType::Semicolon, "Expect ';' after for initializer")?;
        } else if self.is_match(vec![TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after for condition")?;

        let mut increment = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if increment.is_some() {
            body = Rc::new(Block {
                statements: vec![
                    body,
                    Rc::new(Expression {
                        expression: increment.unwrap(),
                    }),
                ],
            });
        }

        if condition.is_none() {
            condition = Some(Rc::new(Literal {
                value: Token {
                    token_type: TokenType::True,
                    lexeme: "true".to_string(),
                    line: 0,
                },
            }));
        }

        body = Rc::new(WhileExpr {
            condition: condition.unwrap(),
            body,
        });

        if initializer.is_some() {
            body = Rc::new(Block {
                statements: vec![initializer.unwrap(), body],
            });
        }

        Ok(body)
    }

    pub fn print_statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Rc::new(Print { expression: expr }))
    }

    pub fn while_statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition")?;
        let body = self.statement()?;
        Ok(Rc::new(WhileExpr { condition, body }))
    }

    pub fn block(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let mut statements = Vec::new();
        while !self.is_end() && !self.check(TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block")?;
        Ok(Rc::new(Block { statements }))
    }

    pub fn expression_statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Rc::new(Expression { expression: expr }))
    }

    pub fn if_statement(&mut self) -> Result<Rc<dyn Stmt>, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition")?;
        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.is_match(vec![TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }
        Ok(Rc::new(IfExpr {
            condition,
            then_branch,
            else_branch,
        }))
    }

    /// parse the tokens using expression's rule
    fn expression(&mut self) -> Result<Rc<dyn Expr>, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Rc<dyn Expr>, Error> {
        let expr = self.or()?;
        if self.is_match(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            if expr.type_name() == std::any::type_name::<crate::ast::expr::VarExpr>() {
                let expr_ptr = Rc::into_raw(expr) as *const crate::ast::expr::VarExpr;
                return Ok(Rc::new(crate::ast::expr::Assignment {
                    name: unsafe { (*expr_ptr).name.clone() },
                    value,
                }));
            } else if expr.type_name() == std::any::type_name::<crate::ast::expr::Get>() {
                let expr_ptr = Rc::into_raw(expr) as *const crate::ast::expr::Get;
                return Ok(Rc::new(crate::ast::expr::Set {
                    object: unsafe { (*expr_ptr).object.clone() },
                    name: unsafe { (*expr_ptr).name.clone() },
                    value,
                }));
            }
            println!(
                "{}",
                Error::report(equals, "Invalid assignment target".to_string(),)
            );
        }
        Ok(expr)
    }

    binary_loop!(or, and, and, Logic, TokenType::Or,);

    binary_loop!(and, equality, equality, Logic, TokenType::And,);

    binary_loop!(
        equality,
        comparison,
        comparison,
        Binary,
        TokenType::BangEqual,
        TokenType::EqualEqual
    );

    binary_loop!(
        comparison,
        term,
        term,
        Binary,
        TokenType::Greater,
        TokenType::GreaterEqual,
        TokenType::Less,
        TokenType::LessEqual
    );

    binary_loop!(
        term,
        factor,
        factor,
        Binary,
        TokenType::Minus,
        TokenType::Plus
    );

    binary_loop!(
        factor,
        unary,
        unary,
        Binary,
        TokenType::Slash,
        TokenType::Star
    );

    fn unary(&mut self) -> Result<Rc<dyn Expr>, Error> {
        if self.is_match(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Rc::new(Unary { operator, right }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Rc<dyn Expr>, Error> {
        let mut expr = self.primary()?;
        loop {
            if self.is_match(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.is_match(vec![TokenType::Dot]) {
                let name = self.consume(
                    TokenType::Identifier("".to_string()),
                    "Expect property name after '.'",
                )?;
                expr = Rc::new(Get { object: expr, name });
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Rc<dyn Expr>) -> Result<Rc<dyn Expr>, Error> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            while {
                if arguments.len() >= 255 {
                    return Err(Error::report(
                        self.peek(),
                        "Can't have more than 255 arguments".to_string(),
                    ));
                }
                arguments.push(self.expression()?);
                self.is_match(vec![TokenType::Comma])
            } {}
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(Rc::new(Call {
            callee,
            arguments,
            paren,
        }))
    }

    fn primary(&mut self) -> Result<Rc<dyn Expr>, Error> {
        if self.is_match(vec![TokenType::False]) {
            Ok(Rc::new(Literal {
                value: self.previous(),
            }))
        } else if self.is_match(vec![TokenType::True]) {
            Ok(Rc::new(Literal {
                value: self.previous(),
            }))
        } else if self.is_match(vec![TokenType::Nil]) {
            Ok(Rc::new(Literal {
                value: self.previous(),
            }))
        } else if self.is_match(vec![
            TokenType::Number(0.0),
            TokenType::String("".to_string()),
        ]) {
            Ok(Rc::new(Literal {
                value: self.previous(),
            }))
        } else if self.is_match(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
            Ok(Rc::new(Grouping { expression: expr }))
        } else if self.is_match(vec![TokenType::Identifier("".to_string())]) {
            Ok(Rc::new(crate::ast::expr::VarExpr {
                name: self.previous(),
            }))
        } else if self.is_match(vec![TokenType::This]) {
            Ok(Rc::new(crate::ast::expr::This {
                keyword: self.previous(),
            }))
        } else {
            Err(Error::report(
                self.tokens[self.current].clone(),
                "Expect Expression".to_string(),
            ))
        }
    }

    /// synchronize the state of parser when error happens
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.tokens[self.current].token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    /// consume the current token if it is match to one of the given token types
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(Error::report(self.previous(), message.to_string()))
        }
    }

    /// check if the current token is match to one of the given token types
    fn is_match(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// check the current token is equal to the given token type
    fn check(&self, token_type: TokenType) -> bool {
        if !self.is_end() {
            self.tokens[self.current].token_type == token_type
        } else {
            false
        }
    }

    /// get the next token and add one to current
    fn advance(&mut self) -> Token {
        if !self.is_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// check if the current token is the last token
    fn is_end(&self) -> bool {
        self.tokens[self.current].token_type == TokenType::Eof
    }

    /// get the previous token
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
}
