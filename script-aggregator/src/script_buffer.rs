use std::cmp;

use crate::indent_engine::IndentEngine;

#[derive(Debug)]
pub struct ScriptBuffer {
    auto_indent: bool,
    indent: Option<String>,
    indent_count: usize,
    indent_enabled: bool,

    eol: Option<String>,
    names: Vec<String>,

    preamble: String,
    body: String,
    postamble: String,

    preamble_indenter: IndentEngine,
    body_indenter: IndentEngine,
    postamble_indenter: IndentEngine,
}

impl ScriptBuffer {
    pub const MAXIMUM_INDENT: &'static str = "                    "; // 20 spaces
    pub const DEFAULT_INDENT: i32 = 4;

    pub fn new() -> Self {
        ScriptBuffer {
            auto_indent: false,
            indent: None,
            indent_count: 0,
            indent_enabled: true,

            eol: Some(String::from("\n")),
            names: Vec::new(),

            preamble: String::new(),
            body: String::new(),
            postamble: String::new(),

            preamble_indenter: IndentEngine::new(String::from("    ")),
            body_indenter: IndentEngine::new(String::from("    ")),
            postamble_indenter: IndentEngine::new(String::from("    ")),
        }
    }

    pub fn set_auto_indent(&mut self, auto_indent: bool) {
        self.auto_indent = auto_indent;
    }

    pub fn change_indent(&mut self, value: i32) {
        let new_indent = cmp::max(
            0,
            cmp::min(
                ScriptBuffer::MAXIMUM_INDENT.chars().count() as i32,
                self.indent_count as i32 + value,
            ),
        ) as usize;
        self.set_indent(new_indent);
    }

    pub fn set_indent(&mut self, value: usize) {
        if value == 0 {
            self.indent_count = 0;
            self.indent = None;
        } else if value > 0 && value <= ScriptBuffer::MAXIMUM_INDENT.chars().count() {
            self.indent_count = value;
            self.indent = Some(ScriptBuffer::MAXIMUM_INDENT.chars().take(value).collect());
        }
    }

    pub fn append(&mut self, statement: String) {
        if self.auto_indent {
            self.body_indenter.apply(&mut self.body, &statement);
        } else if self.indent_enabled && self.indent.is_some() {
            if let Some(ref indent) = self.indent {
                self.body.push_str(indent);
            }
            self.body.push_str(statement.trim());
        } else {
            self.body.push_str(&statement);
        }
        if let Some(eol) = &self.eol {
            self.body.push_str(eol);
        }
    }

    pub fn postpend(&mut self, statement: String) {
        if self.auto_indent {
            self.postamble_indenter
                .apply(&mut self.postamble, &statement);
        } else if self.indent_enabled && self.indent.is_some() {
            if let Some(ref indent) = self.indent {
                self.postamble.push_str(indent);
            }
            self.postamble.push_str(statement.trim());
        } else {
            self.postamble.push_str(&statement);
        }
        if let Some(eol) = &self.eol {
            self.postamble.push_str(eol);
        }
    }

    pub fn prepend(&mut self, statement: String) {
        if self.auto_indent {
            self.preamble_indenter.apply(&mut self.preamble, &statement);
        } else if self.indent_enabled && self.indent.is_some() {
            if let Some(ref indent) = self.indent {
                self.preamble.push_str(indent);
            }
            self.preamble.push_str(statement.trim());
        } else {
            self.preamble.push_str(&statement);
        }
        if let Some(eol) = &self.eol {
            self.preamble.push_str(eol);
        }
    }

    pub fn get_unique_name(&mut self, basename: String) -> String {
        let mut name = basename.clone();
        let mut copy = 1;
        while self.names.contains(&name) {
            name = format!("{}{}", basename, copy);
            copy += 1;
        }
        self.names.push(name.clone());
        name
    }

    pub fn to_string(&self) -> String {
        let mut script = String::new();
        script.push_str(&self.preamble);
        script.push_str(&self.body);
        script.push_str(&self.postamble);
        script
    }
}
