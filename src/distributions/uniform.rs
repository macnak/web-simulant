// Uniform distribution

use super::Distribution;
use rand::Rng;
use std::time::Duration;

pub struct UniformDistribution {
    min_ms: f64,
    max_ms: f64,
}

impl UniformDistribution {
    pub fn new(min_ms: f64, max_ms: f64) -> Self {
        Self { min_ms, max_ms }
    }
}

impl Distribution for UniformDistribution {
    fn sample(&self) -> Duration {
        let value = rand::thread_rng().gen_range(self.min_ms..self.max_ms);
        Duration::from_secs_f64(value / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_distribution() {
        let dist = UniformDistribution::new(10.0, 30.0);
        let samples = 5000;
        let mut total = 0.0;
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for _ in 0..samples {
            let value = dist.sample().as_secs_f64() * 1000.0;
            total += value;
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
        }

        let mean = total / samples as f64;
        assert!(min >= 10.0, "min below range: {}", min);
        assert!(max < 30.0, "max above range: {}", max);
        assert!(mean >= 18.0 && mean <= 22.0, "mean out of range: {}", mean);
    }
}
