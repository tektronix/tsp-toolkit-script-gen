use std::cmp;

#[derive(Debug)]
pub struct ScriptBuffer {
    auto_indent: bool,
    indent: String,
    indent_count: i32,
}

impl ScriptBuffer {
    pub const MAXIMUM_INDENT: &'static str = "                    "; // 20 spaces
    pub const DEFAULT_INDENT: i32 = 4;

    pub fn new() -> Self {
        ScriptBuffer {
            auto_indent: false,
            indent: String::new(),
            indent_count: 0,
        }
    }

    pub fn set_auto_indent(&mut self, auto_indent: bool) {
        self.auto_indent = auto_indent;
    }

    pub fn change_indent(&mut self, val: i32) {
        self.set_indent(cmp::max(
            0,
            cmp::min(
                ScriptBuffer::MAXIMUM_INDENT.chars().count() as i32,
                self.indent_count + val,
            ),
        ));
    }

    pub fn set_indent(&mut self, val: i32) {
        if val == 0 {
            self.indent_count = 0;
            self.indent = String::new();
        } else if val > 0 && val <= ScriptBuffer::MAXIMUM_INDENT.chars().count() as i32 {
            self.indent_count = val;
            self.indent = ScriptBuffer::MAXIMUM_INDENT
                .chars()
                .take(val as usize)
                .collect();
        }
    }
}
