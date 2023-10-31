//! Implementation of Selfish Mining via the [Strategy] trait.

use crate::{block::BlockID, blockchain::Blockchain};

use super::{Action, Strategy};

#[derive(Debug, Clone)]
pub struct Selfish {}

impl Strategy for Selfish {
    fn set_id(&mut self, id: crate::miner::MinerID) {
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
