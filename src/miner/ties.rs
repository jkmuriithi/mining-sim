//! Utilities for describing tie-breaking behavior in strategies.

use rand::Rng;

use crate::{block::BlockID, blockchain::Blockchain, miner::MinerID};

/// Determines how a [Strategy](super::Strategy) implementation breaks ties
/// between blocks of height [Blockchain::tip_height].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum TieBreaker {
    /// Always uses the earliest published block (the first element of
    /// [Blockchain::tips]).
    #[default]
    EarliestPublished,
    /// Uses the earliest published block by the miner with the given ID. If no
    /// such block exists in [Blockchain::tips], the earliest published block is
    /// returned.
    FavorMiner(MinerID),
    /// Uses the earliest block published by the miner with the given ID, with
    /// the given probability. Otherwise, the earliest published block is
    /// returned.
    FavorMinerProb(MinerID, f64),
}

impl TieBreaker {
    /// Returns the block at the tip of this blockchain's longest chain
    /// in accordance with the tie breaking strategy.
    pub fn choose_tip(&self, chain: &Blockchain) -> BlockID {
        use TieBreaker::*;

        match &self {
            EarliestPublished => chain.tips[0],
            FavorMiner(miner) => chain
                .tips
                .iter()
                .find(|&&b| chain[b].block.miner == *miner)
                .cloned()
                .unwrap_or(chain.tips[0]),
            FavorMinerProb(miner, prob) => {
                let favored = chain
                    .tips
                    .iter()
                    .find(|&&b| chain[b].block.miner == *miner);

                match favored {
                    None => chain.tips[0],
                    Some(&id) => {
                        if rand::thread_rng().gen_range(0.0..1.0) < *prob {
                            id
                        } else {
                            chain.tips[0]
                        }
                    }
                }
            }
        }
    }
}
