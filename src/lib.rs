/*!
Simulator for a game theory-based model of blockchain mining.
*/

// ## Todo:
// - Percent trait which turns valid integer ranges into ranges of percentage values
//   (to make describing PowerValue ranges easier)
//     - example: (0..=50).percent()
// - Create example code for each module/submodule
// - Create a version of N-Deficit mining which forks the honest miner whenever
//   possible (as Selfish mining does)
// - For positive recurrent systems (simulations using positive recurrent
//   strategies) the distribution of should approach a normal distribution
// - Estimate the distribution of revenue for a single value of alpha and a
//   set number of rounds
// - try to use Average::Mean as the estimator whe possible to take advantage of
//   the Central Limit Theorem when estimating attacker revenue

// ## Issues:
// - Windows NT kernel seems to be taking up a majority of the execution time
// for bigger multithreaded runs. This seems like an optimization issue either
// within rayon or the kernel itself

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
