use crate::symbol::Segment;
use crate::{Lifetimes, NamedType, Pair, Symbol};
use proc_macro2::{Ident, Span};
use std::fmt::{self, Display};
use std::iter;
use syn::ext::IdentExt;
use syn::parse::{Error, Parser, Result};
use syn::punctuated::Punctuated;

#[derive(Clone)]
pub struct ForeignName {
    text: String,
}

impl Pair {
    pub fn to_symbol(&self) -> Symbol {
        let segments = self
            .namespace
            .iter()
            .map(|ident| ident as &dyn Segment)
            .chain(iter::once(&self.cxx as &dyn Segment));
        Symbol::from_idents(segments)
    }

    pub fn to_fully_qualified(&self) -> String {
        let mut fully_qualified = String::new();
        for segment in &self.namespace {
            fully_qualified += "::";
            fully_qualified += &segment.to_string();
        }
        fully_qualified += "::";
        fully_qualified += &self.cxx.to_string();
        fully_qualified
    }
}

impl NamedType {
    pub fn new(rust: Ident) -> Self {
        let generics = Lifetimes {
            lt_token: None,
            lifetimes: Punctuated::new(),
            gt_token: None,
        };
        NamedType { rust, generics }
    }
}

impl ForeignName {
    pub fn parse(text: &str, span: Span) -> Result<Self> {
        // TODO: support C++ names containing whitespace (`unsigned int`) or
        // non-alphanumeric characters (`operator++`).
        match Ident::parse_any.parse_str(text) {
            Ok(ident) => {
                let text = ident.to_string();
                Ok(ForeignName { text })
            }
            Err(err) => Err(Error::new(span, err)),
        }
    }
}

impl Display for ForeignName {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl PartialEq<str> for ForeignName {
    fn eq(&self, rhs: &str) -> bool {
        self.text == rhs
    }
}
