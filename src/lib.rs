use std::str::CharIndices;

pub trait Logfmt<'a> {
    fn logfmt(&'a self) -> Iter<'a>;
}

impl<'a> Logfmt<'a> for str {
    fn logfmt(&'a self) -> Iter<'a> {
        Iter {
            text: self,
            chars_indices: self.char_indices(),
            state: State::Init,
        }
    }
}

pub struct Iter<'a> {
    text: &'a str,
    chars_indices: CharIndices<'a>,
    state: State,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        for (idx, input) in &mut self.chars_indices {
            let next = self.state.next(idx, input);
            if let State::ValueEnd(ks, ke, vs, ve) = next {
                self.state = State::Init;
                return Some((&self.text[ks..ke], &self.text[vs..ve]));
            } else {
                self.state = next;
            }
        }
        match self.state {
            State::KeyEnd(ks, ke) => {
                self.state = State::Init;
                Some((&self.text[ks..ke], Default::default()))
            }
            State::ValueStart(ks, ke, vs) => {
                self.state = State::Init;
                Some((&self.text[ks..ke], &self.text[vs..]))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum State {
    #[default]
    Init,
    KeyStart(usize),
    KeyEnd(usize, usize),
    ValueStart(usize, usize, usize),
    ValueStartWithQuote(usize, usize, usize),
    ValueEnd(usize, usize, usize, usize),
}

impl State {
    fn next(self, i: usize, c: char) -> Self {
        match self {
            State::Init => match c {
                _ if c.is_whitespace() => State::Init,
                _ => State::KeyStart(i),
            },
            State::KeyStart(ks) => match c {
                '=' => State::KeyEnd(ks, i),
                _ if c.is_whitespace() => State::Init,
                _ => State::KeyStart(ks),
            },
            State::KeyEnd(ks, ke) => match c {
                '"' => State::ValueStartWithQuote(ks, ke, i + c.len_utf8()),
                _ if c.is_whitespace() => State::ValueEnd(ks, ke, i, i),
                _ => State::ValueStart(ks, ke, i),
            },
            State::ValueStart(ks, ke, vs) => match c {
                _ if c.is_whitespace() => State::ValueEnd(ks, ke, vs, i),
                _ => State::ValueStart(ks, ke, vs),
            },
            State::ValueStartWithQuote(ks, ke, vs) => match c {
                '"' => State::ValueEnd(ks, ke, vs, i),
                _ => State::ValueStartWithQuote(ks, ke, vs),
            },
            State::ValueEnd(_, _, _, _) => State::Init,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn collect_pairs(input: &str) -> Vec<(&str, &str)> {
        input.logfmt().collect()
    }

    #[test]
    fn logfmt_collect_pairs_all_cases() {
        let cases: &[(&str, &[(&str, &str)])] = &[
            // Empty / whitespace-only
            ("", &[]),
            (" ", &[]),
            ("   \t  ", &[]),
            // Single pair
            ("a=1", &[("a", "1")]),
            ("key=value", &[("key", "value")]),
            // Leading / trailing / repeated separators
            ("  a=1", &[("a", "1")]),
            ("a=1  ", &[("a", "1")]),
            ("a=1   b=2", &[("a", "1"), ("b", "2")]),
            ("a=1\tb=2", &[("a", "1"), ("b", "2")]),
            ("a=1 \t  b=2   c=3", &[("a", "1"), ("b", "2"), ("c", "3")]),
            // Empty value
            ("a=", &[("a", "")]),
            ("a= b=2", &[("a", ""), ("b", "2")]),
            ("a= b=", &[("a", ""), ("b", "")]),
            // Quoted values (spaces kept inside quotes)
            (r#"msg="hello world""#, &[("msg", "hello world")]),
            (
                r#"a=1 msg="hello world" b=2"#,
                &[("a", "1"), ("msg", "hello world"), ("b", "2")],
            ),
            (
                r#"msg="  leading and  internal   spaces  ""#,
                &[("msg", "  leading and  internal   spaces  ")],
            ),
            // Quotes but empty
            (r#"msg="""#, &[("msg", "")]),
            // Values with punctuation / URL-like content
            ("path=/var/log/syslog", &[("path", "/var/log/syslog")]),
            (
                "url=https://example.com/a?b=c&d=e",
                &[("url", "https://example.com/a?b=c&d=e")],
            ),
            ("ip=127.0.0.1", &[("ip", "127.0.0.1")]),
            // Duplicate keys (iterator should yield in order)
            ("a=1 a=2 a=3", &[("a", "1"), ("a", "2"), ("a", "3")]),
            // Weird-but-valid key shapes (depends on your State machine rules)
            ("a_b=1", &[("a_b", "1")]),
            ("a-b=1", &[("a-b", "1")]),
            ("a.b=1", &[("a.b", "1")]),
        ];

        for (input, expected) in cases {
            let got = collect_pairs(input);
            assert_eq!(
                got.as_slice(),
                *expected,
                "mismatch for input: {input:?}\n  got: {got:?}\n  expected: {expected:?}"
            );
        }
    }
}
