//! Zcash SDK proposal adapters for LeakLens.

pub mod fixture;
mod adapter;
mod canonical;

pub use adapter::{AdapterError, extract_proposal_facts};
pub use canonical::encode_canonical_standard_proposal;
