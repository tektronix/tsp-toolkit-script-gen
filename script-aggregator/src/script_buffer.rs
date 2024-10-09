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

    /// Enables or disables automatic indentation.
    /// If true, will enable indenting and override the manual indentation settings
    ///
    /// # Arguments
    ///
    /// * `auto_indent` - A boolean value indicating whether to enable or disable automatic indentation.
    pub fn set_auto_indent(&mut self, auto_indent: bool) {
        self.auto_indent = auto_indent;
    }

    /// Changes the current indentation level by a specified value.
    /// The absolute value for indentation is limited to between 0 and 20 inclusive.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to change the current indentation level by. It can be positive or negative.
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

    /// Sets the current indentation level (in # of spaces) to a specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to set the current indentation level to. It must be between 0 and the maximum allowed indentation level.
    pub fn set_indent(&mut self, value: usize) {
        if value == 0 {
            self.indent_count = 0;
            self.indent = None;
        } else if value > 0 && value <= ScriptBuffer::MAXIMUM_INDENT.chars().count() {
            self.indent_count = value;
            self.indent = Some(ScriptBuffer::MAXIMUM_INDENT.chars().take(value).collect());
        }
    }

    /// Appends a statement to the "body" portion of the script.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement to be appended to the body.
    pub fn body_append(&mut self, statement: String) {
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

    /// Appends a statement to the "postamble" portion of the script.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement to be appended to the postamble.
    pub fn postamble_append(&mut self, statement: String) {
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

    /// Appends a statement to the "preamble" portion of the script.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement to be appended to the preamble.
    pub fn preamble_append(&mut self, statement: String) {
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

    /// Generates a unique name based on the given basename.
    /// This is used to prevent name collision for methods added by various FunctionModels.
    ///
    /// # Arguments
    ///
    /// * `basename` - The base name to be made unique.
    ///
    /// # Returns
    ///
    /// * A unique name based on the given basename.
    pub fn get_unique_name(&mut self, basename: String) -> String {
        let mut name = basename.clone();
        let mut copy = 1;
        while self.names.contains(&name) {
            name = format!("{}{}", basename, copy);
            copy += 1;
        }
        // Save the name for future duplicate detection
        self.names.push(name.clone());
        name
    }

    /// Converts the script buffer to a single string.
    ///
    /// This function concatenates the preamble, body, and postamble of the script buffer
    /// into a single string and returns it.
    ///
    /// # Returns
    ///
    /// * A `String` containing the entire script buffer content.
    pub fn to_string(&self) -> String {
        let mut script = String::new();
        script.push_str(&self.preamble);
        script.push_str(&self.body);
        script.push_str(&self.postamble);
        script
    }
}
