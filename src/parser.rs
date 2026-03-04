use std::collections::HashMap;

use thiserror::Error;

use crate::{
    cfg::{Cfg, CfgLetter, CfgRange, CfgRule},
    lexer::{Lexer, LexerCtx, Operator, Token},
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Top level declaration not found, no rule with name '{0}'")]
    NoTopLevelDeclaration(Box<str>),
    #[error("Rule {0} referenced but not defined")]
    UndefinedRule(Box<str>),
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

use MaybeUnknown::{Known, Unknown};
enum MaybeUnknown {
    /// The letter is known
    Known(CfgLetter),
    /// The letter is a reference to another rule, which must be retrieved later
    Unknown(Box<str>),
}

impl MaybeUnknown {
    /// Tries to become a known rule.
    /// Returns `Some` if value is known, or found in `map`.
    /// Returns `None` if value is unknown and not found in `map`
    ///
    /// If the value was fetched from the map, the result will always have variant
    /// `CfgLetter::Rule`
    pub fn into_letter(self, map: &HashMap<Box<str>, CfgRule>) -> Result<CfgLetter, ParseError> {
        match self {
            Known(l) => Ok(l),
            Unknown(name) => map
                .get(&name)
                .map(|rule| CfgLetter::Rule(*rule))
                .ok_or(ParseError::UndefinedRule(name)),
        }
    }
}

pub fn parse(
    src: &str,
    filename: &str,
    terms: &[Box<str>],
    top_level: &str,
) -> Result<Cfg, ParseError> {
    let mut lex = Lexer::new(src, filename);
    let mut maybes: Vec<MaybeUnknown> = vec![];

    // All CfgRules with names attached for reference
    let mut map: HashMap<Box<str>, CfgRule> = HashMap::new();

    while let Some(tok) = lex.next() {
        let Token::Ident(ident) = tok else {
            return Err(ParseError::UnexpectedToken(
                lex.ctx(),
                "identifier".into(),
                tok,
            ));
        };

        let Some(tok) = lex.next() else {
            return Err(ParseError::GotEof(lex.ctx()));
        };

        if !matches!(tok, Token::Op(Operator::RuleDeclare)) {
            return Err(ParseError::UnexpectedToken(lex.ctx(), "':'".into(), tok));
        }

        if matches!(lex.peek_next(), Some(Token::Op(Operator::Pipe))) {
            _ = lex.next();
        }

        let rules = parse_rule(&mut lex, &mut maybes)?;

        map.insert(ident.clone(), rules);
    }

    let mut terms_map: HashMap<Box<str>, (usize, usize)> = HashMap::new();
    for term in terms {
        let Some((term_ident, term_str)) = term.split_once(':') else {
            return Err(ParseError::InvalidTerm(term.clone()));
        };

        let mut lex = Lexer::new(term_str, term_ident);

        let term_rule = parse_rule(&mut lex, &mut maybes)?;

        for id in term_rule.0..term_rule.1 {
            if !is_valid_term(id, &maybes) {
                return Err(ParseError::TermRecurse(term_ident.into()));
            }
        }

        if terms_map.contains_key(term_ident) {
            return Err(ParseError::TermDupe(term_ident.into()));
        }

        let begin = maybes.len();
        terms_map.insert(term_ident.into(), (begin, maybes.len()));
    }

    let tld_rule_pos = match map.get(top_level) {
        Some(rule_pos) => *rule_pos,
        None => return Err(ParseError::NoTopLevelDeclaration(top_level.into())),
    };

    let letters: Result<Box<[CfgLetter]>, ParseError> = maybes
        .into_iter()
        .map(|maybe| maybe.into_letter(&map))
        .collect();

    Ok(Cfg::new(letters?, tld_rule_pos, terms_map))
}

fn parse_rule(lex: &mut Lexer, letters: &mut Vec<MaybeUnknown>) -> Result<CfgRule, ParseError> {
    let mut ors = vec![];
    let mut begin = letters.len();
    loop {
        // Unwrap because we know its not None due to last match
        if matches!(lex.peek_next(), Some(Token::Op(Operator::Eol))) {
            lex.next();
            break;
        }
        match parse_letter(lex, letters)? {
            Some(Known(CfgLetter::Or(..))) => {
                ors.push((begin, letters.len()));
                begin = letters.len();
            }
            Some(ltr) => letters.push(ltr),
            None => break,
        }
    }

    if !ors.is_empty() {
        ors.push((begin, letters.len()));
        begin = letters.len();
        letters.push(Known(CfgLetter::Or(ors.into())));
    }

    Ok((begin, letters.len()))
}

fn parse_letter(
    lex: &mut Lexer,
    letters: &mut Vec<MaybeUnknown>,
) -> Result<Option<MaybeUnknown>, ParseError> {
    // match lex.peek_next() {
    //     Some(_) => {}
    //     // None => return Err(ParseError::GotEof(lex.ctx.clone())),
    //     None => return Ok(None),
    // }

    let mut letter = match lex.next() {
        Some(Token::Ident(ident)) => {
            if is_uppercase(&ident) {
                Known(CfgLetter::Term(ident))
            } else {
                Unknown(ident)
            }
        }
        Some(Token::Op(op)) => match op {
            Operator::Pipe => Known(CfgLetter::Or(Box::new([]))),
            Operator::OpenParen => {
                let mut begin = letters.len();
                let mut ors = vec![];
                loop {
                    match lex.peek_next() {
                        Some(Token::Op(Operator::CloseParen)) => {
                            lex.next();
                            break;
                        }
                        None => return Err(ParseError::GotEof(lex.ctx())),
                        _ => {}
                    }

                    match parse_letter(lex, letters)? {
                        Some(Known(CfgLetter::Or(..))) => {
                            ors.push((begin, letters.len()));
                            begin = letters.len();
                        }
                        Some(ltr) => letters.push(ltr),
                        None => return Err(ParseError::GotEof(lex.ctx())),
                    }
                }

                if !ors.is_empty() {
                    ors.push((begin, letters.len()));
                    begin = letters.len();
                    letters.push(Known(CfgLetter::Or(ors.into())));
                }

                Known(CfgLetter::Group((begin, letters.len())))
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
                            ranges.push(CfgRange::new_single(prev));
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
                                shift_char(&mut needs_rhs, digit, lex.ctx())?;
                            }
                        }
                        Token::String(str) | Token::Ident(str) => {
                            for ch in str.chars() {
                                shift_char(&mut needs_rhs, ch, lex.ctx())?;
                            }
                        }
                        Token::Op(_) => {}
                    }
                }

                if let Some(ch) = range_from {
                    ranges.push(CfgRange::new_single(ch));
                }

                Known(CfgLetter::Range(ranges.into()))
            }
            _ => return Err(ParseError::UnexpectedOp(lex.ctx(), op)),
        },
        Some(Token::String(str)) => Known(CfgLetter::StrLit(str)),
        Some(Token::Num(num)) => Known(CfgLetter::StrLit(num.to_string().into())),
        None => return Ok(None),
    };

    while let Some(Token::Op(op)) = lex.peek_next() {
        match op {
            Operator::Star => {
                lex.next();
                let idx = letters.len();
                letters.push(letter);
                letter = Known(CfgLetter::Many(idx));
            }
            Operator::Plus => {
                lex.next();
                let idx = letters.len();
                letters.push(letter);
                letter = Known(CfgLetter::OneOrMore(idx));
            }
            Operator::QuestionMark => {
                lex.next();
                let idx = letters.len();
                letters.push(letter);
                letter = Known(CfgLetter::Optional(idx));
            }
            Operator::Eol
            | Operator::RuleDeclare
            | Operator::OpenParen
            | Operator::CloseParen
            | Operator::OpenRange
            | Operator::CloseRange
            | Operator::Pipe
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

fn is_valid_term(id: usize, maybes: &Vec<MaybeUnknown>) -> bool {
    let Known(letter) = &maybes[id] else {
        return false;
    };
    match letter {
        CfgLetter::Rule(_) | CfgLetter::Term(_) => false,
        CfgLetter::StrLit(_) | CfgLetter::Range(_) => true,
        CfgLetter::Or(items) => items
            .iter()
            .flat_map(|rule| rule.0..rule.1)
            .all(|maybe| is_valid_term(maybe, maybes)),

        CfgLetter::Optional(id) | CfgLetter::Many(id) | CfgLetter::OneOrMore(id) => {
            is_valid_term(*id, maybes)
        }
        CfgLetter::Group(items) => (items.0..items.1).all(|maybe| is_valid_term(maybe, maybes)),
    }
}
