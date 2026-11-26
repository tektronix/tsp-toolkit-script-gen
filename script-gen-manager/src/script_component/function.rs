use std::any::Any;

use script_aggregator::script_buffer::ScriptBuffer;
use script_templates::{error::Result, FunctionTemplate};
use serde::Serialize;

use crate::model::sweep_data::sweep_config::SweepConfig;

pub trait FunctionModel: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    /// Returns the underlying type of the function model (e.g., Initialize, Finalize, etc.)
    fn get_type(&self) -> &str;
    /// Returns the description of the function model.
    fn get_description(&self) -> &str;
    /// Returns the template used to render the function model.
    fn get_template(&self) -> &FunctionTemplate;
    /// Converts the function model to a script.
    ///
    /// # Errors
    /// Returns an error if the function's template could not be rendered.
    fn to_script(
        &mut self,
        sweep_config: &SweepConfig,
        script_buffer: &mut ScriptBuffer,
    ) -> Result<()>;

    /// Converts a description string into a Lua comment format.
    ///
    /// This function takes a multi-line description string and formats it as a Lua comment,
    /// with each line prefixed by `--`.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The description string to be converted.
    ///
    /// # Returns
    ///
    /// A formatted string representing the Lua comment.
    fn to_lua_comment(&self, descriptor: &str) -> String {
        let mut first = true;
        let mut returnval = String::with_capacity(100);

        for token in descriptor.lines() {
            if first {
                first = false;
            } else {
                returnval.push('\n');
            }
            returnval.push_str("-- ");
            returnval.push_str(token);
        }

        returnval
    }

    /// Appends relevant information before the function definition.
    ///
    /// This includes a header with the type of the function model and its description.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    fn start_chunk(&self, script_buffer: &mut ScriptBuffer) {
        script_buffer.body_append(
            "----------------------------------------------------------------------------",
        );
        script_buffer.body_append(&format!(
            "-- START OF {} SEGMENT ... do not modify this section ",
            self.get_type().to_uppercase()
        ));
        script_buffer.body_append(
            "----------------------------------------------------------------------------",
        );
        script_buffer.body_append(
            "--==========================================================================",
        );
        script_buffer.body_append(&self.to_lua_comment(self.get_description()));
        script_buffer.body_append(
            "--==========================================================================",
        );
    }

    /// Appends relevant information after the function definition.
    ///
    /// This includes a footer indicating the end of the function model segment.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    fn finish_chunk(&self, script_buffer: &mut ScriptBuffer) {
        script_buffer.body_append(
            "----------------------------------------------------------------------------",
        );
        script_buffer.body_append(&format!(
            "-- END OF {} SEGMENT ... do not modify code after this point",
            self.get_type().to_uppercase()
        ));
        script_buffer.body_append(
            "----------------------------------------------------------------------------\n",
        );
        script_buffer.body_append("");
    }

    /// Renders the function model template and appends it to the script buffer.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    /// * `context` - The values referenced by the template.
    ///
    /// # Errors
    /// Returns an error if the template could not be rendered with `context`.
    fn build<T: Serialize>(&self, script_buffer: &mut ScriptBuffer, context: &T) -> Result<()>
    where
        Self: Sized,
    {
        let code = self.get_template().render_body(context)?;
        // Insert one line at a time so the target script uses the correct EOL sequence.
        for line in code.lines() {
            script_buffer.body_append(line);
        }
        Ok(())
    }

    /// Formats a floating-point value.
    ///
    /// If the absolute value is greater than 0 and not within the range 0.1 to 1000.0,
    /// it uses scientific notation. Otherwise, it uses the default notation.
    ///
    /// # Arguments
    ///
    /// * `value` - The floating-point value to format.
    ///
    /// # Returns
    ///
    /// A formatted string representing the value.
    fn format(&self, value: f64) -> String {
        let temp = value.abs();
        if temp > 0.0 && !(0.1..=1000.0).contains(&temp) {
            format!("{value:e}") // Scientific notation
        } else {
            format!("{value}") // Default notation
        }
    }
}
