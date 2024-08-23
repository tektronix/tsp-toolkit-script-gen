use crate::{catalog::Catalog, device_io::SimulatedDeviceIO};

#[derive(Debug, Clone)]
struct IdnResponse {
    model: String,
    fw_version: String,
    serial: String,
    description: String,
    //line_freq: f64,
}

impl IdnResponse {
    pub fn new(model: String, fw_version: String, serial: String, description: String) -> Self {
        IdnResponse {
            model,
            fw_version,
            serial,
            description,
        }
    }

    pub fn empty_idn_response() -> Self {
        IdnResponse {
            model: String::new(),
            fw_version: String::new(),
            serial: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SmuDevice {
    catalog: Catalog,

    node_id: String,
    smu_id: String,

    async_meas_supported: bool,
    fast_adc_supported: bool,
    analog_filter_supported: bool,

    idn_response: IdnResponse,
}

impl SmuDevice {
    pub fn new(id: String, catalog: Catalog) -> Self {
        let (node_id, smu_id) = SmuDevice::parse_id(id);
        let idn_response = IdnResponse::empty_idn_response();
        SmuDevice {
            catalog,
            node_id,
            smu_id,
            async_meas_supported: false,
            fast_adc_supported: false,
            analog_filter_supported: false,
            idn_response,
        }
    }

    fn parse_id(id: String) -> (String, String) {
        let node_id: String;
        let smu_id: String;

        let res: Vec<String> = id.split('.').map(|s| s.to_string()).collect();
        if res.len() > 0 {
            node_id = res[0].clone();
            smu_id = id;
        } else {
            node_id = String::from("localnode");
            smu_id = format!("{}.{}", node_id, id);
        }

        (node_id, smu_id)
    }

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
            let description = identify_res[3].clone();

            //Comment taken from TSP Express
            // NOTE: the response also includes line frequency -- but the system uses
            //       the line frequency of the connected node (i.e. localnode.linefreq)
            //let _ = identify_res[4].clone();

            self.idn_response = IdnResponse::new(model, fw_version, serial, description);
        }
    }

    pub fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    pub fn get_model(&self) -> String {
        self.idn_response.model.clone()
    }

    pub fn get_fw_version(&self) -> String {
        self.idn_response.fw_version.clone()
    }
}

// struct CompositeSmuDevice {
//     parallel_configuration: bool,
//     smu_devices: Vec<SmuDevice>,
// }
