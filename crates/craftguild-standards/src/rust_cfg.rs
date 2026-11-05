//! Evaluate only Rust cfg(test) logic; production feature predicates stay possible.

use syn::{punctuated::Punctuated, Attribute, Expr, Meta, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    Yes,
    No,
    Conditional,
    Unsupported,
}

fn contains_test(meta: &Meta) -> std::result::Result<bool, syn::Error> {
    match meta {
        Meta::Path(path) => Ok(path.is_ident("test")),
        Meta::List(list) => {
            let items = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            let mut found = false;
            for item in items {
                found |= contains_test(&item)?;
            }
            Ok(found)
        }
        _ => Ok(false),
    }
}

fn production(meta: &Meta) -> Truth {
    use Truth::*;
    match meta {
        Meta::Path(path) if path.is_ident("test") => No,
        Meta::List(list) => {
            let Ok(items) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                return Unsupported;
            };
            let values: Vec<_> = items.iter().map(production).collect();
            if values.contains(&Unsupported) {
                return Unsupported;
            }
            if list.path.is_ident("all") {
                if values.contains(&No) {
                    No
                } else if values.contains(&Conditional) {
                    Conditional
                } else {
                    Yes
                }
            } else if list.path.is_ident("any") {
                if values.contains(&Yes) {
                    Yes
                } else if values.contains(&Conditional) {
                    Conditional
                } else {
                    No
                }
            } else if list.path.is_ident("not") && values.len() == 1 {
                match values[0] {
                    Yes => No,
                    No => Yes,
                    other => other,
                }
            } else {
                Unsupported
            }
        }
        _ => Conditional,
    }
}

pub fn predicate(attr: &Attribute) -> Option<Truth> {
    if !attr.path().is_ident("cfg") {
        return None;
    }
    let Ok(meta) = attr.parse_args::<Meta>() else {
        return Some(Truth::Unsupported);
    };
    match contains_test(&meta) {
        Ok(true) => Some(production(&meta)),
        Ok(false) => None,
        Err(_) => Some(Truth::Unsupported),
    }
}

pub fn expression_attributes(expr: &Expr) -> &[Attribute] {
    match expr {
        Expr::Array(i) => &i.attrs,
        Expr::Assign(i) => &i.attrs,
        Expr::Async(i) => &i.attrs,
        Expr::Await(i) => &i.attrs,
        Expr::Binary(i) => &i.attrs,
        Expr::Block(i) => &i.attrs,
        Expr::Break(i) => &i.attrs,
        Expr::Call(i) => &i.attrs,
        Expr::Cast(i) => &i.attrs,
        Expr::Closure(i) => &i.attrs,
        Expr::Const(i) => &i.attrs,
        Expr::Continue(i) => &i.attrs,
        Expr::Field(i) => &i.attrs,
        Expr::ForLoop(i) => &i.attrs,
        Expr::Group(i) => &i.attrs,
        Expr::If(i) => &i.attrs,
        Expr::Index(i) => &i.attrs,
        Expr::Infer(i) => &i.attrs,
        Expr::Let(i) => &i.attrs,
        Expr::Lit(i) => &i.attrs,
        Expr::Loop(i) => &i.attrs,
        Expr::Macro(i) => &i.attrs,
        Expr::Match(i) => &i.attrs,
        Expr::MethodCall(i) => &i.attrs,
        Expr::Paren(i) => &i.attrs,
        Expr::Path(i) => &i.attrs,
        Expr::Range(i) => &i.attrs,
        Expr::RawAddr(i) => &i.attrs,
        Expr::Reference(i) => &i.attrs,
        Expr::Repeat(i) => &i.attrs,
        Expr::Return(i) => &i.attrs,
        Expr::Struct(i) => &i.attrs,
        Expr::Try(i) => &i.attrs,
        Expr::TryBlock(i) => &i.attrs,
        Expr::Tuple(i) => &i.attrs,
        Expr::Unary(i) => &i.attrs,
        Expr::Unsafe(i) => &i.attrs,
        Expr::While(i) => &i.attrs,
        Expr::Yield(i) => &i.attrs,
        _ => &[],
    }
}
