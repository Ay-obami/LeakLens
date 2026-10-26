//! Zcash SDK proposal adapters for LeakLens.

mod adapter;
mod canonical;
pub mod fixture;

pub use adapter::{AdapterError, extract_proposal_facts};
pub use canonical::encode_canonical_standard_proposal;
