use std::collections::HashMap;

pub struct SpellChecker {
    dictionary: HashMap<String, String>,
}

impl SpellChecker {
    pub fn new() -> Self {
        let mut dictionary = HashMap::new();

        // Add common misspellings
        dictionary.insert("crate".to_string(), "create".to_string());
        dictionary.insert("creat".to_string(), "create".to_string());
        dictionary.insert("craete".to_string(), "create".to_string());
        dictionary.insert("databse".to_string(), "database".to_string());
        dictionary.insert("datbase".to_string(), "database".to_string());
        dictionary.insert("colection".to_string(), "collection".to_string());
        dictionary.insert("colections".to_string(), "collections".to_string());
        dictionary.insert("clled".to_string(), "called".to_string());
        dictionary.insert("shoow".to_string(), "show".to_string());

        Self { dictionary }
    }

    pub fn check(&self, word: &str) -> Option<String> {
        let lower = word.to_lowercase();

        // Check direct dictionary match
        if let Some(correction) = self.dictionary.get(&lower) {
            return Some(correction.clone());
        }

        // Check Levenshtein distance for close matches
        self.find_closest_match(&lower)
    }

    fn find_closest_match(&self, word: &str) -> Option<String> {
        let keywords = vec![
            "create", "database", "collection", "collections",
            "called", "show", "these", "in", "it"
        ];

        let mut best_match = None;
        let mut best_distance = 2; // Max allowed distance

        for keyword in keywords {
            let distance = self.edit_distance(word, keyword);
            // Only suggest if there's an actual difference (distance > 0)
            if distance > 0 && distance <= best_distance {
                best_distance = distance;
                best_match = Some(keyword.to_string());
            }
        }

        best_match
    }

    fn edit_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        let mut dp = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            dp[i][0] = i;
        }

        for j in 0..=b_len {
            dp[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                if a_chars[i - 1] == b_chars[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1]);
                }
            }
        }

        dp[a_len][b_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_correction() {
        let checker = SpellChecker::new();
        assert_eq!(checker.check("crate"), Some("create".to_string()));
        assert_eq!(checker.check("databse"), Some("database".to_string()));
    }

    #[test]
    fn test_edit_distance() {
        let checker = SpellChecker::new();
        assert_eq!(checker.edit_distance("create", "create"), 0);
        assert_eq!(checker.edit_distance("crate", "create"), 1);
        assert_eq!(checker.edit_distance("databse", "database"), 1);
    }

    #[test]
    fn test_no_correction_needed() {
        let checker = SpellChecker::new();
        assert_eq!(checker.check("create"), None);
        assert_eq!(checker.check("database"), None);
    }
}
