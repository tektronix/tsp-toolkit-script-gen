use crate::{
    device::Device,
    model::system_info::{Root, Slot},
};

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

    pub fn create_device_list(&mut self, system_info: &str) -> bool {
        let mut is_device_found = false;
        let res = serde_json::from_str::<Root>(system_info);

        if let Ok(root) = res {
            for system in root.systems {
                if !system.is_active {
                    continue;
                }

                // Helper closure to process slots
                let mut process_slots =
                    |node_id: String, mainframe: String, slots_opt: &Option<Vec<Slot>>| -> bool {
                        if let Some(slots) = slots_opt {
                            let valid_slots = slots.iter().filter(|slot| slot.module != "Empty");
                            let mut found = false;
                            for slot in valid_slots {
                                for i in 1..=2 {
                                    let device =
                                        Device::new(node_id.clone(), mainframe.clone(), slot, i);
                                    self.device_list.push(device);
                                    is_device_found = true;
                                    found = true;
                                }
                            }
                            return found;
                        }
                        false
                    };

                // Try localnode first
                let found_local = if system.local_node == "MP5103" {
                    let found = process_slots(
                        String::from("localnode"),
                        system.local_node.clone(),
                        &system.slots,
                    );
                    if !found {
                        println!("All modules are empty in localnode. Checking nodes...");
                    }
                    found
                } else {
                    false
                };

                // If not found in localnode, check nodes
                if !found_local {
                    if let Some(nodes) = &system.nodes {
                        for node in nodes {
                            if node.mainframe != "MP5103" {
                                println!("Node {} is not MP5103, skipping.", node.node_id);
                                continue;
                            }
                            let found = process_slots(
                                node.node_id.clone(),
                                node.mainframe.clone(),
                                &node.slots,
                            );
                            if found {
                                break;
                            } else {
                                println!(
                                    "All modules are empty in node {}. Skipping.",
                                    node.node_id
                                );
                            }
                        }
                    } else {
                        println!("No nodes found in the system info.");
                    }
                }
            }
        } else if let Err(e) = res {
            println!("Error: {:#?}", e);
        }

        is_device_found
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
