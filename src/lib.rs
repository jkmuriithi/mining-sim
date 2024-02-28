/*!
Simulator for a game theory-based model of blockchain mining.

# Features
- `rayon`: Enables the parallelization of simulation runs using
  [`rayon`](https://docs.rs/rayon/1.9), typically resulting in a signficant
  performance boost.
- `block-children`: Enables the tracking of the blocks which point to a
  particular [`Block`](blockchain::Block) in a
  [`Blockchain`](blockchain::Blockchain) via
  [`BlockData::children`](blockchain::BlockData::children). This greatly
  increases memory usage, and can affect runtime performance.
- By default, `rayon` is enabled.
*/

// ## Todo:
// - Create example code for each module/submodule
// - Create a version of N-Deficit mining which forks the honest miner whenever
//   possible (as Selfish mining does)
// - For positive recurrent systems (simulations using positive recurrent
//   strategies) the distribution of should approach a normal distribution
// - Estimate the distribution of revenue for a single value of alpha and a
//   set number of rounds
// - try to use Average::Mean as the estimator whe possible to take advantage of
//   the Central Limit Theorem when estimating attacker revenue

// ## Important Tests (Conversation with Weinberg):
// - Honest miners only -> alpha is roughly equal to revenue
// - Selfish Mining revenue -> alpha matches closed form from Eyal paper
// - NSM revenue -> alpha matches closed form from Weinberg-Ferreira

pub mod blockchain;
pub mod miner;
pub mod power_dist;
pub mod prelude;
pub mod results;
pub mod simulation;
pub mod tie_breaker;
pub mod transaction;

pub(crate) mod utils;
