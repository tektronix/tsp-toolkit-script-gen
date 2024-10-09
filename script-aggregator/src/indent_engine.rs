#[derive(Debug)]
pub struct IndentEngine {
    indent: String,
    next_indent: String,
    step: String,
    step_size: u16,
    is_multi_line_comment: bool,
}

impl IndentEngine {
    pub fn new(step: String) -> Self {
        let step_size = step.chars().count() as u16;
        IndentEngine {
            indent: String::new(),
            next_indent: String::new(),
            step,
            step_size,
            is_multi_line_comment: false,
        }
    }

    /// Applies the indentation logic to the specified statement and appends it to the buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to append the indentation and statement to.
    /// * `statement` - The statement to process for increase/decrease indentation.
    pub fn apply(&mut self, buffer: &mut String, statement: &str) {
        let trimmed = statement.trim();
        self.pre_process(trimmed);
        buffer.push_str(&self.indent);
        buffer.push_str(trimmed);
        self.post_process();
    }

    /// Process a statement before it is added to the output and update the indentation accordingly.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement to be pre-processed.
    pub fn pre_process(&mut self, statement: &str) {
        // Starting multiline comment?
        if statement.contains("--[[") {
            self.is_multi_line_comment = true;
        }
        // Ending multiline comment?
        if self.is_multi_line_comment && statement.contains("]]") {
            self.is_multi_line_comment = false;
        }
        if !self.is_multi_line_comment {
            // Trim off comments
            let mut trimmed = statement.to_string();
            if let Some(index) = statement.find("--") {
                trimmed = statement[..index].to_string();
            }

            // Combination statements don't affect the indent...
            let has_do = self.find(&trimmed, "do");
            let has_end = self.find(&trimmed, "end");
            let has_repeat = self.find(&trimmed, "repeat");
            let has_then = self.find(&trimmed, "then");
            let has_until = self.find(&trimmed, "until");

            // A complete statement has no effect... Check for:
            //   [if] ... then ... [else...] end
            //   [for] ... do ... end
            //   repeat ... until
            let mut complete = has_then && has_end;
            complete |= has_do && has_end;
            complete |= has_repeat && has_until;
            if !complete {
                let has_else = self.find(&trimmed, "else");
                let has_else_if = self.find(&trimmed, "elseif");

                // These statements decrease the indent (before the statement is added) ...
                let mut decrease = has_end;
                decrease |= has_else;
                decrease |= has_else_if;
                decrease |= has_until;
                decrease |= trimmed.contains('}') && !trimmed.contains('{');
                if decrease && self.indent.len() >= self.step_size as usize {
                    self.indent = self.indent[self.step_size as usize..].to_string();
                }

                // These statements increase the indent (after the statement is added) ...
                self.next_indent.clone_from(&self.indent);
                let mut increase = self.find(&trimmed, "function");
                increase |= has_do;
                increase |= has_repeat;
                increase |= has_else;
                increase |= has_else_if;
                increase |= has_then;
                increase |= trimmed.contains('{') && !trimmed.contains('}');
                if increase {
                    self.next_indent = format!("{}{}", self.indent, self.step);
                }
            }
        }
    }

    /// Checks if a keyword is present in a statement without being part of a larger alphanumeric word.
    ///
    /// This function searches for the `keyword` within the `statement` and ensures that the keyword
    /// is not part of a larger alphanumeric word. It returns `true` if the keyword is found as a
    /// standalone word, and `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `statement` - The string in which to search for the keyword.
    /// * `keyword` - The keyword to search for.
    ///
    /// # Returns
    ///
    /// * `true` if the keyword is found as a standalone word.
    /// * `false` otherwise.
    fn find(&self, statement: &str, keyword: &str) -> bool {
        if let Some(index) = statement.find(keyword) {
            if index > 0
                && statement
                    .chars()
                    .nth(index - 1)
                    .map_or(false, |c| c.is_alphanumeric())
            {
                return false;
            }
            let end_index = index + keyword.len();
            if end_index < statement.len()
                && statement
                    .chars()
                    .nth(end_index)
                    .map_or(false, |c| c.is_alphanumeric())
            {
                return false;
            }
            return true;
        }
        false
    }

    /// Adjust the indentation after a statement has been added to the output.  
    /// This simply uses a value computed in preprocess.
    pub fn post_process(&mut self) {
        self.indent.clone_from(&self.next_indent);
    }
}
