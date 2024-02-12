//! Honest miner implementation which randomly creates forks at the tip of the
//! blockchain

use rand::Rng;

use crate::{
    block::{Block, BlockId},
    blockchain::Blockchain,
    miner::{Action, Miner, MinerId},
    tie_breaker::TieBreaker,
};

/// An honest miner which mines one behind the longest chain with
/// probability `p`.
#[derive(Debug, Clone, Default)]
pub struct HonestForking {
    id: MinerId,
    p: f64,
    tie_breaker: TieBreaker,
}

impl HonestForking {
    pub fn new(p: f64) -> Self {
        HonestForking {
            p,
            ..Default::default()
        }
    }

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
        format!("Honest Forking, p={}", self.p)
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

                if rand::thread_rng().gen_bool(self.p) {
                    Action::Publish(Block {
                        id: block_id,
                        parent_id: Some(
                            chain[lc].block.parent_id.unwrap_or(lc),
                        ),
                        miner_id: self.id,
                        txns: None,
                    })
                } else {
                    Action::Publish(Block {
                        id: block_id,
                        parent_id: Some(lc),
                        miner_id: self.id,
                        txns: None,
                    })
                }
            }
            None => Action::Wait,
        }
    }
}
