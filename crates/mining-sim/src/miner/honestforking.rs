//! Honest mining with random forking

use rand::Rng;

use crate::{
    blockchain::{Block, BlockId, Blockchain},
    miner::{Action, Miner, MinerId},
    tie_breaker::TieBreaker,
};

/// Mines one behind the longest chain with probability `p`, following the
/// Honest strategy otherwise.
#[derive(Debug, Clone, Default)]
pub struct HonestForking {
    id: MinerId,
    p: f64,
    tie_breaker: TieBreaker,
}

impl HonestForking {
    /// Creates a new honest forking miner.
    pub fn new(p: f64) -> Self {
        HonestForking {
            p,
            ..Default::default()
        }
    }

    /// Creates a new honest forking miner which breaks ties using
    /// `tie_breaker`.
    pub fn with_tie_breaker(p: f64, tie_breaker: TieBreaker) -> Self {
        HonestForking {
            p,
            tie_breaker,
            ..Default::default()
        }
    }
}

impl Miner for HonestForking {
    fn name(&self) -> String {
        format!("Honest Forking (p={})", self.p)
    }

    fn id(&self) -> MinerId {
        self.id
    }

    fn set_id(&mut self, id: MinerId) {
        self.id = id;
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block_mined: Option<BlockId>,
    ) -> Action {
        match block_mined {
            Some(block_id) => {
                let lc = self.tie_breaker.choose(chain);

                Action::Publish(Block {
                    id: block_id,
                    parent_id: if rand::thread_rng().gen_bool(self.p) {
                        chain[lc].block.parent_id.or(Some(lc))
                    } else {
                        Some(lc)
                    },
                    miner_id: self.id,
                    txns: vec![],
                })
            }
            None => Action::Wait,
        }
    }
}
