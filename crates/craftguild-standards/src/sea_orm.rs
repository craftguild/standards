//! Syntax-level SeaORM escape-hatch detection, without comments/string matches.

use craftguild_core::Result;
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeSet;
use syn::{
    visit::{self, Visit},
    UseTree,
};

const RAW: &[&str] = &[
    "execute_raw",
    "execute_unprepared",
    "query_one_raw",
    "query_all_raw",
    "from_raw_sql",
    "from_sql_and_values",
    "from_sql_and_values_with_timezone",
    "get_sqlite_connection_pool",
    "get_rusqlite_connection",
    "raw_sql",
    "raw_query",
    "query_as",
    "query_scalar",
];

struct Policy<'a> {
    drivers: &'a BTreeSet<String>,
    violations: BTreeSet<String>,
}

impl Policy<'_> {
    fn name(&mut self, name: &str) {
        if RAW.contains(&name) || self.drivers.contains(name) || name == "Statement" {
            self.violations.insert(name.to_owned());
        }
    }
    fn path_names(&mut self, names: &[String]) {
        for name in names {
            self.name(name);
        }
        if names
            .windows(2)
            .any(|p| p[0] == "Expr" && p[1].starts_with("cust"))
        {
            self.violations.insert("Expr::cust*".into());
        }
    }
    fn imports(&mut self, tree: &UseTree, mut prefix: Vec<String>) {
        match tree {
            UseTree::Path(p) => {
                prefix.push(p.ident.to_string());
                self.imports(&p.tree, prefix);
            }
            UseTree::Name(n) => {
                prefix.push(n.ident.to_string());
                self.path_names(&prefix);
            }
            UseTree::Rename(n) => {
                prefix.push(n.ident.to_string());
                self.path_names(&prefix);
            }
            UseTree::Group(g) => {
                for item in &g.items {
                    self.imports(item, prefix.clone());
                }
            }
            UseTree::Glob(_) => self.path_names(&prefix),
        }
    }
    fn tokens(&mut self, tokens: TokenStream) {
        let mut names = vec![];
        for token in tokens {
            match token {
                TokenTree::Group(group) => self.tokens(group.stream()),
                TokenTree::Ident(id) => names.push(id.to_string()),
                TokenTree::Literal(_) => {
                    self.path_names(&names);
                    names.clear();
                }
                _ => {}
            }
        }
        self.path_names(&names);
    }
}

impl<'ast> Visit<'ast> for Policy<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.path_names(
            &path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>(),
        );
        visit::visit_path(self, path);
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports(&item.tree, vec![]);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.name(&call.method.to_string());
        visit::visit_expr_method_call(self, call);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.visit_path(&mac.path);
        self.tokens(mac.tokens.clone());
    }
}

pub fn check(source: &str, drivers: &BTreeSet<String>) -> Result<Vec<String>> {
    let syntax = syn::parse_file(source)?;
    let mut policy = Policy {
        drivers,
        violations: BTreeSet::new(),
    };
    policy.visit_file(&syntax);
    Ok(policy.violations.into_iter().collect())
}
