pub trait Matcher {
    fn matches(&self, s: &[char], i: usize) -> bool;
    fn is_epsilon(&self) -> bool;
    fn label(&self) -> String;
}

pub struct ClassUnicodeMatcher {
    pub start: char,
    pub end: char,
}

impl Matcher for ClassUnicodeMatcher {
    fn matches(&self, s: &[char], i: usize) -> bool {
        let c = s[i];
        c >= self.start && c <= self.end
    }

    fn is_epsilon(&self) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("{}-{}", self.start, self.end)
    }
}

pub struct CharacterMatcher {
    pub c: char,
}
impl Matcher for CharacterMatcher {
    fn matches(&self, s: &[char], i: usize) -> bool {
        let c = s[i];
        self.c == c
    }

    fn is_epsilon(&self) -> bool {
        false
    }

    fn label(&self) -> String {
        self.c.to_string()
    }
}

pub struct EpsilonMatcher {}
impl Matcher for EpsilonMatcher {
    fn matches(&self, _s: &[char], _i: usize) -> bool {
        true
    }

    fn is_epsilon(&self) -> bool {
        true
    }

    fn label(&self) -> String {
        "ε".to_string()
    }
}
