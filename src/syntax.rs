use crate::tokens::Token;
use crate::tokens::Literal;

#[derive(Debug)]
pub enum Expr {
	Binary(Box<Expr>, Token, Box<Expr>),
	Ternary(Token, Box<Expr>,Box<Expr>,Box<Expr>),
	Grouping(Box<Expr>),
	Literal(Literal),
	Var(Token),
	Unary(Token, Box<Expr>),
	Assignment(Token, Box<Expr>),
	Logical(Box<Expr>, Token, Box<Expr>),
	Call(Box<Expr>, Token, Vec<Expr>),
}


#[derive(Debug)]
pub enum Stmt {
	Print(Expr),
	Expr(Expr),
	Var(Token, Option<Expr>),
	Block(Vec<Stmt>),
	If(Box<Expr>, Box<Stmt>, Box<Option<Stmt>>),
	While(Box<Expr>, Box<Stmt>),
	Break(usize)
}

impl Stmt {
	pub fn accept<R>(&self, visitor: impl StmtVisitor<R>) -> R {
		match self {
			Stmt::Print(exp) => visitor.visit_print(exp),
			Stmt::Expr(exp) => visitor.visit_expr_statement(exp),
			Stmt::Var(name,expr) => visitor.visit_variable(name, expr),
			Stmt::Block(stmts) => visitor.visit_block_stmt(stmts),
			Stmt::If(cond, then, otherwise) => visitor.visit_if(cond,then,otherwise),
			Stmt::While(cond, then) => visitor.visit_while(cond,then),
			Stmt::Break(line) => visitor.visit_break(*line)
		}
	}
}

pub trait StmtVisitor <R> {
	fn visit_print(self, expr: &Expr) -> R;
	fn visit_expr_statement(self, expr: &Expr) -> R;
	fn visit_variable(self, name: &Token, expr: &Option<Expr>) -> R;
	fn visit_block_stmt(self,stmts: &Vec<Stmt>) -> R; 
	fn visit_if(self, cond: &Expr, then: &Stmt, otherwise: &Option<Stmt>) -> R;
	fn visit_while(self, cond: &Expr, then: &Stmt) -> R;
	fn visit_break(self, line: usize) -> R;
}

pub trait ExprVisitor <R> {
	fn visit_binary(self,left: &Expr, op: &Token, right: &Expr) -> R;
	fn visit_grouping(self,exp: &Expr) -> R;
	fn visit_literal(self,lit: &Literal) -> R;
	fn visit_unary(self,op: &Token, exp: &Expr) -> R;
	fn visit_ternary(self, op: &Token, left: &Expr, middle: &Expr, right: &Expr) -> R;
	fn visit_assignment(self, name: &Token, value: &Expr) -> R;
	fn visit_variable_expr(self, name: &Token) -> R;
	fn visit_logical(self, left: &Expr, op: &Token, right: &Expr) -> R;
	fn visit_call(self, callee: &Expr, paren: &Token, args: &Vec<Expr>) -> R;
}

impl Expr {
	pub fn accept<R,T: ExprVisitor<R>>(&self,visitor: T) -> R {
		match self {
			Expr::Binary(exp,op,exp2) => visitor.visit_binary(exp, op, exp2),
			Expr::Grouping(exp) => visitor.visit_grouping(exp),
			Expr::Literal(lt) => visitor.visit_literal(lt),
			Expr::Unary(op, exp) =>  visitor.visit_unary(op, exp),
			Expr::Ternary(op, left, middle, right) => visitor.visit_ternary(op, left, middle, right),
			Expr::Var(nm) => visitor.visit_variable_expr(nm),
			Expr::Assignment(nm, val) => visitor.visit_assignment(nm, val),
			Expr::Logical(left,op,right) => visitor.visit_logical(left, op, right),
			Expr::Call(callee, paren, args) => visitor.visit_call(callee,paren,args)
		}
	} 
}

pub struct PrettyPrint;


impl ExprVisitor<String> for &PrettyPrint {
	fn visit_binary(self,left: &Expr, op: &Token, right: &Expr) -> String {
		let mut total = String::new();
		total.push('(');
		total.push_str(&op.to_string());
		total.push(' ');
		total.push_str(&left.accept(self));
		total.push(' ');
		total.push_str(&right.accept(self));
		total.push(')');
		total
	}

	fn visit_call(self,left: &Expr, paren: &Token, args: &Vec<Expr>) -> String {
		let mut total = String::new();
		total.push('(');
		total.push_str(&left.accept(self));
		total.push(' ');

		for xp in args.iter() {
			total.push_str(&xp.accept(self));
			total.push(' ');
		}

		total
	}

	fn visit_variable_expr(self, name: &Token) -> String {
		name.get_lexeme().to_owned()
	}

	fn visit_assignment(self, name: &Token, value: &Expr) -> String {
		let mut total = String::new();
		total.push_str("(=");
		total.push_str(&name.get_lexeme());
		total.push(' ');
		total.push_str(&value.accept(self));
		total.push(')');
		total
	}

	fn visit_logical(self, left: &Expr, op: &Token, right: &Expr) -> String {
		let mut total = String::new();
		total.push('(');
		total.push_str(&op.get_lexeme());
		total.push(' ');
		total.push_str(&left.accept(self));
		total.push(' ');
		total.push_str(&right.accept(self));
		total.push(')');
		total
	}

	fn visit_grouping(self,exp: &Expr) ->String {
		let mut total = String::new();
		total.push_str("(group");
		total.push_str(&exp.accept(self));
		total.push(')');
		total
	}

	fn visit_ternary(self, op: &Token, left: &Expr, middle: &Expr, right: &Expr) -> String {
		let mut total = String::new();
		total.push('(');
		total.push_str(&op.to_string());
		total.push(' ');
		total.push_str(&left.accept(self));
		total.push(' ');
		total.push_str(&middle.accept(self));
		total.push(' ');
		total.push_str(&right.accept(self));
		total.push(')');
		total
	}
	
	fn visit_literal(self,lit: &Literal) -> String {
		lit.to_string()
	}

	fn visit_unary(self,op: &Token, exp: &Expr) -> String {
		let mut total = String::new();
		total.push('(');
		total.push_str(&op.to_string());
		total.push(' ');
		total.push_str(&exp.accept(self));
		total.push(')');
		total
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::tokens::TokenType;
	use crate::tokens::Token;
	
	#[test]
	fn simple_pretty_print() {
		let e = Expr::Binary(Box::new(Expr::Literal(Literal::Number(2.0))), Token::new(TokenType::Plus,"+".to_owned(),1),Box::new(Expr::Literal(Literal::Number(2.0))) );
		let visitor = PrettyPrint{};
		let b = e.accept(&visitor);
		assert_eq!(b,"(+ 2 2)");
	}
}