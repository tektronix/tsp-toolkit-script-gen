use script_gen_manager::{
    device_io::SimulatedDeviceIO,
    device_manager::DeviceManager,
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
    Ok(())
}

// returning simulated IO only for now
fn get_initial_path() -> SimulatedDeviceIO {
    SimulatedDeviceIO::new()
}
