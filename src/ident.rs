use std::fmt;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Ident {
    identifier: String,
}

impl Ident {
    pub fn from_str(identifier: &str) -> Self {
        Ident {
            identifier: identifier.to_owned(),
        }
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.identifier.as_ref()
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.identifier.fmt(f)
    }
}
