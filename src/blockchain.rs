use std::{cmp::Ordering, collections::HashMap, ops::Index};

use thiserror::Error;

use crate::{
    block::{Block, BlockID},
    miner::MinerID,
};

/// Representation of a public blockchain which is mined on by a set of
/// [Miners](crate::miner::Miner). [Blocks](Block) are published to this chain
/// via [Blockchain::publish].
#[derive(Debug, Clone)]
pub struct Blockchain {
    /// The genesis block of this blockchain.
    pub genesis: BlockID,
    /// Maximum height of any block on the chain.
    pub max_height: u64,
    /// Map from the ID of a block to its associated data.
    blocks: HashMap<BlockID, BlockData>,
    /// IDs of all blocks in the chain, sorted first by height, then by the
    /// order in which they were published.
    blocks_by_height: Vec<Vec<BlockID>>,
}

/// A block and its associated metadata as held within a [Blockchain] instance.
#[derive(Debug, Clone)]
pub struct BlockData {
    pub block: Block,
    /// Length of the path from `block` to the genesis block of the blockchain.
    pub height: u64,
    /// All blocks which directly point to `block`. Allows for more flexible
    /// traversal over the chain.
    pub children: Vec<BlockID>,
}

#[derive(Debug, Error)]
pub enum BlockInsertionError {
    #[error("block does not contain a parent block ID")]
    NoParentGiven,
    #[error("block's parent was not found in this chain")]
    ParentNotFound,
    #[error("block's parent was mined in the same or later round")]
    InvalidParent,
    #[error("block ID already exists on this chain")]
    DuplicateBlockID,
}

impl Blockchain {
    /// Creates a new blockchain containing a genesis block. The genesis block
    /// has [BlockID] 0, and is associated with an uninstantiated genesis miner
    /// with [MinerID] 0.
    pub fn new() -> Self {
        let genesis = Block::new(0.into(), None, 0.into(), None);
        let blocks = HashMap::from([(
            0.into(),
            BlockData { block: genesis, height: 0, children: vec![] },
        )]);

        Blockchain {
            genesis: 0.into(),
            max_height: 0,
            blocks,
            blocks_by_height: vec![vec![0.into()]],
        }
    }

    /// Creates a new blockchain containing a genesis block, in which the
    /// genesis block contains a [MinerID] of `miner_id`.
    pub fn with_genesis_miner(miner_id: MinerID) -> Self {
        let mut chain = Self::new();
        chain.blocks.get_mut(&chain.genesis).unwrap().block.miner = miner_id;

        chain
    }

    /// Returns true iff the given block ID is associated with a block on the
    /// blockchain.
    #[inline]
    pub fn contains(&self, id: BlockID) -> bool {
        self.blocks.contains_key(&id)
    }

    /// Returns a reference to the [BlockData] associated with the given block
    /// ID on the blockchain.
    #[inline]
    pub fn get(&self, id: BlockID) -> Option<&BlockData> {
        self.blocks.get(&id)
    }

    /// Returns the parent of the block with the given ID.
    #[inline]
    pub fn get_parent(&self, id: BlockID) -> Option<BlockID> {
        self.blocks.get(&id).and_then(|opt| opt.block.parent)
    }

    /// Returns the IDs of all blocks at the specified height.
    ///
    /// ## Panics
    /// Panics if `index` is greater than [Blockchain::max_height].
    #[inline]
    pub fn at_height(&self, index: u64) -> &[BlockID] {
        assert!(
            index <= self.max_height,
            "{} exceeds the maximum height {} of the chain",
            index,
            self.max_height
        );
        &self.blocks_by_height[index as usize]
    }

    /// Returns the IDs of all blocks on the path from the given block ID to the
    /// genesis block, in ascending order of height and including the given
    /// block ID.
    /// ## Panics
    /// If a block with [BlockID] `id` is not present on the chain.
    pub fn ancestors_of(&self, id: BlockID) -> Vec<BlockID> {
        assert!(
            self.contains(id),
            "blockchain does not contain a block with ID: {:?}",
            id
        );

        let mut ancestors = vec![id];

        let mut curr = id;
        while curr != self.genesis {
            curr = self.get_parent(curr).unwrap();
            ancestors.push(curr);
        }

        ancestors.reverse();
        ancestors
    }

    /// Returns the IDs of all blocks on the longest chain, in order from the
    /// genesis block to the tip of the chain, where the tip of the chain is the
    /// earliest published block with height [Blockchain::tip_height].
    #[inline]
    pub fn longest_chain(&self) -> Vec<BlockID> {
        self.ancestors_of(self.blocks_by_height.last().unwrap()[0])
    }

    /// Adds the given block to the blockchain.
    pub fn publish(&mut self, block: Block) -> Result<(), BlockInsertionError> {
        use BlockInsertionError::*;

        // Validate block
        if self.contains(block.id) {
            return Err(DuplicateBlockID);
        }
        let parent = match block.parent {
            None => return Err(NoParentGiven),
            Some(parent) => {
                if !self.contains(parent) {
                    return Err(ParentNotFound);
                } else {
                    self.blocks.get_mut(&parent).unwrap()
                }
            }
        };

        if block <= parent.block {
            return Err(InvalidParent);
        }
        parent.children.push(block.id);

        // Insert block
        let height = parent.height + 1;
        match self.max_height.cmp(&height) {
            Ordering::Less => {
                debug_assert!(height == self.max_height + 1);
                self.blocks_by_height.push(vec![block.id]);
                self.max_height = height;
            }
            Ordering::Equal => {
                self.blocks_by_height.last_mut().unwrap().push(block.id);
            }
            _ => (),
        }

        let id = block.id;
        let data = BlockData { block, height, children: vec![] };
        self.blocks.insert(id, data);

        Ok(())
    }

    /// Returns the IDs of all blocks at the tip of the longest chain.
    #[inline]
    pub fn tip(&self) -> &[BlockID] {
        self.blocks_by_height.last().unwrap()
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<BlockID> for Blockchain {
    type Output = BlockData;

    fn index(&self, index: BlockID) -> &Self::Output {
        self.blocks.index(&index)
    }
}

impl Index<&BlockID> for Blockchain {
    type Output = BlockData;

    fn index(&self, index: &BlockID) -> &Self::Output {
        self.blocks.index(index)
    }
}

impl Index<u64> for Blockchain {
    type Output = BlockData;

    fn index(&self, index: u64) -> &Self::Output {
        self.blocks.index(&index.into())
    }
}

#[cfg(test)]
mod tests {
    use super::Blockchain;

    #[test]
    fn new_instance_longest_chain() {
        let chain = Blockchain::new();
        let lc = chain.longest_chain();

        assert_eq!(lc.len(), 1);
        assert_eq!(lc[0], chain.blocks_by_height[0][0]);
    }
}
