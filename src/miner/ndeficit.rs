//! Implementation of the N-Deficit family of mining strategies

// Wisdom of Weinberg:
//  - Selfish mining doesn't work as prescribed because the "fork" case isn't
//    handled properly.
//  - state can be an instance variable which is updated in constant time using
//    each get_action call; need some "abandonment" condition for when/if the LC
//    changes to a new branch
//  - move on to other parametric strategy spaces from Anthony's work after this
//    one

use std::{collections::VecDeque, num::NonZeroUsize};

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
    tie_breaker::TieBreaker,
};

use super::{Action, Miner, MinerID};

#[derive(Debug, Clone, Default)]
pub struct NDeficit {
    capitulation: Option<BlockID>,
    hidden_blocks: VecDeque<BlockID>,
    i: Option<NonZeroUsize>,
    id: Option<MinerID>,
    tie_breaker: Option<TieBreaker>,
}

impl NDeficit {
    pub fn new(i: usize) -> Self {
        Self {
            i: NonZeroUsize::new(i),
            ..Default::default()
        }
    }

    /// Returns a path of blocks from `parent` through all hidden blocks. Clears
    /// `self.hidden`.
    fn make_block_path(&mut self, parent: BlockID) -> Vec<Block> {
        let miner = self.id();

        let mut blocks = vec![];
        let mut parent = parent;
        for id in self.hidden_blocks.drain(..) {
            blocks.push({
                Block {
                    id,
                    parent_id: Some(parent),
                    miner_id: miner,
                    txns: None,
                }
            });
            parent = id;
        }

        blocks
    }
}

impl Miner for NDeficit {
    fn name(&self) -> String {
        format!("{}-Deficit", self.i.unwrap().get())
    }

    fn id(&self) -> MinerID {
        self.id.expect("miner ID to be set")
    }

    fn set_id(&mut self, id: MinerID) {
        self.id = Some(id);
        self.tie_breaker = Some(TieBreaker::FavorMiner(id));
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> super::Action {
        if let Some(block_id) = block {
            self.hidden_blocks.push_back(block_id);
        }

        let id = self.id();
        let n = self.i.expect("n greater than 0").get();
        let capitulation = self.capitulation.unwrap_or(chain.genesis());

        let tip = self.tie_breaker.unwrap().choose(chain);

        // Handle states B_{0} and B_{0, 1} (and therefore B_{0, x})
        if self.hidden_blocks.is_empty() {
            self.capitulation = Some(tip);
            return Action::Wait;
        }

        // IDs of blocks after the capitulation block
        let mut ids = vec![];
        let mut curr = tip;
        while chain[curr].height > chain[capitulation].height {
            debug_assert!(chain[curr].block.miner_id != id);

            ids.push(curr);
            curr = chain[curr].block.parent_id.unwrap();
        }
        ids.reverse();

        // Handle states B_{x, 0}
        if ids.is_empty() {
            return Action::Wait;
        }

        // Invariant: our chain of hidden blocks starts before any other blocks
        assert!(self.hidden_blocks[0] < ids[0]);

        // Merge hidden block ids with ids on the longest chain
        let mut blocks_are_ours =
            Vec::with_capacity(ids.len() + self.hidden_blocks.len());
        let mut hidden_idx = 0;
        let mut ids_idx = 0;
        for _ in 0..(ids.len() + self.hidden_blocks.len()) {
            if hidden_idx >= self.hidden_blocks.len() {
                blocks_are_ours.push(false);
                ids_idx += 1;
            } else if ids_idx >= ids.len() {
                blocks_are_ours.push(true);
                hidden_idx += 1;
            } else if self.hidden_blocks[hidden_idx] > ids[ids_idx] {
                blocks_are_ours.push(false);
                ids_idx += 1;
            } else {
                blocks_are_ours.push(true);
                hidden_idx += 1;
            }
        }

        // Build up alternating counts of attacker and honest blocks
        // (essentially abbreviated state notation)
        let mut counts = vec![];
        let mut last_block_ours = false;
        for curr_block_ours in blocks_are_ours {
            if last_block_ours == curr_block_ours {
                *counts.last_mut().unwrap() += 1;
            } else {
                counts.push(1);
            }

            last_block_ours = curr_block_ours;
        }

        match &counts[..] {
            [1, x] | [1, x, ..] if *x > n => {
                self.capitulation = Some(tip);
                self.hidden_blocks.clear();
                Action::Wait
            }
            // [1, 1] => { doesn't work
            //     let next = self.hidden.pop_front().unwrap();
            //     self.capitulation = Some(next);
            //     Action::Publish(Block::new(next, Some(capitulation), id, None))
            // }
            [1, _] => Action::Wait,
            [1, 1, 1] => {
                self.capitulation = Some(*self.hidden_blocks.back().unwrap());
                Action::PublishSet(self.make_block_path(capitulation))
            }
            [1, _, 1] => Action::Wait,
            [1, _, 1, 1] => {
                self.hidden_blocks.pop_front();
                self.capitulation = Some(ids[ids.len() - 2]);
                Action::Wait
            }
            [1, x, _] | [1, x, _, ..] => {
                if self.hidden_blocks.len() == ids.len() + 1 {
                    self.capitulation =
                        Some(*self.hidden_blocks.back().unwrap());
                    Action::PublishSet(self.make_block_path(capitulation))
                } else if self.hidden_blocks.len() - 1 == ids.len() - x + 1 {
                    self.hidden_blocks.pop_front();
                    self.capitulation =
                        Some(*self.hidden_blocks.back().unwrap());
                    Action::PublishSet(self.make_block_path(ids[ids.len() - 1]))
                } else {
                    Action::Wait
                }
            }
            [_, ..] => {
                if self.hidden_blocks.len() == ids.len() + 1 {
                    self.capitulation =
                        Some(*self.hidden_blocks.back().unwrap());
                    Action::PublishSet(self.make_block_path(capitulation))
                } else {
                    Action::Wait
                }
            }
            _ => unreachable!("unrecognized state: {:?}", counts),
        }
    }
}
