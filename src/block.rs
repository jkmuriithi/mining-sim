use crate::{miner::MinerID, transaction::Transaction};

/// A block's unique identifier.
pub type BlockID = usize;

/// Representation of a mined block of transactions.
#[derive(Debug, Default, Clone)]
pub struct Block {
    pub id: BlockID,
    pub parent_id: Option<BlockID>,
    pub miner_id: MinerID,
    pub txns: Option<Vec<Transaction>>,
}

impl Block {
    pub fn new(
        id: BlockID,
        parent_id: Option<BlockID>,
        miner_id: MinerID,
        txns: Option<Vec<Transaction>>,
    ) -> Self {
        Block {
            id,
            parent_id,
            miner_id,
            txns,
        }
    }
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
