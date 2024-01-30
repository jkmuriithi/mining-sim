//! Definitions for blocks

use crate::{miner::MinerID, transaction::Transaction};

/// Numeric type for block identifiers.
pub type BlockID = usize;

/// Representation of a mined block of transactions.
#[derive(Debug, Default, Clone)]
pub struct Block {
    /// Unique identifier of this block.
    pub id: BlockID,
    /// ID of this block's parent.
    pub parent_id: Option<BlockID>,
    /// ID of this block's miner.
    pub miner_id: MinerID,
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
