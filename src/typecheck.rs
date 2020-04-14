use std::collections::HashMap;
use std::result;

use crate::ast::{Chain, Expr, If, Not, RecordRef, VariableRef};
use crate::ident::Ident;
use crate::path::{Path, Seg};
use crate::types::Type;

type Result<'a> = result::Result<Type, Error<'a>>;

#[derive(Debug)]
pub struct Error<'a> {
    path: Path,
    error: TypeError<'a>,
}

#[derive(Debug)]
enum TypeError<'a> {
    EmptyExpr,
    TypeMismatch(TypeMismatch),
    Undefined(&'a Ident),
    ExpectedRecord(Type),
}

#[derive(Debug)]
struct TypeMismatch {
    expected: Type,
    received: Type,
}

pub type TypeContext = HashMap<Ident, Type>;

pub fn typecheck<'a>(expr: &'a Expr, ctx: &mut TypeContext) -> Result<'a> {
    let path = Vec::new();
    expr.check(path, ctx)
}

trait Typecheck {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result;
}

impl Typecheck for Expr {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        match &self {
            Expr::Empty => Err(Error {
                error: TypeError::EmptyExpr,
                path: path,
            }),
            Expr::LitBool(_) => Ok(Type::Bool),
            Expr::LitNumber(_) => Ok(Type::Number),
            Expr::LitText(_) => Ok(Type::Text),
            Expr::Not(ref not) => not.check(path, ctx),
            Expr::If(ref if_) => if_.check(path, ctx),
            Expr::Chain(ref chain) => chain.check(path, ctx),
            Expr::VariableRef(ref var) => ctx.get(&var.identifier).cloned().ok_or(Error {
                error: TypeError::Undefined(&var.identifier),
                path: path,
            }),
            Expr::RecordRef(ref record_ref) => record_ref.check(path, ctx),
        }
    }
}

impl Typecheck for Not {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        let mut path = path.clone();
        path.push(Seg::Not);

        assert_bool(&self.not, path, ctx)
    }
}

impl Typecheck for If {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        let mut cond_path = path.clone();
        let mut cons_path = path.clone();

        cond_path.push(Seg::IfCond);
        cons_path.push(Seg::IfCons);

        assert_bool(&self.condition, cond_path, ctx)?;
        self.consequence.check(cons_path, ctx)?;
        Ok(Type::Unit)
    }
}

// FIXME
// https://doc.rust-lang.org/stable/rust-by-example/error/iter_result.html
impl Typecheck for Chain {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        self.chain
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let mut path = path.clone();
                path.push(Seg::ChainN(i as i32));
                expr.check(path, ctx)
            })
            .last()
            // FIXME unwrap as List
            .unwrap_or(Ok(Type::List(Box::new(Type::Unit))))
    }
}

impl Typecheck for VariableRef {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        ctx.get(&self.identifier).cloned().ok_or(Error {
            error: TypeError::Undefined(&self.identifier),
            path: path,
        })
    }
}

impl Typecheck for RecordRef {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        let mut path = path.clone();
        path.push(Seg::RecordRef);

        let p = path.clone();

        match assert_record(&self.record, path, ctx) {
            Err(error) => Err(error),
            Ok(ctx_) => ctx_.get(&self.identifier).cloned().ok_or(Error {
                error: TypeError::Undefined(&self.identifier),
                path: p,
            }),
        }
    }
}

fn assert_bool<'c>(expr: &'c Expr, path: Path, ctx: &mut TypeContext) -> Result<'c> {
    match expr.check(path.clone(), ctx) {
        Ok(Type::Bool) => Ok(Type::Bool),
        Ok(type_) => Err(Error {
            path,
            error: TypeError::TypeMismatch(TypeMismatch {
                expected: Type::Bool,
                received: type_,
            }),
        }),
        Err(error) => Err(error),
    }
}

fn assert_record<'c>(
    expr: &'c Expr,
    path: Path,
    ctx: &mut TypeContext,
) -> result::Result<HashMap<Ident, Type>, Error<'c>> {
    match expr.check(path.clone(), ctx) {
        Ok(Type::Record(record)) => Ok(record),
        Ok(type_) => Err(Error {
            path,
            error: TypeError::ExpectedRecord(type_),
        }),
        Err(error) => Err(error),
    }
}
