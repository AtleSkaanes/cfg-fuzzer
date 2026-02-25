use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use cfg_fuzzer::{cliargs, generate::generate_code, parser::parse};
use clap::Parser;

const RED: &str = "\x1b[31m";
const CLEAR: &str = "\x1b[0m";

fn main() -> std::io::Result<()> {
    let args = cliargs::Args::parse();

    let src = match open_file(&args.grammar) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}err{}: While opening '{}': {}",
                RED, CLEAR, args.grammar, e
            );
            std::process::exit(1);
        }
    };

    let ast = match parse(&src, &args.grammar, &args.terms.into_boxed_slice()) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{}err{}: {}", RED, CLEAR, e);
            std::process::exit(1);
        }
    };

    let generated = match generate_code(ast, &args.start, &mut rand::rng()) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}io err{}: {}", RED, CLEAR, e);
            std::process::exit(1);
        }
    };

    if let Some(out) = args.outfile {
        if let Err(e) = write_file(&out, &generated) {
            eprintln!("{}err{}: While writing to '{}': {}", RED, CLEAR, out, e);
            std::process::exit(1);
        }
    } else {
        println!("{}", generated);
    }

    Ok(())
}

fn open_file(path: &str) -> std::io::Result<Box<str>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut str = String::new();
    let _ = file.read_to_string(&mut str);
    Ok(str.into())
}

fn write_file(path: &str, contents: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    _ = file.write(contents.as_bytes())?;

    Ok(())
}
