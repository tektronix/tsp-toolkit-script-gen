/// <reference types="jasmine" />

import { ICommonChanAttributes } from '../interface';
import { CommonChanAttributes, Option } from './defaultChannel';

describe('CommonChanAttributes model identity', () => {
  function channelData(): ICommonChanAttributes {
    return {
      uuid: 'channel-1',
      chan_name: 'step1',
      device_id: 'node[2].slot[1].smu[1]',
      device_model: 'MSMU60-2',
      source_function: { id: 'source_function', value: 'Voltage', range: [] },
      meas_function: { id: 'meas_function', value: 'Current', range: [] },
      source_range: { value: 'AUTO', range: ['AUTO'] },
      meas_range: { value: 'AUTO', range: ['AUTO'] },
      source_limiti: { id: 'source_limiti', value: 0.1, unit: 'A' },
    };
  }

  it('preserves the assigned model and address through a JSON round trip', () => {
    const data = channelData();
    const channel = new CommonChanAttributes(data);
    const serialized: ICommonChanAttributes = JSON.parse(JSON.stringify(channel));
    const restored = new CommonChanAttributes(serialized);

    expect(serialized.device_model).toBe('MSMU60-2');
    expect(restored.device_model).toBe('MSMU60-2');
    expect(restored.device_id).toBe(data.device_id);
  });

  describe('Device dropdown option', () => {
    it('keeps selection value, address, model and label separate', () => {
      const address = 'node[2].slot[1].smu[1]';
      const options = ['MSMU60-2', 'MSMU200-2'].map((model) => new Option(
        JSON.stringify([address, model]),
        true,
        address,
        model,
        `${address} (${model})`,
      ));

      expect(options[0].value).not.toBe(options[1].value);
      expect(options[0].deviceId).toBe(options[1].deviceId);
      expect(options[0].deviceModel).toBe('MSMU60-2');
      expect(options[1].label).toBe(`${address} (MSMU200-2)`);
    });
  });

  it('leaves a missing legacy model unresolved instead of guessing', () => {
    const data = channelData();
    delete data.device_model;
    const channel = new CommonChanAttributes(data);

    expect(channel.device_model).toBe('');
    expect(channel.toJSON().device_model).toBe('');
    expect(channel.device_id).toBe(data.device_id);
  });
});
