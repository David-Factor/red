use std::collections::HashMap;
use std::fmt;
use std::result;

// TODO:
// [] split into modules
// [] tests
// [] return list of errors
// [x] add type env
fn main() {
    let mut context = TypeContext::new();
    let mut record = TypeContext::new();
    record.insert(String::from("blah"), Type::Boolean);
    context.insert(String::from("hello"), Type::Record(record));

    let var_ref = Expr::VarRef("hello".to_string());
    println!("VARREF {:?}", var_ref.check(Vec::new(), &mut context));

    let mut context1 = context.clone();
    let record_ref = Expr::RecordRef(Box::new(RecordRef {
        ident: "blah".to_string(),
        record: Expr::VarRef("hello".to_string()),
    }));
    println!(
        "RECORDREF {:?}",
        record_ref.check(Vec::new(), &mut context1)
    );

    let bad = Expr::Not(Box::new(Not {
        not: Expr::Number(1),
    }));
    println!("{:?}", bad.check(Vec::new(), &mut TypeContext::new()));

    let good = Expr::Not(Box::new(Not {
        not: Expr::Not(Box::new(Not {
            not: Expr::Bool(true),
        })),
    }));
    println!("{:?}", good.check(Vec::new(), &mut TypeContext::new()));

    let bad1 = Expr::Not(Box::new(Not {
        not: Expr::Not(Box::new(Not { not: Expr::Empty })),
    }));
    println!("{:?}", bad1.check(Vec::new(), &mut TypeContext::new()));
    let bad2 = Expr::If(Box::new(If {
        condition: (Expr::Number(1)),
        consequent: Expr::Number(1),
    }));
    println!(">>> {:?}", bad2.check(Vec::new(), &mut TypeContext::new()));

    let bad3 = Expr::Chain(Chain {
        chain: vec![
            Expr::If(Box::new(If {
                condition: (Expr::Number(1)),
                consequent: Expr::Number(1),
            })),
            Expr::Not(Box::new(Not { not: Expr::Empty })),
        ],
    });
    println!(">>> {:?}", bad3.check(Vec::new(), &mut TypeContext::new()));
}

// AST
enum Expr {
    Empty,
    Bool(bool),
    Number(i32),
    Not(Box<Not>),
    If(Box<If>),
    Chain(Chain),
    VarRef(String),
    RecordRef(Box<RecordRef>),
}

struct Not {
    not: Expr,
}

struct If {
    condition: Expr,
    consequent: Expr,
}

struct Chain {
    chain: Vec<Expr>,
}

struct RecordRef {
    ident: String,
    record: Expr,
}
type Path = Vec<Seg>;

#[derive(Debug, Clone)]
enum Seg {
    Not,
    IfCond,
    IfCons,
    ChainN(i32),
    RecordRef,
}

#[derive(Debug, Clone)]
enum Type {
    Number,
    Boolean,
    Unit,
    Record(HashMap<String, Type>),
    List(Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::Boolean => write!(f, "Boolean"),
            Type::Unit => write!(f, "Unit"),
            Type::Record(record) => write!(f, "{:?}", record),
            Type::List(list) => write!(f, "{:?}", list),
        }
    }
}

type Result = result::Result<Type, Error>;

#[derive(Debug)]
struct Error {
    path: Vec<Seg>,
    error: TypeError,
}

#[derive(Debug)]
enum TypeError {
    EmptyExpr,
    TypeMismatch(TypeMismatch),
    Undefined(String),
    ExpectedRecord(Type),
}

#[derive(Debug)]
struct TypeMismatch {
    expected: Type,
    received: Type,
}

type TypeContext = HashMap<String, Type>;

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
            Expr::Bool(_) => Ok(Type::Boolean),
            Expr::Number(_) => Ok(Type::Number),
            Expr::Not(ref not) => not.check(path, ctx),
            Expr::If(ref if_) => if_.check(path, ctx),
            Expr::Chain(ref chain) => chain.check(path, ctx),
            Expr::VarRef(ref var) => ctx.get(var).cloned().ok_or(Error {
                error: TypeError::Undefined(var.to_string()),
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
        self.consequent.check(cons_path, ctx)?;
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
            .unwrap_or(Ok(Type::Unit))
    }
}

impl Typecheck for RecordRef {
    fn check<'c>(&'c self, path: Path, ctx: &mut TypeContext) -> Result {
        let mut path = path.clone();
        path.push(Seg::RecordRef);

        let p = path.clone();

        match assert_record(&self.record, path, ctx) {
            Err(error) => Err(error),
            Ok(ctx_) => ctx_.get(&self.ident).cloned().ok_or(Error {
                error: TypeError::Undefined(self.ident.to_string()),
                path: p,
            }),
        }
    }
}

fn assert_bool<'c>(expr: &'c Expr, path: Path, ctx: &mut TypeContext) -> Result {
    match expr.check(path.clone(), ctx) {
        Ok(Type::Boolean) => Ok(Type::Boolean),
        Ok(type_) => Err(Error {
            path,
            error: TypeError::TypeMismatch(TypeMismatch {
                expected: Type::Boolean,
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
) -> result::Result<HashMap<String, Type>, Error> {
    match expr.check(path.clone(), ctx) {
        Ok(Type::Record(record)) => Ok(record),
        Ok(type_) => Err(Error {
            path,
            error: TypeError::ExpectedRecord(type_),
        }),
        Err(error) => Err(error),
    }
}
