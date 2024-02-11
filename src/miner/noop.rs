//! Mining strategy that never publishes a block.

use super::{Action, Miner, MinerID};

/// [Noop::get_action] always returns [Action::Wait].
#[derive(Debug, Clone, Default)]
pub struct Noop(Option<MinerID>);

impl Miner for Noop {
    fn name(&self) -> String {
        "No-op".into()
    }

    fn id(&self) -> super::MinerID {
        self.0.expect("Miner ID to be set")
    }

    fn set_id(&mut self, id: super::MinerID) {
        self.0 = Some(id);
    }

    fn get_action(
        &mut self,
        _chain: &crate::Blockchain,
        _block: Option<crate::block::BlockID>,
    ) -> Action {
        Action::Wait
    }
}
