use crate::{block::BlockID, blockchain::Blockchain, miner::MinerID};

use super::{Action, Strategy};

#[derive(Debug, Clone)]
pub struct NothingAtStake {}

impl Strategy for NothingAtStake {
    fn set_id(&mut self, id: MinerID) {
        todo!()
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> Action {
        todo!()
    }
}
