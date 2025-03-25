use super::CodeBlock;
use std::collections::HashSet;

pub struct PropertySpec {
    pub name: String,
    pub type_name: String,
    pub imports: HashSet<String>,
    pub annotations: Vec<String>,
    pub getter: Option<CodeBlock>,
    pub setter: Option<CodeBlock>,
    pub initializer: Option<CodeBlock>,
    pub modifiers: Vec<String>,
    pub mutable: bool,
}

impl PropertySpec {
    pub fn builder<S1: AsRef<str>, S2: AsRef<str>>(name: S1, type_name: S2) -> PropertySpecBuilder {
        PropertySpecBuilder {
            name: name.as_ref().to_string(),
            type_name: type_name.as_ref().to_string(),
            imports: HashSet::new(),
            annotations: Vec::new(),
            getter: None,
            setter: None,
            initializer: None,
            modifiers: Vec::new(),
            mutable: false,
        }
    }
}

pub struct PropertySpecBuilder {
    name: String,
    type_name: String,
    imports: HashSet<String>,
    annotations: Vec<String>,
    getter: Option<CodeBlock>,
    setter: Option<CodeBlock>,
    initializer: Option<CodeBlock>,
    modifiers: Vec<String>,
    mutable: bool,
}

#[allow(unused)]
impl PropertySpecBuilder {
    pub fn require_import<S: AsRef<str>>(mut self, s: S) -> Self {
        self.imports.insert(s.as_ref().to_string());
        self
    }

    pub fn add_annotation<S: AsRef<str>>(mut self, s: S) -> Self {
        self.annotations.push(s.as_ref().to_string());
        self
    }

    pub fn getter(mut self, cb: CodeBlock) -> Self {
        self.getter = Some(cb);
        self
    }

    pub fn setter(mut self, cb: CodeBlock) -> Self {
        self.setter = Some(cb);
        self
    }

    pub fn initializer(mut self, cb: CodeBlock) -> Self {
        self.initializer = Some(cb);
        self
    }

    pub fn add_modifier<S: AsRef<str>>(mut self, s: S) -> Self {
        self.modifiers.push(s.as_ref().to_string());
        self
    }

    pub fn mutable(mut self) -> Self {
        self.mutable = true;
        self
    }

    pub fn build(self) -> PropertySpec {
        PropertySpec {
            name: self.name,
            type_name: self.type_name,
            imports: self.imports,
            annotations: self.annotations,
            getter: self.getter,
            setter: self.setter,
            initializer: self.initializer,
            modifiers: self.modifiers,
            mutable: self.mutable,
        }
    }
}

impl From<PropertySpec> for CodeBlock {
    fn from(value: PropertySpec) -> Self {
        let PropertySpec {
            name,
            type_name,
            imports,
            annotations,
            getter,
            setter,
            initializer,
            modifiers,
            mutable,
        } = value;
        let keyword = if mutable { "var" } else { "val" };
        let modifiers = if modifiers.is_empty() {
            String::new()
        } else {
            format!("{} ", modifiers.join(" "))
        };
        let mut result = if let Some(cb) = initializer {
            Self::builder()
                .add_statements(&annotations)
                .add_statement(format!("{modifiers}{keyword} {name}: {type_name} = "))
                .no_new_line()
                .add_code_block(cb)
        } else {
            Self::builder()
                .add_statements(&annotations)
                .add_statement(format!("{modifiers}{keyword} {name}: {type_name}"))
        };
        result = if let Some(getter) = getter {
            result.indent().add_code_block(getter).unindent()
        } else {
            result
        };
        result = if let Some(setter) = setter {
            result.indent().add_code_block(setter).unindent()
        } else {
            result
        };

        result
            .require_imports(&imports.into_iter().collect::<Vec<_>>())
            .build()
    }
}
