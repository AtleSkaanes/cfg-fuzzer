use std::{fs::OpenOptions, io::Read};

use cfg_fuzzer::{cliargs, generate::generate_code, parser::parse};
use clap::Parser;

fn main() -> std::io::Result<()> {
    let args = cliargs::Args::parse();

    let src = open_file(args.grammar.clone())?;

    // eprintln!("TOKENS:");
    // let lex = Lexer::new(&src, &args.grammar);
    // for (i, tok) in lex.enumerate() {
    //     eprintln!("\t[{}]: {:?}", i, tok);
    // }

    let ast = match parse(&src, &args.grammar) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("[ERROR]: {}", e);
            std::process::exit(1);
        }
    };

    // eprintln!("\nAST:\n{:?}", ast);
    //
    let generated = generate_code(ast, &args.start, &mut rand::rng());
    // eprintln!("\n\nCODE:\n");
    println!("{}", generated);

    Ok(())
}

fn open_file(path: String) -> std::io::Result<String> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut str = String::new();
    let _ = file.read_to_string(&mut str);
    Ok(str)
}
