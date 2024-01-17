//! Implementation of Nothing-at-Stake mining.

use std::collections::VecDeque;

use crate::{block::BlockID, blockchain::Blockchain, miner::MinerID};

use super::{ties::TieBreaker, Action, Miner};

#[derive(Debug, Default, Clone)]
pub struct NothingAtStake {
    id: Option<MinerID>,
    tie_breaker: Option<TieBreaker>,
    blocks: VecDeque<BlockID>,
}

impl NothingAtStake {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Miner for NothingAtStake {
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
        let id = self.id();
        let tb = self.tie_breaker.unwrap();

        todo!()
    }
}
