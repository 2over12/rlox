use crate::syntax::Stmt;
use crate::syntax::Expr;
use crate::tokens::Literal;
use crate::tokens::Token;
use crate::tokens::TokenType;
use crate::ErrorReporter;
use std::collections::VecDeque;

pub struct Parser<'a> {
    tokens: VecDeque<Token>,
    previous: Option<Token>,
    err_rep: &'a mut ErrorReporter,
}

pub struct ParserError;

type Result<T> = std::result::Result<T, ParserError>;

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, err_rep: &mut ErrorReporter) -> Parser {
        Parser {
            tokens: VecDeque::from(tokens),
            previous: None,
            err_rep,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.tokens.is_empty() && !self.curr_match(&vec![TokenType::Eof]){
        	stmts.push(self.declaration()?);
        }

        Ok(stmts)
    }

    fn declaration(&mut self) -> Result<Stmt> {
    	let res = if self.curr_match(&vec![TokenType::Var]) {
    		self.var_declaration()
    	} else {
    		self.statement()
    	};

    	if let Err(_) = res {
    		self.synchronize();
    	}

    	res
    }


    fn var_declaration(&mut self) -> Result<Stmt> {
    	let name = self.consume(TokenType::Identifier, "Expected variable name.")?;

    	let mut init = Expr::Literal(Literal::Nil);
    	if self.curr_match(&vec![TokenType::Equal]) {
    		init = self.expression()?;
    	}

    	self.consume(TokenType::Semicolon, "Expected ';' after the variable declaration")?;
    	Ok(Stmt::Var(name,init))
    }

    fn statement(&mut self) -> Result<Stmt> {
    	if self.curr_match(&vec![TokenType::Print]) {
    		return self.print_statement()
    	} else if self.curr_match(&vec![TokenType::LeftBrace]) {
            return self.block()
        } else {
    		return self.expression_statement()
    	}
    }

    fn block(&mut self) -> Result<Stmt> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }
        self.consume(TokenType::RightBrace, "Expected '}' ater block.")?;
        Ok(Stmt::Block(statements))
    }


    fn is_at_end(&self) -> bool {
        if let Some(_) = self.peek() {
            false
        } else {
            true
        }
    }
    fn print_statement(&mut self) -> Result<Stmt> {
    	let value = self.expression()?;
    	self.consume(TokenType::Semicolon, "Expected ';' after value")?;
    	Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
    	let value = self.expression()?;
    	self.consume(TokenType::Semicolon, "Expected ';' after value")?;
    	Ok(Stmt::Expr(value))
    }

    fn expression(&mut self) -> Result<Expr> {
        return self.assignment();
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.comma()?;

        if self.curr_match(&vec![TokenType::Equal]) {
            let equals = self.previous().unwrap();
            let value = self.assignment()?;

            if let Expr::Var(nm) = expr {
                let lval = Expr::Assignment(nm,Box::new(value));
                return Ok(lval);
            }

            self.error(&equals, "Invalid assignment target.");
        }
        
        Ok(expr)
    }

    fn curr_match(&mut self, tokes: &Vec<TokenType>) -> bool {
        for ty in tokes {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn is_ident(&mut self) -> bool {
        if let Some(x) = self.peek() {
            if let TokenType::Identifier = x.get_type() {
                self.advance();
                return true;
            }
        }

        false
    }

    fn is_literal(&mut self) -> bool {
        if let Some(x) = self.peek() {
            if std::mem::discriminant(&TokenType::Literal(Literal::Nil))
                == std::mem::discriminant(x.get_type())
            {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, ty: &TokenType) -> bool {
        if let Some(x) = self.peek() {
            return ty == x.get_type();
        }

        false
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn advance(&mut self) -> Option<Token> {
        self.previous = self.tokens.pop_front();
        self.previous.clone()
    }

    fn previous(&mut self) -> Option<Token> {
        self.previous.take()
    }

    fn comparison(&mut self) -> Result<Expr> {
        self.match_left_asoc(
            vec![
                TokenType::Less,
                TokenType::LessEqual,
                TokenType::Greater,
                TokenType::GreaterEqual,
            ],
            |x| x.addition(),
        )
    }

    fn addition(&mut self) -> Result<Expr> {
        self.match_left_asoc(vec![TokenType::Plus, TokenType::Minus], |x| {
            x.multiplication()
        })
    }

    fn multiplication(&mut self) -> Result<Expr> {
        self.match_left_asoc(vec![TokenType::Star, TokenType::Slash], |x| x.unary())
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.curr_match(&vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().unwrap();
            let right = self.unary()?;
            Ok(Expr::Unary(op, Box::new(right)))
        } else if self.curr_match(&vec![TokenType::EqualEqual,TokenType::BangEqual,TokenType::Plus,TokenType::Minus,
        	TokenType::LessEqual, TokenType::Less, TokenType::GreaterEqual, TokenType::Greater, TokenType::Star, TokenType::Slash]){
        	let op = self.previous().unwrap();
        	self.error(&op, "Expression expected before binary operator");
            let right = self.unary()?;
            Ok(right)
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.is_literal() {
            if let TokenType::Literal(lt) = self.previous().unwrap().get_type().clone() {
                return Ok(Expr::Literal(lt));
            }
        }

        if self.is_ident() {
            return Ok(Expr::Var(self.previous().unwrap()))
        }

        if self.curr_match(&vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expr")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        let u_tk = &self.peek().unwrap().clone();
        self.error(u_tk, "Unexpected token");
        Err(ParserError)
    }

    fn synchronize(&mut self) {
        self.advance();

        while let Some(_) = self.peek() {
            if let Some(y) = &self.previous {
                if let TokenType::Semicolon = y.get_type() {
                    return;
                }
            }

            match self.peek().unwrap().get_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &'static str) -> Result<Token> {
        if self.check(&ty) {
            return Ok(self.advance().unwrap());
        } else {
        	let errored_tok = self.peek().unwrap().clone();
            self.error(&errored_tok, msg);
            Err(ParserError)
        }
    }

    fn error(&mut self, token: &Token, msg: &'static str) {
        if let TokenType::Eof = token.get_type() {
            self.err_rep.report(token.get_line(), "at end", msg)
        } else {
            self.err_rep
                .report(token.get_line(), token.get_lexeme(), msg)
        }
    }

    fn equality(&mut self) -> Result<Expr> {
        self.match_left_asoc(
            vec![TokenType::BangEqual, TokenType::EqualEqual],
            |x: &mut Parser| x.comparison(),
        )
    }


    fn ternary(&mut self) -> Result<Expr> {
    	let left = self.equality()?;

    	if self.curr_match(&vec![TokenType::QuestionMark]) {
    		let tk = self.previous.take().unwrap();
    		let t_cond = self.equality()?;
    		self.consume(TokenType::Colon, "Expected to find ':' after expr")?;
    		let f_cond = self.equality()?;
    		Ok(Expr::Ternary(tk,Box::new(left), Box::new(t_cond), Box::new(f_cond)))

    	} else {
    		Ok(left)
    	}	
    }

    fn comma(&mut self) -> Result<Expr> {
        self.match_left_asoc(
            vec![TokenType::Comma],
            |x: &mut Parser| x.ternary(),
        )
    }

    fn match_left_asoc<T: Fn(&mut Parser) -> Result<Expr>>(
        &mut self,
        matchees: Vec<TokenType>,
        higher_precedence: T,
    ) -> Result<Expr> {
        let mut expr = higher_precedence(self)?;

        while self.curr_match(&matchees) {
            let op = self.previous().unwrap();
            let right = higher_precedence(self)?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }
}
