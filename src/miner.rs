//! Definitions for representations of blockchain miners.

use std::fmt::Debug;

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
};

use dyn_clone::DynClone;

pub mod honest;
pub mod nsm;
pub mod selfish;
pub mod ties;

pub use honest::Honest;
pub use selfish::Selfish;

/// Representation of a blockchain miner with some [Strategy] implementation.
#[derive(Debug, Clone)]
pub struct Miner {
    /// This miner's mining strategy.
    pub strategy: Box<dyn Strategy>,
    /// This miner's unique identifer. Assigned after initialization.
    id: Option<MinerID>,
}

/// A unique identifier assigned to each [Miner].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct MinerID(u64);

impl From<u64> for MinerID {
    fn from(value: u64) -> Self {
        MinerID(value)
    }
}

impl Miner {
    /// Creates a new miner with the given strategy.
    pub fn new<S: Strategy + 'static>(strategy: S) -> Self {
        Miner {
            id: None,
            strategy: Box::new(strategy),
        }
    }

    /// Set this miner's [MinerID]. Should be called immediately after
    /// instantiation with [Miner::new], before any other methods.
    ///
    /// ## Panics
    /// Panics when called more than once on a specific instance.
    pub fn set_id(&mut self, id: impl Into<MinerID>) {
        if self.id.is_some() {
            panic!("Miner ID cannot be set twice");
        }
        let id = id.into();

        self.id = Some(id);
        self.strategy.set_id(id);
    }

    /// Get this miner's [MinerID].
    ///
    /// ## Panics
    /// Panics if this miner's ID has not been set using [Miner::set_id].
    pub fn id(&self) -> MinerID {
        self.id.expect("Miner ID to be set")
    }

    /// Get the action taken by this miner in this round. `block` is `Some` if
    /// this miner has been selected as the proposer for this round, and `None`
    /// otherwise.
    ///
    /// ## Panics
    /// Panics if the ID of this miner has not been set using [Miner::set_id].
    pub fn get_action(
        &mut self,
        chain: &Blockchain,
        block: Option<BlockID>,
    ) -> Action {
        if self.id.is_none() {
            panic!("Miner ID has not been set")
        }

        self.strategy.get_action(chain, block)
    }
}

impl<S: Strategy + 'static> From<S> for Miner {
    fn from(value: S) -> Self {
        Miner::new(value)
    }
}

impl PartialEq for Miner {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Miner {}

impl PartialOrd for Miner {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.zip(other.id).map(|(a, b)| a.cmp(&b))
    }
}

/// An action taken by a miner on the chain.
#[non_exhaustive]
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

/// A blockchain miner with some specific strategy. Maintains internal state
/// relating to unpublished blocks, if necessary.
pub trait Strategy: Debug + DynClone {
    /// Get this miner's [MinerID].
    ///
    /// ## Panics
    /// Panics if this miner's ID has not been set using [Miner::set_id].
    // pub fn id(&self) -> MinerID;

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
}

dyn_clone::clone_trait_object!(Strategy);
