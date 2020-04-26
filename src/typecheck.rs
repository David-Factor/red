use std::collections::HashMap;
use std::convert::From;
use std::result;

use crate::ast::{Chain, Expr, If, Not, RecordRef, VariableRef};
use crate::ident::Ident;
use crate::path::{Path, Seg};
use crate::types::Type;

#[derive(Debug)]
pub struct Check<'a>(Result<'a>);

type Result<'a> = result::Result<Type, Errors<'a>>;
type Errors<'a> = Vec<Error<'a>>;

impl<'a> From<Result<'a>> for Check<'a> {
    fn from(result: Result<'a>) -> Check<'a> {
        Check(result)
    }
}

impl<'a> From<Check<'a>> for Result<'a> {
    fn from(check: Check<'a>) -> Result<'a> {
        check.0
    }
}

impl<'a> Check<'a> {
    fn fail(error: Error) -> Check {
        let mut errors = Vec::new();
        errors.push(error);
        Check(Err(errors))
    }

    fn succeed(t: Type) -> Self {
        Check(Ok(t))
    }

    fn assert_bool(path: Path, type_: Type) -> Self {
        match type_ {
            Type::Bool => Check::succeed(Type::Bool),
            type_ => Check::fail(Error {
                path,
                error: TypeError::TypeMismatch(TypeMismatch {
                    expected: Type::Bool,
                    received: type_,
                }),
            }),
        }
    }

    fn assert_record<'c>(
        expr: &'c Expr,
        path: Path,
        ctx: &mut TypeContext,
    ) -> result::Result<HashMap<Ident, Type>, Errors<'c>> {
        match expr.check(path.clone(), ctx).0 {
            Ok(Type::Record(record)) => Ok(record),
            Ok(type_) => {
                let mut errors = Vec::new();
                let error = Error {
                    path,
                    error: TypeError::ExpectedRecord(type_),
                };
                errors.push(error);

                Err(errors)
            }
            Err(error) => Err(error),
        }
    }

    fn map<F: FnOnce(Type) -> Type>(self, f: F) -> Self {
        Check(self.0.map(f))
    }

    fn map2<F>(f: F, r1: Check<'a>, r2: Check<'a>) -> Self
    where
        F: FnOnce(Type, Type) -> Type,
    {
        let result = match (r1.0, r2.0) {
            (Ok(a), Ok(b)) => Ok(f(a, b)),
            (Err(a), Err(b)) => {
                let errors = a.into_iter().chain(b).collect();
                Err(errors)
            }
            (Err(a), _) => Err(a),
            (_, Err(a)) => Err(a),
        };
        Check(result)
    }

    fn and_then<F>(self, f: F) -> Self
    where
        F: FnOnce(Type) -> Check<'a>,
    {
        match self.0 {
            Ok(t) => f(t),
            Err(e) => Check(Err(e)),
        }
    }
}

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

pub fn typecheck<'a>(expr: &'a Expr, ctx: &mut TypeContext) -> Check<'a> {
    let path = Vec::new();
    expr.check(path, ctx)
}

trait Typecheck {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check;
}

impl Typecheck for Expr {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check {
        match &self {
            Expr::Empty => Check::fail(Error {
                error: TypeError::EmptyExpr,
                path: path,
            }),
            Expr::LitBool(_) => Check::succeed(Type::Bool),
            Expr::LitNumber(_) => Check::succeed(Type::Number),
            Expr::LitText(_) => Check::succeed(Type::Text),
            Expr::Not(ref not) => not.check(path, ctx),
            Expr::If(ref if_) => if_.check(path, ctx),
            Expr::Chain(ref chain) => chain.check(path, ctx),
            Expr::VariableRef(ref var) => var.check(path, ctx),
            Expr::RecordRef(ref record_ref) => record_ref.check(path, ctx),
        }
    }
}

impl Typecheck for Not {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check {
        let mut path = path.clone();
        path.push(Seg::Not);

        self.not
            .check(path.clone(), ctx)
            .and_then(|type_| Check::assert_bool(path, type_))
            .map(|_| Type::Bool)
    }
}

impl Typecheck for If {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check {
        let mut cond_path = path.clone();
        let mut cons_path = path.clone();

        cond_path.push(Seg::IfCond);
        cons_path.push(Seg::IfCons);

        let condition = self
            .condition
            .check(cond_path.clone(), ctx)
            .and_then(|type_| Check::assert_bool(cond_path, type_));

        let consequence = self.consequence.check(cons_path, ctx);

        Check::map2(|_, _| Type::Unit, condition, consequence)
    }
}

impl Typecheck for Chain {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check {
        self.chain
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let mut path = path.clone();
                path.push(Seg::ChainN(i as i32));
                expr.check(path, ctx)
            })
            .fold(Check::succeed(Type::Unit), |acc, checked| match checked.0 {
                Ok(type_) => Check::succeed(Type::List(Box::new(type_))),
                Err(current_errors) => match acc.0 {
                    Ok(_) => Check(Err(current_errors)),
                    Err(errors) => {
                        let next = current_errors.into_iter().chain(errors).collect();
                        Check(Err(next))
                    }
                },
            })
    }
}

impl Typecheck for VariableRef {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check {
        match ctx.get(&self.identifier).cloned() {
            Some(a) => Check::succeed(a),
            None => Check::fail(Error {
                error: TypeError::Undefined(&self.identifier),
                path: path,
            }),
        }
    }
}

impl Typecheck for RecordRef {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Check {
        let mut path = path.clone();
        path.push(Seg::RecordRef);

        let p = path.clone();

        match Check::assert_record(&self.record, path, ctx) {
            Err(error) => Check::from(Err(error)),
            Ok(ctx_) => match ctx_.get(&self.identifier).cloned() {
                Some(t) => Check(Ok(t)),
                None => Check::fail(Error {
                    error: TypeError::Undefined(&self.identifier),
                    path: p,
                }),
            },
        }
    }
}
