import { IStepGlobalParameters, ISweepGlobalParameters } from '../interface';
import { ParameterFloat, ParameterInt } from './TimingConfig';

export class StepGlobalParameters {
  step_points: ParameterInt;
  step_to_sweep_delay: ParameterFloat;
  list_step: boolean;

  constructor(data: IStepGlobalParameters) {
    this.step_points = new ParameterInt(data.step_points);
    this.step_to_sweep_delay = new ParameterFloat(data.step_to_sweep_delay);
    this.list_step = data.list_step;
  }

  toJSON() {
    return {
      step_points: this.step_points.toJSON(),
      step_to_sweep_delay: this.step_to_sweep_delay.toJSON(),
      list_step: this.list_step,
    };
  }
}

export class SweepGlobalParameters {
  sweep_points: ParameterInt;
  sweep_time_per_point: ParameterFloat;
  list_sweep: boolean;

  constructor(data: ISweepGlobalParameters) {
    this.sweep_points = new ParameterInt(data.sweep_points);
    this.sweep_time_per_point = new ParameterFloat(data.sweep_time_per_point);
    this.list_sweep = data.list_sweep;
  }

  toJSON() {
    return {
      sweep_points: this.sweep_points.toJSON(),
      sweep_time_per_point: this.sweep_time_per_point.toJSON(),
      list_sweep: this.list_sweep,
    };
  }
}
