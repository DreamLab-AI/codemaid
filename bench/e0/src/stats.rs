//! Summary statistics, the commit bootstrap, the Wilson bound and the seeded
//! stratified draw. Randomness is `rand_chacha::ChaCha8Rng::seed_from_u64`
//! with indices drawn as `u32`, so a draw is the same on every platform.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// The pre-registered seed.
pub const SEED: u64 = 20261006;
/// The pre-registered number of bootstrap resamples.
pub const RESAMPLES: usize = 10_000;

/// Median: the middle value, or the mean of the two middle values.
pub fn median(v: &[u32]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_unstable();
    let n = s.len();
    if n % 2 == 1 { f64::from(s[n / 2]) } else { (f64::from(s[n / 2 - 1]) + f64::from(s[n / 2])) / 2.0 }
}

/// 90th percentile by nearest rank: the value at rank ceil(0.9 n).
pub fn p90(v: &[u32]) -> u32 {
    if v.is_empty() {
        return 0;
    }
    let mut s = v.to_vec();
    s.sort_unstable();
    let rank = (s.len() * 9).div_ceil(10);
    s[rank.max(1) - 1]
}

/// Σnum / Σden, `None` when Σden is zero.
pub fn ratio(num: &[u32], den: &[u32]) -> Option<f64> {
    let n: u64 = num.iter().map(|&x| u64::from(x)).sum();
    let d: u64 = den.iter().map(|&x| u64::from(x)).sum();
    (d > 0).then(|| n as f64 / d as f64)
}

/// Percentile bootstrap of Σnum/Σden over resampled commits.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Bootstrap {
    pub resamples: usize,
    pub seed: u64,
    /// 2.5th percentile (nearest rank: the 250th smallest of 10,000).
    pub lo: f64,
    /// 97.5th percentile (nearest rank: the 9,750th smallest).
    pub hi: f64,
    /// Resamples whose denominator was zero (ratio infinite; sorted last).
    pub infinite: usize,
}

/// Resample commit indices with replacement `RESAMPLES` times.
pub fn bootstrap(num: &[u32], den: &[u32]) -> Bootstrap {
    assert_eq!(num.len(), den.len());
    let n = num.len() as u32;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut rs = Vec::with_capacity(RESAMPLES);
    let mut infinite = 0;
    for _ in 0..RESAMPLES {
        let (mut a, mut b) = (0u64, 0u64);
        for _ in 0..n {
            let i = rng.random_range(0..n) as usize;
            a += u64::from(num[i]);
            b += u64::from(den[i]);
        }
        if b == 0 {
            infinite += 1;
            rs.push(f64::INFINITY);
        } else {
            rs.push(a as f64 / b as f64);
        }
    }
    rs.sort_by(f64::total_cmp);
    let lo = rs[RESAMPLES * 25 / 1000 - 1];
    let hi = rs[RESAMPLES * 975 / 1000 - 1];
    Bootstrap { resamples: RESAMPLES, seed: SEED, lo, hi, infinite }
}

/// Wilson score interval upper bound at 95% (z = 1.959964).
pub fn wilson_upper(k: usize, n: usize) -> Option<f64> {
    if n == 0 {
        return None;
    }
    let z = 1.959_963_984_540_054_f64;
    let (k, n) = (k as f64, n as f64);
    let p = k / n;
    let z2 = z * z;
    let centre = p + z2 / (2.0 * n);
    let margin = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    Some((centre + margin) / (1.0 + z2 / n))
}

/// Wilson score interval at 95% (z = 1.959964): `(lower, upper)`.
pub fn wilson(k: usize, n: usize) -> Option<(f64, f64)> {
    if n == 0 {
        return None;
    }
    let z = 1.959_963_984_540_054_f64;
    let (k, n) = (k as f64, n as f64);
    let p = k / n;
    let z2 = z * z;
    let centre = p + z2 / (2.0 * n);
    let margin = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    let d = 1.0 + z2 / n;
    Some((((centre - margin) / d).max(0.0), ((centre + margin) / d).min(1.0)))
}

/// Draw `k` of `items` (or all, if fewer) without replacement by a partial
/// Fisher–Yates shuffle on `rng`; the drawn items come back in input order.
pub fn draw<T: Clone>(items: &[T], k: usize, rng: &mut ChaCha8Rng) -> Vec<T> {
    if items.len() <= k {
        return items.to_vec();
    }
    let mut idx: Vec<u32> = (0..items.len() as u32).collect();
    for i in 0..k {
        let j = rng.random_range(i as u32..idx.len() as u32) as usize;
        idx.swap(i, j);
    }
    let mut chosen: Vec<u32> = idx[..k].to_vec();
    chosen.sort_unstable();
    chosen.into_iter().map(|i| items[i as usize].clone()).collect()
}

/// A fresh generator on the pre-registered seed.
pub fn rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(SEED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_and_p90() {
        assert_eq!(median(&[3, 1, 2]), 2.0);
        assert_eq!(median(&[4, 1, 2, 3]), 2.5);
        let v: Vec<u32> = (1..=100).collect();
        assert_eq!(p90(&v), 90);
        assert_eq!(p90(&[5]), 5);
        assert_eq!(p90(&(1..=11).collect::<Vec<_>>()), 10);
    }

    #[test]
    fn bootstrap_is_seeded_and_brackets_the_point_estimate() {
        let num: Vec<u32> = (0..100).map(|i| 5 + i % 7).collect();
        let den: Vec<u32> = (0..100).map(|i| 1 + i % 3).collect();
        let a = bootstrap(&num, &den);
        assert_eq!(a, bootstrap(&num, &den));
        let r = ratio(&num, &den).unwrap();
        assert!(a.lo < r && r < a.hi, "{a:?} {r}");
    }

    #[test]
    fn wilson_matches_a_known_value() {
        // 0 of 40: upper bound 0.0876 (z = 1.96).
        assert!((wilson_upper(0, 40).unwrap() - 0.08762).abs() < 1e-4);
        // 4 of 40: upper bound 0.2306.
        assert!((wilson_upper(4, 40).unwrap() - 0.2306).abs() < 1e-3);
        // Both bounds: 4 of 40 → 0.0396–0.2306; 40 of 40 → lower 0.9124.
        let (lo, hi) = wilson(4, 40).unwrap();
        assert!((lo - 0.0396).abs() < 1e-3 && (hi - wilson_upper(4, 40).unwrap()).abs() < 1e-12);
        assert!((wilson(40, 40).unwrap().0 - 0.9124).abs() < 1e-3);
        assert_eq!(wilson(0, 0), None);
    }

    #[test]
    fn draw_is_deterministic_and_without_replacement() {
        let items: Vec<u32> = (0..50).collect();
        let a = draw(&items, 30, &mut rng());
        assert_eq!(a, draw(&items, 30, &mut rng()));
        let mut d = a.clone();
        d.dedup();
        assert_eq!(d.len(), 30);
        assert_eq!(draw(&items[..5], 30, &mut rng()).len(), 5);
    }
}
