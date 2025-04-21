use serde::{Deserialize, Serialize};

use super::sweep_config::SweepConfig;

/// The `SweepModel` struct represents the model for sweep configurations.
/// It contains a `SweepConfig` which holds the configuration details for the sweep.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SweepModel {
    //#[serde(deserialize_with = "deserialize_sweep_model")]
    pub sweep_config: SweepConfig,
}

impl SweepModel {
    pub fn new() -> Self {
        SweepModel {
            sweep_config: SweepConfig::new(),
        }
    }
}

// fn deserialize_sweep_model<'de, D>(deserializer: D) -> Result<SweepConfig, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     #[derive(Deserialize)]
//     struct TempTiming {
//         nplc: f64,
//     }

//     #[derive(Deserialize)]
//     struct TempGlobalParameters {
//         auto_range: bool,
//         high_c: bool,
//         timing: TempTiming,
//     }

//     #[derive(Deserialize)]
//     struct TempSweepModel {
//         global_parameters: TempGlobalParameters,
//     }

//     let temp = TempSweepModel::deserialize(deserializer)?;

//     // Enforce constraints within GlobalParameters
//     if temp.global_parameters.auto_range && temp.global_parameters.high_c {
//         return Err(de::Error::custom(
//             "auto_range and high_c cannot both be true",
//         ));
//     }

//     Ok(SweepConfig {
//         global_parameters: GlobalParameters {
//             auto_range: temp.global_parameters.auto_range,
//             high_c: temp.global_parameters.high_c,
//             timing: Timing {
//                 nplc: temp.global_parameters.timing.nplc,
//             },
//         },
//     })
// }
