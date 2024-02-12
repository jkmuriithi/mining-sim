//! Definitions for blocks

use crate::{miner::MinerId, transaction::Transaction};

/// Representation of a mined block of transactions.
#[derive(Debug, Default, Clone)]
pub struct Block {
    /// Unique identifier of this block.
    pub id: BlockId,
    /// ID of this block's parent.
    pub parent_id: Option<BlockId>,
    /// ID of this block's miner.
    pub miner_id: MinerId,
    /// Transaction data included in this block.
    pub txns: Option<Vec<Transaction>>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Block {}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Block {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Unique identifier of a [`Block`].
///
/// # Invariants
///
/// There will never be more than one block with [`BlockId`] `0` on a
/// blockchain, as [`BlockId`] `0` is reserved for
/// [`Blockchain::GENESIS`](crate::blockchain::Blockchain), and a blockchain
/// will not accept a block with a duplicate ID.
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub usize);

impl From<usize> for BlockId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
