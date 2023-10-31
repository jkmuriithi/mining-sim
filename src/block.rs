use crate::{miner::MinerID, transaction::Transaction};

/// Representation of a mined block of transactions.
#[derive(Debug, Clone)]
pub struct Block {
    /// The round this block's parent was mined in.
    pub parent: Option<BlockID>,
    /// The round this block was mined in.
    pub id: BlockID,
    /// The miner of this block.
    pub miner: MinerID,
    /// The transactions contained within this block.
    pub txns: Option<Vec<Transaction>>,
}

/// A unique identifier assigned to each [Block]. Directly corresponds to the
/// round that a block was mined in.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct BlockID(u64);

impl From<u64> for BlockID {
    fn from(value: u64) -> Self {
        BlockID(value)
    }
}

impl Block {
    pub fn new(
        id: BlockID,
        parent: Option<BlockID>,
        miner: MinerID,
        txns: Option<Vec<Transaction>>,
    ) -> Self {
        Block {
            parent,
            id,
            miner,
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
