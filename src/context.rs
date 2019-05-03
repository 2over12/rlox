use crate::ErrorReporter;
use crate::syntax::Stmt;
use crate::tokens::Token;
use crate::syntax::StmtVisitor;

use crate::syntax::Expr;


type Result<T> = std::result::Result<T,ContextError>;

pub enum ContextError {
    BreakOutsideLoop(usize)
}

impl ContextError {
    pub fn report(&self, err_rep: &mut ErrorReporter) {
        match self {
            ContextError::BreakOutsideLoop(line) => err_rep.error(*line, "Break found outside of loop body.")
        }
    }
}

#[derive(Clone)]
struct ContextCheck {
    inside_loop: bool
}

impl StmtVisitor<Result<()>> for ContextCheck {
    fn visit_print(self, _expr: &Expr) -> Result<()> {
        Ok(())
    } 

    fn visit_break(self, line: usize) -> Result<()> {
        if self.get_inside_loop() {
            Ok(())
        } else {
            Err(ContextError::BreakOutsideLoop(line))
        }
    }

    fn visit_expr_statement(self, _expr: &Expr) -> Result<()> {
        Ok(())
    }

    fn visit_variable(self,_name: &Token, _expr: &Option<Expr>) -> Result<()> {
        Ok(())
    }

    fn visit_while(mut self, _cond: &Expr, body: &Stmt) -> Result<()> {
        self.inside_loop = true;
        body.accept(self)
    }

    fn visit_if(self, _cond: &Expr, then: &Stmt, otherwise: &Option<Stmt>) -> Result<()> {
        then.accept(self.clone())?;
        if let Some(other) = otherwise {
            other.accept(self)
        } else {
            Ok(())
        }
    }

    fn visit_block_stmt(self, stmts: &Vec<Stmt>) -> Result<()> {
        for stmt in stmts.iter() {
            stmt.accept(self.clone())?;
        }

        Ok(())
    }

}

impl ContextCheck {
    fn new(inside_loop: bool) -> ContextCheck {
        ContextCheck {
            inside_loop
        }
    }

    fn get_inside_loop(&self) -> bool {
        self.inside_loop
    }
}

pub fn check(stmts: &Vec<Stmt>) -> Vec<ContextError> {
    stmts.iter().filter_map(|x| {
        let checker = ContextCheck::new(false);

        if let Err(er) = x.accept(checker) {
            Some(er)
        } else {
            None
        }
    }).collect()
}