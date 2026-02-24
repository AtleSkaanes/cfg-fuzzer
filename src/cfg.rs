use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug)]
pub struct Cfg {
    pub rules: HashMap<Box<str>, CfgRule>,
}

pub type CfgRule = Box<[CfgLetter]>;

#[derive(Clone, Debug)]
pub enum CfgLetter {
    /// A reference to another rule
    Rule(Box<str>),
    /// String literal
    StrLit(Box<str>),
    /// A chain of options, where only one will be evaluated, denoted by the | operator
    Or(Box<[Box<[CfgLetter]>]>),
    /// An optional letter, denoted by the ? suffix
    Optional(Box<CfgLetter>),
    /// An arbitrary amount of repitions, denoted by the * suffix
    Many(Box<CfgLetter>),
    /// An arbitrary amount of repitions, denoted by the + suffix
    OneOrMore(Box<CfgLetter>),
    /// A group of letters, denoted by surrounding it in ( )
    Group(Box<[CfgLetter]>),
    Range(Box<[CfgRange]>),
    /// A terminating value
    Term(Box<str>),
}

#[derive(Clone, Debug)]
pub struct CfgRange {
    pub from: char,
    pub to: Option<char>,
}

impl CfgRange {
    pub fn new(from: char, to: char) -> Self {
        Self { from, to: Some(to) }
    }

    pub fn new_single(ch: char) -> Self {
        Self { from: ch, to: None }
    }
}

impl Display for CfgRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to {
            Some(to) => write!(f, "{}-{}", self.from, to),
            None => write!(f, "{}", self.from),
        }
    }
}
