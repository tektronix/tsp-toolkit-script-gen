import { BiasChannel } from '../chan_data/biasChannel';
import { StepChannel } from '../chan_data/stepChannel';
import { SweepChannel } from '../chan_data/sweepChannel';
import { Device } from '../device_data/device';
import { StepGlobalParameters, SweepGlobalParameters } from './stepSweepConfig';
import { ParameterFloat, ParameterInt, TimingConfig } from './TimingConfig';

export class GlobalParameters {
  timing_config: TimingConfig;

  constructor(data: any) {
    this.timing_config = new TimingConfig(data.timing_config);
  }

  toJSON() {
    return {
      timing_config: this.timing_config.toJSON(),
    };
  }
}

export class SweepConfig {
  global_parameters: GlobalParameters;
  bias_channels: BiasChannel[];
  step_channels: StepChannel[];
  sweep_channels: SweepChannel[];
  step_global_parameters: StepGlobalParameters;
  sweep_global_parameters: SweepGlobalParameters;
  device_list: Device[];

  constructor(data: any) {
    this.global_parameters = new GlobalParameters(data.global_parameters);
    this.bias_channels = data.bias_channels.map(
      (channel: any) => new BiasChannel(channel)
    );
    this.step_channels = data.step_channels.map(
      (channel: any) => new StepChannel(channel)
    );
    this.sweep_channels = data.sweep_channels.map(
      (channel: any) => new SweepChannel(channel)
    );
    this.step_global_parameters = new StepGlobalParameters(
      data.step_global_parameters
    );
    this.sweep_global_parameters = new SweepGlobalParameters(
      data.sweep_global_parameters
    );
    this.device_list = data.device_list.map(
      (device: any) => new Device(device)
    );
  }

  toJSON() {
    return {
      global_parameters: this.global_parameters.toJSON(),
      bias_channels: this.bias_channels.map((channel) => channel.toJSON()),
      step_channels: this.step_channels.map((channel) => channel.toJSON()),
      sweep_channels: this.sweep_channels.map((channel) => channel.toJSON()),
      step_global_parameters: this.step_global_parameters.toJSON(),
      sweep_global_parameters: this.sweep_global_parameters.toJSON(),
      device_list: this.device_list.map((device) => device.toJSON()),
    };
  }
}
