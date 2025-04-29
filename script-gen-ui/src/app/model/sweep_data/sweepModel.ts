import { ISweepModel } from '../interface';
import { SweepConfig } from './sweepConfig';

export class SweepModel {
  sweep_config: SweepConfig;

  constructor(data: ISweepModel) {
    this.sweep_config = new SweepConfig(data.sweep_config);
  }

  toJSON() {
    return {
      sweep_config: this.sweep_config.toJSON(),
    };
  }
}
