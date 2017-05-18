extern crate mips_assembler;
extern crate clap;

use clap::{Arg, App};
use std::fs::File;
use std::io::Read;
use mips_assembler::Assembler::Parser;

fn main() {
    let matches = App::new("Mips Assembler")
        .version("0.1")
        .about("by LYP.")
        .arg(Arg::with_name("file")
                 .short("f")
                 .long("file")
                 .value_name("FILE")
                 .takes_value(true))
        .get_matches();

    if let Some(file) = matches.value_of("file") {
        let mut parser = Parser::new();
        if let Ok(mut file) = File::open(file) {
            let mut buf = String::new();
            if let Err(e) = file.read_to_string(&mut buf) {
                println!("{:?}", e);
                return;
            }
            match parser.AsmStr(&buf) {
                Ok(result) => {
                    for instr in result {
                        print!("{:X} ", instr);
                    }
                    println!("");
                }
                Err(e) => println!("{:?}", e),
            }
        } else {
            println!("Couldn't open file {}", file);
        }
    } else {
        println!("File argument required.");
    }
}
