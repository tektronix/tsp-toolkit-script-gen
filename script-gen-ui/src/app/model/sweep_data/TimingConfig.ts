export class ParameterInt {
  id: string;
  value: number;

  constructor(data: any) {
    this.id = data.id;
    this.value = data.value;
  }

  toJSON() {
    return {
      id: this.id,
      value: this.value
    };
  }
}

export class ParameterFloat {
  id: string;
  value: number;
  unit: string;

  constructor(data: any) {
    this.id = data.id;
    this.value = data.value;
    this.unit = data.unit;
  }

  toJSON() {
    return {
      id: this.id,
      value: this.value,
      unit: this.unit
    };
  }
}

export class ParameterString {
  id: string;
  value: string;
  range: string[];

  constructor(data: any) {
    this.id = data.id;
    this.value = data.value;
    this.range = data.range;
  }

  toJSON() {
    return {
      id: this.id,
      value: this.value,
      range: this.range
    };
  }
}

export class TimingConfig {
  nplc: ParameterFloat;
  auto_zero: ParameterString;
  source_delay_type: ParameterString;
  source_delay: ParameterFloat;
  measure_count: ParameterInt;
  measure_delay_type: ParameterString;
  measure_delay: ParameterFloat;
  measure_delay_factor: ParameterFloat;
  measure_filter_enable: ParameterString;
  measure_filter_type: ParameterString;
  measure_filter_count: ParameterInt;
  measure_analog_filter: ParameterString;

  high_speed_sampling: boolean;
  sampling_interval: ParameterFloat;
  sampling_count: ParameterInt;
  sampling_delay_type: ParameterString;
  sampling_delay: ParameterFloat;
  sampling_analog_filter: ParameterString;

  constructor(data: any) {
    this.nplc = new ParameterFloat(data.nplc);
    this.auto_zero = new ParameterString(data.auto_zero);
    this.source_delay_type = new ParameterString(data.source_delay_type);
    this.source_delay = new ParameterFloat(data.source_delay);
    this.measure_count = new ParameterInt(data.measure_count);
    this.measure_delay_type = new ParameterString(data.measure_delay_type);
    this.measure_delay = new ParameterFloat(data.measure_delay);
    this.measure_delay_factor = new ParameterFloat(data.measure_delay_factor);
    this.measure_filter_enable = new ParameterString(data.measure_filter_enable);
    this.measure_filter_type = new ParameterString(data.measure_filter_type);
    this.measure_filter_count = new ParameterInt(data.measure_filter_count);
    this.measure_analog_filter = new ParameterString(data.measure_analog_filter);

    this.high_speed_sampling = data.high_speed_sampling;
    this.sampling_interval = new ParameterFloat(data.sampling_interval);
    this.sampling_count = new ParameterInt(data.sampling_count);
    this.sampling_delay_type = new ParameterString(data.sampling_delay_type);
    this.sampling_delay = new ParameterFloat(data.sampling_delay);
    this.sampling_analog_filter = new ParameterString(data.sampling_analog_filter);
  }

  toJSON() {
    return {
      nplc: this.nplc.toJSON(),
      auto_zero: this.auto_zero.toJSON(),
      source_delay_type: this.source_delay_type.toJSON(),
      source_delay: this.source_delay.toJSON(),
      measure_count: this.measure_count.toJSON(),
      measure_delay_type: this.measure_delay_type.toJSON(),
      measure_delay: this.measure_delay.toJSON(),
      measure_delay_factor: this.measure_delay_factor.toJSON(),
      measure_filter_enable: this.measure_filter_enable.toJSON(),
      measure_filter_type: this.measure_filter_type.toJSON(),
      measure_filter_count: this.measure_filter_count.toJSON(),
      measure_analog_filter: this.measure_analog_filter.toJSON(),

      high_speed_sampling: this.high_speed_sampling,
      sampling_interval: this.sampling_interval.toJSON(),
      sampling_count: this.sampling_count.toJSON(),
      sampling_delay_type: this.sampling_delay_type.toJSON(),
      sampling_delay: this.sampling_delay.toJSON(),
      sampling_analog_filter: this.sampling_analog_filter.toJSON()
    };
  }
}
