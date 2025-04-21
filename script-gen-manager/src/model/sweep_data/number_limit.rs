#[derive(Debug, Clone)]
pub struct NumberLimit {
    min: f64,
    max: f64,
    inclusion: bool,
    sublimit: Option<Box<NumberLimit>>,
}

impl NumberLimit {
    /// Construct a NumberLimit with the default min and max values (all checks disabled).
    /// Use set_min() and set_max() to set limit checks.
    pub fn new(min: f64, max: f64, inclusion: bool, sublimit: Option<NumberLimit>) -> Self {
        NumberLimit {
            min,
            max,
            inclusion,
            sublimit: sublimit.map(Box::new),
        }
    }

    /// Set the minimum value which will be enforced by limit(). To
    /// disable the minimum check, set the value to f64::NAN (the default).
    pub fn set_min(&mut self, value: f64) {
        self.min = value;
    }

    /// Get the minimum value which will be enforced by limit(). If the value
    /// is set to f64::NAN (the default), the minimum check is disabled
    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Set the maximum value which will be enforced by limit(). To
    /// disable the maximum check, set the value to f64::NAN (the default).
    pub fn set_max(&mut self, value: f64) {
        self.max = value;
    }

    /// Get the maximum value which will be enforced by limit(). If the value
    /// is set to f64::NAN (the default), the maximum check is disabled
    pub fn get_max(&self) -> f64 {
        self.max
    }

    /// Set this NumberLimit as an inclusion limit (true) or an exclusion limit (false). An
    /// inclusion limit means limit() will enforce min <= value && value <= max -- and both
    /// min and max are optional (i.e. can be NAN). An exclusion limit means limit() will
    /// enforce value <= min || max <= value -- and both min and max are required (i.e. cannot
    /// be NAN)
    pub fn set_inclusion(&mut self, value: bool) {
        self.inclusion = value;
    }

    /// Is this NumberLimit an inclusion limit (true) or an exclusion limit (false)? An
    /// inclusion limit means limit() will enforce min <= value && value <= max -- and both
    /// min and max are optional (i.e. can be NAN). An exclusion limit means limit() will
    /// enforce value <= min || max <= value -- and both min and max are required (i.e. cannot
    /// be NAN)
    pub fn is_inclusion(&self) -> bool {
        self.inclusion
    }

    /// Set the sublimit. limit() will apply the limits defined by min, max, and inclusion
    /// then call sublimit.limit() (i.e. the value is recursively limited).
    pub fn set_sublimit(&mut self, value: NumberLimit) {
        self.sublimit = Some(Box::new(value));
    }

    /// Get the sublimit. limit() will apply the limits defined by min, max, and inclusion
    /// then call sublimit.limit() (i.e. the value is recursively limited).
    pub fn get_sublimit(&self) -> Option<&NumberLimit> {
        self.sublimit.as_deref()
    }

    /// Apply the optional min, max, and sublimit values to the specified value and return the
    /// limited value (i.e. min if value < min, max if value > max, ...). If sublimit is defined
    /// the value is recursively limited (i.e. sublimit is applied and if sublimit has a sublimit
    /// it is applied...). The value is limited by this instance of NumberLimit and all nested
    /// NumberLimit instances (an "and" operation) so to define a complex region define one
    /// inclusion limit that covers the entire range and add one or more exclusion limits to
    /// "carve out" pieces of that range and nest them.
    pub fn limit(&self, value: f64) -> f64 {
        let mut result = value;

        // Optional limit checks
        if self.inclusion {
            // Inclusion ... value must be between min and max (inclusive)
            if !self.min.is_nan() && result < self.min {
                result = self.min;
            }
            if !self.max.is_nan() && result > self.max {
                result = self.max;
            }
        } else {
            // Exclusion ... value must be outside min and max (non-inclusive)
            if self.min < result && result < self.max {
                // Choose the closer of the limits...
                result = if value - self.min < self.max - value {
                    self.min
                } else {
                    self.max
                };
            }
        }

        // Recursively apply sublimit and return the value
        if let Some(ref sublimit) = self.sublimit {
            sublimit.limit(result)
        } else {
            result
        }
    }

    /// Apply the optional min, max, and sublimit values to the specified value and return the
    /// limited value (i.e. min if value < min, max if value > max, ...). If sublimit is defined
    /// the value is recursively limited (i.e. sublimit is applied and if sublimit has a sublimit
    /// it is applied...). The value is limited by this instance of NumberLimit and all nested
    /// NumberLimit instances (an "and" operation) so to define a complex region define one
    /// inclusion limit that covers the entire range and add one or more exclusion limits to
    /// "carve out" pieces of that range and nest them.
    pub fn limit_int(&self, value: i32) -> i32 {
        let result = self.limit(value as f64);
        result as i32
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
