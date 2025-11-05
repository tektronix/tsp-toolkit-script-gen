import { DeviceType } from './device_data/device';

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

export interface ISmuTiming {
  nplc: IParameterFloat;
  aperture: IParameterFloat;
  source_auto_delay: IParameterString;
  source_delay: IParameterFloat;
  measure_auto_delay: IParameterString;
  measure_delay: IParameterFloat;
  nplc_type: IParameterString;
}

export interface IPsuTiming {
  rate: IParameterString;
}

export interface ISweepTimingConfig {
  measure_count: IParameterInt;
  smu_timing: ISmuTiming;
  psu_timing: IPsuTiming;
}

export interface IGlobalParameters {
  sweep_timing_config: ISweepTimingConfig;
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
  source_limitv?: IParameterFloat;
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
  list: IParameterFloat[];
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
  list_sweep: boolean;
}

export interface IDevice {
  node_id: string;
  slot_id: string;
  chan_num: number;
  _id: string;

  model: string;
  device_type: DeviceType;

  in_use: boolean;
  is_valid: boolean;

  fw_version: string;
}

export enum StatusType {
  Info = "Info",
  Warning = "Warning",
  Error = "Error",
}

export interface IStatusMsg {
  status_type: StatusType;
  message: string;
  time_stamp: string;
}

export interface ISweepConfig {
  global_parameters: IGlobalParameters;
  bias_channels: IBiasChannel[];
  step_channels: IStepChannel[];
  sweep_channels: ISweepChannel[];
  step_global_parameters: IStepGlobalParameters;
  sweep_global_parameters: ISweepGlobalParameters;
  device_list: IDevice[];
  status_msg?: IStatusMsg;
}

export interface ISweepModel {
  sweep_config: ISweepConfig;
}

export interface IServerData {
  sweep_model: ISweepModel;
}
