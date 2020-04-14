use red;

// TODO:
// [] split into modules
// [] tests
// [] return list of errors
// [x] add type env
// [] serialise
//
fn main() {
    let chain = r#"{
                    "name": "chain",
                    "left": [{"name": "litNumber", "value": 1}],
                    "center": {"name": "empty" }
                    "right": [{"name": "if",
                               "condition": {"name": "litNumber", "value": 100},
                               "consequence": {"name": "litText", "value": "hello"}
                    }]
                  }"#;

    red::parse(&chain);
}
