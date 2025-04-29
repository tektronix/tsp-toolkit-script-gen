import { DeviceType } from "./device_data/device";

export interface IIpcDataInterface {
  request_type: string;
  additional_info: string;
  json_value: string;
}

export interface IParameterInt {
  id: string;
  value: number;
}

export interface IParameterFloat {
  id: string;
  value: number;
  unit: string;
}

export interface IParameterString {
  id: string;
  value: string;
  range: string[];
}

export interface IChannelRange {
  range: string[];
  value: string;
}

export interface ITimingConfig {
  nplc: IParameterFloat;
  auto_zero: IParameterString;
  source_delay_type: IParameterString;
  source_delay: IParameterFloat;
  measure_count: IParameterInt;
  measure_delay_type: IParameterString;
  measure_delay: IParameterFloat;
  measure_delay_factor: IParameterFloat;
  measure_filter_enable: IParameterString;
  measure_filter_type: IParameterString;
  measure_filter_count: IParameterInt;
  measure_analog_filter: IParameterString;

  high_speed_sampling: boolean;
  sampling_interval: IParameterFloat;
  sampling_count: IParameterInt;
  sampling_delay_type: IParameterString;
  sampling_delay: IParameterFloat;
  sampling_analog_filter: IParameterString;
}

export interface IGlobalParameters {
  timing_config: ITimingConfig;
}

export interface ICommonChanAttributes {
  uuid: string;
  chan_name: string;
  device_id: string;
  source_function: IParameterString;
  meas_function: IParameterString;
  source_range: IChannelRange;
  meas_range: IChannelRange;
  source_limiti: IParameterFloat;
  source_limitv: IParameterFloat;
  sense_mode?: IParameterString;
}

export interface IBiasChannel {
  common_chan_attributes: ICommonChanAttributes;
  bias: IParameterFloat;
}

export interface IStartStopChannel {
  common_chan_attributes: ICommonChanAttributes;
  start: IParameterFloat;
  stop: IParameterFloat;
  style: IParameterString;
}

export interface IStepChannel {
  start_stop_channel: IStartStopChannel;
}

export interface ISweepChannel {
  start_stop_channel: IStartStopChannel;
}

export interface IStepGlobalParameters {
  step_points: IParameterInt;
  step_to_sweep_delay: IParameterFloat;
  list_step: boolean;
}

export interface ISweepGlobalParameters {
  sweep_points: IParameterInt;
  sweep_time_per_point: IParameterFloat;
  list_sweep: boolean;
}

export interface IDevice {
  node_id: string;
  _id: string;
  device_type: DeviceType;

  model: string;
  fw_version: string;
  in_use: boolean;
}

export interface ISweepConfig {
  global_parameters: IGlobalParameters;
  bias_channels: IBiasChannel[];
  step_channels: IStepChannel[];
  sweep_channels: ISweepChannel[];
  step_global_parameters: IStepGlobalParameters;
  sweep_global_parameters: ISweepGlobalParameters;
  device_list: IDevice[];
}

export interface ISweepModel {
    sweep_config: ISweepConfig;
}

export interface IServerData {
    sweep_model: ISweepModel;
}
