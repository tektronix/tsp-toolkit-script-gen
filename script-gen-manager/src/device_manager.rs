use crate::{catalog::Catalog, device::SmuDevice, device_io::SimulatedDeviceIO};

pub struct DeviceManager {
    path: SimulatedDeviceIO,
    catalog: Catalog,
    device_list: Vec<SmuDevice>
}

impl DeviceManager {
    pub fn new(path: SimulatedDeviceIO) -> Self {
        let catalog = Catalog::new();

        // Initialize devices as an empty vector, only non-composite smu devices for now
        let device_list = Vec::new();
        DeviceManager { path, catalog, device_list }
    }

    pub fn search(&mut self) {
        let search_response = self.path.get_query_response("SEARCH".to_string());
        let instruments: Vec<String> = search_response.split(',').map(|s| s.to_string()).collect();
        for instr in instruments {
            let mut device = SmuDevice::new(instr, self.catalog.clone());
            device.determine_attributes(self.path.clone());
            self.device_list.push(device);
        }
    }
}
