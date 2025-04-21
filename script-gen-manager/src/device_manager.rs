use crate::{device::Device, model::mainframe::Mainframe};

/// Manages device search, storage, and retrieval.
#[derive(Debug, Clone)]
pub struct DeviceManager {
    pub device_list: Vec<Device>,
}

impl DeviceManager {
    pub fn new() -> Self {
        // Initialize devices as an empty vector, only non-composite smu devices for now
        let device_list = Vec::new();
        DeviceManager { device_list }
    }

    pub fn create_device_list(&mut self, instr_list: String) {
        let res = serde_json::from_str(&instr_list);
        match res {
            Ok(res) => {
                if let Ok(mainframe_list) = serde_json::from_value::<Vec<Mainframe>>(res) {
                    for mainframe in mainframe_list {
                        for slot in mainframe.slot {
                            for i in 1..=2 {
                                let device = Device::new(
                                    mainframe.name.clone(),
                                    mainframe.model.clone(),
                                    &slot,
                                    i,
                                );
                                self.device_list.push(device);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
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
    pub fn get_device(&self, index: usize) -> &Device {
        &self.device_list[index]
    }

    pub fn get_device_ids(&self) -> Vec<String> {
        self.device_list
            .iter()
            .map(|device| device._id.clone())
            .collect()
    }
}
