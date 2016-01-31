#![feature(type_ascription)]
#![feature(peekable_is_empty)]

use std::env;
use std::fs::File;
use std::io::Read;

fn get_file_content(arg: String) -> std::io::Result<String> {
    let mut content = String::new();
    let mut file = try!(File::open(arg));
    try!(file.read_to_string(&mut content));
    Ok(content)
}

#[derive(Debug)]
enum Token {
    Whitespace(String),
    Blackspace(String),
    LineComment(String),
    BlockComment(String)
}

macro_rules! push_token {
    ($tokens: ident, $tokenType: path, $content: ident) => (
        $tokens.push($tokenType($content.into_iter().collect()));
    )
}

macro_rules! iterate_until {
    ($chars: ident, $current_chars: ident, $until: expr, $execute: stmt) => (
        {
            let mut previous = ' ';
            while !$chars.is_empty() {
                let current = *$chars.peek().unwrap();
                if $until(previous, current) {
                    $execute;
                } else {
                    $chars.next();
                    $current_chars.push(current);
                }
                previous = current;
            }
        }
    )
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    'chariter: while !chars.is_empty() {
        let first_char: char = chars.next().unwrap();
        let is_whitespace = first_char.is_whitespace();
        let mut current_chars: Vec<char> = Vec::new();
        current_chars.push(first_char);

        if first_char == '/' {
            let next_char = chars.next().unwrap();

            current_chars.push(next_char);
            if next_char == '/' {
                iterate_until!(chars, current_chars, |_, c| c == '\n', {
                    push_token!(tokens, Token::LineComment, current_chars);
                    continue 'chariter;
                });
            } else if next_char == '*' {
                iterate_until!(chars, current_chars, |p, c| p == '*' && c == '/', {
                    push_token!(tokens, Token::BlockComment, current_chars);
                    continue 'chariter;
                });
            }
        }

        while !chars.is_empty() {
            if is_whitespace != chars.peek().unwrap().is_whitespace() {
                if is_whitespace {
                    push_token!(tokens, Token::Whitespace, current_chars);
                } else {
                    push_token!(tokens, Token::Blackspace, current_chars);
                }
                continue 'chariter;
            }
            current_chars.push(chars.next().unwrap());
        }
    }

    tokens
}

fn main() {
    for argument in env::args().skip(1) {
        let content = &get_file_content(argument).unwrap();
        println!("{:?}", content);
        println!("{:?}", tokenize(content));
    }
}
