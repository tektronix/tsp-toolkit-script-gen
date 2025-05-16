use std::convert::Into;

use serde::{Deserialize, Serialize};

use super::{
    number_limit::NumberLimit,
    parameters::{ParameterFloat, ParameterInt, ParameterString},
};
use crate::instr_metadata::base_metadata::{BaseMetadata, Metadata};

/// The `TimingConfig` struct represents the configuration for timing parameters in a sweep.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimingConfig {
    pub nplc: ParameterFloat,
    pub auto_zero: ParameterString,
    pub source_delay_type: ParameterString,
    pub source_delay: ParameterFloat,
    pub measure_count: ParameterInt,
    pub measure_delay_type: ParameterString,
    pub measure_delay: ParameterFloat,
    pub measure_delay_factor: ParameterFloat,
    pub measure_filter_enable: ParameterString,
    pub measure_filter_type: ParameterString,
    pub measure_filter_count: ParameterInt,
    pub measure_analog_filter: ParameterString,

    pub high_speed_sampling: bool,
    pub sampling_interval: ParameterFloat,
    pub sampling_count: ParameterInt,
    pub sampling_delay_type: ParameterString,
    pub sampling_delay: ParameterFloat,
    pub sampling_analog_filter: ParameterString,

    #[serde(skip)]
    base_metadata: BaseMetadata,

    #[serde(skip)]
    timing_limits: TimingLimit,
}

impl TimingConfig {
    /// Creates a new instance of `TimingConfig` with default values.
    ///
    /// # Returns
    /// A new `TimingConfig` instance with default timing parameters.
    pub fn new() -> Self {
        TimingConfig {
            nplc: ParameterFloat::new("nplc", 0.1, None),
            auto_zero: ParameterString::new("autoZero"),
            source_delay_type: ParameterString::new("sourceDelayType"),
            source_delay: ParameterFloat::new(
                "sourceDelay",
                0.0,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            measure_count: ParameterInt::new("measureCount", 1),
            measure_delay_type: ParameterString::new("measureDelayType"),
            measure_delay: ParameterFloat::new(
                "measureDelay",
                0.0,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            measure_delay_factor: ParameterFloat::new("measureDelayFactor", 1.0, None),
            measure_filter_enable: ParameterString::new("measureFilterEnable"),
            measure_filter_type: ParameterString::new("measureFilterType"),
            measure_filter_count: ParameterInt::new("measureFilterCount", 1),
            measure_analog_filter: ParameterString::new("measureAnalogFilter"),

            high_speed_sampling: false,
            sampling_interval: ParameterFloat::new(
                "samplingInterval",
                1.0e-6,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            sampling_count: ParameterInt::new("samplingCount", 1000),
            sampling_delay_type: ParameterString::new("samplingDelayType"),
            sampling_delay: ParameterFloat::new(
                "samplingDelay",
                0.0,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            sampling_analog_filter: ParameterString::new("samplingAnalogFilter"),

            base_metadata: BaseMetadata::new(),
            timing_limits: TimingLimit::new(),
        }
    }

    /// Sets default ranges and values for the timing parameters.
    pub fn set_defaults(&mut self) {
        //self.timing_limits.update_timing_limits();

        self.auto_zero.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::ONCE_VALUE.to_string(),
            BaseMetadata::AUTO_VALUE.to_string(),
        ];
        self.auto_zero.value = BaseMetadata::ONCE_VALUE.to_string();

        self.source_delay_type.range = self
            .base_metadata
            .get_option("timing.delay.type:0")
            .unwrap_or(&vec![])
            .iter()
            .map(|&s| s.to_string())
            .collect();
        self.source_delay_type.value = BaseMetadata::OFF_VALUE.to_string();

        self.measure_delay_type.range = self
            .base_metadata
            .get_option("timing.delay.type:0")
            .unwrap_or(&vec![])
            .iter()
            .map(|&s| s.to_string())
            .collect();
        self.measure_delay_type.value = BaseMetadata::OFF_VALUE.to_string();

        self.measure_filter_enable.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::ON_VALUE.to_string(),
        ];
        self.measure_filter_enable.value = BaseMetadata::OFF_VALUE.to_string();

        self.measure_filter_type.range = vec![
            BaseMetadata::MOVING_AVG.to_string(),
            BaseMetadata::REPEAT_AVG.to_string(),
        ];
        self.measure_filter_type.value = BaseMetadata::MOVING_AVG.to_string();

        self.measure_analog_filter.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::ON_VALUE.to_string(),
        ];
        self.measure_analog_filter.value = BaseMetadata::OFF_VALUE.to_string();

        self.sampling_delay_type.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::USER_DEFINED_VALUE.to_string(),
        ];
        self.sampling_delay_type.value = BaseMetadata::OFF_VALUE.to_string();

        self.sampling_analog_filter.range = vec![
            BaseMetadata::OFF_VALUE.to_string(),
            BaseMetadata::ON_VALUE.to_string(),
        ];
        self.sampling_analog_filter.value = BaseMetadata::OFF_VALUE.to_string();
    }

    pub fn evaluate(&mut self) {
        let key = "timing.delay.type";

        self.source_delay_type.range = self
            .base_metadata
            .get_option(key)
            .unwrap_or(&vec![])
            .iter()
            .map(|&s| s.to_string())
            .collect();

        self.measure_delay_type.range = self
            .base_metadata
            .get_option(key)
            .unwrap_or(&vec![])
            .iter()
            .map(|&s| s.to_string())
            .collect();
    }

    /// Returns the sweep time per point to be used in further validation
    pub fn validate(
        &mut self,
        sweep_time_per_point: f64,
        min_buffer_time: f64,
        line_frequency: i32,
    ) -> f64 {
        self.correct_nplc(min_buffer_time, line_frequency);
        self.correct_auto_zero();
        self.correct_source_delay(min_buffer_time, line_frequency);
        self.correct_measure_count(min_buffer_time, line_frequency);
        self.correct_measure_delay(min_buffer_time, line_frequency);
        self.correct_measure_delay_factor();
        self.correct_measure_filter(min_buffer_time, line_frequency);
        self.correct_measure_analog_filter();

        let sweep_time_per_point =
            self.correct_high_speed_sampling(sweep_time_per_point, min_buffer_time, line_frequency);

        self.correct_sampling_interval(min_buffer_time, line_frequency, sweep_time_per_point);
        self.correct_sampling_count(min_buffer_time, line_frequency, sweep_time_per_point);
        self.correct_sampling_delay(min_buffer_time, line_frequency, sweep_time_per_point);
        self.correct_sampling_analog_filter();

        sweep_time_per_point
    }

    fn get_nonhss_value<T>(
        &self,
        min_buffer_time: f64,
        line_frequency: i32,
        value: T,
        cb: impl Fn() -> T,
    ) -> T {
        // TODO When we support pulsing, TMAX = (pulse_width - EPSILON) iff pulsing is enabled
        // NOTE: EPSILON for 2600 was 1e-9
        const TMAX: f64 = f64::MAX;
        if !self.high_speed_sampling && self.compute_t(min_buffer_time, line_frequency) > TMAX {
            cb()
        } else {
            value
        }
    }

    fn f64_to_i32(value: f64) -> i32 {
        if value > i32::MAX.into() {
            i32::MAX
        } else if value < i32::MIN.into() {
            i32::MIN
        } else {
            value.floor() as i32
        }
    }

    fn correct_nplc(&mut self, min_buffer_time: f64, line_frequency: i32) {
        // TODO When we support pulsing, TMAX = (pulse_width - EPSILON) iff pulsing is enabled
        // NOTE: EPSILON for 2600 was 1e-9
        const TMAX: f64 = f64::MAX;
        self.nplc.value =
            self.get_nonhss_value(min_buffer_time, line_frequency, self.nplc.value, || {
                (((TMAX - self.compute_effective_delay())
                    / (Into::<f64>::into(self.compute_effective_filter_count())
                        * Into::<f64>::into(self.measure_count.value)))
                    - min_buffer_time)
                    * Into::<f64>::into(line_frequency)
            });
        self.nplc.value = f64::floor(self.nplc.value / 0.001) * 0.001;
        self.nplc.value = self.timing_limits.nplc_limits.limit(self.nplc.value);
        //No coupling
    }
    fn correct_auto_zero(&mut self) {
        // No validation??? self.auto_zero = self.auto_zero.range;
        // No coupling
    }

    fn compute_measurment_time_per_point(&self, min_buffer_time: f64, line_frequency: i32) -> f64 {
        Into::<f64>::into(self.compute_effective_filter_count())
            * Into::<f64>::into(self.measure_count.value)
            * (min_buffer_time + self.nplc.value / Into::<f64>::into(line_frequency))
    }
    fn compute_minimum_time_per_point(&self, min_buffer_time: f64, line_frequency: i32) -> f64 {
        self.compute_effective_delay()
            + self.compute_measurment_time_per_point(min_buffer_time, line_frequency)
    }

    fn compute_dead_time(&self) -> f64 {
        //TODO need values for Treb
        0.0
    }

    fn compute_effective_delay(&self) -> f64 {
        let mut delay = 0.0;

        if self.high_speed_sampling {
            if self.sampling_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                delay += self.sampling_delay.value;
            }
        } else {
            if self.measure_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                delay += self.measure_delay.value;
            }
            if self.source_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                delay += self.source_delay.value;
            }
        }
        delay
    }

    fn compute_t(&self, min_buffer_time: f64, line_frequency: i32) -> f64 {
        let meas_time_per_count = if self.high_speed_sampling {
            self.sampling_interval.value + self.compute_dead_time()
        } else {
            min_buffer_time + self.nplc.value / Into::<f64>::into(line_frequency)
        };
        if self.high_speed_sampling {
            let sampling_count_contribution = if self.sampling_count.value < 45 {
                Into::<f64>::into(45 - self.sampling_count.value) * 2.0e-6
            } else {
                0.0
            };
            self.compute_effective_delay()
                + Into::<f64>::into(self.compute_effective_filter_count())
                    * Into::<f64>::into(self.sampling_count.value)
                    * meas_time_per_count
                + sampling_count_contribution
        } else {
            self.compute_effective_delay()
                + Into::<f64>::into(self.compute_effective_filter_count())
                    * Into::<f64>::into(self.measure_count.value)
                    * Into::<f64>::into(meas_time_per_count)
        }
    }

    fn correct_source_delay(&mut self, min_buffer_time: f64, line_frequency: i32) {
        if self.source_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
            // TODO When we support pulsing, TMAX = (pulse_width - EPSILON) iff pulsing is enabled
            // NOTE: EPSILON for 2600 was 1e-9
            const TMAX: f64 = f64::MAX;
            // soft limits
            self.source_delay.value = self.get_nonhss_value(
                min_buffer_time,
                line_frequency,
                self.source_delay.value,
                || {
                    let measurement_delay =
                        if self.measure_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                            self.measure_delay.value
                        } else {
                            0.0
                        };
                    TMAX - measurement_delay
                        - Into::<f64>::into(self.compute_effective_filter_count())
                            * Into::<f64>::into(self.measure_count.value)
                            * (min_buffer_time
                                + self.nplc.value / Into::<f64>::into(line_frequency))
                },
            );
            // hard limits
            self.source_delay.value = f64::floor(self.source_delay.value / 1.0e-6) * 1.0e-6;
            self.source_delay.value = self
                .timing_limits
                .source_delay_limits
                .limit(self.source_delay.value);

            //No coupling
        } else {
            self.source_delay.value = 0.0;
        }
    }
    fn correct_measure_count(&mut self, min_buffer_time: f64, line_frequency: i32) {
        // TODO When we support pulsing, TMAX = (pulse_width - EPSILON) iff pulsing is enabled
        const TMAX: f64 = f64::MAX;
        const EPSILON: f64 = 1e-9; //TODO This is for 26xx, is there a different value for Treb?
        self.measure_count.value = self.get_nonhss_value(
            min_buffer_time,
            line_frequency,
            self.measure_count.value,
            || {
                Self::f64_to_i32(
                    (((TMAX - self.compute_effective_delay())
                        / Into::<f64>::into(self.compute_effective_filter_count()))
                        / (min_buffer_time
                            + Into::<f64>::into(self.nplc.value)
                                / Into::<f64>::into(line_frequency)))
                        + EPSILON,
                )
            },
        );
        self.measure_count.value = self
            .timing_limits
            .measure_count_limits
            .limit_int(self.measure_count.value);
        //no coupling
    }
    fn correct_measure_delay(&mut self, min_buffer_time: f64, line_frequency: i32) {
        if self.measure_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
            // TODO When we support pulsing, TMAX = (pulse_width - EPSILON) iff pulsing is enabled
            // NOTE: EPSILON for 2600 was 1e-9
            const TMAX: f64 = f64::MAX;
            // soft limits
            self.measure_delay.value = self.get_nonhss_value(
                min_buffer_time,
                line_frequency,
                self.measure_delay.value,
                || {
                    let source_delay =
                        if self.source_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                            self.source_delay.value
                        } else {
                            0.0
                        };
                    TMAX - source_delay
                        - Into::<f64>::into(self.compute_effective_filter_count())
                            * Into::<f64>::into(self.measure_count.value)
                            * (min_buffer_time
                                + Into::<f64>::into(self.nplc.value)
                                    / Into::<f64>::into(line_frequency))
                },
            );
            self.measure_delay.value = f64::floor(self.measure_delay.value / 1.0e-6) * 1.0e-6;
            self.measure_delay.value = self
                .timing_limits
                .measure_delay_limits
                .limit(self.measure_delay.value);
        //no coupling
        } else {
            self.measure_delay.value = 0.0;
        }
    }
    fn correct_measure_delay_factor(&mut self) {
        self.measure_delay_factor.value = self
            .timing_limits
            .measure_delay_factor_limits
            .limit(self.measure_delay_factor.value);
        //no coupling
    }
    fn correct_measure_filter(&mut self, min_buffer_time: f64, line_frequency: i32) {
        if self.measure_filter_enable.value == BaseMetadata::ON_VALUE {
            const TMAX: f64 = f64::MAX;
            const EPSILON: f64 = 1e-9; //TODO This is for 26xx, is there a different value for Treb?
                                       // soft limits
            self.measure_filter_count.value = self.get_nonhss_value(
                min_buffer_time,
                line_frequency,
                self.measure_filter_count.value,
                || {
                    let temp = (((TMAX - self.compute_effective_delay())
                        / Into::<f64>::into(self.measure_count.value))
                        / ((min_buffer_time + self.nplc.value)
                            / Into::<f64>::into(line_frequency)))
                        + EPSILON;

                    // safely convert f64 to i32
                    Self::f64_to_i32(temp)
                },
            );
            self.measure_filter_count.value = self
                .timing_limits
                .measure_filter_count_limits
                .limit_int(self.measure_filter_count.value);
        } else {
            self.measure_filter_count.value = 1;
        }
        //no coupling
    }
    fn correct_measure_analog_filter(&mut self) {
        // no validation??? self.measure_analog_filter.value =
        // no coupling
    }
    fn correct_high_speed_sampling(
        &mut self,
        sweep_time_per_point: f64,
        min_buffer_time: f64,
        line_frequency: i32,
    ) -> f64 {
        // high speed sampling coupling:
        let tmin = {
            let effective_delay: f64 = {
                let mut delay: f64 = 0.0;
                if self.measure_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                    delay += self.measure_delay.value;
                } else if self.measure_delay_type.value == BaseMetadata::AUTO_VALUE {
                    // we can't know what the delay is for AUTO... In fact, it can vary -- so stick with 0
                }

                if self.source_delay_type.value == BaseMetadata::USER_DEFINED_VALUE {
                    delay += self.source_delay.value;
                } else if self.source_delay_type.value == BaseMetadata::AUTO_VALUE {

                    // we can't know what the delay is for AUTO... In fact, it can vary -- so stick with 0
                }

                delay
            };

            let measurement_time_per_point: f64 = {
                let measurement_time_per_count =
                    min_buffer_time + (self.nplc.value / Into::<f64>::into(line_frequency));
                let effective_filter_count = {
                    if self.measure_filter_enable.value == BaseMetadata::ON_VALUE {
                        self.measure_filter_count.value
                    } else {
                        1
                    }
                };
                Into::<f64>::into(effective_filter_count)
                    * Into::<f64>::into(self.measure_count.value)
                    * measurement_time_per_count
            };

            effective_delay + measurement_time_per_point
        };
        tmin.max(sweep_time_per_point)
    }
    /// Returns `sweep_time_per_point` to be consumed by caller
    fn couple_timing(
        &mut self,
        min_buffer_time: f64,
        line_frequency: i32,
        sweep_time_per_point: f64,
    ) -> f64 {
        f64::max(
            self.compute_minimum_time_per_point(min_buffer_time, line_frequency),
            sweep_time_per_point,
        )
    }

    /// Returns `sweep_time_per_point` to be consumed by caller
    fn correct_sampling_interval(
        &mut self,
        min_buffer_time: f64,
        line_frequency: i32,
        sweep_time_per_point: f64,
    ) -> f64 {
        self.sampling_interval.value = f64::floor(self.sampling_interval.value / 1.0e-6) * 1.0e-6;
        self.sampling_interval.value = self
            .timing_limits
            .sampling_interval_limits
            .limit(self.sampling_interval.value);
        //TODO if pulsing support, add SweepModel::couplePulseWidth() call here
        self.couple_timing(min_buffer_time, line_frequency, sweep_time_per_point)
    }

    /// Returns `sweep_time_per_point` to be consumed by caller
    fn correct_sampling_count(
        &mut self,
        min_buffer_time: f64,
        line_frequency: i32,
        sweep_time_per_point: f64,
    ) -> f64 {
        self.sampling_count.value = self
            .timing_limits
            .sampling_count_limits
            .limit_int(self.sampling_count.value);
        self.couple_timing(min_buffer_time, line_frequency, sweep_time_per_point)
    }

    /// Returns `sweep_time_per_point` to be consumed by caller
    fn correct_sampling_delay(
        &mut self,
        min_buffer_time: f64,
        line_frequency: i32,
        sweep_time_per_point: f64,
    ) -> f64 {
        self.sampling_delay.value = f64::floor(self.sampling_delay.value / 1.0e-6) * 1.0e-6;
        self.sampling_delay.value = self
            .timing_limits
            .sampling_delay_limits
            .limit(self.sampling_delay.value);
        self.couple_timing(min_buffer_time, line_frequency, sweep_time_per_point)
    }
    fn correct_sampling_analog_filter(&mut self) {}

    fn compute_effective_filter_count(&self) -> i32 {
        if !self.high_speed_sampling && self.measure_filter_enable.value == BaseMetadata::ON_VALUE {
            self.measure_filter_count.value
        } else {
            1
        }
    }
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TimingLimit {
    pub nplc_limits: NumberLimit,
    pub source_delay_limits: NumberLimit,
    pub measure_count_limits: NumberLimit,
    pub measure_filter_count_limits: NumberLimit,
    pub measure_delay_limits: NumberLimit,
    pub measure_delay_factor_limits: NumberLimit,

    pub sampling_interval_limits: NumberLimit,
    pub sampling_count_limits: NumberLimit,
    pub sampling_delay_limits: NumberLimit,
}

impl TimingLimit {
    pub fn new() -> Self {
        TimingLimit {
            nplc_limits: NumberLimit::new(0.0, 0.0, true, None),
            source_delay_limits: NumberLimit::new(0.0, 0.0, true, None),
            measure_count_limits: NumberLimit::new(0.0, 0.0, true, None),
            measure_filter_count_limits: NumberLimit::new(0.0, 0.0, true, None),
            measure_delay_limits: NumberLimit::new(0.0, 0.0, true, None),
            measure_delay_factor_limits: NumberLimit::new(0.0, 0.0, true, None),
            sampling_interval_limits: NumberLimit::new(0.0, 0.0, true, None),
            sampling_count_limits: NumberLimit::new(0.0, 0.0, true, None),
            sampling_delay_limits: NumberLimit::new(0.0, 0.0, true, None),
        }
    }

    pub fn update_timing_limits(&mut self) {
        self.nplc_limits.set_min(1e-3);
        self.nplc_limits.set_max(25.0);

        self.source_delay_limits.set_min(0.0);
        self.source_delay_limits.set_max(4294.0);

        self.measure_count_limits.set_min(1.0);
        self.measure_count_limits.set_max(60000.0);

        self.measure_filter_count_limits.set_min(1.0);
        self.measure_filter_count_limits.set_max(100.0);

        self.measure_delay_limits.set_min(0.0);
        self.measure_delay_limits.set_max(4294.0);

        self.measure_delay_factor_limits.set_min(0.0);
        self.measure_delay_factor_limits.set_max(1000.0);
    }
}

impl Default for TimingLimit {
    fn default() -> Self {
        TimingLimit::new()
    }
}
