//! Describing distributions of mining power

use crate::miner::MinerID;

/// Numeric type used to represent mining power.
pub type PowerValue = f64;

/// Determines how mining power is distributed between miners during a
/// simulation.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum PowerDistribution {
    /// Weight each miner equally.
    #[default]
    Equal,
    /// Set the specified miner's power to the given float in \[0, 1\], with
    /// mining power distributed equally between all other miners.
    SetMiner(MinerID, PowerValue),
    /// Set all mining power values to those in the given vector.
    SetValues(Vec<PowerValue>),
}

#[derive(Debug, thiserror::Error)]
pub enum PowerDistributionError {
    #[error("distribution values sum to {0}, not 1.0")]
    BadDistributionSum(PowerValue),
    #[error("power value {0} is not in the range 0.0..=1.0")]
    BadPowerValue(PowerValue),
    #[error("cannot set power for the genesis miner (MinerID 0)")]
    SetMinerGenesisMiner,
    #[error("cannot set power for invalid miner ID {0}")]
    SetMinerBadMinerID(MinerID),
    #[error("cannot set power for a single miner")]
    SetMinerSingleMiner,
    #[error("power distribution size {0} does not match miner count {1}")]
    WrongNumMiners(usize, usize),
    #[error("cannot create a distribution for zero miners")]
    ZeroMinersGiven,
}

impl PowerDistribution {
    /// Allowable difference between a distribution sum and 1.0.
    const EPSILON_POWER: PowerValue = 1e-6;

    #[inline]
    pub fn is_valid(&self, num_miners: usize) -> bool {
        self.validate(num_miners).is_ok()
    }

    pub fn validate(
        &self,
        num_miners: usize,
    ) -> Result<(), PowerDistributionError> {
        use PowerDistributionError::*;

        if num_miners == 0 {
            return Err(ZeroMinersGiven);
        }

        match &self {
            Self::Equal => Ok(()),
            Self::SetValues(dist) => {
                if dist.len() != num_miners {
                    return Err(WrongNumMiners(dist.len(), num_miners));
                }

                if let Some(&val) =
                    dist.iter().find(|&x| x.is_nan() || !(0.0..1.0).contains(x))
                {
                    return Err(BadPowerValue(val));
                }

                let sum = dist.iter().sum();
                if PowerValue::abs(sum - 1.0) > Self::EPSILON_POWER {
                    return Err(BadDistributionSum(sum));
                }

                Ok(())
            }
            Self::SetMiner(miner_id, power) => {
                if num_miners == 1 {
                    return Err(SetMinerSingleMiner);
                }

                let miner_id = *miner_id;

                if miner_id == 0 {
                    return Err(SetMinerGenesisMiner);
                }

                if miner_id > num_miners {
                    return Err(SetMinerBadMinerID(miner_id));
                }

                let power = *power;
                if power.is_nan() || !(0.0..=1.0).contains(&power) {
                    return Err(BadPowerValue(power));
                }

                Ok(())
            }
        }
    }

    pub fn power_of(
        &self,
        miner_id: MinerID,
        num_miners: usize,
    ) -> Result<PowerValue, PowerDistributionError> {
        self.validate(num_miners)?;

        Ok(unsafe { self.power_of_unchecked(miner_id, num_miners) })
    }

    /// # Safety
    /// This function expects the underlying power distribution to be a valid
    /// discrete probability distribution over the given number of miners.
    pub unsafe fn power_of_unchecked(
        &self,
        miner_id: MinerID,
        num_miners: usize,
    ) -> PowerValue {
        match &self {
            Self::Equal => 1.0 / num_miners as PowerValue,
            Self::SetValues(dist) => dist[miner_id - 1],
            Self::SetMiner(id, power) => {
                if miner_id == *id {
                    *power
                } else {
                    (1.0 - power) / (num_miners - 1) as PowerValue
                }
            }
        }
    }

    pub fn values(
        &self,
        num_miners: usize,
    ) -> Result<Vec<PowerValue>, PowerDistributionError> {
        self.validate(num_miners)?;

        Ok(unsafe { self.values_unchecked(num_miners) })
    }

    /// # Safety
    /// This function expects the underlying power distribution to be a valid
    /// discrete probability distribution over the given number of miners.
    pub unsafe fn values_unchecked(
        &self,
        num_miners: usize,
    ) -> Vec<PowerValue> {
        match &self {
            Self::Equal => vec![1.0 / num_miners as PowerValue; num_miners],
            Self::SetValues(dist) => dist.clone(),
            Self::SetMiner(miner_id, power) => {
                let other = (1.0 - power) / (num_miners - 1) as PowerValue;

                let mut dist = Vec::with_capacity(num_miners);
                for i in 1..=num_miners {
                    dist.push(if i == *miner_id { *power } else { other });
                }

                dist
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PowerDistribution;

    #[test]
    fn power_dist_equal_power() {
        assert_eq!(
            PowerDistribution::Equal.values(4).unwrap(),
            vec![0.25, 0.25, 0.25, 0.25]
        )
    }
}
