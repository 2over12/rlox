use crate::syntax::Expr;
use crate::interpreter::Interpreter;
use crate::tokens::Literal;
use crate::interpreter::Result;

pub enum Callable {
	Static(StaticFunction)
}

impl Callable {
	pub fn from(expr: &Expr) -> Result<Callable> {

	}
}


impl LoxCalls for Callable {
	fn call(&self, intepreter: &mut Interpreter, args: Vec<Literal>) -> Result<Literal> {

	}

	fn arity(&self) -> usize {
		1
	}
}

pub trait LoxCalls {
	fn call(&self, intepreter: &mut Interpreter, args: Vec<Literal>) -> Result<Literal>;
	fn arity(&self) -> usize;
}

struct StaticFunction {

}