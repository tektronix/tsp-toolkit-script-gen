import { SweepModel } from './sweep_data/sweepModel';

export class ServerData {
  sweep_model: SweepModel;

  constructor(data: any) {
    this.sweep_model = new SweepModel(data.sweep_model);
  }
}