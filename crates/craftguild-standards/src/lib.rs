//! Distribution tooling. Policies and adapters remain separate from presentation.

#![forbid(unsafe_code)]

pub mod banner;
pub mod command;
pub mod config;
pub mod distribution;
pub mod generated;
pub mod inventory;
pub mod measure;
pub mod npm_bundle;
pub mod policies;
pub mod quality;
pub mod rust_graph;
pub mod rust_regions;
pub mod sea_orm;
