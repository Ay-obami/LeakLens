//! Zcash SDK proposal adapters for LeakLens.

pub mod fixture;
mod adapter;

pub use adapter::{AdapterError, extract_proposal_facts};
