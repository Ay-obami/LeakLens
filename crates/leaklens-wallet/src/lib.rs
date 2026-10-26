//! Minimal persistent, testnet-only Zcash wallet database adapter.
//!
//! This module does not generate, store, export, or print secret material.
//! Transaction synchronization and proposal generation are separate concerns.

pub mod storage;

pub mod proposal;
