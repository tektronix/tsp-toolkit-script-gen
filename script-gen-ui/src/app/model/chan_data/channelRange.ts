import { IChannelRange } from "../interface"

export class ChannelRange {
    range: string[]
    value: string

    constructor(data: IChannelRange) {
        this.range = data.range
        this.value = data.value
    }

    toJSON() {
        return {
            range: this.range,
            value: this.value
        }
    }
}