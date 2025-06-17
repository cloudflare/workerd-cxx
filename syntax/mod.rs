// Functionality that is shared between the cxxbridge macro and the cmd.

// NOTE(workerd-cxx): upstream is really messy
#![allow(
    clippy::cast_sign_loss,
    clippy::enum_glob_use,
    clippy::inherent_to_string,
    clippy::into_iter_without_iter,
    clippy::items_after_statements,
    clippy::match_bool,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_lifetimes,
    clippy::needless_pass_by_value,
    clippy::new_without_default,
    clippy::nonminimal_bool,
    clippy::pub_underscore_fields,
    clippy::redundant_else,
    clippy::should_implement_trait,
    clippy::single_match_else,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::toplevel_ref_arg,
    clippy::uninlined_format_args,
)]

pub mod atom;
pub mod attrs;
pub mod cfg;
pub mod check;
pub mod derive;
mod discriminant;
mod doc;
pub mod error;
pub mod file;
pub mod ident;
mod impls;
mod improper;
pub mod instantiate;
pub mod mangle;
pub mod map;
mod names;
pub mod namespace;
mod parse;
mod pod;
pub mod qualified;
pub mod report;
pub mod resolve;
pub mod set;
pub mod symbol;
mod tokens;
mod toposort;
pub mod trivial;
pub mod types;
mod visit;

use self::attrs::OtherAttrs;
use self::cfg::CfgExpr;
use self::namespace::Namespace;
use self::parse::kw;
use self::symbol::Symbol;
use proc_macro2::{Ident, Span};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{Attribute, Expr, Generics, Lifetime, LitInt, Token, Type as RustType};

pub use self::atom::Atom;
pub use self::derive::{Derive, Trait};
pub use self::discriminant::Discriminant;
pub use self::doc::Doc;
pub use self::names::ForeignName;
pub use self::parse::parse_items;
pub use self::types::Types;

pub enum Api {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    Include(Include),
    Struct(Struct),
    Enum(Enum),
    CxxType(ExternType),
    CxxFunction(ExternFn),
    RustType(ExternType),
    RustFunction(ExternFn),
    TypeAlias(TypeAlias),
    Impl(Impl),
}

pub struct Include {
    pub cfg: CfgExpr,
    pub path: String,
    pub kind: IncludeKind,
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub begin_span: Span,
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub end_span: Span,
}

/// Whether to emit `#include "path"` or `#include <path>`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IncludeKind {
    /// `#include "quoted/path/to"`
    Quoted,
    /// `#include <bracketed/path/to>`
    Bracketed,
}

pub struct ExternType {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub lang: Lang,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub visibility: Token![pub],
    pub type_token: Token![type],
    pub name: Pair,
    pub generics: Lifetimes,
    #[allow(dead_code)]
    pub colon_token: Option<Token![:]>,
    pub bounds: Vec<Derive>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub semi_token: Token![;],
    pub trusted: bool,
}

pub struct Struct {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub visibility: Token![pub],
    pub struct_token: Token![struct],
    pub name: Pair,
    pub generics: Lifetimes,
    pub brace_token: Brace,
    pub fields: Vec<Var>,
}

pub struct Enum {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub visibility: Token![pub],
    pub enum_token: Token![enum],
    pub name: Pair,
    pub generics: Lifetimes,
    pub brace_token: Brace,
    pub variants: Vec<Variant>,
    pub variants_from_header: bool,
    #[allow(dead_code)]
    pub variants_from_header_attr: Option<Attribute>,
    pub repr: EnumRepr,
    pub explicit_repr: bool,
}

pub enum EnumRepr {
    Native {
        atom: Atom,
        repr_type: Type,
    },
    #[cfg(feature = "experimental-enum-variants-from-header")]
    Foreign {
        rust_type: syn::Path,
    },
}

pub struct ExternFn {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub lang: Lang,
    pub doc: Doc,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub visibility: Token![pub],
    pub name: Pair,
    pub sig: Signature,
    pub semi_token: Token![;],
    pub trusted: bool,
}

pub struct TypeAlias {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub doc: Doc,
    pub derives: Vec<Derive>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub visibility: Token![pub],
    pub type_token: Token![type],
    pub name: Pair,
    pub generics: Lifetimes,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub eq_token: Token![=],
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub ty: RustType,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub semi_token: Token![;],
}

pub struct Impl {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub impl_token: Token![impl],
    pub impl_generics: Lifetimes,
    #[allow(dead_code)]
    pub negative: bool,
    pub ty: Type,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub ty_generics: Lifetimes,
    pub brace_token: Brace,
    pub negative_token: Option<Token![!]>,
}

#[derive(Clone, Default)]
pub struct Lifetimes {
    pub lt_token: Option<Token![<]>,
    pub lifetimes: Punctuated<Lifetime, Token![,]>,
    pub gt_token: Option<Token![>]>,
}

pub struct Signature {
    pub asyncness: Option<Token![async]>,
    pub unsafety: Option<Token![unsafe]>,
    pub fn_token: Token![fn],
    pub generics: Generics,
    pub receiver: Option<Receiver>,
    pub args: Punctuated<Var, Token![,]>,
    pub ret: Option<Type>,
    pub throws: bool,
    pub paren_token: Paren,
    pub throws_tokens: Option<(kw::Result, Token![<], Token![>])>,
}

pub struct Var {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub doc: Doc,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub visibility: Token![pub],
    pub name: Pair,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub colon_token: Token![:],
    pub ty: Type,
}

pub struct Receiver {
    pub pinned: bool,
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutable: bool,
    pub var: Token![self],
    pub ty: NamedType,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub colon_token: Token![:],
    pub shorthand: bool,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub pin_tokens: Option<(kw::Pin, Token![<], Token![>])>,
    pub mutability: Option<Token![mut]>,
}

pub struct Variant {
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub cfg: CfgExpr,
    pub doc: Doc,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub attrs: OtherAttrs,
    pub name: Pair,
    pub discriminant: Discriminant,
    #[allow(dead_code)]
    pub expr: Option<Expr>,
}

pub enum Type {
    Ident(NamedType),
    RustBox(Box<Ty1>),
    RustVec(Box<Ty1>),
    UniquePtr(Box<Ty1>),
    SharedPtr(Box<Ty1>),
    WeakPtr(Box<Ty1>),
    Ref(Box<Ref>),
    Ptr(Box<Ptr>),
    Str(Box<Ref>),
    CxxVector(Box<Ty1>),
    Fn(Box<Signature>),
    Void(Span),
    SliceRef(Box<SliceRef>),
    Array(Box<Array>),
    Future(Box<Future>),
}

pub struct Ty1 {
    pub name: Ident,
    pub langle: Token![<],
    pub inner: Type,
    pub rangle: Token![>],
}

pub struct Ref {
    pub pinned: bool,
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutable: bool,
    pub inner: Type,
    pub pin_tokens: Option<(kw::Pin, Token![<], Token![>])>,
    pub mutability: Option<Token![mut]>,
}

pub struct Ptr {
    pub star: Token![*],
    pub mutable: bool,
    pub inner: Type,
    pub mutability: Option<Token![mut]>,
    pub constness: Option<Token![const]>,
}

pub struct SliceRef {
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutable: bool,
    pub bracket: Bracket,
    pub inner: Type,
    pub mutability: Option<Token![mut]>,
}

pub struct Array {
    pub bracket: Bracket,
    pub inner: Type,
    pub semi_token: Token![;],
    pub len: usize,
    pub len_token: LitInt,
}

pub struct Future {
    pub output: Type,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub throws_tokens: Option<(kw::Result, Token![<], Token![>])>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Lang {
    Cxx,
    Rust,
}

// An association of a defined Rust name with a fully resolved, namespace
// qualified C++ name.
#[derive(Clone)]
pub struct Pair {
    pub namespace: Namespace,
    pub cxx: ForeignName,
    pub rust: Ident,
}

// Wrapper for a type which needs to be resolved before it can be printed in
// C++.
#[derive(PartialEq, Eq, Hash)]
pub struct NamedType {
    pub rust: Ident,
    pub generics: Lifetimes,
}
