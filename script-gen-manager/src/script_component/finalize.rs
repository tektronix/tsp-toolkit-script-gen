use std::any::Any;

use xml_handler::group::Group;

use super::FunctionModel;

#[derive(Debug)]
pub struct FinalizeModel {
    type_: String,
    description: String,
    metadata: Group,
}

impl FunctionModel for FinalizeModel {
    fn set_type(&mut self, type_: String) {
        self.type_ = type_;
    }

    fn get_type(&self) -> String {
        self.type_.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_script(&mut self) {
        //TODO!
        //no replacements to be done for finalize snippet, just call the
        //script aggregator with indentation handled
        todo!();
    }
}

impl FinalizeModel {
    pub fn new(group: Group) -> Self {
        FinalizeModel {
            type_: group.type_.clone(),
            description: String::from(
                "The function completes the script and places the instrument in a known state.",
            ),
            metadata: group,
        }
    }
}
