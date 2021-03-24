use crate::{Lox, token::{Token, TokenType, Literal}};
use std::error;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(e) => {
                Some(e)
            },
            Err(parse_error) => {
                None
            }
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var |
                TokenType::For | TokenType::If | TokenType::While |
                TokenType::Print | TokenType::Return // any of these tokens probably means we're at the beginning of the next statement
                => {
                    return
                },
                _ => {}
            }

            self.advance();
        }
    }

    fn error(&self, token: Token, message: String) {
        Lox::token_error(token, message);
    }

    // ======== OPERATORS ========
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        let token_types = vec![TokenType::BangEqual, TokenType::EqualEqual];
        while self.match_(&token_types) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        let token_types = vec![TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual];
        while self.match_(&token_types) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        let token_types = vec![TokenType::Minus, TokenType::Plus];
        while self.match_(&token_types) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        let token_types = vec![TokenType::Slash, TokenType::Star];
        while self.match_(&token_types) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        let token_types = vec![TokenType::Bang, TokenType::Minus];
        if self.match_(&token_types) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_(&vec![TokenType::False]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.match_(&vec![TokenType::True]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }
        if self.match_(&vec![TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        }

        if self.match_(&vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal.ok_or_else(|| ParseError(self.peek().clone(), "".to_owned())).unwrap()));
        }

        if self.match_(&vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.".to_owned());
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        // As the parser descends through the parsing methods for each grammar rule, it eventually hits
        // primary(). If none of the above cases match, it means we are currently sitting on a
        // token that can't start an expression. We need to handle this error too.

        Err(ParseError(self.peek().clone(), "Expect expression.".to_owned()))
    }

    // ======== PRIMITIVE COMBINATORS ========
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        (&self.tokens[self.current-1]).clone()
    }

    fn at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn advance(&mut self) -> Token {
        if !self.at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.at_end() {
            false
        } else {
            self.peek().token_type == *token_type
        }
    }

    fn match_(&mut self, token_types: &Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<(), ParseError> {
        if self.check(&token_type) {
            self.advance();
        }

        Err(ParseError(self.peek().clone(), message))
    }
}

#[derive(Debug)]
struct ParseError(Token, String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.0.to_string(), self.1)
    }
}

impl error::Error for ParseError {}