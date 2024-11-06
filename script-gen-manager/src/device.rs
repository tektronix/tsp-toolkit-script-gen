use tsp_toolkit_kic_lib::instrument::info::InstrumentInfo;

use crate::{catalog::Catalog, device_io::SimulatedDeviceIO};

/// Represents a Source Measure Unit (SMU) device.
#[derive(Debug, Clone)]
pub struct SmuDevice {
    catalog: Catalog,

    node_id: String,
    smu_id: String,

    async_meas_supported: bool,
    fast_adc_supported: bool,
    analog_filter_supported: bool,

    instr_idn_info: InstrumentInfo,
}

impl SmuDevice {
    /// Creates a new `SmuDevice` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - <node_id>.<smu_id> string representing the device.
    /// * `catalog` - a cache of metadata generated from resources in the application (e.g., XML files, structs etc).
    ///
    /// # Returns
    ///
    /// A new `SmuDevice` instance.
    pub fn new(id: String, catalog: Catalog) -> Self {
        let (node_id, smu_id) = SmuDevice::parse_id(id);
        SmuDevice {
            catalog,
            node_id,
            smu_id,
            async_meas_supported: false,
            fast_adc_supported: false,
            analog_filter_supported: false,
            instr_idn_info: InstrumentInfo::default(),
        }
    }

    /// Parses the input string into node ID and SMU ID.
    ///
    /// # Arguments
    ///
    /// * `id` - input string.
    ///
    /// # Returns
    ///
    /// A tuple containing the node ID and SMU ID.
    /// e.g., if id = "node[37].smua", the function returns ("node[37]", "node[37].smua").
    fn parse_id(id: String) -> (String, String) {
        let node_id: String;
        let smu_id: String;

        let res: Vec<String> = id.split('.').map(|s| s.to_string()).collect();
        if !res.is_empty() {
            node_id = res[0].clone();
            smu_id = id;
        } else {
            node_id = String::from("localnode");
            smu_id = format!("{}.{}", node_id, id);
        }

        (node_id, smu_id)
    }

    /// Determines the attributes of the SMU device using simulated device IO query-response mechanism.
    ///
    /// # Arguments
    ///
    /// * `path` - A `SimulatedDeviceIO` instance used to query the device info.
    pub fn determine_attributes(&mut self, path: SimulatedDeviceIO) {
        self.async_meas_supported = false;
        self.fast_adc_supported = false;
        self.analog_filter_supported = false;

        // Try to get the response for "IDENTIFY_{node_id}"
        let identify_response = path.get_query_response(format!("IDENTIFY_{}", self.node_id));

        // If the response is empty, try to get the response for "IDENTIFY" - for localnode
        let identify_response = if identify_response.is_empty() {
            path.get_query_response("IDENTIFY".to_string())
        } else {
            identify_response
        };

        let identify_res: Vec<String> = identify_response
            .split(',')
            .map(|s| s.to_string())
            .collect();

        if identify_res.len() > 4 {
            let model = identify_res[0].clone();
            let fw_version = identify_res[1].clone();
            let serial = identify_res[2].clone();

            //Comment taken from TSP Express
            // NOTE: the response also includes line frequency -- but the system uses
            //       the line frequency of the connected node (i.e. localnode.linefreq)
            //let description = identify_res[3].clone();
            //let _ = identify_res[4].clone();

            //self.idn_response = IdnResponse::new(model, fw_version, serial, description);
            self.instr_idn_info.model = Some(model);
            self.instr_idn_info.firmware_rev = Some(fw_version);
            self.instr_idn_info.serial_number = Some(serial);
        }
    }

    /// Returns the SMU ID of the device.
    ///
    /// # Returns
    ///
    /// A string representing the SMU ID.
    pub fn get_id(&self) -> String {
        self.smu_id.clone()
    }

    /// Returns the node ID of the device.
    ///
    /// # Returns
    ///
    /// A string representing the node ID.
    pub fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    /// Returns the model number of the device.
    ///
    /// # Returns
    ///
    /// A string representing the model number. If the model is not available, returns "Unknown Model".
    pub fn get_model(&self) -> String {
        if let Some(model) = &self.instr_idn_info.model {
            model.clone()
        } else {
            String::from("Unknown Model")
        }
    }

    /// Returns the firmware version of the device.
    ///
    /// # Returns
    ///
    /// A string representing the firmware version. If the firmware version is not available, returns "Unknown Firmware Version".
    pub fn get_fw_version(&self) -> String {
        if let Some(fw_version) = &self.instr_idn_info.firmware_rev {
            fw_version.clone()
        } else {
            String::from("Unknown Firmware Version")
        }
    }
}

// struct CompositeSmuDevice {
//     parallel_configuration: bool,
//     smu_devices: Vec<SmuDevice>,
// }
