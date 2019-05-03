
use crate::tokens::Token;
use crate::tokens::Literal;
use crate::interpreter::Result;
use std::collections::HashMap;
use crate::interpreter::InterpreterError;

pub struct Stack {
	envs: Vec<Environment>,
	current: Environment,
}

impl Stack {
	pub fn new() -> Stack {
		Stack {
			envs: Vec::new(),
			current: Environment::new()
		}
	}

	pub fn push_new(&mut self) {
		let old = std::mem::replace(&mut self.current, Environment::new());
		self.envs.push(old);
	}

	pub fn restore_old(&mut self) {
		let old = self.envs.pop();
		self.current = old.unwrap();
	}

	pub fn define(&mut self, name: String, value: Literal) {
		self.current.define(name, value)
	}

	pub fn assign(&mut self, name: &Token, value: Literal) -> Result<()> {
		if let Ok(_) = self.current.assign(name, value.clone()) {
			Ok(())
		} else {
			for item in self.envs.iter_mut().rev() {
				if let Ok(_) = item.assign(name,value.clone()) {
					return Ok(())
				}
			}

			Err(InterpreterError::new(name, " Undefined variable"))
		}
	}

	pub fn get(&self, tk: &Token) -> Result<Literal> {
		if let Ok(val) = self.current.get(tk) {
			Ok(val)
		} else {
			for item in self.envs.iter().rev() {
				if let Ok(val) = item.get(tk) {
					return Ok(val);
				}
			}

			Err(InterpreterError::new(tk,"Undefined variable"))
		}
	}

}

struct Environment {
	values: HashMap<String, Literal>
}



impl Environment {
	pub fn new() -> Environment {
		Environment {
			values: HashMap::new()
		}
	}

	pub fn define(&mut self, name: String, value: Literal) {
		self.values.insert(name, value);
	}

	pub fn get(&self, tk: &Token) -> Result<Literal> {
		if let Some(val) = self.values.get(tk.get_lexeme()) {
			Ok(val.clone())
		} else {
			Err(InterpreterError::new(tk,"Undefined variable"))
		}
	}

	pub fn assign(&mut self, name: &Token, value: Literal) -> Result<()> {
		if !self.values.contains_key(name.get_lexeme()) {
			Err(InterpreterError::new(name, " Undefined variable"))
		} else {
			self.values.insert(name.get_lexeme().to_owned(), value);
			Ok(())
		}
	}
}