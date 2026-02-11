// Exponential distribution

use super::Distribution;
use rand_distr::Distribution as RandDistribution;
use rand_distr::Exp;
use std::time::Duration;

pub struct ExponentialDistribution {
    rate: f64,
}

impl ExponentialDistribution {
    pub fn new(rate: f64) -> Self {
        Self { rate }
    }
}

impl Distribution for ExponentialDistribution {
    fn sample(&self) -> Duration {
        let exp = Exp::new(self.rate).expect("rate should be validated before use");
        let value = exp.sample(&mut rand::thread_rng());
        let clamped = if value.is_finite() && value >= 0.0 { value } else { 0.0 };
        Duration::from_secs_f64(clamped / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_distribution() {
        let rate = 0.02;
        let expected_mean = 1.0 / rate;
        let dist = ExponentialDistribution::new(rate);
        let mut total = 0.0;
        let samples = 5000;

        for _ in 0..samples {
            total += dist.sample().as_secs_f64() * 1000.0;
        }

        let mean = total / samples as f64;
        let lower = expected_mean * 0.8;
        let upper = expected_mean * 1.2;
        assert!(mean >= lower && mean <= upper, "mean out of range: {}", mean);
    }
}
