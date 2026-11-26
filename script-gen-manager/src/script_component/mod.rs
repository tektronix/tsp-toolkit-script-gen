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

    fn check(actual: &str, expected: &str) {
        assert!(
            !actual.contains('\r'),
            "generated script must use \\n line endings"
        );
        assert!(
            !expected.contains('\r'),
            "expected script must use \\n line endings (see .gitattributes)"
        );
        assert_eq!(actual, expected);
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
        check(
            &render(&mut model),
            include_str!("../../tests/expected/initialize.tsp"),
        );
    }

    #[test]
    fn finalize() {
        let mut model = FinalizeModel::new(template("Finalize"));
        check(
            &render(&mut model),
            include_str!("../../tests/expected/finalize.tsp"),
        );
    }

    #[test]
    fn data_report() {
        let mut model = DataReportModel::new(template("DataReport"));
        check(
            &render(&mut model),
            include_str!("../../tests/expected/data_report.tsp"),
        );
    }
}
