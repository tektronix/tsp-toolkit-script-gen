import { IStepChannel } from "../interface";
import { StartStopChannel } from "./startStopChannel";

export class StepChannel {
    start_stop_channel: StartStopChannel

    constructor(data: IStepChannel) {
        this.start_stop_channel = new StartStopChannel(data.start_stop_channel);
    }

    toJSON() {
        return {
            start_stop_channel: this.start_stop_channel.toJSON()
        }
    }
}