use std::collections::HashMap;

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
    /// TODO: Better comment: A binop that matches either lhs or rhs, denoted by the | operator
    Or(Box<[Box<[CfgLetter]>]>),
    /// An optional letter, denoted by the ? suffix
    Optional(Box<CfgLetter>),
    /// An arbitrary amount of repitions, denoted by the * suffix
    Many(Box<CfgLetter>),
    /// An arbitrary amount of repitions, denoted by the + suffix
    OneOrMore(Box<CfgLetter>),
    /// A group of letters, denoted by surrounding it in ( )
    Group(Box<[CfgLetter]>),
    /// A terminating value
    Term(Box<str>),
}
