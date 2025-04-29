import { ICommonChanAttributes } from '../interface';
import { ParameterFloat, ParameterString } from '../sweep_data/TimingConfig';
import { ChannelRange } from './channelRange';

export class CommonChanAttributes {
  uuid: string;
  chan_name: string;
  device_id: string;
  source_function: ParameterString;
  meas_function: ParameterString;
  source_range: ChannelRange;
  meas_range: ChannelRange;

  source_limiti: ParameterFloat;
  source_limitv: ParameterFloat;
  sense_mode?: ParameterString;

  constructor(data: ICommonChanAttributes) {
    this.uuid = data.uuid;
    this.chan_name = data.chan_name;
    this.device_id = data.device_id;
    this.source_function = new ParameterString(data.source_function);
    this.meas_function = new ParameterString(data.meas_function);
    this.source_range = new ChannelRange(data.source_range);
    this.meas_range = new ChannelRange(data.meas_range);
    this.source_limiti = new ParameterFloat(data.source_limiti);
    this.source_limitv = new ParameterFloat(data.source_limitv);
    this.sense_mode = data.sense_mode
      ? new ParameterString(data.sense_mode)
      : undefined;
  }

  toJSON() {
    const json: ICommonChanAttributes = {
      uuid: this.uuid,
      chan_name: this.chan_name,
      device_id: this.device_id,
      source_function: this.source_function.toJSON(),
      meas_function: this.meas_function.toJSON(),
      source_range: this.source_range.toJSON(),
      meas_range: this.meas_range.toJSON(),
      source_limiti: this.source_limiti.toJSON(),
      source_limitv: this.source_limitv.toJSON(),
    };

    if (this.sense_mode) {
      json.sense_mode = this.sense_mode.toJSON();
    }

    return json;
  }
}
