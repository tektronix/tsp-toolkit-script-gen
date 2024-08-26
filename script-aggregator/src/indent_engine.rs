#[derive(Debug)]
struct IndentEngine {
    indent: String,
    next_indent: String,
    step: String,
    step_size: u16,
    is_multi_line_comment: bool,
}

impl IndentEngine {
    fn new(step: &String) -> Self {
        IndentEngine {
            indent: String::new(),
            next_indent: String::new(),
            step: step.to_string(),
            step_size: step.chars().count() as u16,
            is_multi_line_comment: false,
        }
    }
}
