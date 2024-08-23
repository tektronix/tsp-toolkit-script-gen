use std::any::Any;

pub trait FunctionModel {
    fn set_type(&mut self, type_: String);
    fn get_type(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn to_script(&mut self);
}
