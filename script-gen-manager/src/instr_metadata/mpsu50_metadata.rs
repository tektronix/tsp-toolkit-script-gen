use crate::model::{
    chan_data::region_map::RegionMapMetadata, sweep_data::number_limit::NumberLimit,
};

use super::base_metadata::{BaseMetadata, Metadata};

#[derive(Debug, Clone)]
pub struct Mpsu50Metadata {
    base: BaseMetadata,
    // Additional properties for PsuMetadata
}

impl Mpsu50Metadata {
    pub fn new() -> Self {
        let mut base = BaseMetadata::new();
        // Add additional key-value pairs for Mpsu50Metadata
        base.add_option("source_meas.rangev", vec!["50 V"]);
        base.add_option("source_meas.rangei", vec!["5 A"]);

        base.add_default("source_meas.range.defaultv", "50 V");
        base.add_default("source_meas.range.defaulti", "5 A");

        //TODO: verify for Trebuchet PSU (model: MPSU50-2ST)
        // Add ranges
        let max_supported_voltage = 50.1;
        let max_supported_current = 5.0;
        base.add_range(
            "source.levelv".to_string(),
            -max_supported_voltage,
            max_supported_voltage,
        );
        base.add_range(
            "source.leveli".to_string(),
            -max_supported_current,
            max_supported_current,
        );

        base.add_range("source.limiti".to_string(), 0.01, 5.1);

        base.add_range("source.step_to_sweep_delay".to_string(), 0.0, 100.0);

        // Add region maps
        // when pulse mode is off
        let exclude_i = NumberLimit::new(-10.0e-9, 10.0e-9, false, None);
        let mut region_map_metadata = RegionMapMetadata::new(None, exclude_i);

        region_map_metadata.add_region(1, 0.0, 0.0, 10.0, max_supported_current);
        region_map_metadata.add_region(1, 0.0, 0.0, -10.0, -max_supported_current);

        Self::add_1st_quadrant_curved_region(10.0, 50.0, 0.001, 0.0, &mut region_map_metadata); //First quadrant curve
        Self::add_3rd_quadrant_curved_region(-10.0, -50.0, -0.001, 0.0, &mut region_map_metadata); //Third quadrant curve

        base.add_region_map("50 V", region_map_metadata); //Use source range to identify region map

        base.add_overrange_scale(1.002);

        Mpsu50Metadata {
            base,
            // Initialize additional properties
        }
    }

    fn add_1st_quadrant_curved_region(
        voltage_start: f64,
        voltage_max: f64,
        step: f64,    // Use the step parameter to finely control the approximation
        current: f64, // Fixed current for the curve
        region_map_metadata: &mut RegionMapMetadata,
    ) {
        // Add region maps iteratively as small rectangles to approximate the curve

        let mut v1 = voltage_start;

        while v1.abs() <= voltage_max.abs() {
            let v2 = v1 + step;
            let i2 = voltage_max / v2.abs();
            region_map_metadata.add_region(1, v1, current, v2, i2);
            v1 += step;
        }
    }

    fn add_3rd_quadrant_curved_region(
        voltage_start: f64,
        voltage_max: f64,
        step: f64,    // Use the step parameter to finely control the approximation
        current: f64, // Fixed current for the curve
        region_map_metadata: &mut RegionMapMetadata,
    ) {
        // Add region maps iteratively as small rectangles to approximate the curve

        let mut v1 = voltage_start;

        while v1.abs() <= voltage_max.abs() {
            let v2 = v1 + step;
            let i2 = voltage_max / v2.abs();
            region_map_metadata.add_region(1, v2, i2, v1, current);
            v1 += step;
        }
    }
}

impl Metadata for Mpsu50Metadata {
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
