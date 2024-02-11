//! Definitions for miner implementations

use std::fmt::Debug;

use crate::{
    block::{Block, BlockID},
    blockchain::Blockchain,
};

pub mod honest;
pub mod honestforking;
pub mod ndeficit;
pub mod noop;
pub mod selfish;

/// Numeric type of each miner's unique identifier.
pub type MinerID = usize;

/// A blockchain miner with some strategy.
pub trait Miner: Debug + dyn_clone::DynClone + Send + Sync {
    /// Get this miner's [`MinerID`].
    ///
    /// # Panics
    /// Panics if this miner's ID has not been set using [`Miner::set_id`].
    fn id(&self) -> MinerID;

    /// Set this miner's [`MinerID`]. This ID must be set before any other trait
    /// methods are called.
    fn set_id(&mut self, id: MinerID);

    /// Get the action taken by this miner in this round. `block` is `Some` if
    /// this miner has been selected as the proposer for this round, and `None`
    /// otherwise.
    ///
    /// # Panics
    /// Panics if the ID of this miner has not been set using [`Miner::set_id`].
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

/// An action taken by a miner on the chain.
#[derive(Debug, Clone)]
pub enum Action {
    /// Don't publish a block.
    Wait,
    /// Publish the given block.
    Publish(Block),
    /// Publish the given blocks in order. No parent-child relationships are
    /// created during this process.
    PublishSet(Vec<Block>),
}

/// Returns an instance of the ideal Selfish Miner revenue function from Eyal
/// and Sirer's paper which can be used as input to
/// [`SimulationResultsBuilder::mining_power_func`](crate::SimulationResultsBuilder).
pub fn selfish_revenue(gamma: f64) -> impl Fn(crate::PowerValue) -> f64 {
    move |a: crate::PowerValue| -> f64 {
        (a * (1.0 - a).powi(2) * (4.0 * a + gamma * (1.0 - 2.0 * a))
            - a.powi(3))
            / (1.0 - a * (1.0 + a * (2.0 - a)))
    }
}

/// Ideal Nothing-At-Stake miner revenue function from Weinberg and Ferrera's
/// paper. Can be used as input to
/// [`SimulationResultsBuilder::mining_power_func`](crate::SimulationResultsBuilder).
pub fn nsm_revenue(a: crate::PowerValue) -> f64 {
    (4.0 * a.powi(2) - 8.0 * a.powi(3) - a.powi(4) + 7.0 * a.powi(5)
        - 3.0 * a.powi(6))
        / (1.0 - a - 2.0 * a.powi(2) + 3.0 * a.powi(4) - 3.0 * a.powi(5)
            + a.powi(6))
}
