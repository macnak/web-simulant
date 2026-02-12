// Log-normal distribution

use super::Distribution;
use rand_distr::Distribution as RandDistribution;
use rand_distr::LogNormal;
use std::time::Duration;

pub struct LogNormalDistribution {
    mean_ms: f64,
    stddev_ms: f64,
}

impl LogNormalDistribution {
    pub fn new(mean_ms: f64, stddev_ms: f64) -> Self {
        Self { mean_ms, stddev_ms }
    }

    fn mu_sigma(&self) -> Option<(f64, f64)> {
        if self.mean_ms <= 0.0 || self.stddev_ms < 0.0 {
            return None;
        }
        if self.stddev_ms == 0.0 {
            return Some((self.mean_ms.ln(), 0.0));
        }

        let variance = self.stddev_ms.powi(2);
        let mean_sq = self.mean_ms.powi(2);
        let sigma_sq = (1.0 + (variance / mean_sq)).ln();
        let sigma = sigma_sq.sqrt();
        let mu = (mean_sq / (variance + mean_sq).sqrt()).ln();
        Some((mu, sigma))
    }
}

impl Distribution for LogNormalDistribution {
    fn sample(&self) -> Duration {
        let Some((mu, sigma)) = self.mu_sigma() else {
            return Duration::from_secs_f64(0.0);
        };

        if sigma == 0.0 {
            return Duration::from_secs_f64(self.mean_ms / 1000.0);
        }

        let dist = LogNormal::new(mu, sigma).expect("mu and sigma should be valid");
        let value = dist.sample(&mut rand::thread_rng());
        let clamped = if value.is_finite() && value >= 0.0 { value } else { 0.0 };
        Duration::from_secs_f64(clamped / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_normal_distribution_mean() {
        let dist = LogNormalDistribution::new(100.0, 20.0);
        let samples = 5000;
        let mut total = 0.0;

        for _ in 0..samples {
            total += dist.sample().as_secs_f64() * 1000.0;
        }

        let mean = total / samples as f64;
        assert!(mean >= 80.0 && mean <= 120.0, "mean out of range: {}", mean);
    }
}
