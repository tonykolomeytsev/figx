use super::CodeBlock;
use std::{collections::HashSet, fmt::Display};

pub struct FileSpec {
    pub package: String,
    pub imports: HashSet<String>,
    pub members: Vec<CodeBlock>,
}

impl FileSpec {
    pub fn builder<S: AsRef<str>>(package: S) -> FileSpecBuilder {
        FileSpecBuilder {
            package: package.as_ref().to_string(),
            imports: HashSet::with_capacity(20),
            members: Vec::with_capacity(3),
        }
    }
}

pub struct FileSpecBuilder {
    package: String,
    imports: HashSet<String>,
    members: Vec<CodeBlock>,
}

#[allow(unused)]
impl FileSpecBuilder {
    pub fn add_member(mut self, member: CodeBlock) -> Self {
        member.imports.iter().for_each(|import| {
            self.imports.insert(import.clone());
        });
        self.members.push(member);
        self
    }

    pub fn require_import<S: AsRef<str>>(mut self, s: S) -> Self {
        self.imports.insert(s.as_ref().to_string());
        self
    }

    pub fn build(self) -> FileSpec {
        FileSpec {
            package: self.package,
            imports: self.imports,
            members: self.members,
        }
    }
}

impl Display for FileSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FileSpec {
            package,
            imports,
            members,
        } = self;
        if !package.is_empty() {
            writeln!(f, "package {package}")?;
            writeln!(f)?;
        }

        let mut imports: Vec<_> = imports.iter().collect();
        imports.sort();

        for import in imports {
            writeln!(f, "import {import}")?;
        }
        writeln!(f)?;

        for member in members {
            writeln!(f, "{member}")?;
        }
        Ok(())
    }
}
