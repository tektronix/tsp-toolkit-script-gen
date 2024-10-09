use std::any::Any;

use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::composite::CommonChunk;

pub trait FunctionModel {
    fn as_any(&self) -> &dyn Any;
    fn get_type(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_val_replacement_map(&self) -> &std::collections::HashMap<String, String>;
    fn get_metadata(&self) -> &xml_handler::group::Group;
    fn to_script(&mut self, script_buffer: &mut ScriptBuffer);

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

    fn format(&self, value: f64) -> String {
        let temp = value.abs();
        if temp > 0.0 && !(0.1..=1000.0).contains(&temp) {
            format!("{:e}", value) // Scientific notation
        } else {
            format!("{}", value) // Default notation
        }
    }

    fn aux_build(
        &mut self,
        script_buffer: &mut ScriptBuffer,
        comp: &mut xml_handler::composite::Composite,
    ) {
        script_buffer.change_indent(ScriptBuffer::DEFAULT_INDENT);
        comp.to_script(script_buffer, self.get_val_replacement_map());
        script_buffer.change_indent(-ScriptBuffer::DEFAULT_INDENT);
    }
}
