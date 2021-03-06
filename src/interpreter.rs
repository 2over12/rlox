use crate::syntax::Stmt;
use crate::syntax::StmtVisitor;
use crate::syntax::ExprVisitor;
use crate::tokens::Literal;
use crate::syntax::Expr;
use crate::tokens::Token;
use crate::tokens::TokenType;
use crate::environment::Stack;
use crate::functions::Callable;
use crate::functions::LoxCalls;

pub struct Interpreter {
	env: Stack
}

pub struct InterpreterError {
	msg: String
}

impl InterpreterError {
	pub fn get_msg(&self) -> &str {
		&self.msg
	}

	pub fn new(tk: &Token, err: &str) -> InterpreterError {
		InterpreterError {
			msg: format!("Error: {}, at: '{}' on line {}", err,tk.get_lexeme(), tk.get_line())
		}
	}

	pub fn from(line: usize, lexeme: &str, msg: &str) -> InterpreterError {
		InterpreterError {
			msg: format!("Error: {}, at: '{}' on line {}",msg, lexeme, line)
		}
	}
}

pub enum RuntimeError {
	BreakSentinel,
	InterpreterError(InterpreterError)
}

impl RuntimeError {
	pub fn get_msg(&self) -> &str {
		match self {
			RuntimeError::BreakSentinel => "Break ran without encapsulating loop. Report this bug in the interpreter.",
			RuntimeError::InterpreterError(ie) => ie.get_msg()
		}
	}
}

pub type Result<T> = std::result::Result<T,RuntimeError>;

impl Interpreter {
	fn evaluate(&mut self, expr: &Expr) -> Result<Literal> {
		expr.accept(self)
	}

	fn execute(&mut self, stmt: &Stmt) -> Result<()> {
		stmt.accept(self)
	}
}

impl StmtVisitor<Result<()>> for &mut Interpreter {
	fn visit_print(self, expr: &Expr) -> Result<()> {
		let val = self.evaluate(expr)?;
		println!("{}", val.to_string());
		Ok(())
	}

	fn visit_break(self, _line: usize) -> Result<()> {
		Err(RuntimeError::BreakSentinel)
	}

	fn visit_while(self,cond: &Expr, then: &Stmt) -> Result<()> {
		while is_truthy(&(self.evaluate(cond)?)) {
			let res = self.execute(then);
			if let Err(RuntimeError::BreakSentinel) =  res {
				break;
			} else {
				res?;
			}
		}

		Ok(())
	}

	fn visit_if(self, cond: &Expr,then: &Stmt, otherwise: &Option<Stmt>) -> Result<()> {
		let cond = self.evaluate(cond)?;

		if is_truthy(&cond) {
			self.execute(then)
		} else {
			if let Some(otherwise) = otherwise {
				self.execute(otherwise)
			} else {
				Ok(())
			}
		}
	}

	fn visit_block_stmt(self, stmts: &Vec<Stmt>) -> Result<()> {
		self.env.push_new();

		for st in stmts {
			let res = self.execute(st);

			if let Err(_) = res {
				self.env.restore_old();
				return res;
			}
		}
		
		self.env.restore_old();
		Ok(())
		
	}

	fn visit_variable(self, name: &Token, init: &Option<Expr>) -> Result<()> {
		let value = if let Some(init) = init {
			Some(self.evaluate(init)?)
		} else {
			None
		};

		self.env.define(name.get_lexeme().to_owned(), value);
		Ok(())
	}

	fn visit_expr_statement(self, expr: &Expr) -> Result<()> {
		self.evaluate(expr)?;
		Ok(())
	}

}


impl ExprVisitor<Result<Literal>> for &mut Interpreter {

	fn visit_call(self, callee: &Expr, tk: &Token, args: &Vec<Expr>) -> Result<Literal> {
		let callee = self.evaluate(callee)?;

		let args = args.into_iter().map(|x|self.evaluate(x)).collect::<Result<Vec<_>>>();

		let func = Callable::from(callee)?;
		func.call(self, args)
	}

	fn visit_literal(self, ltrl: &Literal) -> Result<Literal> {
		Ok(ltrl.clone())
	}

	fn visit_logical(self, left: &Expr, op: &Token, right: &Expr) -> Result<Literal> {
		let left = self.evaluate(left)?;

		if let TokenType::Or = op.get_type() {
			if is_truthy(&left) {
				return Ok(left);
			}
		} else {
			if !is_truthy(&left) {
				return Ok(left);
			}
		}

		self.evaluate(right)
	}


	fn visit_variable_expr(self, name: &Token) -> Result<Literal> {
		self.env.get(name)
	}

	fn visit_grouping(self, exp: &Expr) -> Result<Literal> {
		exp.accept(self)
	}

	fn visit_unary(self, op: &Token, exp: &Expr) -> Result<Literal> {
		let right = exp.accept(self)?;
		match op.get_type() {
			TokenType::Minus => {
				let x = unpack_number(right, op)?;
				Ok(Literal::Number(-x))
			},
			TokenType::Bang => Ok(Literal::Boolean(!is_truthy(&right))),
			_ => unreachable!()
		}
	}

	fn visit_assignment(self, name: &Token, value: &Expr) -> Result<Literal> {
		let value = self.evaluate(value)?;

		self.env.assign(name, value.clone())?;
		Ok(value)
	}


	fn visit_ternary(self, _op: &Token, left: &Expr, middle: &Expr, right: &Expr) -> Result<Literal> {
		let left = self.evaluate(left)?;
		if is_truthy(&left) {
			middle.accept(self)
		} else {
			right.accept(self)
		}
	}

	fn visit_binary(self, left: &Expr, op: &Token, right: &Expr) -> Result<Literal> {
		let left = self.evaluate(left)?;
		let right = self.evaluate(right)?;

		match op.get_type() {
			TokenType::Minus | TokenType::Slash | TokenType::Star | TokenType::Greater |
			TokenType::GreaterEqual | TokenType::LessEqual | TokenType::Less => {
				let left = unpack_number(left,op)?;
				let right = unpack_number(right,op)?;
				match op.get_type() {
					TokenType::Minus => Ok(Literal::Number(left - right)),
					TokenType::Slash => if right == 0.0 {
						Err(RuntimeError::InterpreterError(InterpreterError::new(op, "Division by zero")))
					} else {
						Ok(Literal::Number(left / right))
					},
					TokenType::Star => Ok(Literal::Number(left * right)),
					TokenType::Greater => Ok(Literal::Boolean(left > right)),
					TokenType::GreaterEqual => Ok(Literal::Boolean(left >= right)),
					TokenType::Less => Ok(Literal::Boolean(left < right)),
					TokenType::LessEqual => Ok(Literal::Boolean(left <= right)),
					_ => unreachable!()
				}
			},
			TokenType::Plus => {
				let nl = unpack_number(left.clone(), op);
				let nr = unpack_number(right.clone(), op);
				if let (Ok(nl),Ok(nr)) = (nl,nr) {
					Ok(Literal::Number(nl+nr))
				} else {
					let mut sl = unpack_into_string(left, op)?;
					let sr = unpack_into_string(right, op)?;
					sl.push_str(&sr);
					Ok(Literal::String(sl))
				}
			},
			TokenType::EqualEqual => Ok(Literal::Boolean(is_equal(&left,&right))),
			TokenType::BangEqual => Ok(Literal::Boolean(!is_equal(&left,&right))),
			_ => unreachable!()
		}

	}
}

fn is_equal(f: &Literal, s: &Literal) -> bool {
	match f {
		Literal::Number(f) => if let Literal::Number(s) = s {
			f == s
		} else {
			false
		},
		Literal::Boolean(f) => if let Literal::Boolean(s) = s {
			f == s
		} else {
			false
		},
		Literal::String(f) => if let Literal::String(s) = s {
			f == s
		} else {
			false
		},
		Literal::Nil => f==s
	}
}

fn is_truthy(ltl: &Literal) -> bool {
	match ltl {
		Literal::Nil => false,
		Literal::Boolean(x) => *x,
		__ => true
	}
}

fn unpack_number(ltl: Literal, tk: &Token) -> Result<f64> {
	match ltl {
		Literal::Number(x) => Ok(x),
		_ => Err(RuntimeError::InterpreterError(InterpreterError::new(tk, "Expected number"))),
	}
}

fn unpack_string(ltl: Literal, tk: &Token) -> Result<String> {
	match ltl {
		Literal::String(x) => Ok(x),
		_ => Err(RuntimeError::InterpreterError(<InterpreterError>::new(tk, "Expected String"))),
	}
}

fn unpack_into_string(ltl: Literal, tk: &Token) -> Result<String> {
	match ltl {
		Literal::String(x) => Ok(x),
		Literal::Number(x) => Ok(x.to_string()),
		_ => Err(RuntimeError::InterpreterError(<InterpreterError>::new(tk, "Expected value that can be a String"))),
	}
}

fn unpack_bool(ltl: Literal, tk: &Token) -> Result<bool> {
	match ltl {
		Literal::Boolean(x) => Ok(x),
		_ => Err(RuntimeError::InterpreterError(<InterpreterError>::new(tk, "Expected boolean"))),
	}
}

pub fn interpret(statements: &Vec<Stmt>) -> Result<()> {
	let mut visit = Interpreter {
		env: Stack::new()
	};
	
	for stmt in statements {
		visit.execute(stmt)?;
	}

	Ok(())
}