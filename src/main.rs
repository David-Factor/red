use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use red;
use red::ast;
use red::ident::Ident;
use red::typecheck;
use red::types::Type;

// TODO:
// [x] split into modules
// [] tests
// [] return list of errors
// [x] add type context
// [x] deserialise
// [ ] add remaining rules
// [] update type context
fn main() {
    //let empty = r#"{"name": "empty"}";
    let if_ = r#"{"name": "if",
                     "condition": {"name": "nam", "identifier": "xxxxxxxxx"},
                     "consequence": {"name": "litText", "value": "hello"}
                    }"#;

    let chain = r#"{
                    "name": "chain",
                    "right": [{"name": "recordRef",
                              "record": {"name": "variableRef", "identifier": "PaidRight"},
                              "identifier": "organisation"
                              }],
                    "center": {"name": "variableRef", "identifier": "hello" },
                    "left": [{"name": "if",
                               "condition": {"name": "litNumber", "value": 100},
                               "consequence": {"name": "litText", "value": "hello"}
                    }]
                  }"#;

    let expr = ast::parse(&if_).unwrap();
    let mut type_context = read_type_context("../data/env.json").unwrap();
    //  println!("{:#?}", expr);
    //  println!("{:#?}", type_context);
    let check = typecheck::typecheck(&expr, &mut type_context);
    println!("{:?}", check)
}

fn read_type_context<P: AsRef<Path>>(path: P) -> Result<HashMap<Ident, Type>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `TypeContext`.
    let type_context = serde_json::from_reader(reader)?;

    // Return the `TypeContext`.
    Ok(type_context)
}
