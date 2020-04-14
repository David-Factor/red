pub mod ast;
pub mod ident;
pub mod path;
pub mod typecheck;
pub mod types;

pub fn parse(raw_json: &str) {
    println!("{:?}", ast::parse(&raw_json));
}
