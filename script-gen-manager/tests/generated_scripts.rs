//! Renders complete scripts for a set of representative sweep configurations and compares
//! them against the expected output stored in `tests/expected/`.
//!
//! Set `UPDATE_EXPECTED=1` to regenerate the expected output after an intentional change.

use std::path::PathBuf;

use script_gen_manager::{
    catalog::Catalog,
    instr_metadata::base_metadata::BaseMetadata,
    model::sweep_data::{parameters::ParameterFloat, sweep_config::SweepConfig},
    script_component::script::ScriptModel,
};

const LOCALNODE_SYSTEM: &str = r#"{
    "systems": [{
        "name": "System 1",
        "isActive": true,
        "localNode": "MP5103",
        "slots": [
            { "slotId": "slot[1]", "module": "MSMU60-2" },
            { "slotId": "slot[2]", "module": "MPSU50-2" },
            { "slotId": "slot[3]", "module": "MSMU200-2" }
        ],
        "nodes": null
    }]
}"#;

const REMOTE_NODE_SYSTEM: &str = r#"{
    "systems": [{
        "name": "System 1",
        "isActive": true,
        "localNode": "2450",
        "slots": null,
        "nodes": [{
            "nodeId": "node[2]",
            "mainframe": "MP5103",
            "slots": [
                { "slotId": "slot[1]", "module": "MSMU60-2" },
                { "slotId": "slot[2]", "module": "Empty" },
                { "slotId": "slot[3]", "module": "MSMU60-2" }
            ]
        }]
    }]
}"#;

fn new_config(system_info: &str) -> SweepConfig {
    let mut config = SweepConfig::new();
    assert!(config.create_device_list(system_info));
    config
}

fn render(name: &str, config: &SweepConfig) -> String {
    let mut catalog = Catalog::new();
    catalog.refresh_function_metadata();

    let mut script_model = ScriptModel::new(catalog);
    script_model.initialize_scripts();
    script_model.add_sweep();
    script_model.add_data_report();

    let dir = std::env::temp_dir().join(format!("script-gen-test-{}-{name}", std::process::id()));
    let path = dir.join("script.tsp");
    script_model.to_script(config, path.to_str().unwrap());
    let script = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(dir);
    script
}

fn check(name: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("expected")
        .join(format!("{name}.tsp"));
    if std::env::var_os("UPDATE_EXPECTED").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    assert!(
        expected == actual,
        "generated script for '{name}' does not match {}",
        path.display()
    );
}

#[test]
fn no_channels() {
    let config = new_config(LOCALNODE_SYSTEM);
    check("no_channels", &render("no_channels", &config));
}

#[test]
fn auto_configured_step_sweep_bias() {
    let mut config = new_config(LOCALNODE_SYSTEM);
    config.auto_configure();
    config.evaluate();
    check(
        "auto_configured_step_sweep_bias",
        &render("auto_configured_step_sweep_bias", &config),
    );
}

#[test]
fn list_sweep_with_iv_measurement_and_aperture() {
    let mut config = new_config(LOCALNODE_SYSTEM);
    config.add_channel("sweep");
    config.evaluate();

    config.sweep_global_parameters.list_sweep = true;
    config.sweep_global_parameters.sweep_points.value = 3;
    let sweep = &mut config.sweep_channels[0].start_stop_channel;
    sweep.common_chan_attributes.meas_function.value = BaseMetadata::FUNCTION_IV.to_string();
    sweep.list = [0.5, 1.0, 1.5, 2.0]
        .iter()
        .map(|v| ParameterFloat::new("list", *v, None))
        .collect();

    let timing = &mut config.global_parameters.sweep_timing_config.smu_timing;
    timing.nplc_type.value = String::from("Aperture");
    timing.aperture.value = 1e-4;
    timing.source_auto_delay.value = BaseMetadata::OFF_VALUE.to_string();
    timing.source_delay.value = 2e-3;
    timing.measure_auto_delay.value = BaseMetadata::OFF_VALUE.to_string();
    timing.measure_delay.value = 5e-3;

    check(
        "list_sweep_with_iv_measurement_and_aperture",
        &render("list_sweep_with_iv_measurement_and_aperture", &config),
    );
}

#[test]
fn multiple_biases_and_log_sweeps() {
    let mut config = new_config(LOCALNODE_SYSTEM);
    config.add_channel("bias");
    config.add_channel("bias");
    config.add_channel("step");
    config.add_channel("sweep");
    config.add_channel("sweep");
    config.add_channel("sweep");
    config.evaluate();

    config.step_global_parameters.list_step = true;
    config.step_channels[0].start_stop_channel.list = [1.0, 2.0]
        .iter()
        .map(|v| ParameterFloat::new("list", *v, None))
        .collect();
    for sweep in &mut config.sweep_channels {
        let sweep = &mut sweep.start_stop_channel;
        sweep.style.value = BaseMetadata::STYLE_LOG.to_string();
        sweep.start.value = 1e-3;
        sweep.stop.value = 10.0;
        sweep.common_chan_attributes.meas_function.value =
            BaseMetadata::FUNCTION_VOLTAGE.to_string();
    }
    config.bias_channels[1]
        .common_chan_attributes
        .meas_function
        .value = BaseMetadata::FUNCTION_IV.to_string();

    check(
        "multiple_biases_and_log_sweeps",
        &render("multiple_biases_and_log_sweeps", &config),
    );
}

#[test]
fn remote_node_sweep() {
    let mut config = new_config(REMOTE_NODE_SYSTEM);
    config.auto_configure();
    config.evaluate();
    check("remote_node_sweep", &render("remote_node_sweep", &config));
}
