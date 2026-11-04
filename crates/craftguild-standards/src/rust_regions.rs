//! Remove only syntactically certain test items while preserving original source lines.

use craftguild_core::Result;
use proc_macro2::Span;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Attribute, ImplItem, Item, TraitItem,
};

struct Regions {
    spans: Vec<Span>,
    unsupported: bool,
}

impl Regions {
    fn test(&mut self, attrs: &[Attribute], span: Span) -> bool {
        let mut test = false;
        for attr in attrs {
            let path = attr
                .path()
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if matches!(path.as_str(), "test" | "tokio::test" | "async_std::test") {
                test = true;
            } else if attr
                .path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "test")
            {
                self.unsupported = true;
            }
            if attr.path().is_ident("cfg") {
                match attr.parse_args::<syn::Meta>() {
                    Ok(syn::Meta::Path(p)) if p.is_ident("test") => test = true,
                    Ok(meta)
                        if quote::quote!(#meta)
                            .to_string()
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .any(|s| s == "test") =>
                    {
                        self.unsupported = true
                    }
                    Err(_) => self.unsupported = true,
                    _ => {}
                }
            }
        }
        if test {
            self.spans.push(span);
        }
        test
    }
}

impl<'ast> Visit<'ast> for Regions {
    fn visit_item(&mut self, item: &'ast Item) {
        let attrs = match item {
            Item::Const(i) => &i.attrs,
            Item::Enum(i) => &i.attrs,
            Item::ExternCrate(i) => &i.attrs,
            Item::Fn(i) => &i.attrs,
            Item::ForeignMod(i) => &i.attrs,
            Item::Impl(i) => &i.attrs,
            Item::Macro(i) => &i.attrs,
            Item::Mod(i) => &i.attrs,
            Item::Static(i) => &i.attrs,
            Item::Struct(i) => &i.attrs,
            Item::Trait(i) => &i.attrs,
            Item::TraitAlias(i) => &i.attrs,
            Item::Type(i) => &i.attrs,
            Item::Union(i) => &i.attrs,
            Item::Use(i) => &i.attrs,
            _ => {
                self.unsupported = true;
                return;
            }
        };
        if !self.test(attrs, item.span()) {
            visit::visit_item(self, item);
        }
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        let attrs = match item {
            ImplItem::Const(i) => &i.attrs,
            ImplItem::Fn(i) => &i.attrs,
            ImplItem::Type(i) => &i.attrs,
            ImplItem::Macro(i) => &i.attrs,
            _ => {
                self.unsupported = true;
                return;
            }
        };
        if !self.test(attrs, item.span()) {
            visit::visit_impl_item(self, item);
        }
    }
    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        let attrs = match item {
            TraitItem::Const(i) => &i.attrs,
            TraitItem::Fn(i) => &i.attrs,
            TraitItem::Type(i) => &i.attrs,
            TraitItem::Macro(i) => &i.attrs,
            _ => {
                self.unsupported = true;
                return;
            }
        };
        if !self.test(attrs, item.span()) {
            visit::visit_trait_item(self, item);
        }
    }
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr") {
            let tokens = quote::quote!(#attr).to_string();
            if tokens
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|s| s == "test")
            {
                // An expression/field-level test predicate needs finer context than
                // item removal. Refuse to pretend this source was fully measured.
                self.unsupported = true;
            }
        }
    }
}

pub fn implementation(source: &str) -> Result<String> {
    let syntax = syn::parse_file(source)?;
    let mut regions = Regions {
        spans: vec![],
        unsupported: false,
    };
    regions.visit_file(&syntax);
    if regions.unsupported {
        return Err("Cannot reliably separate conditional test syntax".into());
    }
    let mut offsets = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(index + 1);
        }
    }
    let mut bytes = source.as_bytes().to_vec();
    for span in regions.spans {
        let start = span.start();
        let end = span.end();
        let start = offsets
            .get(start.line.checked_sub(1).ok_or("Unavailable Rust span")?)
            .ok_or("Invalid Rust start span")?
            + start.column;
        let end = offsets
            .get(end.line.checked_sub(1).ok_or("Unavailable Rust span")?)
            .ok_or("Invalid Rust end span")?
            + end.column;
        for byte in bytes
            .get_mut(start..end)
            .ok_or("Rust span outside source")?
        {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    Ok(String::from_utf8(bytes)?)
}
