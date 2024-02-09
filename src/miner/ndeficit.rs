//! Implementation of the N-Deficit family of mining strategies

// Wisdom of Weinberg:
//  - Selfish mining doesn't work as prescribed because the "fork" case isn't
//    handled properly.
//  - state can be an instance variable which is updated in constant time using
//    each get_action call; need some "abandonment" condition for when/if the LC
//    changes to a new branch
//  - move on to other parametric strategy spaces from Anthony's work after this
//    one

use std::collections::{HashSet, VecDeque};

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
    tie_breaker::TieBreaker,
};

use super::{Action, Miner, MinerID};

#[derive(Debug, Clone, Default)]
pub struct NDeficit {
    i: usize,
    id: Option<MinerID>,
    tie_breaker: Option<TieBreaker>,

    // Blockchain state tracking
    capitulation: BlockID,
    state: Vec<StateEntry>,
    seen: HashSet<BlockID>,
    our_blocks: VecDeque<BlockID>,
    honest_blocks: Vec<BlockID>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StateEntry {
    /// Count of consecutive "attacker" (our) blocks
    A(usize),
    /// Count of consecutive "honest" (other) blocks
    H(usize),
}

impl NDeficit {
    pub fn new(i: usize) -> Self {
        Self {
            i,
            ..Default::default()
        }
    }

    fn clear_state(&mut self) {
        self.state.clear();
        self.seen.clear();
        self.our_blocks.clear();
        self.honest_blocks.clear();
    }

    /// Capitulates to B_{0, 0} with `head` as the genesis block.
    fn capitulate(&mut self, genesis: BlockID) {
        self.capitulation = genesis;
        self.clear_state();
    }

    fn update_state(&mut self, chain: &Blockchain, block: Option<&BlockID>) {
        let tip = self.tie_breaker.unwrap().choose(chain);
        let cap_height = chain[self.capitulation].height;

        if !self.our_blocks.is_empty() {
            let mut unseen_blocks = vec![];
            let mut curr = tip;
            while curr != self.capitulation && !self.seen.contains(&curr) {
                // Clear state if we're no longer on the longest chain
                if chain[curr].height <= cap_height {
                    self.capitulate(tip);
                    return;
                }

                unseen_blocks.push(curr);
                self.seen.insert(curr);

                curr = chain[curr].block.parent_id.unwrap();
            }

            if !unseen_blocks.is_empty() {
                let unseen = unseen_blocks.len();

                match self.state.last_mut() {
                    Some(StateEntry::H(count)) => *count += unseen,
                    _ => self.state.push(StateEntry::H(unseen)),
                }

                while let Some(block) = unseen_blocks.pop() {
                    self.honest_blocks.push(block);
                }
            }
        } else {
            self.capitulation = tip;
        }

        if let Some(block_id) = block {
            self.seen.insert(*block_id);
            self.our_blocks.push_back(*block_id);

            match self.state.last_mut() {
                Some(StateEntry::A(count)) => *count += 1,
                _ => self.state.push(StateEntry::A(1)),
            }
        }
    }

    /// Returns a path of blocks from `parent` through all hidden blocks. Clears
    /// `self.hidden`.
    fn block_path_to(&mut self, parent: BlockID) -> Vec<Block> {
        let miner_id = self.id();

        let mut blocks = vec![];
        let mut parent = parent;
        self.our_blocks.drain(..).for_each(|id| {
            blocks.push({
                Block {
                    id,
                    parent_id: Some(parent),
                    miner_id,
                    txns: None,
                }
            });
            parent = id;
        });

        blocks
    }

    fn publish_all(&mut self) -> Action {
        let path = self.block_path_to(self.capitulation);
        self.capitulate(path.last().unwrap().id);
        Action::PublishSet(path)
    }

    fn map_state(&mut self) -> Action {
        use StateEntry::*;

        // All non-empty states should be of the form [A(x), ..]
        match &self.state[..] {
            [] => Action::Wait,
            [A(1)] => Action::Wait,
            [A(2..), ..] => {
                if self.our_blocks.len() == self.honest_blocks.len() + 1 {
                    self.publish_all()
                } else {
                    Action::Wait
                }
            }
            [A(1), H(x)] => {
                if *x > self.i {
                    self.capitulate(self.honest_blocks[x - 1]);
                }

                Action::Wait
            }
            [A(1), H(1), A(1)] => self.publish_all(),
            [A(1), H(x), A(1)] => {
                if *x > self.i {
                    self.capitulate(self.honest_blocks[x - 1]);
                }

                Action::Wait
            }
            [A(1), H(x), A(2..), ..] => {
                assert!(*x <= self.i);

                let ours = self.our_blocks.len();
                let honest = self.honest_blocks.len();

                if ours == honest + 1 {
                    self.publish_all()
                } else if ours - 1 == honest - x + 1 {
                    self.our_blocks.pop_front();
                    let path = self.block_path_to(self.honest_blocks[x - 1]);
                    self.capitulate(path.last().unwrap().id);

                    Action::PublishSet(path)
                } else {
                    Action::Wait
                }
            }
            [A(1), H(x), A(1), H(1)] => {
                assert!(*x <= self.i);

                // Manually capitulate to B_{1, 1}
                let temp_x = *x;
                self.state = vec![A(1), H(1)];

                self.capitulation = self.honest_blocks[temp_x - 1];
                self.our_blocks.pop_front();
                self.honest_blocks.drain(..temp_x);
                self.seen.clear();
                self.seen.insert(self.our_blocks[0]);
                self.seen.insert(self.honest_blocks[0]);

                Action::Wait
            }
            _ => panic!("illegal n-deficit state: {:?}", &self.state),
        }
    }
}

impl Miner for NDeficit {
    fn name(&self) -> String {
        format!("{}-Deficit", self.i)
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
        self.update_state(chain, block.as_ref());

        // Handle selfish mining fork case
        if self.our_blocks.len() == 1 {
            let miner_id = self.id();
            let lc = chain.tip();

            let ours_at_lc =
                lc.iter().find(|&&b| chain[b].block.miner_id == miner_id);
            let othr_at_lc =
                lc.iter().find(|&&b| chain[b].block.miner_id != miner_id);

            if let (Some(parent_id), Some(_)) = (ours_at_lc, othr_at_lc) {
                let block_id = self.our_blocks[0];
                self.capitulate(block_id);

                return Action::Publish(Block {
                    id: block_id,
                    miner_id,
                    parent_id: Some(*parent_id),
                    txns: None,
                });
            }
        }

        self.map_state()
    }
}
