use std::str;
use std::fmt;
use memchr;

#[derive(PartialEq, Eq)]
pub struct Token<'a> {
    ty: TokenType,
    value: &'a str
}

impl<'a> Token<'a> {
    fn new(ty: TokenType, value: &'a str) -> Self {
        Token { ty: ty, value: value }
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}Token: {:?}", self.ty, self.value)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TokenizerType {
    Whitespace,
    Blackspace,
    LineComment,
    BlockComment
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    Whitespace,
    Keyword,
    Identifier,
    Blackspace, // XXX
    LineComment,
    BlockComment
}

#[inline]
fn is_id(c: u8) -> bool {
    (c as char).is_alphabetic() || match c {
        b'$' | b'_' => true,
        _ => false
    }
}

fn is_keyword(s: &str) -> bool {
    s == "var" ||
    s == "let" ||
    s == "function" ||
    s == "return"
}

enum State {
    Unknown,
    Identifier
}

fn tokenize_blackspace<'a>(input: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();

    let mut start_index = 0;
    let mut state;
    while start_index < bytes.len() {
        if start_index != 0 {
            tokens.push(Token::new(TokenType::Whitespace, ""));
        }

        let mut end_index = start_index + 1;
        if is_id(bytes[start_index]) {
            state = State::Identifier;
            while end_index < bytes.len() && is_id(bytes[end_index]) {
                end_index += 1;
            }
        } else {
            state = State::Unknown;
        }

        let content = unsafe { str::from_utf8_unchecked(&bytes[start_index..end_index]) };
        match state {
            State::Unknown => {
                tokens.push(Token::new(TokenType::Blackspace, content));
            },
            State::Identifier => {
                tokens.push(Token::new(TokenType::Identifier, content));
            }
        }

        start_index = end_index;
    }

    tokens
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();

    let mut start_index = 0;
    let mut state = TokenizerType::Whitespace;
    while start_index < bytes.len() {
        let mut end_index = start_index;

        if start_index + 1 < bytes.len() && bytes[start_index] == b'/'
            && (bytes[start_index + 1] == b'/' || bytes[start_index + 1] == b'*') {
            if bytes[start_index + 1] == b'/' {
                state = TokenizerType::LineComment;
                end_index = end_index + memchr::memchr(b'\n', &bytes[end_index..]).unwrap_or(bytes.len() - end_index);
            } else if bytes[start_index + 1] == b'*' {
                end_index += 1; // Increment since we can guarantee it's at least one more
                state = TokenizerType::BlockComment;
                loop {
                    let next_position = memchr::memchr(b'/', &bytes[end_index..]);
                    if let Some(pos) = next_position {
                        let star_pos = end_index + pos - 1; // Right before the found slash

                        if bytes[star_pos] == b'*' {
                            end_index = end_index + pos + 1;
                            break;
                        } else {
                            end_index += 1; // Didn't find it here, try again.
                        }
                    } else {
                        end_index = bytes.len() - 1;
                        break;
                    }
                }
            }
        } else {
            while end_index < bytes.len() {
                let is_whitespace = (bytes[end_index] as char).is_whitespace();

                if (state == TokenizerType::Whitespace) != is_whitespace {
                    break;
                }

                end_index += 1;
            }
        }

        assert!(start_index < bytes.len(), "Start index is within range.");
        assert!(end_index <= bytes.len(), "End index is within range.");

        let content = unsafe { str::from_utf8_unchecked(&bytes[start_index..end_index]) };
        match state {
            TokenizerType::Whitespace => {
                tokens.push(Token::new(TokenType::Whitespace, content));
            },
            TokenizerType::Blackspace => {
                if is_keyword(content) {
                    tokens.push(Token::new(TokenType::Keyword, content));
                } else {
                    tokens.append(&mut tokenize_blackspace(content));
                }
            },
            TokenizerType::LineComment => {
                tokens.push(Token::new(TokenType::LineComment, content));
            },
            TokenizerType::BlockComment => {
                tokens.push(Token::new(TokenType::BlockComment, content));
            }
        };

        state = if state == TokenizerType::Whitespace {
            TokenizerType::Blackspace
        } else {
            TokenizerType::Whitespace
        };

        start_index = end_index;
    }

    tokens
}

#[cfg(test)]
mod bench {
    use super::*;
    use test::Bencher;

    macro_rules! _benchmark {
        ($name: ident, $toRun: expr) => (
            #[bench]
            fn $name(b: &mut Bencher) {
                b.iter(|| $toRun);
            }
        );
    }

    macro_rules! benchmark_tokenize_blackspace {
        ($name: ident, $toRun: expr) => (
            _benchmark!($name, super::tokenize_blackspace($toRun));
        )
    }

    macro_rules! benchmark_tokenize {
        ($name: ident, $toRun: expr) => (
            _benchmark!($name, tokenize($toRun));
        )
    }

    mod tokenize {
        use test::Bencher;
        use super::super::{tokenize};

        benchmark_tokenize!(function, "function test() {}");
        benchmark_tokenize!(keyword, "function");
        benchmark_tokenize!(empty, "");
        benchmark_tokenize!(space, " ");
        benchmark_tokenize!(comment_line, "// testing");
        benchmark_tokenize!(comment_block, "/* testi*/");
        benchmark_tokenize!(comment_long_block, "/* testitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitestitesti*/");
        benchmark_tokenize!(sample, include_str!("../input.js"));
    }


    benchmark_tokenize!(tokenize_ident, "$_very_Z_complex$$ident");
    benchmark_tokenize_blackspace!(tokenize_ident_blackspace, "$_very_Z_complex$$ident");
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenType};

    #[test]
    fn tokenize_line_comment() {
        let mut tokens = tokenize("// test");
        assert_eq!(tokens.remove(0), Token::new(TokenType::LineComment, "// test"));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_sample() {
        let input = "function test() {
            // test
            /*
             * testing
             * multiline BlockComment
             */
            return this.foobar.TeSt;
        }";
        let mut tokens = tokenize(input);
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Keyword, "function"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, " "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "test"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, "("));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, ")"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, " "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, "{"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::LineComment, "// test"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::BlockComment, "/*\n             * testing\n             * multiline BlockComment\n             */"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Keyword, "return"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, " "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "this"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, "."));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "foobar"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, "."));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "TeSt"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, ";"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n        "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Blackspace, "}"));
        assert_eq!(tokens.len(), 0);
    }
}
