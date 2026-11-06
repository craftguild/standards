//! Distribution tooling. Policies and adapters remain separate from presentation.

#![forbid(unsafe_code)]

pub mod banner;
pub mod build_config;
pub mod build_settings;
pub mod command;
pub mod config;
pub mod contract;
pub mod dispatch;
pub mod distribution;
pub mod execution;
pub mod execution_context;
pub mod execution_drivers;
pub mod execution_hooks;
pub mod execution_install;
pub mod execution_process;
pub mod execution_record;
pub mod execution_transaction;
pub mod generated;
pub mod html_regions;
mod init_baseline;
pub mod init_plan;
pub mod init_policy;
pub mod init_transaction;
pub mod initialization;
pub mod installer;
pub mod installer_config;
pub mod inventory;
pub mod measure;
pub mod npm_bundle;
pub mod npm_regions;
pub mod operation_config;
pub mod operations;
pub mod package;
pub mod policies;
pub mod prime;
pub mod publication;
pub mod quality;
pub mod release;
pub mod repository_pin;
pub mod rust_artifact;
mod rust_cfg;
pub mod rust_graph;
pub mod rust_regions;
pub mod sea_orm;
pub mod verification;
