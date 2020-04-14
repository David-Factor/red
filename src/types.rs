use std::collections::HashMap;
use std::fmt;

use crate::ident::Ident;

#[derive(Debug, Clone)]
pub enum Type {
    Number,
    Boolean,
    Text,
    Unit,
    Record(HashMap<Ident, Type>),
    List(Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::Boolean => write!(f, "Boolean"),
            Type::Text => write!(f, "Text"),
            Type::Unit => write!(f, "Unit"),
            Type::Record(record) => write!(f, "Record {:?}", record),
            Type::List(list) => write!(f, "List {:?}", list),
        }
    }
}
