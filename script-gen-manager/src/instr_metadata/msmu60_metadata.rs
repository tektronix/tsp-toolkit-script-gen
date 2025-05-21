use crate::model::{
    chan_data::region_map::RegionMapMetadata, sweep_data::number_limit::NumberLimit,
};

use super::base_metadata::{BaseMetadata, Metadata};

#[derive(Debug, Clone)]
pub struct Msmu60Metadata {
    base: BaseMetadata,
    // Additional properties for SmuMetadata
}

impl Msmu60Metadata {
    pub fn new() -> Self {
        let mut base = BaseMetadata::new();
        // Add additional key-value pairs for MSmu60Metadata
        base.add_option(
            "source_meas.rangev",
            vec!["AUTO", "200 mV", "2 V", "6 V", "20 V", "60 V"],
        );
        base.add_option(
            "source_meas.rangei",
            vec![
                "AUTO", "100 nA", "1 uA", "10 uA", "100 uA", "1 mA", "10 mA", "100 mA", "1 A",
                "1.5 A",
            ],
        );

        base.add_default("source_meas.range.defaultv", "2 V");
        base.add_default("source_meas.range.defaulti", "100 mA");

        // Add ranges
        base.add_range("source.levelv".to_string(), -60.0, 60.0);
        base.add_range("source.leveli".to_string(), -1.5, 1.5);

        // Add region maps
        // when pulse mode is off
        let exclude_v = Some(NumberLimit::new(-0.01, 0.01, false, None));
        let exclude_i = NumberLimit::new(-10.0e-9, 10.0e-9, false, None);
        let mut region_map_metadata = RegionMapMetadata::new(exclude_v, exclude_i);
        region_map_metadata.add_region(1, -60.0, -0.1, 60.0, 0.1);
        region_map_metadata.add_region(1, -20.0, -1.5, 20.0, 1.5);
        base.add_region_map("smu.region", region_map_metadata);

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
}
