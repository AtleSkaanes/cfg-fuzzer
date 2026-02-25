use std::collections::HashMap;

use thiserror::Error;

use crate::{
    cfg::{Cfg, CfgLetter, CfgRange, CfgRule},
    lexer::{Lexer, LexerCtx, Operator, Token},
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0} -> Expected {1} but got {2:?}")]
    UnexpectedToken(LexerCtx, Box<str>, Token),
    #[error("{0} -> Got unexpected operator {1:?}")]
    UnexpectedOp(LexerCtx, Operator),
    #[error("{0} -> Got range operator '-' without left hand side argument")]
    UnexpectedRange(LexerCtx),
    #[error("{0} -> Got invalid range '{1}'")]
    InvalidRange(LexerCtx, CfgRange),
    #[error("Term '{0}' is not valid. The syntax is 'TERM_NAME:VAL'")]
    InvalidTerm(Box<str>),
    #[error("Term '{0}' depends on the a rule. A term can only consit of terminal values")]
    TermRecurse(Box<str>),
    #[error("Term '{0}' defined multiple times")]
    TermDupe(Box<str>),
    #[error("{0} -> Got unexpected EOF")]
    GotEof(LexerCtx),
}

pub fn parse(src: &str, filename: &str, terms: &[Box<str>]) -> Result<Cfg, ParseError> {
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

    let mut terms_map = HashMap::new();
    for term in terms {
        let Some((term_ident, term_str)) = term.split_once(':') else {
            return Err(ParseError::InvalidTerm(term.clone()));
        };

        let mut lex = Lexer::new(term_str, term_ident);

        let Some(term_rule) = parse_rule(&mut lex)? else {
            return Err(ParseError::InvalidTerm(term.clone()));
        };

        for ltr in &term_rule {
            if !is_valid_term(ltr) {
                return Err(ParseError::TermRecurse(term_ident.into()));
            }
        }

        if terms_map.insert(term_ident.into(), term_rule).is_some() {
            return Err(ParseError::TermDupe(term_ident.into()));
        };
    }

    Ok(Cfg {
        rules: map,
        terms: terms_map,
    })
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
            Operator::OpenRange => {
                let mut ranges = vec![];
                let mut range_from = None;
                let mut needs_rhs = false;

                let is_related = |first: char, second: char| {
                    first.is_numeric() == second.is_numeric()
                        && first.is_lowercase() == second.is_lowercase()
                        && first <= second
                        && first.is_alphanumeric()
                        && second.is_alphanumeric()
                };

                let mut shift_char = |nrhs: &mut _, ch, ctx| {
                    if *nrhs {
                        if let Some(prev) = range_from {
                            if !is_related(prev, ch) {
                                return Err(ParseError::InvalidRange(ctx, CfgRange::new(prev, ch)));
                            }
                            ranges.push(CfgRange::new(prev, ch));
                        } else {
                            return Err(ParseError::UnexpectedRange(ctx));
                        }
                        range_from = None;
                        *nrhs = false;
                    } else {
                        if let Some(prev) = range_from {
                            if !(prev.is_alphanumeric() || prev == '_') {
                                return Err(ParseError::InvalidRange(
                                    ctx,
                                    CfgRange::new_single(prev),
                                ));
                            }
                            ranges.push(CfgRange::new_single(prev))
                        }
                        range_from = Some(ch);
                    }
                    Ok(())
                };

                while let Some(token) = lex.next() {
                    match token {
                        Token::Op(Operator::CloseRange) => {
                            break;
                        }
                        Token::Op(Operator::RangeTo) => {
                            needs_rhs = true;
                        }
                        Token::Num(num) => {
                            for digit in num.to_string().chars() {
                                shift_char(&mut needs_rhs, digit, lex.ctx.clone())?
                            }
                        }
                        Token::String(str) | Token::Ident(str) => {
                            for ch in str.chars() {
                                shift_char(&mut needs_rhs, ch, lex.ctx.clone())?
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(ch) = range_from {
                    ranges.push(CfgRange::new_single(ch))
                }

                CfgLetter::Range(ranges.into())
            }
            _ => return Err(ParseError::UnexpectedOp(lex.ctx.clone(), op)),
        },
        Some(Token::String(str)) => CfgLetter::StrLit(str),
        Some(Token::Num(num)) => CfgLetter::StrLit(num.to_string().into()),
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
            Operator::Eol
            | Operator::RuleDeclare
            | Operator::OpenParen
            | Operator::CloseParen
            | Operator::OpenRange
            | Operator::CloseRange
            | Operator::RangeTo => {
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

fn is_valid_term(letter: &CfgLetter) -> bool {
    match letter {
        CfgLetter::Rule(_) | CfgLetter::Term(_) => false,
        CfgLetter::StrLit(_) => true,
        CfgLetter::Or(items) => {
            for rule in items {
                for ltr in rule {
                    if !is_valid_term(ltr) {
                        return false;
                    }
                }
            }
            true
        }
        CfgLetter::Optional(ltr) => is_valid_term(ltr),

        CfgLetter::Many(ltr) => is_valid_term(ltr),
        CfgLetter::OneOrMore(ltr) => is_valid_term(ltr),
        CfgLetter::Group(items) => {
            for ltr in items {
                if !is_valid_term(ltr) {
                    return false;
                }
            }
            true
        }
        CfgLetter::Range(_) => true,
    }
}
