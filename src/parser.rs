use std::collections::HashMap;

use thiserror::Error;

use crate::{
    cfg::{Cfg, CfgLetter, CfgRule},
    lexer::{Lexer, LexerCtx, Operator, Token},
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0} -> Expected {1} but got {2:?}")]
    UnexpectedToken(LexerCtx, Box<str>, Token),
    #[error("{0} -> Got unexpected operator {1:?}")]
    UnexpectedOp(LexerCtx, Operator),
    #[error("{0} -> Got unexpected EOF")]
    GotEof(LexerCtx),
}

pub fn parse(src: &str, filename: &str) -> Result<Cfg, ParseError> {
    let mut lex = Lexer::new(src, filename);

    let mut map = HashMap::new();

    while let Some(tok) = lex.next() {
        let ident = match tok {
            Token::Ident(ref ident) => ident,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    lex.ctx,
                    "identifier".into(),
                    tok,
                ));
            }
        };

        let Some(tok) = lex.next() else {
            return Err(ParseError::GotEof(lex.ctx.clone()));
        };

        if !matches!(tok, Token::Op(Operator::RuleDeclare)) {
            return Err(ParseError::UnexpectedToken(lex.ctx, "':'".into(), tok));
        }

        if matches!(lex.peek_next(), Some(Token::Op(Operator::Pipe))) {
            let _ = lex.next();
        }

        let rules = parse_rule(&mut lex)?;
        map.insert(ident.clone(), rules.unwrap());
    }

    Ok(Cfg { rules: map })
}

fn parse_rule(lex: &mut Lexer) -> Result<Option<CfgRule>, ParseError> {
    let mut letters = vec![];
    let mut ors = vec![];
    loop {
        // Unwrap because we know its not None due to last match
        if matches!(lex.peek_next(), Some(Token::Op(Operator::Eol))) {
            _ = lex.next();
            break;
        }
        match parse_letter(lex)? {
            Some(CfgLetter::Or(or)) => {
                letters.push(or[0][0].clone());
                ors.push(std::mem::take(&mut letters).into());
            }
            Some(ltr) => letters.push(ltr),
            None => break,
        }
    }

    if !ors.is_empty() {
        ors.push(std::mem::take(&mut letters).into());
        letters.push(CfgLetter::Or(ors.into()))
    }

    Ok(Some(letters.into()))
}

fn parse_letter(lex: &mut Lexer) -> Result<Option<CfgLetter>, ParseError> {
    match lex.peek_next() {
        Some(_) => {}
        // None => return Err(ParseError::GotEof(lex.ctx.clone())),
        None => return Ok(None),
    }

    let mut letter = match lex.next() {
        Some(Token::Ident(ident)) => {
            if is_uppercase(&ident) {
                CfgLetter::Term(ident)
            } else {
                CfgLetter::Rule(ident)
            }
        }
        Some(Token::Op(op)) => match op {
            Operator::OpenParen => {
                let mut letters = vec![];
                let mut ors = vec![];
                loop {
                    match lex.peek_next() {
                        Some(Token::Op(Operator::CloseParen)) => {
                            let _ = lex.next();
                            break;
                        }
                        None => return Err(ParseError::GotEof(lex.ctx.clone())),
                        _ => {}
                    }

                    // Unwrap because we know its not None due to last match
                    match parse_letter(lex)? {
                        Some(CfgLetter::Or(or)) => {
                            letters.push(or[0][0].clone());
                            ors.push(std::mem::take(&mut letters).into());
                        }
                        Some(ltr) => letters.push(ltr),
                        None => return Err(ParseError::GotEof(lex.ctx.clone())),
                    }
                }

                if ors.is_empty() {
                    CfgLetter::Group(letters.into())
                } else {
                    ors.push(std::mem::take(&mut letters).into());
                    CfgLetter::Group([CfgLetter::Or(ors.into())].into())
                }
            }
            _ => return Err(ParseError::UnexpectedOp(lex.ctx.clone(), op)),
        },
        Some(Token::String(str)) => CfgLetter::StrLit(str),
        None => return Ok(None),
    };

    while let Some(Token::Op(op)) = lex.peek_next() {
        match op {
            Operator::Star => {
                let _ = lex.next();
                letter = CfgLetter::Many(Box::new(letter))
            }
            Operator::Plus => {
                let _ = lex.next();
                letter = CfgLetter::OneOrMore(Box::new(letter))
            }
            Operator::QuestionMark => {
                let _ = lex.next();
                letter = CfgLetter::Optional(Box::new(letter))
            }
            Operator::Pipe => {
                let _ = lex.next();
                letter = CfgLetter::Or(Box::new([Box::new([letter])]));
                break;
            }
            Operator::Eol | Operator::RuleDeclare | Operator::OpenParen | Operator::CloseParen => {
                break;
            }
        }
    }

    Ok(Some(letter))
}

fn is_uppercase(str: &str) -> bool {
    for ch in str.chars() {
        if ch.is_lowercase() {
            return false;
        }
    }
    true
}
