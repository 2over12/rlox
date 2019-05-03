use super::ErrorReporter;

use std::collections::HashMap;
use std::string::ToString;


lazy_static! {
  static ref KEYWORD_MAP: HashMap<&'static str, TokenType> = {
    let mut m = HashMap::new();
    m.insert("and", TokenType::And);
    m.insert("class", TokenType::Class);
    m.insert("else", TokenType::Else);
    m.insert("false", TokenType::Literal(Literal::Boolean(false)));
    m.insert("for", TokenType::For);
    m.insert("fun", TokenType::Fun);
    m.insert("if", TokenType::If);
    m.insert("nil", TokenType::Literal(Literal::Nil));
    m.insert("or", TokenType::Or);
    m.insert("print", TokenType::Print);
    m.insert("return", TokenType::Return);
    m.insert("super", TokenType::Super);
    m.insert("this", TokenType::This);
    m.insert("true", TokenType::Literal(Literal::Boolean(true)));
    m.insert("var", TokenType::Var);
    m.insert("while", TokenType::While);
    m
  };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
  // Single Character
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,
  QuestionMark,
  Colon,

  // One or two character tokens.
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,

  // Literals.
  Identifier,
  Literal(Literal),

  // Keywords.
  And,
  Class,
  Else,
  Fun,
  For,
  If,
  Or,
  Print,
  Return,
  Super,
  This,
  Var,
  While,

  Eof,
}

#[derive(Debug, Clone)]
pub enum Literal {
  Number(f64),
  String(String),
  Nil,
  Boolean(bool)
}


impl std::cmp::Eq for Literal {

}

impl std::cmp::PartialEq for Literal {
  fn eq(&self, other: &Literal) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(other)
  }
}

impl ToString for Literal {
    fn to_string(&self) -> String {
      match self {
        Literal::Number(val) => val.to_string(),
        Literal::String(s) => s.clone(),
        Literal::Boolean(t) => t.to_string(),
        Literal::Nil => "nil".to_owned(),
      }
    }
}

impl ToString for Token {
  fn to_string(&self) -> String {
    self.lexeme.clone()
  }
}

#[derive(Debug,Clone)]
pub struct Token {
  t_type: TokenType,
  line: usize,
  lexeme: String,
}

impl Token {
  pub fn new(tk: TokenType, lexeme: String, line: usize) -> Token {
    Token {
      t_type: tk,
      line,
      lexeme,
    }
  }

  pub fn get_type(&self) -> &TokenType {
    &self.t_type
  }

  pub fn get_line(&self) -> usize {
    self.line
  }

  pub fn get_lexeme(&self) -> &str {
    &self.lexeme
  }
}

fn is_alpha(c: char) -> bool {
  (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
  is_digit(c) || is_alpha(c)
}

fn is_digit(c: char) -> bool {
  c >= '0' && c <= '9'
}

pub struct Scanner<'a> {
  src: String,
  tokens: Vec<Token>,
  start: usize,
  current: usize,
  line: usize,
  err_rep: &'a mut ErrorReporter,
}

impl<'a> Scanner<'a> {
  pub fn new(src: String, err_hand: &'a mut ErrorReporter) -> Scanner {
    Scanner {
      line: 1,
      current: 0,
      start: 0,
      src,
      tokens: Vec::new(),
      err_rep: err_hand,
    }
  }

  pub fn scan_tokens(mut self) -> Vec<Token> {
    while !self.is_at_end() {
      self.start = self.current;
      self.grab_token();
    }

    self
      .tokens
      .push(Token::new(TokenType::Eof, "".to_owned(), self.line));
    self.tokens
  }

  fn is_at_end(&self) -> bool {
    self.current >= self.src.len()
  }

  fn add_token(&mut self, t_type: TokenType) {
    let text = self.src[self.start..self.current].to_owned();
    self.tokens.push(Token::new(t_type, text, self.line))
  }

  fn grab_token(&mut self) {
    let next = self.advance().unwrap();

    match next {
      '(' => self.add_token(TokenType::LeftParen),
      ')' => self.add_token(TokenType::RightParen),
      '{' => self.add_token(TokenType::LeftBrace),
      '}' => self.add_token(TokenType::RightBrace),
      ',' => self.add_token(TokenType::Comma),
      '.' => self.add_token(TokenType::Dot),
      '-' => self.add_token(TokenType::Minus),
      '+' => self.add_token(TokenType::Plus),
      ';' => self.add_token(TokenType::Semicolon),
      '*' => self.add_token(TokenType::Star),
      '?' => self.add_token(TokenType::QuestionMark),
      ':' => self.add_token(TokenType::Colon),
      '!' => {
        let tk = if self.match_char('=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };
        self.add_token(tk)
      }
      '=' => {
        let tk = if self.match_char('=') {
          TokenType::EqualEqual
        } else {
          TokenType::Equal
        };
        self.add_token(tk)
      }
      '<' => {
        let tk = if self.match_char('=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };
        self.add_token(tk)
      }
      '>' => {
        let tk = if self.match_char('=') {
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        };
        self.add_token(tk);
      }
      '/' => {
        if self.match_char('/') {
          while self.get_current_char() != Some('\n') && !self.is_at_end() {
            self.advance();
          }
        } else if self.match_char('*') {
          let mut term = 1;
          while let (Some(curr), Some(next)) = (self.get_current_char(), self.peek_next()) {
            if curr == '*' && next == '/' {
              term -= 1;
              self.advance();
              self.advance();
            } else if curr == '/' && next == '*' {
              term += 1;
              self.advance();
              self.advance();
            } else if curr == '\n' {
              self.line += 1;
            } else {
              self.advance();
            };

            if term == 0 {
              break;
            }
          }

          if term != 0 {
            self.err_rep.error(self.line, "Unclosed block comment.")
          }
        } else {
          self.add_token(TokenType::Slash);
        }
      }
      ' ' | '\r' | '\t' => (),
      '\n' => {
        self.line += 1;
        ()
      }
      '"' => self.string(),
      '0'...'9' => self.number(),
      x if is_alpha(x) => self.identifier(),
      _ => self.err_rep.error(self.line, "Unexpected character."),
    }
  }

  fn identifier(&mut self) {
    while let Some(x) = self.get_current_char() {
      if !is_alpha_numeric(x) {
        break;
      } else {
        self.advance();
      }
    }

    let name = &self.src[self.start..self.current];

    if let Some(tok) = KEYWORD_MAP.get(name) {
      self.add_token(tok.clone());
    } else {
      self.add_token(TokenType::Identifier)
    }
  }

  fn number(&mut self) {
    while let Some(x) = self.get_current_char() {
      if !is_digit(x) {
        break;
      } else {
        self.advance();
      }
    }

    if let (Some(curr), Some(next)) = (self.get_current_char(), self.peek_next()) {
      if curr == '.' && is_digit(next) {
        self.advance();
        while let Some(x) = self.get_current_char() {
          if !is_digit(x) {
            break;
          } else {
            self.advance();
          }
        }
      }
    }

    self.add_token(TokenType::Literal(Literal::Number(
      self.src[self.start..self.current].parse().unwrap(),
    )));
  }

  fn string(&mut self) {
    let mut terminated = false;

    while let Some(next_char) = self.advance() {
      if next_char == '"' {
        terminated = true;
        break;
      } else if next_char == '\n' {
        self.line += 1;
      }
    }

    if !terminated {
      self.err_rep.error(self.line, "Unterminated string.");
      return;
    }

    let value = self.src[self.start + 1..self.current - 1].to_owned();
    self.add_token(TokenType::Literal(Literal::String(value)));
  }

  fn match_char(&mut self, c: char) -> bool {
    if let Some(x) = self.get_current_char() {
      if x == c {
        self.advance();
        true 
      } else {
        false
      }
    } else {
      false
    }
  }

  fn get_current_char(&self) -> Option<char> {
    self.src[(self.current)..].chars().next()
  }

  fn peek_next(&self) -> Option<char> {
    self.src[self.current + 1..].chars().next()
  }

  fn advance(&mut self) -> Option<char> {
    let c = self.get_current_char();
    self.current += 1;
    c
  }
}
