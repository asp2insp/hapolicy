use std::vec::Vec;

/// Returns true iff the given pattern will accept the candidate.
/// The pattern and the candidate can be multi-segment paths, separated
/// by the given separator string. In this case, a "*" segment in the pattern
/// will match any sequence of characters in the corresponding segment of the
/// candidate, and a "**" segment in the pattern will match any set of segments.
/// **Using an empty seperator will result in no string splitting**
/// in the candidate.
/// # Examples
/// ## Simple matching
/// ```
/// use hapolicy::glob::matches;
/// assert_eq!(false, matches("a", "b", ""));
/// assert_eq!(false, matches("a/b", "b/a", "/"));
/// assert_eq!(true, matches("a/b", "a/b", "/"));
/// ```
/// ## Matching with globs
/// ```
/// use hapolicy::glob::matches;
/// assert_eq!(true, matches("*", "b", ""));
/// assert_eq!(true, matches("a/*", "a/foo", "/"));
/// assert_eq!(true, matches("*/*", "foo/bar", "/"));
/// assert_eq!(false, matches("*/*", "foo/bar/baz", "/"));
/// ```
/// ## Matching with multi-glob
/// ```
/// use hapolicy::glob::matches;
/// assert_eq!(true, matches("**", "b", ""));
/// assert_eq!(true, matches("a/**", "a/foo/bar", "/"));
/// assert_eq!(true, matches("a/**/*.jpg", "a/foo/bar/baz.jpg", "/"));
/// assert_eq!(false, matches("a/**/*.jpg", "a/foo/bar/baz", "/"));
/// ```
pub fn matches(pattern: &str, candidate: &str, sep: &str) -> bool {
    let pattern: Vec<&str> = match sep {
        "" => vec!(pattern),
        _ => pattern.split(sep).collect(),
    };
    let candidate: Vec<&str> = match sep {
        "" => vec!(candidate),
        _ => candidate.split(sep).collect(),
    };
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
    use super::matches;

    #[test]
    fn matches_use_cases() {
        assert_eq!(true, matches("**", "b", ""));
        assert_eq!(true, matches("a/**", "a/foo/bar", "/"));
        assert_eq!(true, matches("foo/**/*.jpg", "foo/bar/baz.jpg", "/"));
        assert_eq!(false, matches("foo/**/*.jpg", "foo/bar/baz", "/"));
    }

    #[test]
    fn matches_works_on_empty_strings() {
        assert_eq!(true, matches("", "", ""));
        assert_eq!(false, matches("a", "", ""));
        assert_eq!(false, matches("", "a", ""));

        assert_eq!(true, matches("", "", ":"));
        assert_eq!(false, matches("a", "", ":"));
        assert_eq!(false, matches("", "a", ":"));
    }

    #[test]
    fn matches_works_with_single_wildcards() {
        assert_eq!(true, matches("a/*/c", "a/b/c", "/"));
        assert_eq!(true, matches("a/*/c", "a/booooo/c", "/"));
        assert_eq!(true, matches("a/*/c", "a/*/c", "/"));

        assert_eq!(true, matches("a/*/c/*", "a/*/c/", "/"));
        assert_eq!(true, matches("a/*/c/*", "a/*/c/foo", "/"));
        assert_eq!(false, matches("a/*/c/*", "a/*/c", "/"));
    }

    #[test]
    fn matches_works_with_dual_wildcards() {
        assert_eq!(true, matches("a/**/c", "a/c", "/"));
        assert_eq!(true, matches("a/**/c", "a/b/c", "/"));
        assert_eq!(true, matches("a/**/c", "a/b/d/c", "/"));

        assert_eq!(true, matches("a/**/c/*", "a/b/c/", "/"));
        assert_eq!(true, matches("a/**/c/*", "a/b/df/c/foo", "/"));
        assert_eq!(false, matches("a/**/c/*", "a/b/d/c", "/"));

        assert_eq!(true, matches("a/b/c/**", "a/b/c", "/"));
        assert_eq!(true, matches("a/b/c/**", "a/b/c/", "/"));
        assert_eq!(true, matches("a/b/c/**", "a/b/c/d", "/"));
        assert_eq!(true, matches("a/b/c/**", "a/b/c/d/e", "/"));
    }

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
