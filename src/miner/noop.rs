//! Strategy which never publishes a block

use crate::{
    blockchain::{BlockId, Blockchain},
    miner::{Action, Miner, MinerId},
};

/// [`.get_action`](Noop::get_action) always returns [`Action::Wait`].
#[derive(Debug, Clone, Default)]
pub struct Noop(MinerId);

impl Noop {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Miner for Noop {
    fn name(&self) -> String {
        "No-op".to_string()
    }

    fn id(&self) -> MinerId {
        self.0
    }

    fn set_id(&mut self, id: MinerId) {
        self.0 = id;
    }

    fn get_action(&mut self, _: &Blockchain, _: Option<BlockId>) -> Action {
        Action::Wait
    }
}
