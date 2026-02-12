// Distribution implementations for latency generation

mod fixed;
mod normal;
mod exponential;
mod uniform;
mod log_normal;

pub use fixed::FixedDistribution;
pub use normal::NormalDistribution;
pub use exponential::ExponentialDistribution;
pub use uniform::UniformDistribution;
pub use log_normal::LogNormalDistribution;

use std::time::Duration;

/// Trait for latency distributions
pub trait Distribution: Send + Sync {
    /// Generate a latency duration
    fn sample(&self) -> Duration;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let dist = FixedDistribution::new(1.0);
        let sampled = dist.sample();
        assert_eq!(sampled, Duration::from_millis(1));
    }
}
