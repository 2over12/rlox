#[macro_use]
extern crate lazy_static;

mod tokens;
mod syntax;
mod parser;
mod interpreter;
mod environment;
mod context;

use std::io::Write;


use parser::Parser;

use tokens::Scanner;

pub struct ErrorReporter {
	had_error: bool
}

impl ErrorReporter {

	pub fn new() -> ErrorReporter {
		ErrorReporter {
			had_error: false
		}
	}

	fn error(&mut self, line: usize, message: &str) {
		self.report(line,"",message)
	}

	fn report(&mut self,line: usize, place: &str, msg: &str) {
		eprintln!("[line {}] Error {}: {}",line,place,msg);
		self.had_error = true;
	}

}
pub fn run(src: String) {
	let mut err_hand = ErrorReporter::new();

	let scanner = Scanner::new(src,&mut err_hand);
	let tokens = scanner.scan_tokens();
	let mut parser = Parser::new(tokens,&mut err_hand);
	let stmts = parser.parse();

	if let Ok(stmts) = &stmts {
		let context_errors = context::check(stmts);
		for err in context_errors {
			err.report(&mut err_hand)
		}
	}
	
	match stmts {
		Ok(ref stmts) if !err_hand.had_error => {
			let res = interpreter::interpret(stmts);
			match res {
				Err(er) => eprintln!("{}",er.get_msg()),
				Ok(_) => (),
			}
		},
		_ => { std::io::stderr().flush().unwrap(); return;},
	}

}