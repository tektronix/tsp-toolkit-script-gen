use serde::{Deserialize, Serialize};

use crate::instr_metadata::base_metadata::BaseMetadata;

use super::{
    number_limit::{CommonTimingLimit, NumberLimit, SmuTimingLimit},
    parameters::{ParameterFloat, ParameterString},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SweepTimingConfig {
    pub measure_count: ParameterFloat,
    pub smu_timing: SmuTiming,
    pub psu_timing: PsuTiming,
}

impl SweepTimingConfig {
    pub fn new() -> Self {
        SweepTimingConfig {
            measure_count: ParameterFloat::new("measureCount", 1.0, None),
            smu_timing: SmuTiming::new(),
            psu_timing: PsuTiming::new(),
        }
    }

    pub fn evaluate(&mut self) {
        let common_timing_limits = CommonTimingLimit::new();
        //TODO: verify if additional validation is needed
        self.measure_count.value = common_timing_limits
            .measure_count_limits
            .limit(self.measure_count.value);
        self.smu_timing.evaluate();
        self.psu_timing.evaluate();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmuTiming {
    pub nplc: ParameterFloat,
    pub aperture: ParameterFloat,
    pub source_auto_delay: ParameterString,
    pub source_delay: ParameterFloat,
    pub measure_auto_delay: ParameterString,
    pub measure_delay: ParameterFloat,
    pub nplc_type: ParameterString,
}

impl SmuTiming {
    pub fn new() -> Self {
        let mut smu_timing = SmuTiming {
            nplc: ParameterFloat::new("nplc", 1.0, None),
            aperture: ParameterFloat::new(
                "aperture",
                1e-6,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            source_auto_delay: ParameterString::new("sourceAutoDelay"),
            source_delay: ParameterFloat::new(
                "sourceDelay",
                0.0,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            measure_auto_delay: ParameterString::new("measureAutoDelay"),
            measure_delay: ParameterFloat::new(
                "measureDelay",
                0.0,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            nplc_type: ParameterString::new("nplcType"),
        };
        smu_timing.set_defaults();
        smu_timing
    }

    pub fn set_defaults(&mut self) {
        self.source_auto_delay.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::ON_VALUE.to_string(),
        ];
        self.source_auto_delay.value = BaseMetadata::ON_VALUE.to_string();

        self.measure_auto_delay.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::ON_VALUE.to_string(),
        ];
        self.measure_auto_delay.value = BaseMetadata::ON_VALUE.to_string();

        self.nplc_type.range = vec![String::from("NPLC"), String::from("Aperture")];
        self.nplc_type.value = String::from("NPLC");
    }

    pub fn evaluate(&mut self) {
        let smu_limits = SmuTimingLimit::new();
        //TODO: verify if additional validation is needed
        self.evaluate_nplc(&smu_limits.nplc_limits);
        self.evaluate_aperture(&smu_limits.aperture_limits);
        self.evaluate_source_delay(&smu_limits.source_delay_limits);
        self.evaluate_measure_delay(&smu_limits.measure_delay_limits);
    }

    // fn compute_effective_delay(&self) -> f64 {
    //     let mut delay = 0.0;
    //     if self.measure_auto_delay.value == BaseMetadata::OFF_VALUE.to_string() {
    //         delay += self.measure_delay.value;
    //     }
    //     if self.source_auto_delay.value == BaseMetadata::OFF_VALUE.to_string() {
    //         delay += self.source_delay.value;
    //     }
    //     delay
    // }

    // fn compute_t(&self) -> f64 {
    //     let mut meas_time_per_count = 0.0;
    // 	let mut effective_total_time = 0.0;
    //     //TODO: If we are using these computations, we need to obtain this value from the UI
    //     let line_freq = 60.0;

    //     meas_time_per_count = BaseMetadata::MIN_BUFFER_TIME + self.nplc.value * self.aperture.value / line_freq;
    //     //TODO: Should measure_count be a separate timing parameter for both SMU and PSU?
    //     effective_total_time = self.compute_effective_delay() + 1 * self.measure_count.value * meas_time_per_count;

    //     effective_total_time
    // }

    fn evaluate_nplc(&mut self, limit: &NumberLimit) {
        self.nplc.value = limit.limit(self.nplc.value);

        // The NPLC resolution for our instrument is 0.00001 -> round *down* the new NPLC value
        // to closest (N * 0.00001)
        let res = self.nplc.value.rem_euclid(0.00001);
        if res != 0.0 {
            let temp = (self.nplc.value / 0.00001).floor() * 0.00001;
            println!("NPLC value rounded to: {}", temp);
            // Make sure it is still within hard limits
            self.nplc.value = limit.limit(temp);
        }
    }

    fn evaluate_aperture(&mut self, limit: &NumberLimit) {
        self.aperture.value = limit.limit(self.aperture.value);
    }

    fn evaluate_source_delay(&mut self, limit: &NumberLimit) {
        self.source_delay.value = limit.limit(self.source_delay.value);
    }

    fn evaluate_measure_delay(&mut self, limit: &NumberLimit) {
        self.measure_delay.value = limit.limit(self.measure_delay.value);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PsuTiming {
    rate: ParameterString,
}

impl PsuTiming {
    pub fn new() -> Self {
        let mut psu_timing = PsuTiming {
            rate: ParameterString::new("rate"),
        };
        psu_timing.set_defaults();
        psu_timing
    }

    pub fn set_defaults(&mut self) {
        self.rate.range = vec![
            BaseMetadata::RATE_NORMAL.to_string(),
            BaseMetadata::RATE_FAST.to_string(),
        ];
        self.rate.value = BaseMetadata::RATE_NORMAL.to_string();
    }

    pub fn evaluate(&mut self) {
        //let psu_limits = PsuTimingLimit::new();
        //TODO: verify if additional validation is needed
    }
}
