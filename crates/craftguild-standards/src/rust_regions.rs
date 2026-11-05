//! Remove only syntactically certain test items while preserving original source lines.

use crate::rust_cfg::{expression_attributes, predicate, Truth};
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
    commas: Vec<(Span, Span)>,
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
            match predicate(attr) {
                Some(Truth::No) => test = true,
                Some(Truth::Unsupported) => self.unsupported = true,
                _ => {}
            }
        }
        if test {
            self.spans.push(span);
            if let Some((_, comma)) = self
                .commas
                .iter()
                .find(|(node, _)| node.start() == span.start() && node.end() == span.end())
            {
                self.spans.push(*comma);
            }
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
    fn visit_fields_named(&mut self, fields: &'ast syn::FieldsNamed) {
        for pair in fields.named.pairs() {
            let (field, comma) = pair.into_tuple();
            if let Some(comma) = comma {
                self.commas.push((field.span(), comma.span()));
            }
        }
        visit::visit_fields_named(self, fields);
    }
    fn visit_fields_unnamed(&mut self, fields: &'ast syn::FieldsUnnamed) {
        for pair in fields.unnamed.pairs() {
            let (field, comma) = pair.into_tuple();
            if let Some(comma) = comma {
                self.commas.push((field.span(), comma.span()));
            }
        }
        visit::visit_fields_unnamed(self, fields);
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        if !self.test(&field.attrs, field.span()) {
            visit::visit_field(self, field);
        }
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for pair in item.variants.pairs() {
            let (variant, comma) = pair.into_tuple();
            if let Some(comma) = comma {
                self.commas.push((variant.span(), comma.span()));
            }
        }
        visit::visit_item_enum(self, item);
    }
    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        if !self.test(&variant.attrs, variant.span()) {
            visit::visit_variant(self, variant);
        }
    }
    fn visit_expr_struct(&mut self, expr: &'ast syn::ExprStruct) {
        for pair in expr.fields.pairs() {
            let (field, comma) = pair.into_tuple();
            if let Some(comma) = comma {
                self.commas.push((field.span(), comma.span()));
            }
        }
        visit::visit_expr_struct(self, expr);
    }
    fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
        if !self.test(&field.attrs, field.span()) {
            visit::visit_field_value(self, field);
        }
    }
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        if !self.test(&arm.attrs, arm.span()) {
            visit::visit_arm(self, arm);
        } else if let Some(comma) = arm.comma {
            self.spans.push(comma.span);
        }
    }
    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        let attrs = match stmt {
            syn::Stmt::Local(i) => &i.attrs,
            syn::Stmt::Macro(i) => &i.attrs,
            syn::Stmt::Expr(i, _) => expression_attributes(i),
            syn::Stmt::Item(_) => &[],
        };
        if !self.test(attrs, stmt.span()) {
            visit::visit_stmt(self, stmt);
        }
    }
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if matches!(predicate(attr), Some(Truth::No | Truth::Unsupported)) {
            self.unsupported = true;
        }
        if attr.path().is_ident("cfg_attr") {
            let tokens = quote::quote!(#attr).to_string();
            if tokens
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|s| s == "test")
            {
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
        commas: vec![],
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
    let implementation = String::from_utf8(bytes)?;
    syn::parse_file(&implementation)?;
    Ok(implementation)
}
