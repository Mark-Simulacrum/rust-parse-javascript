use std::str;
use std::fmt;
use memchr;

#[derive(Debug, PartialEq, Eq)]
enum TokenizerType {
    Whitespace,
    StringLiteral,
    RegexLiteral,
    TemplateLiteral,
    Blackspace,
    LineComment,
    BlockComment
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    Whitespace,
    Shebang,
    Keyword,
    Identifier,
    NumericLiteral,
    StringLiteral,
    Equal,
    DeIncrement,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXOR,
    BitwiseAnd,
    BitwiseNot,
    Equality,
    Relational,
    BitShift,
    PlusMin,
    Modulo,
    Star,
    Slash,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Dot,
    Comma,
    QuestionMark,
    Colon,
    ExclamationMark,
    RegexLiteral,
    LineComment,
    BlockComment,
    TemplateLiteral
}

impl TokenType {
    fn before_expression(&self) -> bool {
        match *self {
            TokenType::LeftBracket | TokenType::LeftBrace | TokenType::LeftParen |
            TokenType::Comma | TokenType::Semicolon | TokenType::Colon | TokenType::QuestionMark |
            TokenType::Equal =>
                true,
            _ => false
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct Token<'a> {
    ty: TokenType,
    value: &'a str
}

impl<'a> Token<'a> {
    fn new(ty: TokenType, value: &'a str) -> Self {
        Token { ty: ty, value: value }
    }

    fn before_expression(&self) -> bool {
        self.ty.before_expression()
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}Token: {:?}", self.ty, self.value)
    }
}


fn is_id(c: u8) -> bool {
    (c as char).is_alphabetic() ||
    c == b'$' ||
    c == b'_'
}

fn is_keyword(s: &str) -> bool {
    s == "var" ||
    s == "let" ||
    s == "function" ||
    s == "return" ||
    s == "for" ||
    s == "in" ||
    s == "undefined" ||
    s == "break" ||
    s == "case" ||
    s == "continue" ||
    s == "debugger" ||
    s == "default" ||
    s == "do" ||
    s == "if" ||
    s == "finally" ||
    s == "switch" ||
    s == "throw" ||
    s == "try" ||
    s == "const" ||
    s == "while" ||
    s == "with" ||
    s == "new" ||
    s == "this" ||
    s == "super" ||
    s == "class" ||
    s == "extends" ||
    s == "export" ||
    s == "import" ||
    s == "yield" ||
    s == "null" ||
    s == "true" ||
    s == "false" ||
    s == "instanceof" ||
    s == "typeof" ||
    s == "void" ||
    s == "delete"
}

fn is_num(c: u8) -> bool {
    // 100 and 10e10 are both valid numbers
    (c as char).is_numeric() || c == b'e' || c == b'E'
}

#[derive(PartialEq, Eq)]
enum BlackspaceState {
    Unknown,
    Identifier,
    StringLiteral,
    TemplateLiteral,
    RegexLiteral,
    Number
}

fn find_string_literal(bytes: &[u8], start_index: usize) -> usize {
    let mut ignore_next = true;
    let mut end_index = start_index;
    while end_index < bytes.len() {
        if (bytes[end_index] == b'"' || bytes[end_index] == b'\'') && !ignore_next {
            end_index += 1;
            break;
        }
        ignore_next = !ignore_next && bytes[end_index] == b'\\';
        end_index += 1;
    }

    end_index
}

fn find_template_string_literal<'a>(bytes: &[u8], start_index: usize) -> usize {
    let mut ignore_next = true;
    let mut end_index = start_index;
    while end_index < bytes.len() {
        if bytes[end_index] == b'`' && !ignore_next {
            end_index += 1;
            break;
        }
        ignore_next = !ignore_next && bytes[end_index] == b'\\';
        end_index += 1;
    }

    end_index
}

fn find_regex_literal(bytes: &[u8], start_index: usize) -> usize {
    let mut end_index = start_index;

    let mut ignore_next = true;
    while end_index < bytes.len() {
        if bytes[end_index] == b'/' && !ignore_next {
            end_index += 1;
            break;
        }
        ignore_next = !ignore_next && bytes[end_index] == b'\\';
        end_index += 1;
    }

    while end_index < bytes.len() && !(bytes[end_index] as char).is_whitespace() {
        end_index += 1;
    }

    end_index
}

fn previous_non_whitespace_token<'a>(tokens: &'a Vec<Token>) -> Option<&'a Token<'a>> {
    tokens.iter().rev().find(|x| x.ty != TokenType::Whitespace)
}

fn tokenize_blackspace<'a>(input: &'a str, prev_token: Option<&Token>, position: usize) -> Vec<Token<'a>> {
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
            state = BlackspaceState::Identifier;
            while end_index < bytes.len() && is_id(bytes[end_index]) {
                end_index += 1;
            }
        } else if is_num(bytes[start_index]) {
            state = BlackspaceState::Number;
            while end_index < bytes.len() && is_num(bytes[end_index]) {
                end_index += 1;
            }
        } else if bytes[start_index] == b'"' || bytes[start_index] == b'\'' {
            state = BlackspaceState::StringLiteral;

            end_index = find_string_literal(&bytes, end_index);
        } else if bytes[start_index] == b'/' &&
            prev_token.map_or(true, |tok| tok.before_expression()) {
                state = BlackspaceState::RegexLiteral;
                end_index = find_regex_literal(&bytes, end_index);
        } else if bytes[start_index] == b'`' {
            state = BlackspaceState::TemplateLiteral;
            end_index = find_template_string_literal(&bytes, end_index);
            // tokens.append(&mut tokenize_template_string_literal(&bytes[start_index..end_index]));
        } else {
            state = BlackspaceState::Unknown;
        }

        let content = unsafe { str::from_utf8_unchecked(&bytes[start_index..end_index]) };
        match state {
            BlackspaceState::Unknown => {
                let ty = match content {
                    "=" => TokenType::Equal,
                    "==" | "!=" => TokenType::Equality,
                    ";" => TokenType::Semicolon,
                    "++" | "--" => TokenType::DeIncrement,
                    "||" => TokenType::LogicalOr,
                    "&&" => TokenType::LogicalAnd,
                    "|" => TokenType::BitwiseOr,
                    "&" => TokenType::BitwiseAnd,
                    "^" => TokenType::BitwiseXOR,
                    "<" | ">" => TokenType::Relational,
                    "<<" | ">>" => TokenType::BitShift,
                    "+" | "-" => TokenType::PlusMin,
                    "%" => TokenType::Modulo,
                    "*" => TokenType::Star,
                    "/" => TokenType::Slash,
                    "(" => TokenType::LeftParen,
                    ")" => TokenType::RightParen,
                    "{" => TokenType::LeftBrace,
                    "}" => TokenType::RightBrace,
                    "[" => TokenType::LeftBracket,
                    "]" => TokenType::RightBracket,
                    "." => TokenType::Dot,
                    "," => TokenType::Comma,
                    ":" => TokenType::Colon,
                    "?" => TokenType::QuestionMark,
                    "!" => TokenType::ExclamationMark,
                    "~" => TokenType::BitwiseNot,
                    _ => panic!("Unknown Blackspace Token \"{}\" at {},", content, position)
                };

                tokens.push(Token::new(ty, content));
            },
            BlackspaceState::Identifier => {
                tokens.push(Token::new(TokenType::Identifier, content));
            },
            BlackspaceState::Number => {
                tokens.push(Token::new(TokenType::NumericLiteral, content));
            },
            BlackspaceState::StringLiteral => {
                tokens.push(Token::new(TokenType::StringLiteral, content));
            },
            BlackspaceState::RegexLiteral => {
                tokens.push(Token::new(TokenType::RegexLiteral, content));
            },
            BlackspaceState::TemplateLiteral => {
                tokens.push(Token::new(TokenType::TemplateLiteral, content));
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

    if bytes.len() >= 2 && bytes[start_index] == b'#' && bytes[start_index + 1] == b'!' {
        let nearest_newline = memchr::memchr(b'\n', &bytes).unwrap_or(bytes.len());
        let content = unsafe { str::from_utf8_unchecked(&bytes[start_index..nearest_newline]) };
        tokens.push(Token::new(TokenType::Shebang, content));
        start_index += content.len();
    }

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
                            end_index += pos + 1; // Didn't find it here, try again.
                        }
                    } else {
                        end_index = bytes.len() - 1;
                        break;
                    }
                }
            }
        } else if bytes[start_index] == b'"' || bytes[start_index] == b'\'' {
            if state == TokenizerType::Whitespace {
                tokens.push(Token::new(TokenType::Whitespace, ""));
            }

            state = TokenizerType::StringLiteral;

            end_index = find_string_literal(&bytes, end_index);
        } else if bytes[start_index] == b'/' &&
            previous_non_whitespace_token(&tokens).map_or(true, |tok| tok.before_expression()) {
                if state == TokenizerType::Whitespace {
                    tokens.push(Token::new(TokenType::Whitespace, ""));
                }

                state = TokenizerType::RegexLiteral;

                end_index = find_regex_literal(&bytes, end_index);
        } else if bytes[start_index] == b'`' {
            state = TokenizerType::TemplateLiteral;
            end_index = find_template_string_literal(&bytes, end_index);
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
                    let mut to_append = {
                        tokenize_blackspace(content, previous_non_whitespace_token(&tokens), start_index)
                    };
                    tokens.append(&mut to_append);
                }
            },
            TokenizerType::LineComment => {
                tokens.push(Token::new(TokenType::LineComment, content));
            },
            TokenizerType::BlockComment => {
                tokens.push(Token::new(TokenType::BlockComment, content));
            },
            TokenizerType::StringLiteral => {
                tokens.push(Token::new(TokenType::StringLiteral, content));
            },
            TokenizerType::RegexLiteral => {
                tokens.push(Token::new(TokenType::RegexLiteral, content));
            },
            TokenizerType::TemplateLiteral => {
                tokens.push(Token::new(TokenType::TemplateLiteral, content));
            }
        };

        state = if state == TokenizerType::Whitespace {
            TokenizerType::Blackspace
        } else {
            TokenizerType::Whitespace
        };

        start_index = end_index;
    }

    if tokens.len() > 0 && tokens.last().unwrap().ty != TokenType::Whitespace {
        tokens.push(Token::new(TokenType::Whitespace, ""));
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
    // benchmark_tokenize_blackspace!(tokenize_ident_blackspace, "$_very_Z_complex$$ident");
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenType};

    #[test]
    fn tokenize_shebang() {
        let mut tokens = tokenize("#! testing");
        assert_eq!(tokens.remove(0), Token::new(TokenType::Shebang, "#! testing"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_template_literal_with_expression() {
        let mut tokens = tokenize("`test${test}test`");
        assert_eq!(tokens.remove(0), Token::new(TokenType::TemplateLiteral, "`test${test}test`"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_line_comment() {
        let mut tokens = tokenize("// test");
        assert_eq!(tokens.remove(0), Token::new(TokenType::LineComment, "// test"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_line_comment_complex() {
        let mut tokens = tokenize("// CSS escapes http://www.w3.org/TR/CSS21/syndata.html#escaped-characters");
        assert_eq!(tokens.remove(0), Token::new(TokenType::LineComment, "// CSS escapes http://www.w3.org/TR/CSS21/syndata.html#escaped-characters"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_empty_string() {
        let mut tokens = tokenize("\"\"");
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::StringLiteral, "\"\""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_normal_string() {
        let mut tokens = tokenize("\"test foobar\"");
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::StringLiteral, "\"test foobar\""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_normal_regex() {
        let mut tokens = tokenize(r#"/(=)\?(?=&|$) |\?\?/"#);
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::RegexLiteral, r#"/(=)\?(?=&|$) |\?\?/"#));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_modified_regex() {
        let mut tokens = tokenize("/te st/mgi");
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::RegexLiteral, "/te st/mgi"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_non_quote_escape_string() {
        let mut tokens = tokenize("\"\n\"");
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::StringLiteral, "\"\n\""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_quote_escape_string() {
        let mut tokens = tokenize(r#""\"""#);
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::StringLiteral, r#""\"""#));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_blackspace_embedded_string() {
        let mut tokens = tokenize(r#"("auto")"#);
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::LeftParen, "("));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::StringLiteral, r#""auto""#));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::RightParen, ")"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
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
            `test`;
        }";
        let mut tokens = tokenize(input);
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Keyword, "function"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, " "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "test"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::LeftParen, "("));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::RightParen, ")"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, " "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::LeftBrace, "{"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::LineComment, "// test"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::BlockComment, "/*\n             * testing\n             * multiline BlockComment\n             */"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Keyword, "return"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, " "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "this"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Dot, "."));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "foobar"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Dot, "."));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Identifier, "TeSt"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Semicolon, ";"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n            "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::TemplateLiteral, "`test`"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Semicolon, ";"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, "\n        "));
        assert_eq!(tokens.remove(0), Token::new(TokenType::RightBrace, "}"));
        assert_eq!(tokens.remove(0), Token::new(TokenType::Whitespace, ""));
        assert_eq!(tokens.len(), 0);
    }
}
