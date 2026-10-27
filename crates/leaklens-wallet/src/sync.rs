//! Bounded testnet-only lightwalletd synchronization for a persistent wallet.
//!
//! This component never signs or broadcasts. Network requests expose the
//! client's IP address to the configured lightwalletd operator.

use std::path::Path;

use anyhow::{Context, Result, bail};
use futures_util::TryStreamExt;
use orchard::tree::MerkleHashOrchard;
use prost::Message;
use rusqlite::{Connection, params};
use sapling::Node;
use tonic::transport::Channel;
use zcash_client_backend::{
    data_api::{
        WalletCommitmentTrees, WalletRead, WalletWrite,
        chain::{CommitmentTreeRoot, scan_cached_blocks},
    },
    proto::service::{self, compact_tx_streamer_client::CompactTxStreamerClient},
};
use zcash_client_sqlite::{BlockDb, chain::init::init_cache_database};
use zcash_primitives::merkle_tree::HashSer;
use zcash_protocol::consensus::{BlockHeight, Network};

use crate::storage::TestnetWalletDb;

/// A deliberately bounded sync request. A sync never silently rewinds wallet
/// history or assumes a mainnet chain is acceptable.
#[derive(Clone, Debug)]
pub struct SyncConfig {
    endpoint: String,
    max_blocks: usize,
}

impl SyncConfig {
    pub fn new(endpoint: impl Into<String>, max_blocks: usize) -> Result<Self> {
        let endpoint = endpoint.into();
        if !endpoint.starts_with("https://") {
            bail!("lightwalletd must use an https endpoint");
        }
        if max_blocks == 0 {
            bail!("scan budget must be nonzero");
        }
        if max_blocks > 1000 {
            bail!("scan budget cannot exceed 1000 compact blocks per run");
        }
        Ok(Self {
            endpoint,
            max_blocks,
        })
    }

    pub fn max_blocks(&self) -> usize {
        self.max_blocks
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncReport {
    pub remote_tip: BlockHeight,
    pub scanned_blocks: usize,
}

/// Refresh completed Sapling and Orchard commitment-tree subtrees.
///
/// Lightwalletd streams subtree roots in index order. Passing index zero to
/// the wallet's insertion routines makes this operation repeatable.
async fn update_subtree_roots(
    client: &mut CompactTxStreamerClient<Channel>,
    wallet_db: &mut TestnetWalletDb,
) -> Result<()> {
    let mut sapling_request = service::GetSubtreeRootsArg::default();
    sapling_request.set_shielded_protocol(service::ShieldedProtocol::Sapling);
    let sapling_roots: Vec<CommitmentTreeRoot<Node>> = client
        .get_subtree_roots(sapling_request)
        .await
        .context("fetching Sapling subtree roots")?
        .into_inner()
        .and_then(|root| async move {
            let node = Node::read(&root.root_hash[..])?;
            Ok(CommitmentTreeRoot::from_parts(
                BlockHeight::from_u32(root.completing_block_height as u32),
                node,
            ))
        })
        .try_collect()
        .await
        .context("decoding Sapling subtree roots")?;
    wallet_db
        .put_sapling_subtree_roots(0, &sapling_roots)
        .context("persisting Sapling subtree roots")?;

    let mut orchard_request = service::GetSubtreeRootsArg::default();
    orchard_request.set_shielded_protocol(service::ShieldedProtocol::Orchard);
    let orchard_roots: Vec<CommitmentTreeRoot<MerkleHashOrchard>> = client
        .get_subtree_roots(orchard_request)
        .await
        .context("fetching Orchard subtree roots")?
        .into_inner()
        .and_then(|root| async move {
            let node = MerkleHashOrchard::read(&root.root_hash[..])?;
            Ok(CommitmentTreeRoot::from_parts(
                BlockHeight::from_u32(root.completing_block_height as u32),
                node,
            ))
        })
        .try_collect()
        .await
        .context("decoding Orchard subtree roots")?;
    wallet_db
        .put_orchard_subtree_roots(0, &orchard_roots)
        .context("persisting Orchard subtree roots")?;
    Ok(())
}

/// Reject an accidental mainnet or non-testnet backend before touching the
/// caller's wallet database. lightwalletd documents `chainName` as "test".
fn validate_chain_name(chain_name: &str) -> Result<()> {
    if chain_name != "test" {
        bail!("lightwalletd is not on Zcash testnet (chainName={chain_name:?})");
    }
    Ok(())
}

/// Synchronize at most `config.max_blocks()` blocks, preserving wallet state
/// across calls. `cache_path` must be separate from the wallet database path.
/// On a reorganization or non-contiguous stream we surface an error rather
/// than deleting data behind the user's back.
pub async fn sync_testnet_wallet(
    wallet_db: &mut TestnetWalletDb,
    cache_path: &Path,
    config: &SyncConfig,
) -> Result<SyncReport> {
    let mut client = CompactTxStreamerClient::connect(config.endpoint.clone())
        .await
        .context("connecting to TLS lightwalletd")?;

    let info = client
        .get_lightd_info(service::Empty::default())
        .await
        .context("reading lightwalletd network identity")?
        .into_inner();
    validate_chain_name(&info.chain_name)?;

    update_subtree_roots(&mut client, wallet_db).await?;

    let tip_reply = client
        .get_latest_block(service::ChainSpec::default())
        .await
        .context("reading lightwalletd chain tip")?
        .into_inner();
    let remote_tip: BlockHeight = tip_reply
        .height
        .try_into()
        .context("remote height out of range")?;
    wallet_db
        .update_chain_tip(remote_tip)
        .context("recording latest testnet chain tip")?;

    // The SQLite block cache is distinct from the wallet's account database.
    let cache = BlockDb::for_path(cache_path).context("opening compact block cache")?;
    init_cache_database(&cache).context("initializing compact block cache")?;

    let ranges = wallet_db
        .suggest_scan_ranges()
        .context("determining wallet scan ranges")?;
    let Some(range) = ranges.first() else {
        return Ok(SyncReport {
            remote_tip,
            scanned_blocks: 0,
        });
    };
    let from = range.block_range().start;
    let desired_end = range.block_range().end;
    let end = BlockHeight::from_u32(
        u32::from(from)
            .saturating_add(config.max_blocks as u32)
            .min(u32::from(desired_end))
            .min(u32::from(remote_tip).saturating_add(1)),
    );
    if from >= end {
        return Ok(SyncReport {
            remote_tip,
            scanned_blocks: 0,
        });
    }
    let predecessor = u32::from(from)
        .checked_sub(1)
        .context("scan range begins at genesis; no predecessor tree state")?;
    let tree_state_reply = client
        .get_tree_state(service::BlockId {
            height: BlockHeight::from_u32(predecessor).into(),
            hash: vec![],
        })
        .await
        .context("fetching authenticated preceding tree state")?
        .into_inner();
    if tree_state_reply.network != "test" {
        bail!(
            "lightwalletd provided non-testnet tree state: {:?}",
            tree_state_reply.network
        );
    }
    let tree_state = tree_state_reply
        .to_chain_state()
        .context("decoding preceding tree state")?;

    let block_range = service::BlockRange {
        start: Some(service::BlockId {
            height: from.into(),
            hash: vec![],
        }),
        end: Some(service::BlockId {
            height: (end - 1).into(),
            hash: vec![],
        }),
        ..Default::default()
    };
    let mut blocks = client
        .get_block_range(block_range)
        .await
        .context("requesting compact block range")?
        .into_inner();
    let mut writer = Connection::open(cache_path).context("opening cache insertion connection")?;
    let transaction = writer
        .transaction()
        .context("starting atomic compact-block cache write")?;
    let mut expected = u32::from(from);
    while let Some(block) = blocks
        .try_next()
        .await
        .context("streaming compact blocks")?
    {
        let height = u32::try_from(block.height).context("compact block height out of bounds")?;
        if height != expected {
            bail!(
                "non-contiguous lightwalletd compact block stream at {height}; expected {expected}"
            );
        }
        transaction
            .execute(
                "INSERT OR REPLACE INTO compactblocks (height, data) VALUES (?1, ?2)",
                params![i64::from(height), block.encode_to_vec()],
            )
            .context("caching compact block")?;
        expected += 1;
    }
    if expected != u32::from(end) {
        bail!(
            "lightwalletd block stream ended early; expected end {}, got {}",
            u32::from(end),
            expected
        );
    }
    transaction
        .commit()
        .context("committing contiguous compact blocks")?;
    scan_cached_blocks(
        &Network::TestNetwork,
        &cache,
        wallet_db,
        from,
        &tree_state,
        usize::try_from(u32::from(end) - u32::from(from))?,
    )
    .context("scanning compact blocks into wallet database")?;
    Ok(SyncReport {
        remote_tip,
        scanned_blocks: usize::try_from(u32::from(end) - u32::from(from))?,
    })
}

#[cfg(test)]
mod tests {
    use super::validate_chain_name;

    #[test]
    fn refuses_mainnet_or_empty_chain_name() {
        assert!(validate_chain_name("main").is_err());
        assert!(validate_chain_name("").is_err());
    }

    #[test]
    fn accepts_documented_testnet_chain_name() {
        validate_chain_name("test").expect("expected lightwalletd testnet identifier");
    }
}
