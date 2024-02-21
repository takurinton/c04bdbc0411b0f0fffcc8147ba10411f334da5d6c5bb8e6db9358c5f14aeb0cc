use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Regix {
    Literal(String),
    Concat(Vec<Regix>),
    Or(Vec<Regix>),
    Star(Box<Regix>),
    Plus(Box<Regix>),
    Question(Box<Regix>),
    Any,
    Char(char),
    Range(char, char),
    NotRange(char, char),
    Set(Vec<Regix>),
    NotSet(Vec<Regix>),
}

impl Regix {
    fn new(pattern_str: &str) -> Regix {
        let mut pattern_str = pattern_str.to_string();
        let mut pattern = Vec::new();
        while pattern_str.len() > 0 {
            let c = pattern_str.remove(0);
            match c {
                '|' => {
                    let mut v = Vec::new();
                    let mut s = String::new();
                    while pattern_str.len() > 0 {
                        let c = pattern_str.remove(0);
                        if c == '|' {
                            break;
                        }
                        s.push(c);
                    }
                    v.push(Regix::new(&s));
                    pattern.push(Regix::Or(v));
                }
                '*' => {
                    let r = pattern.pop().unwrap();
                    pattern.push(Regix::Star(Box::new(r)));
                }
                '+' => {
                    let r = pattern.pop().unwrap();
                    pattern.push(Regix::Plus(Box::new(r)));
                }
                '?' => {
                    let r = pattern.pop().unwrap();
                    pattern.push(Regix::Question(Box::new(r)));
                }
                '.' => pattern.push(Regix::Any),
                '[' => {
                    let mut v = Vec::new();
                    let mut s = String::new();
                    while pattern_str.len() > 0 {
                        let c = pattern_str.remove(0);
                        if c == ']' {
                            break;
                        }
                        s.push(c);
                    }
                    let mut s = s.chars();
                    let c1 = s.next().unwrap();
                    if s.next() == Some('-') {
                        let c2 = s.next().unwrap();
                        v.push(Regix::Range(c1, c2));
                    } else {
                        v.push(Regix::Char(c1));
                    }
                    pattern.push(Regix::Set(v));
                }
                '^' => {
                    let mut v = Vec::new();
                    let mut s = String::new();
                    while pattern_str.len() > 0 {
                        let c = pattern_str.remove(0);
                        if c == ']' {
                            break;
                        }
                        s.push(c);
                    }
                    let mut s = s.chars();
                    let c1 = s.next().unwrap();
                    if s.next() == Some('-') {
                        let c2 = s.next().unwrap();
                        v.push(Regix::NotRange(c1, c2));
                    } else {
                        v.push(Regix::Char(c1));
                    }
                    pattern.push(Regix::NotSet(v));
                }
                _ => pattern.push(Regix::Char(c)),
            }
        }

        if pattern.len() == 1 {
            pattern.pop().unwrap()
        } else {
            Regix::Concat(pattern)
        }
    }

    fn match_string(&self, s: &str) -> bool {
        let mut map = HashMap::new();
        self.match_string_with_map(s, &mut map)
    }

    fn match_string_with_map(&self, s: &str, map: &mut HashMap<usize, usize>) -> bool {
        match self {
            Regix::Literal(l) => s == l,
            Regix::Concat(v) => {
                let mut s = s;
                for r in v {
                    if !r.match_string_with_map(s, map) {
                        return false;
                    }
                    s = &s[map[&s.len()]..];
                }
                true
            }
            Regix::Or(v) => {
                for r in v {
                    if r.match_string_with_map(s, map) {
                        return true;
                    }
                }
                false
            }
            Regix::Star(r) => {
                let mut s = s;
                while r.match_string_with_map(s, map) {
                    s = &s[map[&s.len()]..];
                }
                true
            }
            Regix::Plus(r) => {
                let mut s = s;
                while r.match_string_with_map(s, map) {
                    s = &s[map[&s.len()]..];
                }
                s.len() != s.len()
            }
            Regix::Question(r) => {
                r.match_string_with_map(s, map);
                true
            }
            Regix::Any => s.len() > 0,
            Regix::Char(c) => s.starts_with(*c),
            Regix::Range(c1, c2) => {
                let c = s.chars().next().unwrap();
                c >= *c1 && c <= *c2
            }
            Regix::NotRange(c1, c2) => {
                let c = s.chars().next().unwrap();
                c < *c1 || c > *c2
            }
            Regix::Set(v) => {
                for r in v {
                    if r.match_string_with_map(s, map) {
                        return true;
                    }
                }
                false
            }
            Regix::NotSet(v) => {
                for r in v {
                    if r.match_string_with_map(s, map) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl fmt::Display for Regix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Regix::Literal(s) => write!(f, "{}", s),
            Regix::Concat(v) => {
                let mut s = String::new();
                for r in v {
                    s.push_str(&format!("{}", r));
                }
                write!(f, "{}", s)
            }
            Regix::Or(v) => {
                let mut s = String::new();
                for r in v {
                    s.push_str(&format!("|{}", r));
                }
                write!(f, "({})", s)
            }
            Regix::Star(r) => write!(f, "({})*", r),
            Regix::Plus(r) => write!(f, "({})+", r),
            Regix::Question(r) => write!(f, "({})?", r),
            Regix::Any => write!(f, "."),
            Regix::Char(c) => write!(f, "{}", c),
            Regix::Range(c1, c2) => write!(f, "[{}-{}]", c1, c2),
            Regix::NotRange(c1, c2) => write!(f, "[^{}-{}]", c1, c2),
            Regix::Set(v) => {
                let mut s = String::new();
                for r in v {
                    s.push_str(&format!("{}", r));
                }
                write!(f, "[{}]", s)
            }
            Regix::NotSet(v) => {
                let mut s = String::new();
                for r in v {
                    s.push_str(&format!("{}", r));
                }
                write!(f, "[^{}]", s)
            }
        }
    }
}

#[cfg(test)]
mod regix {
    use super::*;

    #[test]
    fn test_new() {
        let regix = Regix::new("a");
        assert_eq!(regix, Regix::Char('a'));

        let regix = Regix::new("ab");
        assert_eq!(
            regix,
            Regix::Concat(vec![Regix::Char('a'), Regix::Char('b')])
        );

        let regix = Regix::new("a|b");
        assert_eq!(regix, Regix::Or(vec![Regix::Char('a'), Regix::Char('b')]));

        let regix = Regix::new("a*");
        assert_eq!(regix, Regix::Star(Box::new(Regix::Char('a'))));

        let regix = Regix::new("a+");
        assert_eq!(regix, Regix::Plus(Box::new(Regix::Char('a'))));

        let regix = Regix::new("a?");
        assert_eq!(regix, Regix::Question(Box::new(Regix::Char('a'))));

        let regix = Regix::new(".");
        assert_eq!(regix, Regix::Any);

        let regix = Regix::new("[a-z]");
        assert_eq!(regix, Regix::Range('a', 'z'));

        let regix = Regix::new("[^a-z]");
        assert_eq!(regix, Regix::NotRange('a', 'z'));

        let regix = Regix::new("[a-z0-9]");
        assert_eq!(
            regix,
            Regix::Concat(vec![Regix::Range('a', 'z'), Regix::Range('0', '9')])
        );
    }

    // #[test]
    // fn test_fmt() {
    //     let regix = Regix::new("a");
    //     assert_eq!(format!("{}", regix), "a");

    //     let regix = Regix::new("ab");
    //     assert_eq!(format!("{}", regix), "ab");

    //     let regix = Regix::new("a|b");
    //     assert_eq!(format!("{}", regix), "(a|b)");

    //     let regix = Regix::new("a*");
    //     assert_eq!(format!("{}", regix), "(a)*");

    //     let regix = Regix::new("a+");
    //     assert_eq!(format!("{}", regix), "(a)+");
    // }

    #[test]
    fn test_match_string() {
        let regix = Regix::new("a");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("b"), false);

        let regix = Regix::new("ab");
        assert_eq!(regix.match_string("ab"), true);
        assert_eq!(regix.match_string("a"), false);
        assert_eq!(regix.match_string("b"), false);

        let regix = Regix::new("a|b");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("b"), true);
        assert_eq!(regix.match_string("c"), false);

        let regix = Regix::new("a*");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("aa"), true);
        assert_eq!(regix.match_string("b"), false);

        let regix = Regix::new("a+");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("aa"), true);
        assert_eq!(regix.match_string("b"), false);

        let regix = Regix::new("a?");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("b"), true);
        assert_eq!(regix.match_string("aa"), false);

        let regix = Regix::new(".");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("b"), true);
        assert_eq!(regix.match_string(""), false);

        let regix = Regix::new("[a-z]");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("z"), true);
        assert_eq!(regix.match_string("A"), false);

        let regix = Regix::new("[^a-z]");
        assert_eq!(regix.match_string("a"), false);
        assert_eq!(regix.match_string("z"), false);
        assert_eq!(regix.match_string("A"), true);

        let regix = Regix::new("[a-z0-9]");
        assert_eq!(regix.match_string("a"), true);
        assert_eq!(regix.match_string("z"), true);
        assert_eq!(regix.match_string("0"), true);
        assert_eq!(regix.match_string("A"), false);
    }
}
