import { IBiasChannel } from '../interface';
import { ParameterFloat } from '../sweep_data/SweepTimingConfig';
import { CommonChanAttributes } from './defaultChannel';

export class BiasChannel {
  common_chan_attributes: CommonChanAttributes;
  bias: ParameterFloat;

  constructor(data: IBiasChannel) {
    this.common_chan_attributes = new CommonChanAttributes(
      data.common_chan_attributes
    );
    this.bias = new ParameterFloat(data.bias);
  }

  toJSON() {
    return {
      common_chan_attributes: this.common_chan_attributes.toJSON(),
      bias: this.bias.toJSON(),
    };
  }
}
