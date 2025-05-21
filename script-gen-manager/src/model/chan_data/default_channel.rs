use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    device::{Device, DeviceType},
    instr_metadata::{
        base_metadata::{BaseMetadata, Metadata},
        enum_metadata::MetadataEnum,
    },
    model::sweep_data::parameters::{ParameterFloat, ParameterString},
};

use super::{channel_range::ChannelRange, region_map::RegionMapMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonChanAttributes {
    pub uuid: String,
    pub chan_name: String,
    pub source_function: ParameterString,
    pub meas_function: ParameterString,
    pub source_range: ChannelRange,
    meas_range: ChannelRange,
    pub source_limiti: ParameterFloat,
    pub source_limitv: Option<ParameterFloat>,
    pub sense_mode: Option<ParameterString>,

    #[serde(skip)]
    pub device: Device,
    pub device_id: String,
}

impl CommonChanAttributes {
    pub fn new(chan_name: String, device: Device) -> Self {
        let device_id = &device.get_id();

        CommonChanAttributes {
            uuid: Uuid::new_v4().to_string(),
            chan_name,
            source_function: ParameterString::new("source_function"),
            meas_function: ParameterString::new("meas_function"),
            source_range: ChannelRange::new(),
            meas_range: ChannelRange::new(),
            source_limiti: ParameterFloat::new(
                "source_limiti",
                1e-1,
                Some(BaseMetadata::UNIT_AMPERES.to_string()),
            ),
            source_limitv: None,
            sense_mode: None,

            device,
            device_id: device_id.to_string(),
        }
    }

    pub fn set_defaults(&mut self) {
        match self.device.device_type {
            DeviceType::Smu => {
                self.source_function.range = vec![
                    BaseMetadata::FUNCTION_VOLTAGE.to_string(),
                    BaseMetadata::FUNCTION_CURRENT.to_string(),
                ];
                self.sense_mode = self.initialize_sense_mode();
                self.source_limitv = Some(ParameterFloat::new(
                    "source_limitv",
                    20.0,
                    Some(BaseMetadata::UNIT_VOLTS.to_string()),
                ));
            }
            DeviceType::Psu => {
                self.source_function.range = vec![BaseMetadata::FUNCTION_VOLTAGE.to_string()];
            }
            DeviceType::Unknown => {
                //todo: handle error
                println!("Unknown device type");
            }
        }
        self.source_function.value = BaseMetadata::FUNCTION_VOLTAGE.to_string();

        self.meas_function.range = vec![
            BaseMetadata::FUNCTION_VOLTAGE.to_string(),
            BaseMetadata::FUNCTION_CURRENT.to_string(),
            BaseMetadata::FUNCTION_IV.to_string(),
        ];
        self.meas_function.value = BaseMetadata::FUNCTION_CURRENT.to_string();

        let device_metadata = self.device.get_metadata();
        self.set_source_range(&device_metadata);
        self.set_meas_range(&device_metadata);
        self.set_source_range_value();
        self.set_meas_range_value();
    }

    pub fn evaluate(&mut self) {
        self.evaluate_source_function();
        self.evaluate_measure_function();
    }

    fn evaluate_source_function(&mut self) {
        let device_metadata = self.device.get_metadata();
        self.set_source_range(&device_metadata);
        self.set_source_range_limits(&device_metadata);
        self.set_source_range_value();
    }

    fn set_source_range(&mut self, metadata: &MetadataEnum) {
        self.source_range.unit = self.determine_units(&self.source_function.value);
        if self.source_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
            self.source_range.range = self.get_range(metadata, "source_meas.rangev");
        } else {
            self.source_range.range = self.get_range(metadata, "source_meas.rangei");
        }
    }

    fn set_source_range_limits(&mut self, metadata: &MetadataEnum) {
        let key = if self.source_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
            "source.levelv"
        } else {
            "source.leveli"
        };

        if let Some((min, max)) = self.get_range_limits(metadata, key) {
            self.source_range.set_min(min);
            self.source_range.set_max(max);
        }
    }

    fn set_source_range_value(&mut self) {
        if !self.source_range.range.contains(&self.source_range.value) {
            let key = if self.source_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
                "source_meas.range.defaultv"
            } else {
                "source_meas.range.defaulti"
            };
            if let Some(default_value) = self.get_range_defaults(&self.device.get_metadata(), key) {
                self.source_range.value = default_value.to_string();
            }
        }
    }

    fn evaluate_measure_function(&mut self) {
        if self.meas_function.value == self.source_function.value {
            self.meas_range.range = self.source_range.range.clone();
            self.meas_range.value = self.source_range.value.clone();
        } else {
            let device_metadata = self.device.get_metadata();
            self.set_meas_range(&device_metadata);
            self.set_meas_range_value();
        }
    }

    fn set_meas_range(&mut self, metadata: &MetadataEnum) {
        if self.meas_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
            self.meas_range.range = self.get_range(metadata, "source_meas.rangev");
        } else {
            self.meas_range.range = self.get_range(metadata, "source_meas.rangei");
        }
    }

    fn set_meas_range_value(&mut self) {
        if !self.meas_range.range.contains(&self.meas_range.value) {
            let key = if self.meas_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
                "source_meas.range.defaultv"
            } else {
                "source_meas.range.defaulti"
            };
            if let Some(default_value) = self.get_range_defaults(&self.device.get_metadata(), key) {
                self.meas_range.value = default_value.to_string();
            }
        }
    }

    fn determine_units(&self, function_name: &String) -> String {
        if *function_name == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
            BaseMetadata::UNIT_VOLTS.to_string()
        } else {
            BaseMetadata::UNIT_AMPERES.to_string()
        }
    }

    fn get_range(&self, metadata: &MetadataEnum, key: &str) -> Vec<String> {
        match metadata {
            MetadataEnum::Base(base_metadata) => {
                // Handle base_metadata if needed
                vec![]
            }
            MetadataEnum::Msmu60(msmu60_metadata) => msmu60_metadata
                .get_option(key)
                .unwrap_or(&vec![])
                .iter()
                .map(|s| s.to_string())
                .collect(),
            MetadataEnum::Mpsu50(mpsu50_metadata) => mpsu50_metadata
                .get_option(key)
                .unwrap_or(&vec![])
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    fn get_range_limits(&self, metadata: &MetadataEnum, key: &str) -> Option<(f64, f64)> {
        match metadata {
            MetadataEnum::Base(base_metadata) => base_metadata.get_range(key),
            MetadataEnum::Msmu60(msmu60_metadata) => msmu60_metadata.get_range(key),
            MetadataEnum::Mpsu50(mpsu50_metadata) => mpsu50_metadata.get_range(key),
        }
    }

    fn get_range_defaults(&self, metadata: &MetadataEnum, key: &str) -> Option<&'static str> {
        match metadata {
            MetadataEnum::Base(base_metadata) => base_metadata.get_default(key),
            MetadataEnum::Msmu60(msmu60_metadata) => msmu60_metadata.get_default(key),
            MetadataEnum::Mpsu50(mpsu50_metadata) => mpsu50_metadata.get_default(key),
        }
    }

    pub fn get_name_for(&self, key: &str) -> Option<&'static str> {
        let metadata = self.device.get_metadata();
        match metadata {
            MetadataEnum::Base(base_metadata) => base_metadata.get_name(key),
            MetadataEnum::Msmu60(msmu60_metadata) => msmu60_metadata.get_name(key),
            MetadataEnum::Mpsu50(mpsu50_metadata) => mpsu50_metadata.get_name(key),
        }
    }

    pub fn get_region_map(&self, metadata: &MetadataEnum, key: &str) -> Option<RegionMapMetadata> {
        match metadata {
            MetadataEnum::Base(base_metadata) => base_metadata.get_region_map(key),
            MetadataEnum::Msmu60(msmu60_metadata) => msmu60_metadata.get_region_map(key),
            MetadataEnum::Mpsu50(mpsu50_metadata) => mpsu50_metadata.get_region_map(key),
        }
    }

    /// Initializes the `sense_mode` parameter for SMU devices.
    fn initialize_sense_mode(&self) -> Option<ParameterString> {
        let mut sense_mode = ParameterString::new("sense_mode");
        sense_mode.range = vec![
            BaseMetadata::SENSE_MODE_TWO_WIRE.to_string(),
            BaseMetadata::SENSE_MODE_FOUR_WIRE.to_string(),
        ];
        sense_mode.value = BaseMetadata::SENSE_MODE_TWO_WIRE.to_string();
        Some(sense_mode)
    }

    pub fn update_region_constraints(&mut self, min_value: f64, max_value: f64) -> bool {
        let mut changed = false;
        let mut region_id = -1;

        match self.device.device_type {
            DeviceType::Smu => {
                if let Some(region_map) =
                    self.get_region_map(&self.device.get_metadata(), "smu.region")
                {
                    let mut value = 0.0;
                    if self.source_range.is_range_auto()
                        || self.source_range.is_range_follow_limiti()
                    {
                        value = if min_value.abs() > max_value.abs() {
                            min_value
                        } else {
                            max_value
                        };
                    } else {
                        if let Some(res) = self.source_range.get_scaled_value() {
                            value = res
                        }
                    }

                    let old_id = region_id;
                    let old_limiti = self.source_limiti.value;
                    let old_limitv = self
                        .source_limitv
                        .as_ref()
                        .map(|v| v.value)
                        .unwrap_or(f64::NAN);

                    if self.source_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
                        let source_limit_ilimits = region_map.get_current_limit(value);
                        self.source_limiti.value =
                            source_limit_ilimits.limit(self.source_limiti.value);
                        region_id = region_map.find_region(value, self.source_limiti.value);
                    } else {
                        let source_limit_vlimits = region_map.get_voltage_limit(value);
                        if let Some(ref mut limitv) = self.source_limitv {
                            limitv.value = source_limit_vlimits.limit(limitv.value);
                            region_id = region_map.find_region(limitv.value, value);
                        }
                    }

                    if (region_id != old_id)
                        || (old_limiti != self.source_limiti.value)
                        || (old_limitv
                            != self
                                .source_limitv
                                .as_ref()
                                .map(|v| v.value)
                                .unwrap_or(f64::NAN))
                    {
                        changed = true;
                    }
                }
            }
            DeviceType::Psu => {
                if let Some(region_map) =
                    self.get_region_map(&self.device.get_metadata(), "psu.region")
                {
                    let mut value = 0.0;
                    if self.source_range.is_range_auto()
                        || self.source_range.is_range_follow_limiti()
                    {
                        value = if min_value.abs() > max_value.abs() {
                            min_value
                        } else {
                            max_value
                        };
                    } else {
                        if let Some(res) = self.source_range.get_scaled_value() {
                            value = res
                        }
                    }

                    let old_id = region_id;
                    let old_limiti = self.source_limiti.value;

                    if self.source_function.value == BaseMetadata::FUNCTION_VOLTAGE.to_string() {
                        let source_limit_ilimits = region_map.get_current_limit(value);
                        self.source_limiti.value =
                            source_limit_ilimits.limit(self.source_limiti.value);
                        region_id = region_map.find_region(value, self.source_limiti.value);
                    }

                    if (region_id != old_id) || (old_limiti != self.source_limiti.value) {
                        changed = true;
                    }
                }
            }
            DeviceType::Unknown => {
                //todo: handle error
            }
        }
        changed
    }
}
