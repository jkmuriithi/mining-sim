use rand::Rng;

/// Simple honest miner which randomly creates forks in the blockchain
use crate::{block::Block, tie_breaker::TieBreaker, Miner};

use super::{Action, MinerID};

/// An honest miner which mines one behind the longest chain with
/// probability `p`.
#[derive(Debug, Clone, Default)]
pub struct HonestForking {
    id: Option<MinerID>,
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

    fn id(&self) -> MinerID {
        self.id.expect("Miner ID to be set")
    }

    fn set_id(&mut self, id: MinerID) {
        self.id = Some(id);
    }

    fn get_action(
        &mut self,
        chain: &crate::Blockchain,
        block: Option<crate::block::BlockID>,
    ) -> Action {
        match block {
            Some(block_id) => {
                let miner_id = self.id();
                let lc = self.tie_breaker.choose(chain);

                if rand::thread_rng().gen_bool(self.p) {
                    Action::Publish(Block {
                        id: block_id,
                        parent_id: Some(
                            chain[lc].block.parent_id.unwrap_or(lc),
                        ),
                        miner_id,
                        txns: None,
                    })
                } else {
                    Action::Publish(Block {
                        id: block_id,
                        parent_id: Some(lc),
                        miner_id,
                        txns: None,
                    })
                }
            }
            None => Action::Wait,
        }
    }
}
