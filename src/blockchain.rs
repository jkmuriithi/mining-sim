use std::{collections::HashMap, ops::Index};

use crate::block::{Block, BlockID};

/// Representation of a public blockchain which is mined on by a set of
/// [Miners](crate::miner::Miner). [Blocks](Block) are published to this chain
/// via [Blockchain::publish].
#[derive(Debug, Clone)]
pub struct Blockchain {
    genesis_id: BlockID,
    max_height: usize,
    blocks: HashMap<BlockID, BlockData>,
    blocks_by_height: Vec<Vec<BlockID>>,
}

/// A block and its associated metadata as stored in a [Blockchain] instance.
#[derive(Debug, Default, Clone)]
pub struct BlockData {
    pub block: Block,
    pub height: usize,
    /// All blocks which directly point to `block`. Allows for more flexible
    /// traversal over the chain.
    pub children: Vec<BlockID>,
}

#[derive(Debug, thiserror::Error)]
pub enum BlockPublishingError {
    #[error("block {0} does not contain a parent block ID")]
    NoParentGiven(BlockID),
    #[error("block {child}'s parent {parent} was not found in this chain")]
    ParentNotFound { child: BlockID, parent: BlockID },
    #[error("block {child} cannot have block {parent} as its parent")]
    InvalidParent { child: BlockID, parent: BlockID },
    #[error("block ID {0} already exists on this chain")]
    DuplicateBlockID(BlockID),
}

impl Blockchain {
    /// Creates a new blockchain containing a genesis block. The genesis block
    /// has [BlockID] 0, and is associated with an uninstantiated genesis miner
    /// with [MinerID] 0.
    pub fn new() -> Self {
        // Default BlockData matches the genesis block's BlockData
        let blocks = HashMap::from([(0, BlockData::default())]);

        Blockchain {
            genesis_id: 0,
            max_height: 0,
            blocks,
            blocks_by_height: vec![vec![0]],
        }
    }

    /// Returns true iff the given block ID is associated with a block on the
    /// blockchain.
    #[inline]
    pub fn contains(&self, id: BlockID) -> bool {
        self.blocks.contains_key(&id)
    }

    #[inline]
    /// ID of the genesis block.
    pub fn genesis(&self) -> BlockID {
        self.genesis_id
    }

    #[inline]
    /// Maximum height of any block on the blockchain.
    pub fn max_height(&self) -> usize {
        self.max_height
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
        self.blocks.get(&id).and_then(|opt| opt.block.parent_id)
    }

    /// Returns the IDs of all blocks at the specified height.
    ///
    /// # Panics
    /// Panics if `index` is greater than [Blockchain::max_height].
    #[inline]
    pub fn at_height(&self, index: usize) -> &[BlockID] {
        debug_assert!(
            index <= self.max_height,
            "{} exceeds the maximum height {} of the chain",
            index,
            self.max_height
        );

        &self.blocks_by_height[index]
    }

    /// Returns the IDs of all blocks on the path from the given block ID to the
    /// genesis block, in ascending order of height and including the given
    /// block ID.
    ///
    /// # Panics
    /// If a block with [BlockID] `id` is not present on the chain.
    // TODO: Investigate ways of optimizing this loop
    pub fn ancestors_of(&self, id: BlockID) -> Vec<BlockID> {
        debug_assert!(
            self.contains(id),
            "blockchain does not contain a block with ID: {:?}",
            id
        );

        let mut ancestors = vec![id];

        let mut curr = id;
        while curr != self.genesis_id {
            curr = self.blocks[&curr].block.parent_id.unwrap();
            ancestors.push(curr);
        }

        ancestors.reverse();
        ancestors
    }

    /// Returns the IDs of all blocks on the longest chain, where the tip of the
    /// longest chain is defined as the earliest block published at
    /// [Blockchain::max_height].
    #[inline]
    pub fn longest_chain(&self) -> Vec<BlockID> {
        self.ancestors_of(self.blocks_by_height.last().unwrap()[0])
    }

    /// Adds the given block to the blockchain.
    pub fn publish(
        &mut self,
        block: Block,
    ) -> Result<(), BlockPublishingError> {
        use BlockPublishingError::*;

        if self.contains(block.id) {
            return Err(DuplicateBlockID(block.id));
        }

        let parent_id = match block.parent_id {
            Some(parent_id) => parent_id,
            None => return Err(NoParentGiven(block.id)),
        };

        let parent_data = match self.blocks.get_mut(&parent_id) {
            Some(parent_data) => parent_data,
            None => {
                return Err(ParentNotFound {
                    child: block.id,
                    parent: parent_id,
                })
            }
        };

        if block.id <= parent_data.block.id {
            return Err(InvalidParent {
                child: block.id,
                parent: parent_id,
            });
        }

        parent_data.children.push(block.id);

        // Insert block
        let height = parent_data.height + 1;
        if height > self.max_height {
            debug_assert!(height == self.max_height + 1);

            self.blocks_by_height.push(vec![block.id]);
            self.max_height = height;
        } else {
            self.blocks_by_height[height].push(block.id);
        }

        self.blocks.insert(
            block.id,
            BlockData {
                block,
                height,
                children: vec![],
            },
        );

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

impl Index<&BlockID> for Blockchain {
    type Output = BlockData;

    fn index(&self, index: &BlockID) -> &Self::Output {
        self.blocks.index(index)
    }
}

impl Index<BlockID> for Blockchain {
    type Output = BlockData;

    fn index(&self, index: BlockID) -> &Self::Output {
        self.blocks.index(&index)
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
