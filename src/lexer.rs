pub struct Lexer {
    pub ctx: LexerCtx,
    src: Box<[char]>,
    index: usize,
}

impl Lexer {
    pub fn new(src: &str, filename: &str) -> Self {
        let src_string = src.to_owned();
        Self {
            ctx: LexerCtx {
                line: 1,
                column: 1,
                file: filename.into(),
            },
            src: src_string.chars().collect(),
            index: 0,
        }
    }

    pub fn peek_next(&mut self) -> Option<Token> {
        let old_idx = self.index;
        let token = self.next();
        self.index = old_idx;

        token
    }
}

impl std::iter::Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.src.len() {
            if self.src[self.index] == '#' {
                while self.src[self.index] != '\n' {
                    self.index += 1;
                }
                break;
            }
            // if self.index < self.src.len() + 1
            //     && matches!(
            //         self.src[self.index..],
            //         ['\n', '\n', ..]
            //             | ['\r', '\n', '\r', '\n', ..]
            //             | ['\r', '\n', '\n', ..]
            //             | ['\n', '\r', '\n', ..]
            //     )
            // {
            //     self.index += 2;
            //     return Some(Token::Op(Operator::Eol));
            // }

            if self.src[self.index].is_whitespace() {
                if self.src[self.index] == '\n' {
                    self.ctx.column = 1;
                    self.ctx.line += 1;
                }
                self.index += 1;
                continue;
            }

            if self.src[self.index].is_numeric() {
                let start_idx = self.index;
                while self.src[self.index].is_numeric() {
                    self.index += 1;
                }
                let num: usize = match &self.src[start_idx..self.index]
                    .iter()
                    .collect::<String>()
                    .parse()
                {
                    Ok(n) => *n,
                    Err(_) => return None,
                };

                return Some(Token::Num(num));
            }

            // String
            if self.src[self.index] == '"' || self.src[self.index] == '\'' {
                let delim = self.src[self.index];
                self.index += 1;
                let start_idx = self.index;
                while self.src[self.index] != delim {
                    if self.src[self.index] == '\\' {
                        self.index += 2;
                    } else {
                        self.index += 1
                    }
                }
                let str: String = self.src[start_idx..self.index].iter().collect();
                self.index += 1;
                return Some(Token::String(str.into()));
            }

            if is_ident_start(self.src[self.index]) {
                let start_idx = self.index;

                while is_ident(self.src[self.index]) {
                    self.index += 1;
                }

                let str: String = self.src[start_idx..self.index].iter().collect();
                return Some(Token::Ident(str.into()));
            }

            let op = match self.src[self.index] {
                '*' => Some(Operator::Star),
                '+' => Some(Operator::Plus),
                '?' => Some(Operator::QuestionMark),
                '|' => Some(Operator::Pipe),
                '(' => Some(Operator::OpenParen),
                ')' => Some(Operator::CloseParen),
                '[' => Some(Operator::OpenRange),
                ']' => Some(Operator::CloseRange),
                '-' => Some(Operator::RangeTo),
                ':' => Some(Operator::RuleDeclare),
                ';' => Some(Operator::Eol),
                _ => None,
            };

            if let Some(o) = op {
                self.index += 1;
                return Some(Token::Op(o));
            }

            break;
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct LexerCtx {
    pub line: usize,
    pub column: usize,
    pub file: Box<str>,
}

impl std::fmt::Display for LexerCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[derive(Clone, Debug)]
pub enum Operator {
    Star,
    Plus,
    QuestionMark,
    Pipe,
    OpenParen,
    CloseParen,
    OpenRange,
    CloseRange,
    RangeTo,
    RuleDeclare,
    Eol,
}

#[derive(Clone, Debug)]
pub enum Token {
    String(Box<str>),
    Num(usize),
    Ident(Box<str>),
    Op(Operator),
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident(ch: char) -> bool {
    is_ident_start(ch) || ch.is_numeric()
}
