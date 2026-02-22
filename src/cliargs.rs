use clap::Parser;

/// A progam to generate random valid source code, based on a Context-Free-Grammar
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the grammarfile
    pub grammar: String,

    /// Starting rule
    #[arg(short, long)]
    pub start: String,

    /// Where to output the generated source code
    #[arg(short = 'o')]
    pub outfile: Option<String>,

    /// Specify a term
    #[arg(short = 'T')]
    pub terms: Vec<String>,
}
