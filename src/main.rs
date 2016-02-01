#![feature(test)]

extern crate test;

use std::env;
use std::fs::File;
use std::io::Read;

fn get_file_content(arg: String) -> std::io::Result<String> {
    let mut content = String::new();
    let mut file = try!(File::open(arg));
    try!(file.read_to_string(&mut content));
    Ok(content)
}

mod tokenizer;
use tokenizer::tokenize;

fn main() {
    for argument in env::args().skip(1) {
        let content = &get_file_content(argument).unwrap();
        println!("Started tokenization!");
        let tokens = tokenize(content);
        println!("Tokenized!");
        // for token in &tokens {
        //     println!("{:?}", token);
        // }
        println!("{:#?}", tokens.len());
    }
}
