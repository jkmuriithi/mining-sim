//! Implementation of selfish mining.

use std::collections::VecDeque;

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
};

use super::{ties::TieBreaker, Action, Miner, MinerID};

#[derive(Debug, Default, Clone)]

pub struct Selfish {
    id: Option<MinerID>,
    tie_breaker: Option<TieBreaker>,
    /// All blocks which are mined, but unpublished.
    hidden_blocks: VecDeque<Block>,
    /// Height of the highest block on the private chain.
    private_height: usize,
}

impl Selfish {
    pub fn new() -> Self {
        Selfish {
            ..Default::default()
        }
    }
}

impl Miner for Selfish {
    fn name(&self) -> String {
        "Selfish".into()
    }

    #[inline]
    fn id(&self) -> MinerID {
        self.id.expect("Miner ID to be set")
    }

    fn set_id(&mut self, id: MinerID) {
        self.id = Some(id);
        self.tie_breaker = Some(TieBreaker::FavorMiner(id));
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> Action {
        if self.private_height < chain.max_height() {
            self.hidden_blocks.clear();
        }

        match block {
            Some(block_id) => {
                let parent = if self.hidden_blocks.is_empty() {
                    let p = self.tie_breaker.unwrap().choose(chain);
                    self.private_height = chain[p].height + 1;
                    p
                } else {
                    self.private_height += 1;
                    self.hidden_blocks.back().unwrap().id
                };

                let id = self.id();
                let tip = chain.tip();
                let fork = tip.iter().any(|&b| chain[b].block.miner_id == id)
                    && tip.iter().any(|&b| chain[b].block.miner_id != id);

                let block = Block::new(block_id, Some(parent), id, None);
                if fork && self.hidden_blocks.is_empty() {
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
