import {
  IParameterFloat,
  IParameterInt,
  IParameterString,
  IPsuTiming,
  ISmuTiming,
  ISweepTimingConfig,
} from '../interface';

export class ParameterInt {
  id: string;
  value: number;

  constructor(data: IParameterInt) {
    this.id = data.id;
    this.value = data.value;
  }

  toJSON() {
    return {
      id: this.id,
      value: this.value,
    };
  }
}

export class ParameterFloat {
  id: string;
  value: number;
  unit: string;

  constructor(data: IParameterFloat) {
    this.id = data.id;
    this.value = data.value;
    this.unit = data.unit;
  }

  toJSON() {
    return {
      id: this.id,
      value: this.value,
      unit: this.unit,
    };
  }
}

export class ParameterString {
  id: string;
  value: string;
  range: string[];

  constructor(data: IParameterString) {
    this.id = data.id;
    this.value = data.value;
    this.range = data.range;
  }

  toJSON() {
    return {
      id: this.id,
      value: this.value,
      range: this.range,
    };
  }
}

export class SweepTimingConfig {
  measure_count: ParameterInt;
  smu_timing: SmuTiming;
  psu_timing: PsuTiming;

  constructor(data: ISweepTimingConfig) {
    this.measure_count = new ParameterInt(data.measure_count);
    this.smu_timing = new SmuTiming(data.smu_timing);
    this.psu_timing = new PsuTiming(data.psu_timing);
  }

  toJSON() {
    return {
      measure_count: this.measure_count.toJSON(),
      smu_timing: this.smu_timing.toJSON(),
      psu_timing: this.psu_timing.toJSON(),
    };
  }
}

export class SmuTiming {
  nplc: ParameterFloat;
  aperture: ParameterFloat;
  source_auto_delay: ParameterString;
  source_delay: ParameterFloat;
  measure_auto_delay: ParameterString;
  measure_delay: ParameterFloat;
  nplc_type: ParameterString;

  constructor(data: ISmuTiming) {
    this.nplc = new ParameterFloat(data.nplc);
    this.aperture = new ParameterFloat(data.aperture);
    this.source_auto_delay = new ParameterString(data.source_auto_delay);
    this.source_delay = new ParameterFloat(data.source_delay);
    this.measure_auto_delay = new ParameterString(data.measure_auto_delay);
    this.measure_delay = new ParameterFloat(data.measure_delay);
    this.nplc_type = new ParameterString(data.nplc_type);
  }

  toJSON() {
    return {
      nplc: this.nplc.toJSON(),
      aperture: this.aperture.toJSON(),
      source_auto_delay: this.source_auto_delay.toJSON(),
      source_delay: this.source_delay.toJSON(),
      measure_auto_delay: this.measure_auto_delay.toJSON(),
      measure_delay: this.measure_delay.toJSON(),
      nplc_type: this.nplc_type.toJSON(),
    };
  }
}

export class PsuTiming {
  rate: ParameterString;
  aperture_value: number[];

  constructor(data: IPsuTiming) {
    this.rate = new ParameterString(data.rate);
    this.aperture_value = data.aperture_value;
  }

  toJSON() {
    return {
      rate: this.rate.toJSON(),
      aperture_value: this.aperture_value,
    };
  }
}
