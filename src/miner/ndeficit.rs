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
    blockchain::{Block, BlockId, Blockchain},
    miner::{Action, Miner, MinerId},
    tie_breaker::TieBreaker,
};

#[derive(Debug, Clone, Default)]
pub struct NDeficit {
    i: usize,
    id: MinerId,
    tie_breaker: TieBreaker,

    // Blockchain state tracking
    capitulation: BlockId,
    state: Vec<StateEntry>,
    seen: HashSet<BlockId>,
    our_blocks: VecDeque<BlockId>,
    honest_blocks: Vec<BlockId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    fn capitulate(&mut self, genesis: BlockId) {
        self.capitulation = genesis;
        self.clear_state();
    }

    fn update_state(
        &mut self,
        chain: &Blockchain,
        block_mined: Option<&BlockId>,
    ) {
        let tip = self.tie_breaker.choose(chain);
        let cap_height = chain[self.capitulation].height;

        // Ignore states of the form [H(x), ..]
        if !self.our_blocks.is_empty() {
            let mut unseen_blocks = vec![];

            for curr in chain.ancestors_of(tip) {
                if curr == self.capitulation || self.seen.contains(&curr) {
                    break;
                }
                if chain[curr].height <= cap_height {
                    self.capitulate(tip);
                    return;
                }

                unseen_blocks.push(curr);
                self.seen.insert(curr);
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
            debug_assert!(self.state.is_empty());
            debug_assert!(self.seen.is_empty());
            debug_assert!(self.honest_blocks.is_empty());

            self.capitulation = tip;
        }

        if let Some(block_id) = block_mined {
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
    fn block_path_to(&mut self, parent: BlockId) -> Vec<Block> {
        let mut blocks = vec![];
        let mut parent = parent;
        self.our_blocks.drain(..).for_each(|id| {
            blocks.push({
                Block {
                    id,
                    parent_id: Some(parent),
                    miner_id: self.id,
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
                debug_assert!(*x <= self.i);

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
                debug_assert!(*x <= self.i);

                let x = *x;

                // Manually capitulate to B_{1, 1}
                self.state = vec![A(1), H(1)];
                self.capitulation = self.honest_blocks[x - 1];
                self.our_blocks.pop_front();
                self.honest_blocks.drain(..x);
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

    fn id(&self) -> MinerId {
        self.id
    }

    fn set_id(&mut self, id: MinerId) {
        self.id = id;
        self.tie_breaker = TieBreaker::FavorMiner(id);
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block_mined: Option<BlockId>,
    ) -> super::Action {
        self.update_state(chain, block_mined.as_ref());

        // Handle selfish mining fork case
        // FIXME: Forks are never encountered when up against an honest miner,
        // may need to implement "aggressive" strategy
        // if self.our_blocks.len() == 1 {
        //     let lc = chain.tip();

        //     let ours_at_lc =
        //         lc.iter().find(|&&b| chain[b].block.miner_id == self.id);
        //     let othr_at_lc =
        //         lc.iter().find(|&&b| chain[b].block.miner_id != self.id);

        //     if let (Some(parent_id), Some(_)) = (ours_at_lc, othr_at_lc) {
        //         let block_id = self.our_blocks[0];
        //         self.capitulate(block_id);

        //         return Action::Publish(Block {
        //             id: block_id,
        //             miner_id: self.id,
        //             parent_id: Some(*parent_id),
        //             txns: None,
        //         });
        //     }
        // }

        self.map_state()
    }
}
