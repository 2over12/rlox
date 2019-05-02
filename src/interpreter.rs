use crate::syntax::Stmt;
use crate::syntax::StmtVisitor;
use crate::syntax::ExprVisitor;
use crate::tokens::Literal;
use crate::syntax::Expr;
use crate::tokens::Token;
use crate::tokens::TokenType;

struct Interpreter;

pub struct InterpreterError {
	msg: String
}

impl InterpreterError {
	pub fn get_msg(&self) -> &str {
		&self.msg
	}

	fn new(tk: &Token, err: &str) -> InterpreterError {
		InterpreterError {
			msg: format!("Error: {}, at: '{}' on line {}", err,tk.get_lexeme(), tk.get_line())
		}
	}
}

type Result<T> = std::result::Result<T,InterpreterError>;

impl Interpreter {
	fn evaluate(&self, expr: &Expr) -> Result<Literal> {
		expr.accept(self)
	}

	fn execute(&self, stmt: &Stmt) -> Result<()> {
		stmt.accept(self)
	}
}

impl StmtVisitor<Result<()>> for &Interpreter {
	fn visit_print(self, expr: &Expr) -> Result<()> {
		let val = self.evaluate(expr)?;
		println!("{}", val.to_string());
		Ok(())
	}

	fn visit_expr_statement(self, expr: &Expr) -> Result<()> {
		self.evaluate(expr)?;
		Ok(())
	}

}


impl ExprVisitor<Result<Literal>> for &Interpreter {

	fn visit_literal(self, ltrl: &Literal) -> Result<Literal> {
		Ok(ltrl.clone())
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


	fn visit_ternary(self, op: &Token, left: &Expr, middle: &Expr, right: &Expr) -> Result<Literal> {
		let left = left.accept(self)?;
		let left = unpack_bool(left, op)?;
		if left {
			middle.accept(self)
		} else {
			right.accept(self)
		}
	}

	fn visit_binary(self, left: &Expr, op: &Token, right: &Expr) -> Result<Literal> {
		let left = left.accept(self)?;
		let right = right.accept(self)?;

		match op.get_type() {
			TokenType::Minus | TokenType::Slash | TokenType::Star | TokenType::Greater |
			TokenType::GreaterEqual | TokenType::LessEqual | TokenType::Less => {
				let left = unpack_number(left,op)?;
				let right = unpack_number(right,op)?;
				match op.get_type() {
					TokenType::Minus => Ok(Literal::Number(left - right)),
					TokenType::Slash => if right == 0.0 {
						Err(InterpreterError::new(op, "Division by zero"))
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
		Literal::Identifier => f==s,
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
		_ => Err(InterpreterError::new(tk, "Expected number")),
	}
}

fn unpack_string(ltl: Literal, tk: &Token) -> Result<String> {
	match ltl {
		Literal::String(x) => Ok(x),
		_ => Err(InterpreterError::new(tk, "Expected String")),
	}
}

fn unpack_into_string(ltl: Literal, tk: &Token) -> Result<String> {
	match ltl {
		Literal::String(x) => Ok(x),
		Literal::Number(x) => Ok(x.to_string()),
		_ => Err(InterpreterError::new(tk, "Expected value that can be a String")),
	}
}

fn unpack_bool(ltl: Literal, tk: &Token) -> Result<bool> {
	match ltl {
		Literal::Boolean(x) => Ok(x),
		_ => Err(InterpreterError::new(tk, "Expected boolean")),
	}
}

pub fn interpret(statements: &Vec<Stmt>) -> Result<()> {
	let visit = Interpreter;
	
	for stmt in statements {
		visit.execute(stmt)?;
	}

	Ok(())
}