import { IDevice } from '../interface';

export enum DeviceType {
  Smu,
  Psu,
  Unknown,
}

export class Device {
  node_id: string;
  slot_id: string;
  chan_num: number;
  _id: string;

  model: string;
  device_type: DeviceType;

  in_use: boolean;
  is_valid: boolean;

  fw_version: string;

  constructor(data: IDevice) {
    this.node_id = data.node_id;
    this.slot_id = data.slot_id;
    this.chan_num = data.chan_num;
    this._id = data._id;
    this.model = data.model;
    this.device_type = data.device_type;
    this.in_use = data.in_use;
    this.is_valid = data.is_valid;
    this.fw_version = data.fw_version;
  }

  toJSON() {
    return {
      node_id: this.node_id,
      slot_id: this.slot_id,
      chan_num: this.chan_num,
      _id: this._id,
      model: this.model,
      device_type: this.device_type,
      in_use: this.in_use,
      is_valid: this.is_valid,
      fw_version: this.fw_version,
    };
  }
}
