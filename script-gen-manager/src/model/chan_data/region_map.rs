use crate::model::sweep_data::number_limit::NumberLimit;

#[derive(Debug, Clone)]
pub struct VoltageCurrentRegion {
    id: i32,
    v1: f64,
    i1: f64,
    v2: f64,
    i2: f64,
}

impl VoltageCurrentRegion {
    pub fn new(id: i32, v1: f64, i1: f64, v2: f64, i2: f64) -> Self {
        VoltageCurrentRegion { id, v1, i1, v2, i2 }
    }
}

#[derive(Debug, Clone)]
pub struct RegionMapMetadata {
    exclude_v: Option<NumberLimit>,
    exclude_i: NumberLimit,
    regions: Vec<VoltageCurrentRegion>,
}

impl RegionMapMetadata {
    pub fn new(exclude_v: Option<NumberLimit>, exclude_i: NumberLimit) -> Self {
        RegionMapMetadata {
            exclude_v,
            exclude_i,
            regions: Vec::new(),
        }
    }

    pub fn add_region(&mut self, id: i32, v1: f64, i1: f64, v2: f64, i2: f64) {
        let region = VoltageCurrentRegion::new(id, v1, i1, v2, i2);
        self.regions.push(region);
    }

    /// Find the least restrictive (i.e. largest) current limit for the specified voltage.
    ///
    /// # Arguments
    ///
    /// * `value` - The voltage value for which to determine the current limit.
    ///
    /// # Returns
    ///
    /// * `NumberLimit` - The least restrictive current limit for the specified voltage.
    pub fn get_current_limit(&self, value: f64) -> NumberLimit {
        let mut first = true;
        let mut limit = NumberLimit::default();
        limit.set_sublimit(self.exclude_i.clone());

        for region in &self.regions {
            if region.v1 <= value && value <= region.v2 {
                if first {
                    first = false;
                    limit.set_min(region.i1);
                    limit.set_max(region.i2);
                } else {
                    limit.set_min(region.i1.min(limit.get_min()));
                    limit.set_max(region.i2.max(limit.get_max()));
                }
            }
        }

        limit
    }

    /// Find the least restrictive (i.e. largest) voltage limit for the specified current.
    ///
    /// # Arguments
    ///
    /// * `value` - The current value for which to determine the voltage limit.
    ///
    /// # Returns
    ///
    /// * `NumberLimit` - The least restrictive voltage limit for the specified current.
    pub fn get_voltage_limit(&self, value: f64) -> NumberLimit {
        let mut first = true;
        let mut limit = NumberLimit::default();
        if let Some(ref exclude_v) = self.exclude_v {
            limit.set_sublimit(exclude_v.clone());
        }

        for region in &self.regions {
            if region.i1 <= value && value <= region.i2 {
                if first {
                    first = false;
                    limit.set_min(region.v1);
                    limit.set_max(region.v2);
                } else {
                    limit.set_min(region.v1.min(limit.get_min()));
                    limit.set_max(region.v2.max(limit.get_max()));
                }
            }
        }

        limit
    }

    /// Find the region identifier for the specified voltage and current.
    ///
    /// # Arguments
    ///
    /// * `vpoint` - The voltage value.
    /// * `ipoint` - The current value.
    ///
    /// # Returns
    ///
    /// * `i32` - The region identifier, or -1 if not found.
    pub fn find_region(&self, vpoint: f64, ipoint: f64) -> i32 {
        for region in &self.regions {
            if region.v1 <= vpoint
                && region.i1 <= ipoint
                && vpoint <= region.v2
                && ipoint <= region.i2
            {
                return region.id;
            }
        }
        -1
    }
}
