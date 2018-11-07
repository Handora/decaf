extern crate parser;
extern crate util;

use std::io;
use std::mem;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn string_to_static_str(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

fn main() {
    let mut input = String::new();
    {
        let filename = env::args().nth(1).unwrap_or_else(|| {
            eprintln!("Please specify input filename");
            std::process::exit(1);
        });
        let mut f = File::open(filename).unwrap();
        f.read_to_string(&mut input).unwrap();
    }
    let input = string_to_static_str(input);

    let mut parser = parser::Parser::new();

    let _ = parser.parse(input)
        .map_err(|errors| { for error in errors { println!("{}", error); } })
        .map(|program| {
            let mut printer = util::IndentPrinter::new();
            program.print_to(&mut printer);
            printer.flush(&mut io::stdout());
        });
}
