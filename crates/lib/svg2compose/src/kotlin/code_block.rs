use std::{collections::HashSet, fmt::Display};

pub enum Token {
    Indent,
    Unindent,
    Statement(String),
    Text(String),
    NoNewLine,
}

pub struct CodeBlock {
    pub(super) tokens: Vec<Token>,
    pub(super) imports: HashSet<String>,
}

impl CodeBlock {
    pub fn builder() -> CodeBlockBuilder {
        CodeBlockBuilder {
            tokens: Vec::new(),
            imports: HashSet::new(),
        }
    }
}

pub struct CodeBlockBuilder {
    tokens: Vec<Token>,
    imports: HashSet<String>,
}

#[allow(unused)]
impl CodeBlockBuilder {
    #[inline]
    fn add(mut self, t: Token) -> Self {
        self.tokens.push(t);
        self
    }

    pub fn add_statement<S: AsRef<str>>(self, s: S) -> Self {
        self.add(Token::Statement(s.as_ref().to_string()))
    }

    pub fn add_statements<S: AsRef<str>>(mut self, s: &[S]) -> Self {
        s.iter()
            .for_each(|s| self.tokens.push(Token::Statement(s.as_ref().to_string())));
        self
    }

    pub fn add_code_block(mut self, cb: CodeBlock) -> Self {
        let mut cb = cb;
        self.imports.extend(cb.imports);
        self.tokens.append(&mut cb.tokens);
        self
    }

    pub fn add_code_blocks(mut self, cbs: Vec<CodeBlock>) -> Self {
        for cb in cbs {
            let mut cb = cb;
            self.imports.extend(cb.imports);
            self.tokens.append(&mut cb.tokens);
        }
        self
    }

    pub fn indent(self) -> Self {
        self.add(Token::Indent)
    }

    pub fn unindent(self) -> Self {
        self.add(Token::Unindent)
    }

    pub fn begin_control_flow<S: AsRef<str>>(mut self, s: S) -> Self {
        let s = s.as_ref();
        if s.ends_with('}') {
            return self;
        }
        if s.contains('{') {
            self.tokens.push(Token::Text(format!("{s}\n")));
        } else {
            self.tokens.push(Token::Text(format!("{s} {{\n")));
        }
        self.tokens.push(Token::NoNewLine);
        self.indent()
    }

    pub fn next_control_flow<S: AsRef<str>>(self, s: S) -> Self {
        self.unindent()
            .add(Token::Text(format!("}} {} {{", s.as_ref())))
            .indent()
    }

    pub fn end_control_flow(self) -> Self {
        self.unindent().add(Token::Text("}".to_string()))
    }

    pub fn no_new_line(self) -> Self {
        self.add(Token::NoNewLine)
    }

    pub fn require_import<S: AsRef<str>>(mut self, s: S) -> Self {
        self.imports.insert(s.as_ref().to_string());
        self
    }

    pub fn require_imports<S: AsRef<str>>(mut self, s: &[S]) -> Self {
        self.imports
            .extend(s.iter().map(|it| it.as_ref().to_string()));
        self
    }

    pub fn build(self) -> CodeBlock {
        CodeBlock {
            tokens: self.tokens,
            imports: self.imports,
        }
    }
}

impl Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut depth = 0usize;

        let mut iter1 = self.tokens.iter();
        let mut iter2 = self.tokens.iter();
        iter2.next();
        let mut no_new_line = false;

        loop {
            let current = if let Some(item) = iter1.next() {
                item
            } else {
                break;
            };
            let next = iter2.next();

            match current {
                Token::Indent => depth += 1,
                Token::Unindent => depth -= 1,
                Token::Text(str) => {
                    if !no_new_line {
                        write!(f, "{}", "    ".repeat(depth))?;
                    }
                    write!(f, "{str}")?;
                    match next {
                        Some(Token::NoNewLine) => (),
                        _ => writeln!(f)?,
                    }
                }
                Token::Statement(str) => {
                    if !no_new_line {
                        write!(f, "{}", "    ".repeat(depth))?;
                    }
                    write!(f, "{str}")?;
                    match next {
                        Some(Token::NoNewLine) => (),
                        _ => writeln!(f)?,
                    }
                }
                Token::NoNewLine => {
                    no_new_line = true;
                    continue;
                }
            }
            no_new_line = false;
        }
        Ok(())
    }
}
