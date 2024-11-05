use crate::{catalog::Catalog, device::SmuDevice, device_io::SimulatedDeviceIO};

/// Manages device search, storage, and retrieval.
#[derive(Debug, Clone)]
pub struct DeviceManager {
    path: SimulatedDeviceIO,
    pub catalog: Catalog,
    pub device_list: Vec<SmuDevice>,
    line_frequency: i32,
}

impl DeviceManager {
    pub fn new(path: SimulatedDeviceIO) -> Self {
        let catalog = Catalog::new();

        // Initialize devices as an empty vector, only non-composite smu devices for now
        let device_list = Vec::new();
        DeviceManager {
            path,
            catalog,
            device_list,
            line_frequency: 60,
        }
    }

    /// Retrieves the line frequency.
    ///
    /// # Returns
    ///
    /// The line frequency as an integer.
    pub fn get_line_frequency(&self) -> i32 {
        self.line_frequency
    }

    /// Searches for devices using simulated device IO query-response mechanism
    /// and populates the device list.
    pub fn search(&mut self) {
        let search_response = self.path.get_query_response("SEARCH".to_string());
        let instruments: Vec<String> = search_response.split(',').map(|s| s.to_string()).collect();
        for instr in instruments {
            let mut device = SmuDevice::new(instr, self.catalog.clone());
            device.determine_attributes(self.path.clone());
            self.device_list.push(device);
        }
    }

    /// Retrieves a device based on input index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the device in the device list.
    ///
    /// # Returns
    ///
    /// A reference to the `SmuDevice` at the specified index.
    pub fn get_device(&self, index: usize) -> &SmuDevice {
        &self.device_list[index]
    }
}
