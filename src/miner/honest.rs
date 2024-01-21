//! Implementation of the Honest (or Frontier) mining strategy.

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
    miner::MinerID,
    tie_breaker::TieBreaker,
};

use super::{Action, Miner};

/// Publishes all blocks as soon as possible at the tip of the longest chain.
#[derive(Debug, Default, Clone)]
pub struct Honest {
    id: Option<MinerID>,
    tie_breaker: TieBreaker,
}

impl Honest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tie_breaker(tie_breaker: TieBreaker) -> Self {
        Honest {
            id: None,
            tie_breaker,
        }
    }
}

impl Miner for Honest {
    fn name(&self) -> String {
        "Honest".into()
    }

    fn id(&self) -> MinerID {
        self.id.expect("Miner ID to be set")
    }

    fn set_id(&mut self, id: MinerID) {
        self.id = Some(id);
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> Action {
        let miner_id = self.id();
        match block {
            Some(block_id) => Action::Publish(Block {
                id: block_id,
                parent_id: Some(self.tie_breaker.choose(chain)),
                miner_id,
                txns: None,
            }),
            None => Action::Wait,
        }
    }
}
