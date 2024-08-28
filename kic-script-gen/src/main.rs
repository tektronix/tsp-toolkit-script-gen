use script_gen_manager::{
    device_io::SimulatedDeviceIO, device_manager::DeviceManager, script_component::ScriptModel,
};

fn main() -> anyhow::Result<()> {
    eprintln!("Welcome to TSP script generator");

    run()?;

    Ok(())
}

fn run() -> anyhow::Result<()> {
    eprintln!("Running the script generator...");

    let initial_path = get_initial_path();
    let mut device_manager = DeviceManager::new(initial_path);
    device_manager.search();

    let mut script_model = ScriptModel::new(device_manager);
    script_model.initialize_scripts();
    script_model.to_script();

    Ok(())
}

// returning simulated IO only for now
fn get_initial_path() -> SimulatedDeviceIO {
    SimulatedDeviceIO::new()
}
