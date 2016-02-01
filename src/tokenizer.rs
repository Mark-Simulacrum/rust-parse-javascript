use std::str;
use std::fmt;

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

    if is_keyword(input) {
        tokens.push(Token::new(TokenType::Keyword, input));
    } else {
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
    }

    tokens
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();

    let mut line_comment_starts: Vec<usize> = input.match_indices("//").map(|(i, _)| i).collect();
    let mut block_comment_starts: Vec<usize> = input.match_indices("/*").map(|(i, _)| i).collect();

    let mut start_index = 0;
    let mut state = TokenizerType::Whitespace;
    while start_index < bytes.len() {
        let mut end_index = start_index;

        if bytes[start_index] == b'/' {
            if line_comment_starts.len() > 0 && line_comment_starts[0] == start_index {
                line_comment_starts.remove(0);
                state = TokenizerType::LineComment;
            } else if block_comment_starts.len() > 0 && block_comment_starts[0] == start_index {
                block_comment_starts.remove(0);
                state = TokenizerType::BlockComment;
            }
        }

        while end_index < bytes.len() {
            if state == TokenizerType::LineComment {
                if bytes[end_index] == b'\n' {
                    break;
                }
            } else if state == TokenizerType::BlockComment {
                if end_index + 1 < bytes.len() && bytes[end_index] == b'*' && bytes[end_index + 1] == b'/' {
                    end_index += 2; // We want to include the slash
                    break;
                }
            } else {
                let is_whitespace = (bytes[end_index] as char).is_whitespace();

                if (state == TokenizerType::Whitespace) != is_whitespace {
                    break;
                }
            }

            end_index += 1;
        }

        assert!(start_index < bytes.len(), "Start index is within range.");
        assert!(end_index <= bytes.len(), "End index is within range.");

        let content = unsafe { str::from_utf8_unchecked(&bytes[start_index..end_index]) };
        start_index = end_index;
        match state {
            TokenizerType::Whitespace => {
                tokens.push(Token::new(TokenType::Whitespace, content));
            },
            TokenizerType::Blackspace => {
                tokens.append(&mut tokenize_blackspace(content));
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
    }

    tokens
}

#[cfg(test)]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn tokenize_function(b: &mut Bencher) {
        b.iter(|| tokenize("function () {}"));
    }

    #[bench]
    fn tokenize_ident(b: &mut Bencher) {
        b.iter(|| tokenize("$_very_Z_complex$$ident"));
    }

    #[bench]
    fn tokenize_blackspace_ident(b: &mut Bencher) {
        b.iter(|| super::tokenize_blackspace("$_very_Z_complex$$ident"));
    }

    #[bench]
    fn tokenize_comment_line(b: &mut Bencher) {
        b.iter(|| tokenize("// testing"));
    }

    #[bench]
    fn tokenize_comment_block(b: &mut Bencher) {
        b.iter(|| tokenize("/* testi*/")); // This is the same length as the line comment
    }

    #[bench]
    fn tokenize_sample(b: &mut Bencher) {
        let input = "function test() {
        	// test
        	/*
        	 * testing
        	 * multiline BlockComment
        	 */
        	return this.foobar.TeSt;
        }";
        b.iter(|| tokenize(input));
    }
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenType};

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
    }
}
