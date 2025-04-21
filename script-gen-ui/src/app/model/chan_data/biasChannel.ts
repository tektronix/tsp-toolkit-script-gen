import { ParameterFloat } from '../sweep_data/TimingConfig';
import { CommonChanAttributes } from './defaultChannel';

export class BiasChannel {
  common_chan_attributes: CommonChanAttributes;
  bias: ParameterFloat;

  constructor(data: any) {
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
