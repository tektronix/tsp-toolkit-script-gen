use std::fmt;

use crate::{
    instr_metadata::{
        base_metadata::{BaseMetadata, MODEL_MAP},
        enum_metadata::MetadataEnum,
        mpsu50_metadata::Mpsu50Metadata,
        msmu60_metadata::Msmu60Metadata,
    },
    model::mainframe::Slot,
};
use serde::{Deserialize, Deserializer, Serialize};
// use tsp_toolkit_kic_lib::instrument::info::InstrumentInfo;

/// Represents the type of device in the mainframe slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Smu,
    Psu,
    Unknown,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Smu => write!(f, "smu"),
            DeviceType::Psu => write!(f, "psu"),
            DeviceType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Represents a device in one of the mainframe slots.
#[derive(Debug, Clone, Serialize)]
pub struct Device {
    node_id: String,
    pub _id: String,
    pub device_type: DeviceType,

    model: String,
    fw_version: String,
    pub in_use: bool,

    #[serde(skip)]
    pub metadata: MetadataEnum,
}

impl<'de> Deserialize<'de> for Device {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeviceData {
            node_id: String,
            _id: String,
            device_type: DeviceType,
            model: String,
            fw_version: String,
            in_use: bool,
        }

        let device_data = DeviceData::deserialize(deserializer)?;

        let metadata = match device_data.device_type {
            DeviceType::Smu => MetadataEnum::Msmu60(Msmu60Metadata::new()),
            DeviceType::Psu => MetadataEnum::Mpsu50(Mpsu50Metadata::new()),
            DeviceType::Unknown => MetadataEnum::Base(BaseMetadata::default()),
        };

        Ok(Device {
            node_id: device_data.node_id,
            _id: device_data._id,
            device_type: device_data.device_type,
            model: device_data.model,
            fw_version: device_data.fw_version,
            in_use: device_data.in_use,
            metadata,
        })
    }
}

impl Default for Device {
    fn default() -> Self {
        Device {
            node_id: String::new(),
            _id: String::new(),
            device_type: DeviceType::Unknown,

            model: String::new(),
            fw_version: String::new(),
            in_use: false,
            metadata: MetadataEnum::Base(BaseMetadata::default()),
        }
    }
}

impl Device {
    /// Creates a new `Device` instance.
    ///
    /// # Arguments
    ///
    /// * `mainframe_name` - name representing Trebuchet mainframe (e.g., localnode, node[37] etc).
    /// * `mainframe_model` - model number of the mainframe.
    /// * `slot` - slot information.
    /// * `channel_id` - channel ID of slot.
    ///
    /// # Returns
    ///
    /// A new `Device` instance.
    pub fn new(
        mainframe_name: String,
        mainframe_model: String,
        slot: &Slot,
        channel_id: i32,
    ) -> Self {
        let device_type = match MODEL_MAP.get(&slot.model) {
            Some(&"Smu") => DeviceType::Smu,
            Some(&"Psu") => DeviceType::Psu,
            _ => DeviceType::Unknown, // Handle unknown device types
        };
        let (node_id, _id) = Device::parse_id(mainframe_name, slot, channel_id, &device_type);
        let metadata = match device_type {
            DeviceType::Smu => MetadataEnum::Msmu60(Msmu60Metadata::new()),
            DeviceType::Psu => MetadataEnum::Mpsu50(Mpsu50Metadata::new()),
            DeviceType::Unknown => MetadataEnum::Base(BaseMetadata::default()),
        };
        Device {
            node_id,
            _id,
            device_type,

            model: slot.model.clone(),
            fw_version: String::new(),
            in_use: false,
            metadata,
        }
    }

    /// Parses the input string into node ID and ID.
    ///
    /// # Arguments
    ///
    /// * `mainframe_name` - name representing Trebuchet mainframe (e.g., localnode, node[37] etc).
    /// * `slot` - slot information.
    /// * `channel_id` - channel ID of slot.
    ///
    /// # Returns
    ///
    /// A tuple containing the node ID and ID.
    /// e.g., if mainframe_name = "node[37]", slot.name = slot[1], id = 1 and device_type = Smu
    /// the function returns ("node[37]", "node[37].slot[1].smu[1]").
    fn parse_id(
        mainframe_name: String,
        slot: &Slot,
        id: i32,
        device_type: &DeviceType,
    ) -> (String, String) {
        let node_id = format!("{}", mainframe_name);
        let chan_id = format!("{}[{}]", device_type, id);
        let _id = format!("{}.{}.{}", mainframe_name, slot.name, chan_id);

        (node_id, _id)
    }

    /// Returns the ID of the device.
    ///
    /// # Returns
    ///
    /// A string representing the ID.
    pub fn get_id(&self) -> String {
        self._id.clone()
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
        self.model.clone()
    }

    /// Returns the firmware version of the device.
    ///
    /// # Returns
    ///
    /// A string representing the firmware version. If the firmware version is not available, returns "Unknown Firmware Version".
    pub fn get_fw_version(&self) -> String {
        self.fw_version.clone()
    }

    /// Returns metadata associated with this device type
    pub fn get_metadata(&self) -> MetadataEnum {
        self.metadata.clone()
    }
}

// struct CompositeSmuDevice {
//     parallel_configuration: bool,
//     smu_devices: Vec<SmuDevice>,
// }
