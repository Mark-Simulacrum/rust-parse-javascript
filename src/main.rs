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
    Blackspace(String)
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while !chars.is_empty() {
        let first_char: char = chars.next().unwrap();
        let is_whitespace = first_char.is_whitespace();
        let mut current_chars: String = first_char.to_string();

        while !chars.is_empty() {
            if is_whitespace != chars.peek().unwrap().is_whitespace() {
                if is_whitespace {
                    tokens.push(Token::Whitespace(current_chars));
                } else {
                    tokens.push(Token::Blackspace(current_chars));
                }
                break;
            }
            current_chars.push_str(&chars.next().unwrap().to_string());
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
