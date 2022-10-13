pub struct SuffixMatcher {
    target: Vec<char>,
    backtrack: Vec<usize>,
}

impl SuffixMatcher {
    pub fn new(target: &str) -> Self {
        let chars: Vec<_> = target.chars().collect();
        let mut backs = vec![0; chars.len() + 1];

        chars.iter().enumerate().skip(1).for_each(|(nth, &ch)| {
            let mut idx = backs[nth];
            let back_idx = loop {
                if chars[idx] == ch {
                    break idx + 1;
                } else if idx > 0 {
                    idx = backs[idx];
                } else {
                    break 0;
                }
            };
            backs[idx + 1] = back_idx;
        });

        assert_eq!(chars.len() + 1, backs.len());
        Self {
            target: chars,
            backtrack: backs,
        }
    }

    pub fn try_match(&self, query: &str) -> bool {
        let Self { backtrack, target } = self;

        if target.is_empty() {
            return true;
        }

        let mut idx = 0;

        for qch in query.chars() {
            loop {
                let tch = target[idx];

                if tch == qch {
                    idx += 1;

                    if idx == target.len() {
                        return true;
                    } else {
                        break;
                    }
                } else if idx > 0 {
                    idx = backtrack[idx];
                } else {
                    break;
                }
            }
        }

        idx > 0
    }
}

#[cfg(test)]
mod tests {
    use crate::SuffixMatcher;

    #[test]
    fn suffix_matching_test() {
        {
            let matcher = SuffixMatcher::new("");
            assert!(matcher.try_match(""));
            assert!(matcher.try_match("a"));
            assert!(matcher.try_match("aa"));
            assert!(matcher.try_match("aaa"));
        }

        {
            let matcher = SuffixMatcher::new("abc");
            assert!(!matcher.try_match(""));
            assert!(matcher.try_match("a"));
            assert!(matcher.try_match("ab"));
            assert!(matcher.try_match("abc"));
            assert!(matcher.try_match("abcab"));
            assert!(!matcher.try_match("bb"));
        }

        {
            let matcher = SuffixMatcher::new("aaa");
            assert!(!matcher.try_match(""));
            assert!(matcher.try_match("a"));
            assert!(matcher.try_match("aa"));
            assert!(matcher.try_match("aaa"));
            assert!(matcher.try_match("aaaa"));
            assert!(matcher.try_match("baaa"));
            assert!(matcher.try_match("baaa"));
            assert!(matcher.try_match("ba"));
            assert!(!matcher.try_match("b"));
        }
    }
}
