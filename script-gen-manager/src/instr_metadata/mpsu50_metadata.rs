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
        base.add_range("source.levelv".to_string(), -50.1, 50.1);
        base.add_range("source.leveli".to_string(), -5.0, 5.0);

        base.add_range("source.limiti".to_string(), 0.01, 5.1);

        // Add region maps
        // when pulse mode is off
        let exclude_i = NumberLimit::new(-10.0e-9, 10.0e-9, false, None);
        let mut region_map_metadata = RegionMapMetadata::new(None, exclude_i);
        region_map_metadata.add_region(1, 0.0, 0.0, 50.0, 1.0);
        region_map_metadata.add_region(1, 0.0, 0.0, 10.0, 5.0);
        region_map_metadata.add_region(1, 0.0, 0.0, -10.0, -5.0);
        region_map_metadata.add_region(1, 0.0, 0.0, -50.0, -1.0);
        base.add_region_map("psu.region", region_map_metadata);

        base.add_overrange_scale(1.002);

        Mpsu50Metadata {
            base,
            // Initialize additional properties
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
