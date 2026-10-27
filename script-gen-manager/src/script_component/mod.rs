pub mod function;
pub mod script;

mod data_report;
mod finalize;
mod initialize;
mod sweep;

#[cfg(test)]
mod tests {
    use script_aggregator::script_buffer::ScriptBuffer;
    use script_templates::FunctionTemplate;

    use super::{
        data_report::DataReportModel, finalize::FinalizeModel, function::FunctionModel,
        initialize::InitializeModel,
    };
    use crate::model::sweep_data::sweep_config::SweepConfig;

    /// Renders a single function model, as it would appear in a script.
    fn render(model: &mut dyn FunctionModel) -> String {
        let mut script_buffer = ScriptBuffer::new();
        script_buffer.set_auto_indent(true);
        model
            .to_script(&SweepConfig::new(), &mut script_buffer)
            .unwrap();
        script_buffer.to_string()
    }

    fn template(type_: &str) -> FunctionTemplate {
        script_templates::load_templates()
            .unwrap()
            .into_iter()
            .find(|t| t.type_() == type_)
            .unwrap()
    }

    #[test]
    fn initialize() {
        let mut model = InitializeModel::new(template("Initialize"));
        assert_eq!(
            render(&mut model),
            include_str!("../../tests/expected/initialize.tsp")
        );
    }

    #[test]
    fn finalize() {
        let mut model = FinalizeModel::new(template("Finalize"));
        assert_eq!(
            render(&mut model),
            include_str!("../../tests/expected/finalize.tsp")
        );
    }

    #[test]
    fn data_report() {
        let mut model = DataReportModel::new(template("DataReport"));
        assert_eq!(
            render(&mut model),
            include_str!("../../tests/expected/data_report.tsp")
        );
    }
}
