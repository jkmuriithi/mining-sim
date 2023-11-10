//! Definitions for representations of blockchain miners.

pub mod honest;
pub mod nsm;
pub mod selfish;
pub mod ties;

pub use honest::Honest;
pub use selfish::Selfish;

use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
};

/// An action taken by a miner on the chain.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Action {
    /// Don't publish any blocks.
    Wait,
    /// Publish the specified block.
    Publish(Block),
    /// Publish the specified set of blocks. The blocks given in this
    /// action will be published in the given order in a
    /// [Simulation](crate::simulation::Simulation), but no new parent-child
    /// relationships will be created between them during this process.
    PublishSet(Vec<Block>),
}

/// A unique identifier assigned to each [Miner].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct MinerID(u64);

impl From<u64> for MinerID {
    fn from(value: u64) -> Self {
        MinerID(value)
    }
}

/// A blockchain miner with some specific strategy. Maintains internal state
/// relating to unpublished blocks, if necessary.
pub trait Miner: Debug + DynClone {
    /// Get this miner's [MinerID].
    ///
    /// ## Panics
    /// Panics if this miner's ID has not been set using [Miner::set_id].
    fn id(&self) -> MinerID;

    /// Set this miner's [MinerID]. Should be called before any other trait
    /// methods.
    ///
    /// ## Panics
    /// Panics when called more than once on a specific instance.
    fn set_id(&mut self, id: MinerID);

    /// Get the action taken by this miner in this round. `block` is `Some` if
    /// this miner has been selected as the proposer for this round, and `None`
    /// otherwise.
    ///
    /// ## Panics
    /// Panics if the ID of this miner has not been set using [Miner::set_id].
    fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> Action;

    /// Returns the name of the miner's strategy.
    fn name(&self) -> String {
        "Name not set".into()
    }
}

dyn_clone::clone_trait_object!(Miner);
