import { IDevice } from "../interface"

export enum DeviceType {
    Smu,
    Psu,
    Unknown,
}

export class Device {
    node_id: string
    _id: string
    device_type: DeviceType

    model: string
    fw_version: string
    in_use: boolean

    constructor(data: IDevice) {
        this.node_id = data.node_id
        this._id = data._id
        this.device_type = data.device_type
        this.model = data.model
        this.fw_version = data.fw_version
        this.in_use = data.in_use
    }

    toJSON() {
        return {
            node_id: this.node_id,
            _id: this._id,
            device_type: this.device_type,
            model: this.model,
            fw_version: this.fw_version,
            in_use: this.in_use,
        }
    }
}