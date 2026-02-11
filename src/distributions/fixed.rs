// Fixed latency distribution

use super::Distribution;
use std::time::Duration;

pub struct FixedDistribution {
    delay: Duration,
}

impl FixedDistribution {
    pub fn new(delay_ms: f64) -> Self {
        Self {
            delay: Duration::from_secs_f64(delay_ms / 1000.0),
        }
    }
}

impl Distribution for FixedDistribution {
    fn sample(&self) -> Duration {
        self.delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_distribution() {
        let dist = FixedDistribution::new(25.0);
        let sampled = dist.sample();
        assert_eq!(sampled, Duration::from_millis(25));
    }
}
