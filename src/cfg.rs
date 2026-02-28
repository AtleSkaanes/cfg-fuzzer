use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug)]
pub struct Cfg {
    letters: Box<[CfgLetter]>,
    pub top_level: CfgRule,
    pub terms: HashMap<Box<str>, CfgRule>,
}

impl Cfg {
    pub fn new(letters: Box<[CfgLetter]>, top_level: CfgRule, terms: HashMap<Box<str>, CfgRule>) -> Self {
        Cfg { letters, top_level, terms }
    }

    /// Get the slice of letters, which `rule` refers to.
    pub fn rule_slice(&self, rule: &CfgRule) -> &[CfgLetter] {
        &self.letters[rule.0 .. rule.1]
    }

    pub fn get_letter(&self, id: usize) -> &CfgLetter {
        &self.letters[id]
    }
}

/// A grouped list of letters
pub type CfgRule = (usize, usize);
pub type CfgTermID = u16;

#[derive(Clone, Debug)]
pub enum CfgLetter {
    /// A reference to another rule
    Rule(CfgRule),
    /// String literal
    StrLit(Box<str>),
    /// A chain of options, where only one will be evaluated, denoted by the | operator
    Or(Box<[CfgRule]>),
    /// An optional letter, denoted by the ? suffix
    Optional(usize),
    /// An arbitrary amount of repitions, denoted by the * suffix
    Many(usize),
    /// An arbitrary amount of repitions, denoted by the + suffix
    OneOrMore(usize),
    /// A group of letters, denoted by surrounding it in ( )
    Group(CfgRule),
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
