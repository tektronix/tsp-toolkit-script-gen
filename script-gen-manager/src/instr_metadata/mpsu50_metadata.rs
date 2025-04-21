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
        base.add_option("source_meas.rangev", vec!["AUTO", "50 V"]);
        base.add_option("source_meas.rangei", vec!["AUTO", "5 A"]);

        base.add_default("source_meas.range.defaultv", "50 V");
        base.add_default("source_meas.range.defaulti", "5 A");

        //TODO: verify for Trebuchet PSU (model: MPSU50-2ST)
        // Add ranges
        base.add_range("source.levelv".to_string(), -50.0, 50.0);
        base.add_range("source.leveli".to_string(), -5.0, 5.0);

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
}
