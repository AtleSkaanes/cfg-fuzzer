use rand::{RngExt, seq::IndexedRandom};
use thiserror::Error;

use crate::cfg::{Cfg, CfgLetter};

// TODO: Remove GenerateError, as Terms should also be parsed properly in the parser
#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Unknown term '{0}'. You can define it with -T=\"{0}:RULE\"")]
    UnknownTerm(Box<str>),
}

pub fn generate_code(
    cfg: Cfg,
    rng: &mut dyn rand::Rng,
) -> Result<Box<str>, GenerateError> {
    let mut code = String::new();

    for r in cfg.rule_slice(&cfg.top_level) {
        generate_from_letter(&cfg, r, &mut code, rng)?;
    }

    Ok(code.into_boxed_str())
}

fn generate_from_letter(
    cfg: &Cfg,
    ltr: &CfgLetter,
    out: &mut String,
    rng: &mut dyn rand::Rng,
) -> Result<(), GenerateError> {
    match ltr {
        CfgLetter::Rule(rule) => {
            for r in cfg.rule_slice(rule) {
                generate_from_letter(cfg, r, out, rng)?;
            }
        }
        CfgLetter::StrLit(str) => {
            out.push(' ');
            out.push_str(str);
        }
        CfgLetter::Or(items) => {
            let Some(rule) = items.choose(rng) else {
                return Ok(());
            };

            for r in cfg.rule_slice(rule) {
                generate_from_letter(cfg, r, out, rng)?;
            }
        }
        CfgLetter::Optional(id) => {
            if rng.random_bool(0.50) {
                generate_from_letter(cfg, cfg.get_letter(*id), out, rng)?;
            }
        }
        CfgLetter::Many(id) => {
            while rng.random_bool(0.50) {
                generate_from_letter(cfg, cfg.get_letter(*id), out, rng)?;
            }
        }
        CfgLetter::OneOrMore(id) => {
            generate_from_letter(cfg, cfg.get_letter(*id), out, rng)?;

            while rng.random_bool(0.50) {
                generate_from_letter(cfg, cfg.get_letter(*id), out, rng)?;
            }
        }
        CfgLetter::Group(group) => {
            for ltr in cfg.rule_slice(group) {
                generate_from_letter(cfg, ltr, out, rng)?;
            }
        }
        CfgLetter::Range(ranges) => {
            let mut choices = vec![];
            for range in ranges {
                match range.to {
                    Some(to) => {
                        for ch in range.from..=to {
                            choices.push(ch)
                        }
                    }
                    None => choices.push(range.from),
                }
            }
            let ch = choices.choose(rng).unwrap_or(&' ');
            out.push(*ch);
        }
        CfgLetter::Term(term) => {
            if let Some(term_rule) = cfg.terms.get(term) {
                for r in cfg.rule_slice(term_rule) {
                    generate_from_letter(cfg, r, out, rng)?;
                }
                return Ok(());
            }

            let str: &str = &term.clone().into_string();
            out.push(' ');
            match str {
                "IDENT" => {
                    let idents = [
                        "foo", "bar", "baz", "foobar", "x", "y", "a", "b", "node", "item", "elem",
                        "i", "index", "str", "out", "buf", "ptr", "get", "set",
                    ];
                    out.push_str(idents.choose(rng).unwrap_or(&"foo"));
                    while rng.random_bool(0.20) {
                        out.push('_');
                        out.push_str(idents.choose(rng).unwrap_or(&"foo"))
                    }
                }
                "CAPS_IDENT" => {
                    let idents = ["FOO", "BAR", "BAZ", "FOOBAR", "VAR", "X", "Y", "A", "B"];
                    out.push_str(idents.choose(rng).unwrap_or(&"FOO"));
                    while rng.random_bool(0.20) {
                        out.push('_');
                        out.push_str(idents.choose(rng).unwrap_or(&"FOO"))
                    }
                }
                "TYPE_IDENT" => {
                    let idents = [
                        "Foo", "Bar", "Baz", "Foobar", "Node", "List", "X", "Y", "A", "B",
                    ];
                    out.push_str(idents.choose(rng).unwrap_or(&"foo"));
                    while rng.random_bool(0.20) {
                        out.push_str(idents.choose(rng).unwrap_or(&"foo"))
                    }
                }
                "NUMBER" => {
                    let number: f64 = rng.random();
                    let rounded = (number * 100.0).round() / 100.0;
                    out.push_str(&rounded.to_string())
                }
                "INT" => {
                    let int: u8 = rng.random();
                    out.push_str(&int.to_string())
                }
                "NEWLINE" | "NL" => out.push('\n'),
                "TAB" => out.push('\t'),
                _ => return Err(GenerateError::UnknownTerm(term.clone())),
            }
        }
    }
    Ok(())
}
