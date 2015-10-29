use std::vec::Vec;

pub fn matches(pattern: &str, candidate: &str) -> bool {
    let pattern: Vec<&str> = pattern.split("/").collect();
    let candidate: Vec<&str> = candidate.split("/").collect();
    matches_segments(&pattern[..], &candidate[..])
}

fn matches_segments(pattern: &[&str], candidate: &[&str]) -> bool {
    if pattern.len() == 0 {
        // We need to run out of pattern and candidate at the same time
        candidate.len() == 0
    } else if candidate.len() == 0 {
        // If the candidate ends before the pattern, reject it. The candidate
        // must not be more general than the pattern. Only exception is ending on "**".
        pattern.len() == 1 && pattern[0] == "**"
    } else if pattern[0] == "**" {
        // In this case the path can span multiple directories. For simplicity,
        // it's not allowed to have a partial match here, so the pattern must be
        // exactly "**". Here we'll follow a "use it or lose it strategy" where
        // each recursive time we either consume a candidate segment, or we stop consuming.
        matches_segments(&pattern[1..], candidate) || matches_segments(pattern, &candidate[1..])
    } else if pattern[0].contains("*") {
        // This is the case where we have a single wildcard in in the pattern. We'll defer
        // to a helper function to help
        matches_glob(pattern[0], candidate[0]) && matches_segments(&pattern[1..], &candidate[1..])
    } else {
        // If there are no wildcards, we can do a straight comparison on the segments
        pattern[0] == candidate[0] && matches_segments(&pattern[1..], &candidate[1..])
    }
}

fn matches_glob(pattern: &str, candidate: &str) -> bool {
    if pattern.len() == 0 {
        candidate.len() == 0
    } else if candidate.len() == 0 {
        pattern == "*"
    } else if pattern.chars().nth(0).unwrap() == '*' {
        // Use it or lose it. We either consume a candidate charactor or stop consuming
        matches_glob(&pattern[1..], candidate) || matches_glob(pattern, &candidate[1..])
    } else {
        pattern.chars().nth(0).unwrap() == candidate.chars().nth(0).unwrap() &&
        matches_glob(&pattern[1..], &candidate[1..])
    }
}

#[cfg(test)]
mod tests {
    use super::matches_glob;
    use super::matches_segments;

    #[test]
    fn matches_segments_works_on_empty_slices() {
        assert_eq!(true, matches_segments(&[], &[]));
        assert_eq!(true, matches_segments(&[""], &[""]));
        assert_eq!(false, matches_segments(&["a"], &[""]));
        assert_eq!(false, matches_segments(&["a"], &[]));
        assert_eq!(false, matches_segments(&[""], &["a"]));
        assert_eq!(false, matches_segments(&[], &["a"]));
    }

    #[test]
    fn matches_glob_works_on_empty_strings() {
        assert_eq!(true, matches_glob("", ""));
        assert_eq!(false, matches_glob("a", ""));
        assert_eq!(false, matches_glob("", "a"));
    }

    #[test]
    fn matches_glob_works_with_full_wildcard_pattern() {
        assert_eq!(true, matches_glob("*", "*"));
        assert_eq!(true, matches_glob("*", "a"));
        assert_eq!(true, matches_glob("*", "abc*"));
        assert_eq!(true, matches_glob("*", "*abc"));
    }

    #[test]
    fn matches_segments_works_with_single_wildcard_pattern() {
        assert_eq!(true, matches_segments(&["*"], &["*"]));
        assert_eq!(true, matches_segments(&["*"], &["a"]));
        assert_eq!(true, matches_segments(&["*"], &["abcde"]));
        assert_eq!(true, matches_segments(&["*"], &["abc*"]));
        assert_eq!(true, matches_segments(&["*"], &["*abc"]));

        // Shouldn't cross segment boundaries
        assert_eq!(false, matches_segments(&["*"], &["a", "b"]));
    }

    #[test]
    fn matches_segments_works_with_dual_wildcard_pattern() {
        assert_eq!(true, matches_segments(&["**"], &["*"]));
        assert_eq!(true, matches_segments(&["**"], &["a"]));
        assert_eq!(true, matches_segments(&["**"], &["ab", "cd", "de"]));
        assert_eq!(true, matches_segments(&["**"], &["abc", "*"]));
        assert_eq!(true, matches_segments(&["**"], &["*", "abc"]));

        assert_eq!(false, matches_segments(&["**b"], &["abc"]));
        assert_eq!(false, matches_segments(&["b**"], &["abc"]));
    }

    #[test]
    fn matches_glob_works_with_partial_wildcard_pattern() {
        assert_eq!(true, matches_glob("*a", "cba"));
        assert_eq!(true, matches_glob("*a", "a"));
        assert_eq!(false, matches_glob("*a", "b"));

        assert_eq!(true, matches_glob("a*", "abc"));
        assert_eq!(true, matches_glob("a*", "a"));
        assert_eq!(false, matches_glob("a*", "b"));

        assert_eq!(true, matches_glob("a*b", "afb"));
        assert_eq!(true, matches_glob("a*b", "afghb"));
        assert_eq!(true, matches_glob("a*b", "ab"));
        assert_eq!(false, matches_glob("a*b", "afghbf"));
    }

    #[test]
    fn matches_segments_works_with_partial_single_wildcard_pattern() {
        assert_eq!(true, matches_segments(&["a", "*"], &["a", "foo"]));
        assert_eq!(true, matches_segments(&["a", "*"], &["a", ""]));
        assert_eq!(false, matches_segments(&["a", "*"], &["b", "foo"]));
        assert_eq!(false, matches_segments(&["a", "*"], &["a"]));

        assert_eq!(false, matches_segments(&["*", "a"], &["a"]));
        assert_eq!(false, matches_segments(&["*", "a"], &["ab", "a", "de"]));
        assert_eq!(true, matches_segments(&["*", "a"], &["abc", "a"]));
        assert_eq!(true, matches_segments(&["*", "a"], &["*", "a"]));

        assert_eq!(true, matches_segments(&["b", "*", "a"], &["b", "*", "a"]));
        assert_eq!(true, matches_segments(&["b", "*", "a"], &["b", "asdf", "a"]));
        assert_eq!(true, matches_segments(&["b", "*", "a"], &["b", "a", "a"]));
        assert_eq!(false, matches_segments(&["b", "*", "a"], &["foo", "a", "a"]));
        assert_eq!(false, matches_segments(&["b", "*", "a"], &["b", "a", "foo"]));

        assert_eq!(true, matches_segments(&["b", "*", "a", "*"], &["b", "foo", "a", "bar"]));
        assert_eq!(true, matches_segments(&["b", "*", "a", "*"], &["b", "a", "a", "a"]));
    }

    #[test]
    fn matches_segments_works_with_partial_dual_wildcard_pattern() {
        assert_eq!(true, matches_segments(&["a", "**"], &["a", "foo"]));
        assert_eq!(true, matches_segments(&["a", "**"], &["a", "foo", "bar"]));
        assert_eq!(true, matches_segments(&["a", "**"], &["a", ""]));
        assert_eq!(false, matches_segments(&["a", "**"], &["b", "foo"]));
        assert_eq!(true, matches_segments(&["a", "**"], &["a"]));

        assert_eq!(true, matches_segments(&["**", "a"], &["a"]));
        assert_eq!(false, matches_segments(&["**", "a"], &["ab", "a", "de"]));
        assert_eq!(true, matches_segments(&["**", "a"], &["abc", "a"]));
        assert_eq!(true, matches_segments(&["**", "a"], &["*", "a"]));
        assert_eq!(true, matches_segments(&["**", "a"], &["*", "a"]));
        assert_eq!(true, matches_segments(&["**", "a"], &["def", "jkl", "a"]));

        assert_eq!(true, matches_segments(&["b", "**", "a"], &["b", "jkl", "a"]));
        assert_eq!(false, matches_segments(&["b", "**", "a"], &["b", "jkl", "foo"]));
        assert_eq!(false, matches_segments(&["b", "**", "a"], &["b", "jkl", "", "foo"]));
    }

    #[test]
    fn matches_glob_works_with_multiple_wildcards() {
        assert_eq!(true, matches_glob("*a*", "cba"));
        assert_eq!(true, matches_glob("*a*", "a"));
        assert_eq!(true, matches_glob("*a*", "abc"));
        assert_eq!(true, matches_glob("*a*", "bac"));
        assert_eq!(false, matches_glob("*a*", "b"));
        assert_eq!(false, matches_glob("*a*", ""));

        assert_eq!(true, matches_glob("a*a*", "aba"));
        assert_eq!(true, matches_glob("a*a*", "abaf"));
    }

    #[test]
    fn matches_segments_works_with_multiple_dual_wildcards() {
        assert_eq!(true, matches_segments(&["a", "**", "**"], &["a", "foo"]));
        assert_eq!(true, matches_segments(&["a", "**", "**"], &["a", "bar", "foo"]));
        assert_eq!(true, matches_segments(&["**", "**", "b"], &["a", "bar", "blah", "b"]));

        assert_eq!(false, matches_segments(&["a", "**", "b"], &["a", "bar", "blah", "foo"]));
        assert_eq!(true, matches_segments(&["a", "**", "b"], &["a", "bar", "blah", "foo", "b"]));
    }
}
