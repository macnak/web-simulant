// Normal (Gaussian) distribution

use super::Distribution;
use rand_distr::Distribution as RandDistribution;
use rand_distr::Normal;
use std::time::Duration;

pub struct NormalDistribution {
    mean_ms: f64,
    stddev_ms: f64,
}

impl NormalDistribution {
    pub fn new(mean_ms: f64, stddev_ms: f64) -> Self {
        Self { mean_ms, stddev_ms }
    }
}

impl Distribution for NormalDistribution {
    fn sample(&self) -> Duration {
        let normal = Normal::new(self.mean_ms, self.stddev_ms)
            .expect("mean and stddev should be validated before use");

        for _ in 0..10 {
            let value = normal.sample(&mut rand::thread_rng());
            if value.is_finite() && value >= 0.0 {
                return Duration::from_secs_f64(value / 1000.0);
            }
        }

        Duration::from_secs_f64(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_distribution() {
        let dist = NormalDistribution::new(50.0, 10.0);
        let mut samples = Vec::with_capacity(5000);

        for _ in 0..5000 {
            let ms = dist.sample().as_secs_f64() * 1000.0;
            samples.push(ms);
        }

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance = samples
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / samples.len() as f64;
        let stddev = variance.sqrt();

        assert!(mean >= 42.5 && mean <= 57.5, "mean out of range: {}", mean);
        assert!(stddev >= 7.0 && stddev <= 13.5, "stddev out of range: {}", stddev);
    }
}
