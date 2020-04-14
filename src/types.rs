use std::collections::HashMap;

use serde::Deserialize;
use serde_json::error::Error;

use crate::ident::Ident;

pub fn parse(exp: &str) -> Result<HashMap<Ident, Type>, Error> {
    serde_json::from_str(&exp)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "parameter")]
#[serde(rename_all = "UPPERCASE")]
pub enum Type {
    Unit,
    NonRule,
    Bool,
    Date,
    DateTime,
    Time,
    Duration,
    Weekday,
    Number,
    Text,
    Record(HashMap<Ident, Type>),
    List(Box<Type>),
    Unknown,
}
