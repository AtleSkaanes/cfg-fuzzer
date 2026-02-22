use rand::{RngExt, seq::IndexedRandom};

use crate::cfg::{Cfg, CfgLetter};

pub fn generate_code(cfg: Cfg, start: &str, rng: &mut dyn rand::Rng) -> Box<str> {
    let mut code = String::new();

    let Some(rule) = cfg.rules.get(start) else {
        panic!("UNKNOWN RULE: '{}'", start)
    };

    for r in rule {
        generate_from_letter(&cfg, r, &mut code, rng);
    }

    code.into_boxed_str()
}

fn generate_from_letter(cfg: &Cfg, ltr: &CfgLetter, out: &mut String, rng: &mut dyn rand::Rng) {
    match ltr {
        CfgLetter::Rule(ident) => {
            let Some(rule) = cfg.rules.get(ident) else {
                panic!("UNKNOWN RULE: '{}'", ident)
            };
            for r in rule {
                generate_from_letter(cfg, r, out, rng);
            }
        }
        CfgLetter::StrLit(str) => {
            out.push(' ');
            out.push_str(str);
        }
        CfgLetter::Or(items) => {
            let Some(rule) = items.choose(rng) else {
                return;
            };

            for r in rule {
                generate_from_letter(cfg, r, out, rng);
            }
        }
        CfgLetter::Optional(cfg_letter) => {
            if rng.random_bool(0.50) {
                generate_from_letter(cfg, cfg_letter, out, rng);
            }
        }
        CfgLetter::Many(cfg_letter) => {
            // let bools: Box<[bool]> = rng.random_iter().take(10).collect();
            // for b in bools {
            //     if b {
            //         generate_from_letter(cfg, cfg_letter, out, rng);
            //     }
            // }
            while rng.random_bool(0.50) {
                generate_from_letter(cfg, cfg_letter, out, rng);
            }
        }
        CfgLetter::OneOrMore(cfg_letter) => {
            // let mut bools: Box<[bool]> = rng.random_iter().take(10).collect();
            // bools[0] = true; // guarentee atleast one
            // for b in bools {
            //     if b {
            //         generate_from_letter(cfg, cfg_letter, out, rng);
            //     }
            // }

            generate_from_letter(cfg, cfg_letter, out, rng);

            while rng.random_bool(0.50) {
                generate_from_letter(cfg, cfg_letter, out, rng);
            }
        }
        CfgLetter::Group(letters) => {
            for ltr in letters {
                generate_from_letter(cfg, ltr, out, rng);
            }
        }
        CfgLetter::Term(term) => {
            let str: &str = &term.clone().into_string();
            out.push(' ');
            match str {
                "IDENT" => {
                    let idents = [
                        "foo", "bar", "baz", "foobar", "var", "x", "y", "a", "b", "node", "item",
                        "elem", "i", "index", "str", "out", "buf", "ptr",
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
                _ => panic!("UNKNOWN TERM: '{}'", str),
            }
        }
    }
}
