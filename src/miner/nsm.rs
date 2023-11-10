//! Implementation of Nothing-at-Stake mining.

use crate::{block::BlockID, blockchain::Blockchain, miner::MinerID};

use super::{ties::TieBreaker, Action, Miner};

#[derive(Debug, Default, Clone)]
pub struct NothingAtStake {
    id: Option<MinerID>,
    tie_breaker: TieBreaker,
}

impl NothingAtStake {
    pub fn new() -> Self {
        NothingAtStake { id: None, tie_breaker: Default::default() }
    }
}

impl Miner for NothingAtStake {
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
        assert!(self.id.is_some(), "Miner ID must be set");

        let id = self.id.unwrap();

        todo!()
    }
}
