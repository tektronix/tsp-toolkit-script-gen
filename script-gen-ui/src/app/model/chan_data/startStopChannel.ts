import { IStartStopChannel } from "../interface";
import { ParameterFloat, ParameterString } from "../sweep_data/SweepTimingConfig"
import { CommonChanAttributes } from "./defaultChannel";

export class StartStopChannel {
    common_chan_attributes: CommonChanAttributes
    start: ParameterFloat
    stop: ParameterFloat
    style: ParameterString
    list: ParameterFloat[] = []

    constructor(data: IStartStopChannel) {
        this.common_chan_attributes = new CommonChanAttributes(data.common_chan_attributes);
        this.start = new ParameterFloat(data.start);
        this.stop = new ParameterFloat(data.stop);
        this.style = new ParameterString(data.style);
        this.list = data.list.map(item => new ParameterFloat(item));
    }

    toJSON() {
        return {
            common_chan_attributes: this.common_chan_attributes.toJSON(),
            start: this.start.toJSON(),
            stop: this.stop.toJSON(),
            style: this.style.toJSON(),
            list: this.list
        }
    }
}
