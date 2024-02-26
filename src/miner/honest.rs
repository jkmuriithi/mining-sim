//! Honest/Frontier mining strategy

use crate::{
    blockchain::{Block, BlockId, Blockchain},
    miner::{Action, Miner, MinerId},
    tie_breaker::TieBreaker,
};

/// Publishes all blocks as soon as possible at the tip of the longest chain.
#[derive(Debug, Default, Clone)]
pub struct Honest {
    id: MinerId,
    tie_breaker: TieBreaker,
}

impl Honest {
    /// Creates a new honest miner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new honest miner which breaks ties using `tie_breaker`.
    pub fn with_tie_breaker(tie_breaker: TieBreaker) -> Self {
        Honest {
            tie_breaker,
            ..Default::default()
        }
    }
}

impl Miner for Honest {
    fn name(&self) -> String {
        "Honest".to_string()
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
            Some(block_id) => Action::Publish(Block {
                id: block_id,
                parent_id: Some(self.tie_breaker.choose(chain)),
                miner_id: self.id,
                txns: vec![],
            }),
            None => Action::Wait,
        }
    }
}
