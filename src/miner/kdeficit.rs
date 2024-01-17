/*!
## Wisdom of Weinberg:
 - Selfish mining doesn't work as prescribed because the "fork" case isn't
   handled properly.
 - state can be an instance variable which is updated in constant time using
   each get_action call; need some "abandonment" condition for when/if the LC
   changes to a new branch
 - move on to other parametric strategy spaces from Anthony's work after this
   one
*/

use std::collections::VecDeque;

use crate::block::{Block, BlockID};

use super::{ties::TieBreaker, Action, Miner, MinerID};

#[derive(Debug, Clone)]
pub struct KDeficit {
    id: Option<MinerID>,
    tie_breaker: Option<TieBreaker>,
    name: String,
    k: usize,
    capitulation: Option<BlockID>,
    hidden: VecDeque<BlockID>,
}

impl KDeficit {
    pub fn new(k: usize) -> Self {
        assert_ne!(k, 0, "k must be greater than 0");
        Self {
            id: None,
            tie_breaker: None,
            name: format!("{}-Deficit", k),
            k,
            capitulation: None,
            hidden: VecDeque::new(),
        }
    }

    /// Returns a path of blocks from `start` through all hidden blocks. Clears
    /// `self.hidden`.
    fn path(&mut self, start: BlockID) -> Vec<Block> {
        let mut blocks = vec![];

        let mut parent = start;
        let miner = self.id();
        for id in self.hidden.drain(..) {
            blocks.push(Block::new(id, Some(parent), miner, None));
            parent = id;
        }

        blocks
    }
}

impl Default for KDeficit {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Miner for KDeficit {
    fn name(&self) -> String {
        self.name.clone()
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
        chain: &crate::blockchain::Blockchain,
        block: Option<BlockID>,
    ) -> super::Action {
        let id = self.id();
        let tb = self.tie_breaker.unwrap();
        let capitulation = self.capitulation.unwrap_or(chain.genesis_id);

        if let Some(block_id) = block {
            self.hidden.push_back(block_id);
        }

        // IDs of blocks after the capitulation block
        let mut ids = vec![];
        let tip = tb.choose(chain);
        let mut curr = tip;
        while chain[curr].height > chain[capitulation].height {
            assert!(chain[curr].block.miner_id != id);
            ids.push(curr);
            curr = chain[curr].block.parent_id.unwrap();
        }
        ids.reverse();

        // Handle states B_{x, 0}
        if ids.is_empty() {
            return Action::Wait;
        }
        // Handle states B_{0} and B_{0, 1} (and therefore B_{0, x})
        if self.hidden.is_empty() {
            self.capitulation = Some(tip);
            return Action::Wait;
        }
        // Invariant: the earliest block after capitulation belongs to us
        assert!(self.hidden[0] < ids[0]);

        // Merge hidden block ids with ids on the longest chain
        let mut blocks_are_ours =
            Vec::with_capacity(ids.len() + self.hidden.len());
        let mut hidden_idx = 0;
        let mut ids_idx = 0;
        for _ in 0..(ids.len() + self.hidden.len()) {
            if hidden_idx >= self.hidden.len() {
                blocks_are_ours.push(false);
                ids_idx += 1;
            } else if ids_idx >= ids.len() {
                blocks_are_ours.push(true);
                hidden_idx += 1;
            } else if self.hidden[hidden_idx] > ids[ids_idx] {
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
            match (last_block_ours, curr_block_ours) {
                (true, true) | (false, false) => {
                    *counts.last_mut().unwrap() += 1;
                }
                (true, false) | (false, true) => {
                    counts.push(1);
                    last_block_ours = !last_block_ours;
                }
            }
        }

        match &counts[..] {
            [1, x] | [1, x, ..] if *x > self.k => {
                self.capitulation = Some(tip);
                self.hidden.clear();
                Action::Wait
            }
            // [1, 1] => { doesn't work
            //     let next = self.hidden.pop_front().unwrap();
            //     self.capitulation = Some(next);
            //     Action::Publish(Block::new(next, Some(capitulation), id, None))
            // }
            [1, _] => Action::Wait,
            [1, 1, 1] => {
                self.capitulation = Some(*self.hidden.back().unwrap());
                Action::PublishSet(self.path(capitulation))
            }
            [1, _, 1] => Action::Wait,
            [1, _, 1, 1] => {
                self.hidden.pop_front();
                self.capitulation = Some(ids[ids.len() - 2]);
                Action::Wait
            }
            [1, x, _] | [1, x, _, ..] => {
                if self.hidden.len() == ids.len() + 1 {
                    self.capitulation = Some(*self.hidden.back().unwrap());
                    Action::PublishSet(self.path(capitulation))
                } else if self.hidden.len() - 1 == ids.len() - x + 1 {
                    self.hidden.pop_front();
                    self.capitulation = Some(*self.hidden.back().unwrap());
                    Action::PublishSet(self.path(ids[ids.len() - 1]))
                } else {
                    Action::Wait
                }
            }
            [_, ..] => {
                if self.hidden.len() == ids.len() + 1 {
                    self.capitulation = Some(*self.hidden.back().unwrap());
                    Action::PublishSet(self.path(capitulation))
                } else {
                    Action::Wait
                }
            }
            _ => panic!("unrecognized state: {:?}", counts),
        }
    }
}
