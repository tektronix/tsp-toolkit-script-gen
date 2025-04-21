export class ChannelRange {
    range: string[]
    value: string

    constructor(data: any) {
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