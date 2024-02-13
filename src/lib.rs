/*!
Simulator for a game theory-based model of blockchain mining.
*/

// ## Todo:
// - Percent trait which turns valid integer ranges into ranges of percentage values
//   (to make describing PowerValue ranges easier)
//     - example: (0..=50).percent()
// - Create example code for each module/submodule
// - Honest forking strategy (pretends it didn't hear certain blocks) and/or
//   global simulation latency modeling
//     - one latency model: exponential distribution of blocks which
//     - forking introduces the question of chain throughput (easy metric to implement)
// - Handling all selfish miners:
//     - A strategy is "excessively patient" if it never publishes the first
//       block
//     - Can we tell when a strategy is "excessively patient"?
//       (Or use a flag to let users know when that's a problem)

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
