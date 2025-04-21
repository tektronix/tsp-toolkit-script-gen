use super::{
    base_metadata::BaseMetadata, mpsu50_metadata::Mpsu50Metadata, msmu60_metadata::Msmu60Metadata,
};

#[derive(Debug, Clone)]
pub enum MetadataEnum {
    Base(BaseMetadata),
    Msmu60(Msmu60Metadata),
    Mpsu50(Mpsu50Metadata),
}

// impl Metadata for MetadataEnum {
//     fn get_option(&self, key: &str) -> Option<&Vec<&'static str>> {
//         match self {
//             MetadataEnum::Base(metadata) => metadata.get_option(key),
//             MetadataEnum::Msmu60(metadata) => metadata.get_option(key),
//             MetadataEnum::Mpsu50(metadata) => metadata.get_option(key),
//         }
//     }

//     fn get_range(&self, key: &str) -> Option<(f64, f64)> {
//         match self {
//             MetadataEnum::Base(metadata) => metadata.get_range(key),
//             MetadataEnum::Msmu60(metadata) => metadata.get_range(key),
//             MetadataEnum::Mpsu50(metadata) => metadata.get_range(key),
//         }
//     }
// }

impl Default for MetadataEnum {
    fn default() -> Self {
        MetadataEnum::Base(BaseMetadata::default())
    }
}
