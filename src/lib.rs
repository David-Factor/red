pub mod ast;
pub mod ident;
pub mod path;
pub mod typecheck;
pub mod types;

pub fn parse(raw_json: &str) {
    println!("{:?}", ast::parse(&raw_json));
}

pub fn parse_env(raw_json: &str) {
    println!("{:?}", types::parse(&raw_json));
}
