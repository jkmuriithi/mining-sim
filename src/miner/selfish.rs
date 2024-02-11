//! Implementation of selfish mining

use std::collections::VecDeque;

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
    tie_breaker::TieBreaker,
};

use super::{Action, Miner, MinerID};

#[derive(Debug, Default, Clone)]
pub struct Selfish {
    hidden_blocks: VecDeque<Block>,
    id: Option<MinerID>,
    private_height: usize,
    tie_breaker: Option<TieBreaker>,
}

impl Selfish {
    pub fn new() -> Self {
        Self::default()
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
        // If hidden_blocks only contains blocks that are
        if self.private_height < chain.max_height() {
            self.hidden_blocks.clear();
        }

        match block {
            Some(block_id) => {
                let parent_id = if self.hidden_blocks.is_empty() {
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

                let block = Block {
                    id: block_id,
                    parent_id: Some(parent_id),
                    miner_id: id,
                    txns: None,
                };

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

/// Returns the ideal Selfish Miner revenue function from Eyal and Sirer's
/// paper. To be used with [SimulationResultsBuilder::mining_power_func](crate::SimulationResultsBuilder).
pub fn selfish_revenue(gamma: f64) -> impl Fn(crate::PowerValue) -> f64 {
    move |a: crate::PowerValue| -> f64 {
        (a * (1.0 - a).powi(2) * (4.0 * a + gamma * (1.0 - 2.0 * a))
            - a.powi(3))
            / (1.0 - a * (1.0 + a * (2.0 - a)))
    }
}
