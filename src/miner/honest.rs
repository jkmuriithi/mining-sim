//! Implementation of the HONEST (or FRONTIER) mining strategy via the
//! [Strategy] trait.

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
    miner::MinerID,
};

use super::{ties::TieBreaker, Action, Strategy};

/// Publishes all blocks as soon as possible at the tip of the longest chain.
#[derive(Debug, Clone)]
pub struct Honest {
    miner: Option<MinerID>,
    tie_breaker: TieBreaker,
}

impl Honest {
    pub fn new() -> Self {
        Honest {
            miner: None,
            tie_breaker: TieBreaker::default(),
        }
    }

    pub fn with_tie_breaker(tie_breaker: TieBreaker) -> Self {
        Honest {
            miner: None,
            tie_breaker,
        }
    }
}

impl Default for Honest {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for Honest {
    fn set_id(&mut self, id: MinerID) {
        self.miner = Some(id);
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> Action {
        let miner = match self.miner {
            None => panic!("Miner ID not set"),
            Some(id) => id,
        };

        match block {
            None => Action::Wait,
            Some(block_id) => Action::Publish(Block::new(
                block_id,
                Some(self.tie_breaker.choose_tip(chain)),
                miner,
                None,
            )),
        }
    }
}
