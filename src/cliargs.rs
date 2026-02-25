use clap::Parser;

/// A progam to generate random valid source code, based on a Context-Free-Grammar
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the grammarfile
    pub grammar: Box<str>,

    /// Starting rule
    #[arg(short, long)]
    pub start: Box<str>,

    /// Where to output the generated source code
    #[arg(short = 'o')]
    pub outfile: Option<Box<str>>,

    /// Specify a term
    #[arg(short = 'T')]
    pub terms: Vec<Box<str>>,
}
