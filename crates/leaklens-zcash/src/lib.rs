//! Adapts actual Zcash wallet proposals into key-free facts for LeakLens.

mod adapter;

pub use adapter::{AdapterError, extract_proposal_facts};
