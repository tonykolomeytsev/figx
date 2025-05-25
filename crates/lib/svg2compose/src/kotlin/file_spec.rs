use super::CodeBlock;
use std::{collections::HashSet, fmt::Display};

pub struct FileSpec {
    pub suppressions: Vec<String>,
    pub package: String,
    pub imports: HashSet<String>,
    pub members: Vec<CodeBlock>,
}

impl FileSpec {
    pub fn builder<S: AsRef<str>>(package: S) -> FileSpecBuilder {
        FileSpecBuilder {
            suppressions: Vec::new(),
            package: package.as_ref().to_string(),
            imports: HashSet::with_capacity(20),
            members: Vec::with_capacity(3),
        }
    }
}

pub struct FileSpecBuilder {
    suppressions: Vec<String>,
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

    pub fn add_suppressions(mut self, list: Vec<String>) -> Self {
        let mut list = list;
        self.suppressions.append(&mut list);
        self
    }

    pub fn build(self) -> FileSpec {
        FileSpec {
            suppressions: self.suppressions,
            package: self.package,
            imports: self.imports,
            members: self.members,
        }
    }
}

impl Display for FileSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FileSpec {
            suppressions,
            package,
            imports,
            members,
        } = self;
        if !suppressions.is_empty() {
            writeln!(f, "@file:Suppress(")?;
            for s in suppressions {
                writeln!(f, "    \"{s}\"")?;
            }
            writeln!(f, ")")?;
            writeln!(f)?;
        }

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
