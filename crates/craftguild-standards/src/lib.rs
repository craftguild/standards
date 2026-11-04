//! Distribution tooling. Policies and adapters remain separate from presentation.

#![forbid(unsafe_code)]

pub mod banner;
pub mod command;
pub mod config;
pub mod contract;
pub mod distribution;
pub mod generated;
pub mod html_regions;
pub mod inventory;
pub mod measure;
pub mod npm_bundle;
pub mod npm_regions;
pub mod policies;
pub mod publication;
pub mod quality;
pub mod release;
pub mod rust_graph;
pub mod rust_regions;
pub mod sea_orm;
