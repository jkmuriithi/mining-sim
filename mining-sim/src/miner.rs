/*!
Definitions for miner implementations

A miner is any type which implements the [`Miner`] trait. Each miner should keep
track of any internal state necessary to produce the desired strategic behavior.

# Examples
A miner implementation which breaks longest chain ties using
[`TieBreaker`](crate::tie_breaker::TieBreaker).

```
use mining_sim::prelude::*;

#[derive(Debug, Clone)]
struct MyMiner {
    id: MinerId,
    tie_breaker: TieBreaker,
}

impl Miner for MyMiner {
    fn name(&self) -> String {
        "MyMiner".to_string()
    }

    fn id(&self) -> MinerId {
        self.id
    }

    fn set_id(&mut self, id: MinerId) {
        self.id = id;
        self.tie_breaker = TieBreaker::FavorMiner(id);
    }

    fn get_action(
        &mut self,
        chain: &Blockchain,
        block_mined: Option<BlockId>,
    ) -> Action {
        match block_mined {
            Some(block_id) => Action::Publish(Block {
                id: block_id,
                parent_id: Some(self.tie_breaker.choose(chain)),
                miner_id: self.id,
                txns: vec![],
            }),
            None => Action::Wait,
        }
    }
}
```

# Built-In Strategies

A variety of strategies have already been implemented in this module. These
include:
- Honest Mining [`honest::Honest`]
- Honest Mining with Probabilistic Forks [`honestforking::HonestForking`]
- Selfish Mining [`selfish::Selfish`]
- N-Deficit Mining [`ndeficit::NDeficit`]
- Noop [`noop::Noop`]
*/

use std::fmt::Debug;

use crate::blockchain::{Block, BlockId, Blockchain};

pub mod honest;
pub mod honestforking;
pub mod ndeficit;
pub mod ndeficiteager;
pub mod noise;
pub mod noop;
pub mod selfish;

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

/// Defines the behavior of a mining strategy.
pub trait Miner: Debug + dyn_clone::DynClone + Send + Sync {
    /// Returns the name of this miner's strategy.
    ///
    /// The return value of this method will appear in the "Strategy Name"
    /// column of [`ResultsTable`](crate::results::ResultsTable).
    fn name(&self) -> String;

    /// Returns this miner's [`MinerId`].
    fn id(&self) -> MinerId;

    /// Sets this miner's [`MinerId`].
    ///
    /// This method is guaranteed to be called after a [`Miner`]
    /// implementation is added to a
    /// [`SimulationBuilder`](crate::simulation::SimulationBuilder).
    fn set_id(&mut self, id: MinerId);

    /// Returns the action taken by this miner in this round.
    ///
    /// Called once in each round of each simulation.
    ///
    /// `chain` is a reference to the simulation's blockchain. `block_mined` is
    /// `Some(block_id)` if this miner has been selected as the proposer in the
    /// current simulation round, and `None` otherwise.
    fn get_action(
        &mut self,
        chain: &Blockchain,
        block_mined: Option<BlockId>,
    ) -> Action;
}

dyn_clone::clone_trait_object!(Miner);

/// Unique identifier of a [`Miner`] implementation. Corresponds to a [`usize`].
///
/// # Invariants
///
/// `MinerId(0)` is reserved for [`Blockchain::GENESIS_MINER`],
/// and as such `MinerId(0)` cannot be instantiated outside of this crate, and
/// [`MinerId::default`] returns `MinerId(1)`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinerId(pub(crate) usize);

impl MinerId {
    /// Returns the [`usize`] corresponding to this [`MinerId`].
    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for MinerId {
    fn from(value: usize) -> Self {
        assert_ne!(value, 0, "newly made MinerId must be greater than 0");
        Self(value)
    }
}

impl Default for MinerId {
    fn default() -> Self {
        Self(1)
    }
}
impl std::fmt::Display for MinerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
