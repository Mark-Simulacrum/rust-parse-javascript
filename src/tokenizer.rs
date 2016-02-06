use std::str;
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
pub enum Token<'a> {
    Whitespace(&'a str),
    Shebang(&'a str),
    Keyword(&'a str),
    Identifier(&'a str),
    NumericLiteral(&'a str),
    StringLiteral(&'a str),
    DeIncrement(&'a str),
    RegexLiteral(&'a str),
    Equality(&'a str),
    BitShift(&'a str),
    Relational(char),
    PlusMin(char),
    LineComment(&'a str),
    BlockComment(&'a str),
    TemplateLiteral(&'a str),
    Equal,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXOR,
    BitwiseAnd,
    BitwiseNot,
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
    ExclamationMark
}

impl<'a> Token<'a> {
    fn before_expression(&self) -> bool {
        match *self {
            Token::LeftBracket | Token::LeftBrace | Token::LeftParen |
            Token::Comma | Token::Semicolon | Token::Colon | Token::QuestionMark |
            Token::Equal | Token::ExclamationMark =>
                true,
            _ => false
        }
    }

    fn is_whitespace(&self) -> bool {
        match *self {
            Token::Whitespace(_) => true,
            _ => false
        }
    }
}

fn is_id(c: u8) -> bool {
    (c as char).is_alphabetic() ||
    c == b'$' ||
    c == b'_'
}

fn is_keyword(s: &str) -> bool {
    match s {
        "var" |
        "let" |
        "function" |
        "return" |
        "for" |
        "undefined" |
        "in" |
        "break" |
        "case" |
        "continue" |
        "debugger" |
        "default" |
        "do" |
        "if" |
        "finally" |
        "switch" |
        "throw" |
        "try" |
        "const" |
        "while" |
        "with" |
        "new" |
        "this" |
        "super" |
        "class" |
        "extends" |
        "export" |
        "import" |
        "yield" |
        "null" |
        "true" |
        "false" |
        "instanceof" |
        "typeof" |
        "void" |
        "delete" => true,
        _ => false
    }
}

fn is_num(c: u8) -> bool {
    // 100 and 10e10 are both valid numbers
    (c as char).is_numeric() || c == b'e' || c == b'E'
}

fn find_string_literal(bytes: &[u8], start_index: usize, quote_type: u8) -> usize {
    let mut ignore_next = true;
    let mut end_index = start_index;
    while end_index < bytes.len() {
        if bytes[end_index] == quote_type && !ignore_next {
            end_index += 1;
            break;
        }
        ignore_next = !ignore_next && bytes[end_index] == b'\\';
        end_index += 1;
    }

    end_index
}

fn find_template_string_literal(bytes: &[u8], start_index: usize) -> usize {
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

fn tokenize_byte<'a>(input: u8, position: usize) -> Token<'a> {
    match input {
        b'.' => Token::Dot,
        b'(' => Token::LeftParen,
        b')' => Token::RightParen,
        b'{' => Token::LeftBrace,
        b'}' => Token::RightBrace,
        b'[' => Token::LeftBracket,
        b']' => Token::RightBracket,
        b';' => Token::Semicolon,
        b'<' | b'>' => Token::Relational(input as char),
        b'+' | b'-' => Token::PlusMin(input as char),
        b'=' => Token::Equal,
        b'*' => Token::Star,
        b'%' => Token::Modulo,
        b'/' => Token::Slash,
        b',' => Token::Comma,
        b':' => Token::Colon,
        b'?' => Token::QuestionMark,
        b'!' => Token::ExclamationMark,
        b'~' => Token::BitwiseNot,
        b'&' => Token::BitwiseAnd,
        b'|' => Token::BitwiseOr,
        b'^' => Token::BitwiseXOR,
        _ => panic!("Unknown Blackspace Token \"{}\" at {}",
            input as char, position)
    }
}

fn as_str(bytes: &[u8]) -> &str {
    unsafe { str::from_utf8_unchecked(bytes) }
}

fn tokenize_blackspace(input: &str, position: usize, is_possible_expression: bool) -> Vec<Token> {
    let mut tokens = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();

    let mut start_index = 0;
    while start_index < bytes.len() {
        if start_index != 0 {
            tokens.push(Token::Whitespace(""));
        }

        let mut end_index = start_index + 1;
        if is_id(bytes[start_index]) {
            while end_index < bytes.len() && is_id(bytes[end_index]) {
                end_index += 1;
            }

            tokens.push(Token::Identifier(as_str(&bytes[start_index..end_index])));
        } else if is_num(bytes[start_index]) {
            while end_index < bytes.len() && is_num(bytes[end_index]) {
                end_index += 1;
            }

            tokens.push(Token::NumericLiteral(as_str(&bytes[start_index..end_index])));
        } else if bytes[start_index] == b'"' || bytes[start_index] == b'\'' {
            end_index = find_string_literal(&bytes, end_index, bytes[start_index]);

            tokens.push(Token::StringLiteral(as_str(&bytes[start_index..end_index])));
        } else if bytes[start_index] == b'/' && is_possible_expression {
            end_index = find_regex_literal(&bytes, end_index);

            tokens.push(Token::RegexLiteral(as_str(&bytes[start_index..end_index])));
        } else if bytes[start_index] == b'`' {
            end_index = find_template_string_literal(&bytes, end_index);

            tokens.push(Token::TemplateLiteral(as_str(&bytes[start_index..end_index])));
        } else {
            if end_index < bytes.len() {
                let curr = bytes[start_index];
                let next = bytes[end_index];

                let token = match (curr, next) {
                    (b'=', b'=') => Token::Equality("=="),
                    (b'!', b'=') => Token::Equality("!="),
                    (b'+', b'+') => Token::DeIncrement("++"),
                    (b'-', b'-') => Token::DeIncrement("--"),
                    (b'<', b'<') => Token::BitShift("<<"),
                    (b'>', b'>') => Token::BitShift(">>"),
                    (b'|', b'|') => Token::LogicalOr,
                    (b'&', b'&') => Token::LogicalAnd,
                    _ => {
                        end_index = start_index;
                        tokenize_byte(curr, position)
                    }
                };

                tokens.push(token);
            } else {
                end_index = start_index;
                tokens.push(tokenize_byte(bytes[start_index], position));
            }

            end_index += 1;
        }

        start_index = end_index;
    }

    tokens
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let bytes = input.as_bytes();

    let mut start_index = 0;

    if bytes.len() >= 2 && bytes[start_index] == b'#' && bytes[start_index + 1] == b'!' {
        let nearest_newline = memchr::memchr(b'\n', &bytes).unwrap_or(bytes.len());
        let content = as_str(&bytes[start_index..nearest_newline]);
        tokens.push(Token::Shebang(content));
        start_index += content.len();
    }

    let mut state = TokenizerType::Whitespace;
    let mut last_broke_at_index = start_index;
    let mut is_possible_expression = true;
    while start_index < bytes.len() {
        let mut end_index = start_index;

        if start_index + 1 < bytes.len() && bytes[start_index] == b'/' && bytes[start_index + 1] == b'/' {
            state = TokenizerType::LineComment;

            end_index += memchr::memchr(b'\n', &bytes[end_index..]).unwrap_or(bytes.len() - end_index);
        } else if start_index + 1 < bytes.len() && bytes[start_index] == b'/' && bytes[start_index + 1] == b'*' {
            end_index += 1;

            state = TokenizerType::BlockComment;
            loop {
                if let Some(pos) = memchr::memchr(b'/', &bytes[end_index..]) {
                    let star_pos = end_index + pos - 1; // Right before the found slash

                    end_index += pos + 1; // Increment to the slash
                    if bytes[star_pos] == b'*' {
                        break;
                    }
                } else {
                    end_index = bytes.len() - 1;
                    break;
                }
            }
        } else if bytes[start_index] == b'"' || bytes[start_index] == b'\'' {
            if state == TokenizerType::Whitespace {
                tokens.push(Token::Whitespace(""));
            }

            state = TokenizerType::StringLiteral;

            end_index = find_string_literal(&bytes, end_index, bytes[start_index]);
        } else if bytes[start_index] == b'/' && is_possible_expression {
            if state == TokenizerType::Whitespace {
                tokens.push(Token::Whitespace(""));
            }

            state = TokenizerType::RegexLiteral;

            end_index = find_regex_literal(&bytes, end_index);
        } else if bytes[start_index] == b'`' {
            if state == TokenizerType::Whitespace {
                tokens.push(Token::Whitespace(""));
            }

            state = TokenizerType::TemplateLiteral;
            end_index = find_template_string_literal(&bytes, end_index);
        } else {
            while end_index < bytes.len() {
                let b = bytes[end_index];
                if last_broke_at_index != end_index && (b == b'/' || b == b'"' || b == b'\'' || b == b'`') {
                    last_broke_at_index = end_index;

                    break;
                }

                let is_whitespace = (b as char).is_whitespace();

                if (state == TokenizerType::Whitespace) != is_whitespace {
                    break;
                }

                end_index += 1;
            }
        }

        assert!(start_index < bytes.len(), "Start index is within range.");
        assert!(end_index <= bytes.len(), "End index is within range.");

        let content = as_str(&bytes[start_index..end_index]);
        match state {
            TokenizerType::Whitespace => {
                tokens.push(Token::Whitespace(content));
            },
            TokenizerType::Blackspace => {
                if is_keyword(content) {
                    tokens.push(Token::Keyword(content));
                } else {
                    let mut to_append = {
                        tokenize_blackspace(content, start_index, is_possible_expression)
                    };
                    tokens.append(&mut to_append);
                }
            },
            TokenizerType::LineComment => {
                tokens.push(Token::LineComment(content));
            },
            TokenizerType::BlockComment => {
                tokens.push(Token::BlockComment(content));
            },
            TokenizerType::StringLiteral => {
                tokens.push(Token::StringLiteral(content));
            },
            TokenizerType::RegexLiteral => {
                tokens.push(Token::RegexLiteral(content));
            },
            TokenizerType::TemplateLiteral => {
                tokens.push(Token::TemplateLiteral(content));
            }
        };

        state = if state == TokenizerType::Whitespace {
            TokenizerType::Blackspace
        } else {
            if let Some(token) = tokens.last() {
                is_possible_expression = token.before_expression();
            }
            TokenizerType::Whitespace
        };

        start_index = end_index;
    }

    if !tokens.is_empty() && !tokens.last().unwrap().is_whitespace() {
        tokens.push(Token::Whitespace(""));
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
            _benchmark!($name, super::tokenize_blackspace($toRun, 0, true));
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

        benchmark_tokenize!(shebang, "#! testing");
        benchmark_tokenize!(template_literal, "`test${test}test`");
        benchmark_tokenize!(regex, r#"/(=)\?(?=&|$) |\?\?/"#);
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
    use super::{tokenize, Token};

    #[test]
    fn tokenize_shebang() {
        let mut tokens = tokenize("#! testing");
        assert_eq!(tokens.remove(0), Token::Shebang("#! testing"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_template_literal_with_expression() {
        let mut tokens = tokenize("`test${test}test`");
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::TemplateLiteral("`test${test}test`"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_line_comment() {
        let mut tokens = tokenize("// test");
        assert_eq!(tokens.remove(0), Token::LineComment("// test"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_line_comment_complex() {
        let mut tokens = tokenize("// CSS escapes http://www.w3.org/TR/CSS21/syndata.html#escaped-characters");
        assert_eq!(tokens.remove(0), Token::LineComment("// CSS escapes http://www.w3.org/TR/CSS21/syndata.html#escaped-characters"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_empty_string() {
        let mut tokens = tokenize("\"\"");
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::StringLiteral("\"\""));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_normal_string() {
        let mut tokens = tokenize("\"test foobar\"");
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::StringLiteral("\"test foobar\""));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_normal_regex() {
        let mut tokens = tokenize(r#"/(=)\?(?=&|$) |\?\?/"#);
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::RegexLiteral(r#"/(=)\?(?=&|$) |\?\?/"#));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_modified_regex() {
        let mut tokens = tokenize("/te st/mgi");
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::RegexLiteral("/te st/mgi"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_non_quote_escape_string() {
        let mut tokens = tokenize("\"\n\"");
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::StringLiteral("\"\n\""));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_quote_escape_string() {
        let mut tokens = tokenize(r#""\"""#);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::StringLiteral(r#""\"""#));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_blackspace_embedded_string() {
        let mut tokens = tokenize(r#"("auto")"#);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::LeftParen);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::StringLiteral(r#""auto""#));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::RightParen);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_operators() {
        let mut tokens = tokenize("a == b; !a;");
        println!("tokens = {:?}", tokens);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Identifier("a"));
        assert_eq!(tokens.remove(0), Token::Whitespace(" "));
        assert_eq!(tokens.remove(0), Token::Equality("=="));
        assert_eq!(tokens.remove(0), Token::Whitespace(" "));
        assert_eq!(tokens.remove(0), Token::Identifier("b"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Semicolon);
        assert_eq!(tokens.remove(0), Token::Whitespace(" "));
        assert_eq!(tokens.remove(0), Token::ExclamationMark);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Identifier("a"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Semicolon);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_block_comment() {
        let mut tokens = tokenize("/* test * * * */");
        println!("{:?}", tokens);
        assert_eq!(tokens.remove(0), Token::BlockComment("/* test * * * */"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
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
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Keyword("function"));
        assert_eq!(tokens.remove(0), Token::Whitespace(" "));
        assert_eq!(tokens.remove(0), Token::Identifier("test"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::LeftParen);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::RightParen);
        assert_eq!(tokens.remove(0), Token::Whitespace(" "));
        assert_eq!(tokens.remove(0), Token::LeftBrace);
        assert_eq!(tokens.remove(0), Token::Whitespace("\n            "));
        assert_eq!(tokens.remove(0), Token::LineComment("// test"));
        assert_eq!(tokens.remove(0), Token::Whitespace("\n            "));
        assert_eq!(tokens.remove(0), Token::BlockComment("/*\n             * testing\n             * multiline BlockComment\n             */"));
        assert_eq!(tokens.remove(0), Token::Whitespace("\n            "));
        assert_eq!(tokens.remove(0), Token::Keyword("return"));
        assert_eq!(tokens.remove(0), Token::Whitespace(" "));
        assert_eq!(tokens.remove(0), Token::Identifier("this"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Dot);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Identifier("foobar"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Dot);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Identifier("TeSt"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Semicolon);
        assert_eq!(tokens.remove(0), Token::Whitespace("\n            "));
        assert_eq!(tokens.remove(0), Token::TemplateLiteral("`test`"));
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.remove(0), Token::Semicolon);
        assert_eq!(tokens.remove(0), Token::Whitespace("\n        "));
        assert_eq!(tokens.remove(0), Token::RightBrace);
        assert_eq!(tokens.remove(0), Token::Whitespace(""));
        assert_eq!(tokens.len(), 0);
    }
}
