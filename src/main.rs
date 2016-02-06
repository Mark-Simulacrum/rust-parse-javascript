#![feature(test)]
#![feature(plugin)]

#![plugin(clippy)]

extern crate test;
extern crate memchr;

use std::env;
use std::fs::File;
use std::io::Read;

fn get_file_content(arg: &str) -> std::io::Result<String> {
    let mut content = String::new();
    let mut file = try!(File::open(arg));
    try!(file.read_to_string(&mut content));
    Ok(content)
}

mod tokenizer;
use tokenizer::tokenize;

fn main() {
    for argument in env::args().skip(1) {
        let content = &get_file_content(&argument).unwrap_or(argument);
        let tokens = tokenize(content);
        if tokens.len() < 20 {
            for token in tokens {
                println!("{:?}", token);
            }
        } else {
            println!("Token amount: {:#?}", tokens.len());
        }
    }
}
