use std::any::Any;

use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::composite::CommonChunk;

pub trait FunctionModel {
    fn as_any(&self) -> &dyn Any;
    /// Returns the underlying type of the function model (e.g., Initialize, Finalize, etc.)
    fn get_type(&self) -> &str;
    /// Returns the description of the function model.
    fn get_description(&self) -> &str;
    /// Returns the value replacement map containing the key-value pairs used for substitution.
    fn get_val_replacement_map(&self) -> &std::collections::HashMap<String, String>;
    /// Returns the metadata (parsed Group information) of the function model.
    fn get_metadata(&self) -> &xml_handler::group::Group;
    /// Converts the function model to a script.
    fn to_script(&mut self, script_buffer: &mut ScriptBuffer);

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
            if !first {
                returnval.push('\n');
            } else {
                first = false;
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
            "----------------------------------------------------------------------------"
                .to_owned(),
        );
        script_buffer.body_append(format!(
            "-- START OF {} SEGMENT ... do not modify this section ",
            self.get_type().to_uppercase()
        ));
        script_buffer.body_append(
            "----------------------------------------------------------------------------"
                .to_owned(),
        );
        script_buffer.body_append(
            "--=========================================================================="
                .to_owned(),
        );
        script_buffer.body_append(self.to_lua_comment(self.get_description()));
        script_buffer.body_append(
            "--=========================================================================="
                .to_owned(),
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
            "----------------------------------------------------------------------------"
                .to_owned(),
        );
        script_buffer.body_append(format!(
            "-- END OF {} SEGMENT ... do not modify code after this point",
            self.get_type().to_uppercase()
        ));
        script_buffer.body_append(
            "----------------------------------------------------------------------------\n"
                .to_owned(),
        );
        script_buffer.body_append("".to_owned());
    }

    /// Builds the function model script and appends it to the script buffer.
    ///
    /// This method generates a unique function name, appends the start chunk, processes the metadata,
    /// and appends the end chunk to the script buffer.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    fn build(&mut self, script_buffer: &mut ScriptBuffer) {
        let chunk_name = script_buffer.get_unique_name(format!("_{}", self.get_type()));
        self.start_chunk(script_buffer);

        script_buffer.body_append(format!("function {}()\n", chunk_name));
        script_buffer.change_indent(ScriptBuffer::DEFAULT_INDENT);

        let metadata = self.get_metadata();
        let val_replacement_map = self.get_val_replacement_map();

        let mut metadata = metadata.clone();
        for child in metadata.children.iter_mut() {
            if let xml_handler::group::IncludeResult::Composite(comp) = child {
                //not aux type
                if comp.type_.is_none() {
                    comp.to_script(script_buffer, val_replacement_map);
                }
            }
        }

        script_buffer.change_indent(-ScriptBuffer::DEFAULT_INDENT);
        script_buffer.body_append("end".to_owned());

        self.finish_chunk(script_buffer);
        script_buffer.postamble_append(format!("{}()", chunk_name));
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
            format!("{:e}", value) // Scientific notation
        } else {
            format!("{}", value) // Default notation
        }
    }
}
