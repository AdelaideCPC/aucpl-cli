//! Fuzzy string matching for "Did you mean?" suggestions

use strsim::{jaro_winkler, levenshtein};

/// Generate "Did you mean?" suggestions
/// Returns up to `max_suggestions` closest matches
pub fn suggest_corrections(
    query: &str,
    candidates: &[&str],
    max_suggestions: usize,
) -> Vec<String> {
    if candidates.is_empty() {
        return vec![];
    }

    // Calculate scores using a combination of Jaro-Winkler and edit distance
    let mut scored: Vec<(f64, &str)> = candidates
        .iter()
        .map(|&candidate| {
            let jw_score = jaro_winkler(query, candidate);
            let edit_dist = levenshtein(query, candidate) as f64;
            let max_len = query.len().max(candidate.len()) as f64;
            let edit_score = 1.0 - (edit_dist / max_len);

            // Combine scores, weighting Jaro-Winkler higher
            let combined_score = jw_score * 0.7 + edit_score * 0.3;
            (combined_score, candidate)
        })
        .filter(|(score, _)| *score > 0.5) // Filter out very poor matches
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    scored
        .into_iter()
        .take(max_suggestions)
        .map(|(_, name)| name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_corrections() {
        let candidates = vec!["my-problem", "other-problem", "test-problem"];
        let suggestions = suggest_corrections("my-problm", &candidates, 3);

        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0], "my-problem");
    }
}
