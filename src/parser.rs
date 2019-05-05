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

    	let mut init = None;
    	if self.curr_match(&vec![TokenType::Equal]) {
    		init = Some(self.expression()?);
    	}

    	self.consume(TokenType::Semicolon, "Expected ';' after the variable declaration")?;
    	Ok(Stmt::Var(name,init))
    }


    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;
        let init = if self.curr_match(&vec![TokenType::Semicolon]) {
            None
        } else if self.curr_match(&vec![TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let cond = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after loop condition")?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expected ')' after for clauses.")?;

        let body = self.statement()?;

        let body = if let Some(increment) = increment {
            Stmt::Block(vec![body,Stmt::Expr(increment)])
        } else {
            body
        };

        let cond = Box::new(cond.unwrap_or(Expr::Literal(Literal::Boolean(true))));
        let body = Box::new(body);

        let wstmt = Stmt::While(cond,body);

        let full_stmt = if let Some(init) = init {
            Stmt::Block(vec![init,wstmt])
        } else {
            wstmt
        };

        Ok(full_stmt)
    }

    fn statement(&mut self) -> Result<Stmt> {
    	if self.curr_match(&vec![TokenType::Print]) {
    		return self.print_statement()
    	} else if self.curr_match(&vec![TokenType::LeftBrace]) {
            return self.block()
        } else if self.curr_match(&vec![TokenType::If]) {
            return self.if_statement()
        } else if self.curr_match(&vec![TokenType::While]) {
            self.while_statement()
        } else if self.curr_match(&vec![TokenType::For]) {
            self.for_statement()
        } else if self.curr_match(&vec![TokenType::Break]) {
            self.break_statement()
        }
         else {
    		return self.expression_statement()
    	}
    }


    fn break_statement(&mut self) -> Result<Stmt> {
        let line = self.previous().unwrap().get_line();
        self.consume(TokenType::Semicolon, "Expected ';' after break.")?;
        Ok(Stmt::Break(line))
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after while")?;
        let cond = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;
        let then = Box::new(self.statement()?);
        Ok(Stmt::While(cond,then))
    }


    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after if")?;
        let cond = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;
        let then = Box::new(self.statement()?);

        Ok(Stmt::If(cond,then, Box::new(if self.curr_match(&vec![TokenType::Else]) {
            let otherwise = self.statement()?;
            Some(otherwise)
        } else {
            None
        })))

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
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary();

        while true {
            if self.curr_match(&vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr> {
        let mut args = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {

                 if args.len() >= 8 {
                    self.error(self.peek().unwrap(), "Cannot have more than 8 arguments.");
                }

                args.push(self.expression()?);
                if !self.curr_match(&vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let token = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;
        Ok(Expr::Call(Box::new(expr), token, args))
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

    
    fn logic_and(&mut self) -> Result<Expr> {
        self.match_two_operand(vec![TokenType::And], |x: &mut Parser| x.equality(),|left,op,right| Expr::Logical(left,op,right))
    }

    fn logic_or(&mut self) -> Result<Expr> {
        self.match_two_operand(vec![TokenType::Or], |x: &mut Parser| x.logic_and(), |left,op,right| Expr::Logical(left,op,right))
    }

    fn ternary(&mut self) -> Result<Expr> {
    	let left = self.logic_or()?;

    	if self.curr_match(&vec![TokenType::QuestionMark]) {
    		let tk = self.previous.take().unwrap();
    		let t_cond = self.logic_or()?;
    		self.consume(TokenType::Colon, "Expected to find ':' after expr")?;
    		let f_cond = self.logic_or()?;
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
        self.match_two_operand(matchees, higher_precedence, |left,op,right| Expr::Binary(left,op,right))
    }

    fn match_two_operand<T: Fn(&mut Parser) -> Result<Expr>, V: Fn(Box<Expr>,Token,Box<Expr>) -> Expr>(
        &mut self,
        matchees: Vec<TokenType>,
        higher_precedence: T,
        combinator: V,
    ) -> Result<Expr> {
        let mut expr = higher_precedence(self)?;

        while self.curr_match(&matchees) {
            let op = self.previous().unwrap();
            let right = higher_precedence(self)?;
            expr = combinator(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }
}

