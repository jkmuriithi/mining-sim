use rand::Rng;

use crate::{
    blockchain::{Block, BlockId, Blockchain},
    miner::{Action, Miner, MinerId},
};

/// Publishes blocks immediately upon mining them, selecting the parent block
/// uniformly at random across all published blocks.
#[derive(Debug, Clone)]
pub struct Noise {
    id: MinerId,
}

impl Miner for Noise {
    fn name(&self) -> String {
        "Noise".into()
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
            None => Action::Wait,
            Some(block_id) => {
                let block_num = block_id.get();

                let mut parent = block_num;
                while !chain.contains(parent.into()) {
                    parent = rand::thread_rng().gen_range(0..block_num);
                }

                Action::Publish(Block {
                    id: block_id,
                    parent_id: Some(parent.into()),
                    miner_id: self.id,
                    txns: vec![],
                })
            }
        }
    }
}
