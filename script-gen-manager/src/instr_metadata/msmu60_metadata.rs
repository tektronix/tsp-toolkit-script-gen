use crate::model::{
    chan_data::region_map::RegionMapMetadata, sweep_data::number_limit::NumberLimit,
};

use super::base_metadata::{BaseMetadata, Metadata};

// Epsilon for floating-point comparisons
const EPSILON: f64 = 1e-15;

// Voltage range constants for MSMU60
const VOLTAGE_AUTO: &str = "AUTO";
const VOLTAGE_200_MV: &str = "200 mV";
const VOLTAGE_2_V: &str = "2 V";
const VOLTAGE_6_V: &str = "6 V";
const VOLTAGE_20_V: &str = "20 V";
const VOLTAGE_60_V: &str = "60 V";

// Current range constants for MSMU60
const CURRENT_AUTO: &str = "AUTO";
const CURRENT_100_NA: &str = "100 nA";
const CURRENT_1_UA: &str = "1 \u{00B5}A"; // Unicode character for micro (µ)
const CURRENT_10_UA: &str = "10 \u{00B5}A";
const CURRENT_100_UA: &str = "100 \u{00B5}A";
const CURRENT_1_MA: &str = "1 mA";
const CURRENT_10_MA: &str = "10 mA";
const CURRENT_100_MA: &str = "100 mA";
const CURRENT_1_A: &str = "1 A";
const CURRENT_1_5_A: &str = "1.5 A";

// Range arrays built from constants
const VOLTAGE_RANGES: &[&str] = &[
    VOLTAGE_AUTO,
    VOLTAGE_200_MV,
    VOLTAGE_2_V,
    VOLTAGE_6_V,
    VOLTAGE_20_V,
    VOLTAGE_60_V,
];
const CURRENT_RANGES: &[&str] = &[
    CURRENT_AUTO,
    CURRENT_100_NA,
    CURRENT_1_UA,
    CURRENT_10_UA,
    CURRENT_100_UA,
    CURRENT_1_MA,
    CURRENT_10_MA,
    CURRENT_100_MA,
    CURRENT_1_A,
    CURRENT_1_5_A,
];

#[derive(Debug, Clone)]
pub struct Msmu60Metadata {
    base: BaseMetadata,
    // Additional properties for SmuMetadata
}

impl Msmu60Metadata {
    pub fn new() -> Self {
        let mut base = BaseMetadata::new();
        // Add additional key-value pairs for MSmu60Metadata
        base.add_option("source_meas.rangev", VOLTAGE_RANGES.to_vec());
        base.add_option("source_meas.rangei", CURRENT_RANGES.to_vec());

        base.add_default("source_meas.range.defaultv", "AUTO");
        base.add_default("source_meas.range.defaulti", "AUTO");

        // Add ranges
        base.add_range("source.levelv".to_string(), -60.6, 60.6);
        base.add_range("source.leveli".to_string(), -1.515, 1.515);

        base.add_range("source.limiti".to_string(), 10.0e-9, 1.515);
        base.add_range("source.limitv".to_string(), 0.02, 60.6);

        base.add_range("source.step_to_sweep_delay".to_string(), 0.0, 100.0);

        // Add region maps
        // when pulse mode is off
        let exclude_v = Some(NumberLimit::new(-0.01, 0.01, false, None));
        let exclude_i = NumberLimit::new(-10.0e-9, 10.0e-9, false, None);

        let mut inner_region = RegionMapMetadata::new(exclude_v.clone(), exclude_i.clone());
        inner_region.add_region(
            1,
            -60.6 - EPSILON,
            -0.101 - EPSILON,
            60.6 + EPSILON,
            0.101 + EPSILON,
        );
        base.add_region_map(VOLTAGE_AUTO, inner_region.clone());
        base.add_region_map(VOLTAGE_60_V, inner_region.clone());
        base.add_region_map(CURRENT_100_NA, inner_region.clone());
        base.add_region_map(CURRENT_1_UA, inner_region.clone());
        base.add_region_map(CURRENT_10_UA, inner_region.clone());
        base.add_region_map(CURRENT_100_UA, inner_region.clone());
        base.add_region_map(CURRENT_1_MA, inner_region.clone());
        base.add_region_map(CURRENT_10_MA, inner_region.clone());
        base.add_region_map(CURRENT_100_MA, inner_region.clone());

        let mut outer_region = RegionMapMetadata::new(exclude_v.clone(), exclude_i.clone());
        outer_region.add_region(
            1,
            -20.2 - EPSILON,
            -1.515 - EPSILON,
            20.2 + EPSILON,
            1.515 + EPSILON,
        );
        base.add_region_map(VOLTAGE_200_MV, outer_region.clone());
        base.add_region_map(VOLTAGE_2_V, outer_region.clone());
        base.add_region_map(VOLTAGE_6_V, outer_region.clone());
        base.add_region_map(VOLTAGE_20_V, outer_region.clone());
        base.add_region_map(CURRENT_1_A, outer_region.clone());
        base.add_region_map(CURRENT_1_5_A, outer_region.clone());

        base.add_overrange_scale(1.01);

        Msmu60Metadata {
            base,
            // Initialize additional properties
        }
    }
}

impl Metadata for Msmu60Metadata {
    fn get_option(&self, key: &str) -> Option<&Vec<&'static str>> {
        self.base.get_option(key)
    }

    fn get_range(&self, key: &str) -> Option<(f64, f64)> {
        self.base.get_range(key)
    }

    fn get_default(&self, key: &str) -> Option<&'static str> {
        self.base.get_default(key)
    }

    fn get_name(&self, key: &str) -> Option<&'static str> {
        self.base.get_name(key)
    }

    fn get_region_map(&self, key: &str) -> Option<RegionMapMetadata> {
        self.base.get_region_map(key)
    }

    fn get_overrange_scale(&self) -> f64 {
        self.base.get_overrange_scale()
    }
}
