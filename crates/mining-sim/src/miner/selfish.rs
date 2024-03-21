//! Selfish mining implementation

use std::collections::VecDeque;

use crate::{
    blockchain::{Block, BlockId, Blockchain},
    miner::{Action, Miner, MinerId},
    tie_breaker::TieBreaker,
};

/// Follows the selfish mining strategy described by
/// [Eyal and Sirer](https://doi.org/10.48550/arXiv.1311.0243).
#[derive(Debug, Default, Clone)]
pub struct Selfish {
    hidden_blocks: VecDeque<Block>,
    id: MinerId,
    private_height: usize,
    tie_breaker: TieBreaker,
}

impl Selfish {
    /// Creates a new selfish miner.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Miner for Selfish {
    fn name(&self) -> String {
        "Selfish".to_string()
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
    ) -> Action {
        if self.private_height < chain.max_height() {
            self.hidden_blocks.clear();
        }

        match block_mined {
            Some(block_id) => {
                let parent_id = if self.hidden_blocks.is_empty() {
                    let p = self.tie_breaker.choose(chain);
                    self.private_height = chain[p].height + 1;
                    p
                } else {
                    self.private_height += 1;
                    self.hidden_blocks.back().unwrap().id
                };

                let block = Block {
                    id: block_id,
                    parent_id: Some(parent_id),
                    miner_id: self.id,
                    txns: vec![],
                };

                let lc = chain.tip();
                if self.hidden_blocks.is_empty()
                    && lc.iter().any(|b| chain[b].block.miner_id == self.id)
                    && lc.iter().any(|b| chain[b].block.miner_id != self.id)
                {
                    Action::Publish(block)
                } else {
                    self.hidden_blocks.push_back(block);
                    Action::Wait
                }
            }
            None => match self.hidden_blocks.len() {
                0 => Action::Wait,
                2 => Action::PublishSet(self.hidden_blocks.drain(..).collect()),
                _ => Action::Publish(self.hidden_blocks.pop_front().unwrap()),
            },
        }
    }
}
