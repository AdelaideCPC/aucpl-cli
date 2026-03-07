//! Shared difficulty utilities for problem management.

use anyhow::{bail, Result};

const MIN_DIFFICULTY_BUCKET: u16 = 800;
const MAX_DIFFICULTY_BUCKET: u16 = 5000;
const DIFFICULTY_BUCKET_INTERVAL: u16 = 200;

/// Calculate the difficulty bucket and string representation for a given difficulty.
///
/// Returns a tuple of (bucketed_difficulty, difficulty_string).
/// - If difficulty is 0, returns (0, "unrated")
/// - Otherwise returns the bucketed difficulty (e.g., 800, 1000, 1200) and its 4-digit string
///
/// # Errors
/// Returns an error if the difficulty exceeds MAX_DIFFICULTY_BUCKET.
pub fn calculate_difficulty_bucket(difficulty: u16) -> Result<(u16, String)> {
    if difficulty == 0 {
        return Ok((0, "unrated".to_string()));
    }

    let mut bucketed_difficulty = MIN_DIFFICULTY_BUCKET;
    if difficulty > MIN_DIFFICULTY_BUCKET {
        bucketed_difficulty += ((difficulty - MIN_DIFFICULTY_BUCKET) / DIFFICULTY_BUCKET_INTERVAL)
            * DIFFICULTY_BUCKET_INTERVAL;
    }

    if bucketed_difficulty >= MAX_DIFFICULTY_BUCKET {
        bail!("You cannot have a difficulty of over {MAX_DIFFICULTY_BUCKET}");
    }

    let difficulty_str = format!("{bucketed_difficulty:0>4}");
    Ok((bucketed_difficulty, difficulty_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unrated_difficulty() {
        let (bucket, str) = calculate_difficulty_bucket(0).unwrap();
        assert_eq!(bucket, 0);
        assert_eq!(str, "unrated");
    }

    #[test]
    fn test_min_difficulty() {
        let (bucket, str) = calculate_difficulty_bucket(800).unwrap();
        assert_eq!(bucket, 800);
        assert_eq!(str, "0800");
    }

    #[test]
    fn test_difficulty_bucket_rounding() {
        // 801-999 should bucket to 800
        let (bucket, _) = calculate_difficulty_bucket(850).unwrap();
        assert_eq!(bucket, 800);

        // 1000-1199 should bucket to 1000
        let (bucket, str) = calculate_difficulty_bucket(1000).unwrap();
        assert_eq!(bucket, 1000);
        assert_eq!(str, "1000");

        let (bucket, _) = calculate_difficulty_bucket(1199).unwrap();
        assert_eq!(bucket, 1000);
    }

    #[test]
    fn test_max_difficulty() {
        let (bucket, str) = calculate_difficulty_bucket(4999).unwrap();
        assert_eq!(bucket, 4800);
        assert_eq!(str, "4800");
    }

    #[test]
    fn test_exceeds_max_difficulty() {
        assert!(calculate_difficulty_bucket(5000).is_err());
        assert!(calculate_difficulty_bucket(6000).is_err());
    }
}
