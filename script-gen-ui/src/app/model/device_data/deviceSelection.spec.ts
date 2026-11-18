import { ElementRef, SimpleChange } from '@angular/core';
// Load the application entry first to honor the existing WebSocketService/AppComponent cycle.
import '../../app.component';
import { BiasComponent } from '../../components/main-sweep/bias/bias.component';
import { MainSweepComponent } from '../../components/main-sweep/main-sweep.component';
import { StepComponent } from '../../components/main-sweep/step/step.component';
import { SweepComponent } from '../../components/main-sweep/sweep/sweep.component';
import { StatusService } from '../../services/status.service';
import { WebSocketService } from '../../websocket.service';
import { BiasChannel } from '../chan_data/biasChannel';
import { CommonChanAttributes } from '../chan_data/defaultChannel';
import { StepChannel } from '../chan_data/stepChannel';
import { SweepChannel } from '../chan_data/sweepChannel';
import { StepGlobalParameters } from '../sweep_data/stepSweepConfig';
import { SweepConfig } from '../sweep_data/sweepConfig';
import { Device, DeviceType } from './device';
import { channelDeviceOptions, resolveChannelDevice } from './deviceSelection';

function attributes(model: string | undefined = '2450') {
  return new CommonChanAttributes({
    uuid: `channel-${model}`,
    chan_name: `Channel ${model}`,
    device_id: 'smu[1]',
    device_model: model,
    source_function: { id: 'source', value: 'Voltage', range: ['Voltage'] },
    meas_function: { id: 'measure', value: 'Current', range: ['Current'] },
    source_range: { value: 'Auto', range: ['Auto'] },
    meas_range: { value: 'Auto', range: ['Auto'] },
    source_limiti: { id: 'limit', value: 1, unit: 'A' },
  });
}

function device(model: string, isValid = true, inUse = false) {
  return new Device({
    _id: 'smu[1]', model, is_valid: isValid, in_use: inUse,
    node_id: '', slot_id: '', chan_num: 1,
    device_type: DeviceType.Smu, fw_version: '',
  });
}

function bias(model = '2450') {
  return new BiasChannel({
    common_chan_attributes: attributes(model),
    bias: { id: 'bias', value: 0, unit: 'V' },
  });
}

function startStop(model = '2450') {
  return {
    common_chan_attributes: attributes(model),
    start: { id: 'start', value: 0, unit: 'V' },
    stop: { id: 'stop', value: 1, unit: 'V' },
    style: { id: 'style', value: 'Linear', range: ['Linear'] },
    list: [],
  };
}

describe('Model-aware device identity', () => {
  it('round-trips model without changing the TSP address and defaults absent legacy model to empty', () => {
    const channel = attributes();
    expect(channel.toJSON().device_id).toBe('smu[1]');
    expect(channel.toJSON().device_model).toBe('2450');
    const legacy = channel.toJSON();
    delete legacy.device_model;
    expect(new CommonChanAttributes(legacy).toJSON().device_model).toBe('');
    expect(device('2450').toJSON()._id).toBe('smu[1]');
  });

  it('uses distinct address/model values and labels for a shared address', () => {
    const { options, selected } = channelDeviceOptions(attributes(), [device('2450'), device('2460')]);
    expect(new Set(options.map((option) => option.value)).size).toBe(2);
    expect(options.map((option) => option.label)).toEqual(['smu[1] (2450)', 'smu[1] (2460)']);
    expect(selected.deviceModel).toBe('2450');
  });

  it('retains an unavailable current model separately from a replacement and excludes occupied alternatives', () => {
    const { options, selected } = channelDeviceOptions(attributes(), [
      device('2450', false, true), device('2460'), device('2470', true, true),
    ]);
    expect(options.length).toBe(2);
    expect(selected.isValid).toBeFalse();
    expect(selected.deviceModel).toBe('2450');
    expect(selected.value).not.toBe(options[1].value);
    expect(options[1].deviceModel).toBe('2460');
  });

  it('retains a missing current device even when absent from the device list', () => {
    const result = channelDeviceOptions(attributes(), [device('2460')]);
    expect(result.selected.deviceId).toBe('smu[1]');
    expect(result.selected.isValid).toBeFalse();
    expect(result.selected.error).toContain('unavailable');
  });

  it('never infers a missing legacy model, even with one matching address', () => {
    const legacy = attributes('');
    const unique = channelDeviceOptions(legacy, [device('2450')]);
    expect(unique.selected.deviceModel).toBe('');
    expect(unique.selected.isValid).toBeFalse();
    expect(unique.selected.error).toContain('model is missing');
    expect(legacy.toJSON().device_model).toBe('');
    const ambiguous = channelDeviceOptions(legacy, [device('2450'), device('2460')]);
    expect(ambiguous.selected.deviceModel).toBe('');
    expect(ambiguous.selected.isValid).toBeFalse();
    expect(ambiguous.selected.error).toContain('model is missing');
    expect(channelDeviceOptions(legacy, [device('2450', false), device('2460')]).selected.error).toContain('model is missing');
    expect(resolveChannelDevice(attributes('2460'), [device('2450')]).device).toBeUndefined();
  });

  it('reports duplicate exact identities rather than silently choosing one', () => {
    const result = channelDeviceOptions(attributes(), [device('2450'), device('2450'), device('2460')]);
    expect(result.selected.isValid).toBeFalse();
    expect(result.selected.error).toContain('Ambiguous device');
    expect(result.options.map((option) => option.deviceModel)).toEqual(['2450', '2460']);
    expect(resolveChannelDevice(attributes(), [device('2450', false), device('2450')]).error).toContain('Ambiguous');
  });

  for (const role of ['bias', 'step', 'sweep'] as const) {
    it(`carries model through ${role} change, deletion and channel serialization`, () => {
      const element = new ElementRef(document.createElement('div'));
      const devices = [device('2450', false, true), device('2460')];
      const old = { deviceId: 'smu[1]', deviceModel: '2450' };
      const change = {
        oldChanId: 'smu[1]', newChanId: 'smu[1]', oldModel: '2450', newModel: '2460',
      };
      if (role === 'bias') {
        const component = new BiasComponent(element);
        component.biasChannel = bias();
        component.deviceList = devices;
        component.updateAll();
        const changed = spyOn(component.emitBiasChannelIdChange, 'emit');
        const removed = spyOn(component.emitBiasChannelDelete, 'emit');
        component.selectedDeviceOption = component.deviceOptionList[1];
        component.onDeviceIDChange(component.deviceOptionList[1]);
        component.removeBias();
        expect(changed).toHaveBeenCalledWith(change);
        expect(removed).toHaveBeenCalledWith(old);
        expect(component.getBiasChanConfigFromComponent().common_chan_attributes.toJSON().device_model).toBe('2450');
        expect(component.selectedDeviceOption?.deviceModel).toBe('2460');
        component.updateAll();
        expect(component.selectedDeviceOption?.deviceModel).toBe('2450');
        expect(component.getBiasChanConfigFromComponent().common_chan_attributes.device_id).toBe('smu[1]');
        component.onDeviceIDChange(component.deviceOptionList[0]);
        expect(changed).toHaveBeenCalledTimes(1);
      } else if (role === 'step') {
        const component = new StepComponent(element);
        component.stepChannel = new StepChannel({ start_stop_channel: startStop() });
        component.stepGlobalParameters = new StepGlobalParameters({
          step_points: { id: 'points', value: 2 },
          step_to_sweep_delay: { id: 'delay', value: 0, unit: 's' },
          list_step: false,
        });
        component.deviceList = devices;
        component.updateAll();
        const changed = spyOn(component.emitStepChannelIdChange, 'emit');
        const removed = spyOn(component.emitStepChannelDelete, 'emit');
        component.selectedDeviceOption = component.deviceOptionList[1];
        component.onDeviceIDChange(component.deviceOptionList[1]);
        component.removeStep();
        expect(changed).toHaveBeenCalledWith(change);
        expect(removed).toHaveBeenCalledWith(old);
        expect(component.getStepChanConfigFromComponent().start_stop_channel.common_chan_attributes.toJSON().device_model).toBe('2450');
        expect(component.selectedDeviceOption?.deviceModel).toBe('2460');
        component.updateAll();
        expect(component.selectedDeviceOption?.deviceModel).toBe('2450');
        expect(component.getStepChanConfigFromComponent().start_stop_channel.common_chan_attributes.device_id).toBe('smu[1]');
        component.onDeviceIDChange(component.deviceOptionList[0]);
        expect(changed).toHaveBeenCalledTimes(1);
        expect(component.stepListsWithNames[0].isDeviceValid).toBeFalse();
      } else {
        const component = new SweepComponent(element);
        component.sweepChannel = new SweepChannel({ start_stop_channel: startStop() });
        component.deviceList = devices;
        component.updateAll();
        const changed = spyOn(component.emitSweepChannelIdChange, 'emit');
        const removed = spyOn(component.emitSweepChannelDelete, 'emit');
        component.selectedDeviceOption = component.deviceOptionList[1];
        component.onDeviceIDChange(component.deviceOptionList[1]);
        component.removeSweep();
        expect(changed).toHaveBeenCalledWith(change);
        expect(removed).toHaveBeenCalledWith(old);
        expect(component.getSweepChanConfigFromComponent().start_stop_channel.common_chan_attributes.toJSON().device_model).toBe('2450');
        expect(component.selectedDeviceOption?.deviceModel).toBe('2460');
        component.updateAll();
        expect(component.selectedDeviceOption?.deviceModel).toBe('2450');
        expect(component.getSweepChanConfigFromComponent().start_stop_channel.common_chan_attributes.device_id).toBe('smu[1]');
        component.onDeviceIDChange(component.deviceOptionList[0]);
        expect(changed).toHaveBeenCalledTimes(1);
        component.deviceList = [device('2450')];
        component.ngOnChanges({ deviceList: new SimpleChange(devices, component.deviceList, false) });
        expect(component.selectedDeviceOption?.isValid).toBeTrue();
      }
    });
  }

  describe('Main sweep IPC and lookups', () => {
    let main: MainSweepComponent;
    let submit: jasmine.Spy;
    let socket: jasmine.SpyObj<WebSocketService>;

    beforeEach(() => {
      socket = jasmine.createSpyObj<WebSocketService>('socket', ['send']);
      main = new MainSweepComponent(socket, new StatusService());
      submit = spyOn(main, 'submitSweepConfigAsJson');
      main.biasChannels = [bias('2450'), bias('2460')];
      main.stepChannels = ['2450', '2460'].map((model) => new StepChannel({ start_stop_channel: startStop(model) }));
      main.sweepChannels = ['2450', '2460'].map((model) => new SweepChannel({ start_stop_channel: startStop(model) }));
    });

    it('updates channel data by model as well as address for every role', () => {
      const updatedBias = bias('2460');
      const updatedStep = new StepChannel({ start_stop_channel: startStop('2460') });
      const updatedSweep = new SweepChannel({ start_stop_channel: startStop('2460') });
      main.updateBiasChannelsConfig(updatedBias);
      main.updateStepChannelsConfig(updatedStep);
      main.updateSweepChannelsConfig(updatedSweep);
      expect(main.biasChannels[1]).toBe(updatedBias);
      expect(main.stepChannels[1]).toBe(updatedStep);
      expect(main.sweepChannels[1]).toBe(updatedSweep);
      expect(main.biasChannels[0].common_chan_attributes.device_model).toBe('2450');
    });

    it('leaves add operation payloads unchanged', () => {
      main.addBias();
      main.addStep();
      main.addSweep();
      expect(submit.calls.allArgs()).toEqual([
        ['reallocation', 'add,bias'],
        ['reallocation', 'add,step'],
        ['reallocation', 'add,sweep'],
      ]);
    });

    it('sends exact extended update strings, preserving legacy empty fields', () => {
      const change = { oldChanId: 'smu[1]', newChanId: 'smu[1]', oldModel: '2450', newModel: '2460' };
      main.updateBiasChannelId(change);
      main.updateStepChannelId(change);
      main.updateSweepChannelId({ ...change, oldModel: '' });
      expect(submit.calls.allArgs()).toEqual([
        ['reallocation', 'update,bias,smu[1],smu[1],2450,2460'],
        ['reallocation', 'update,step,smu[1],smu[1],2450,2460'],
        ['reallocation', 'update,sweep,smu[1],smu[1],,2460'],
      ]);
    });

    it('removes only the requested model and sends exact extended remove strings', () => {
      const identity = { deviceId: 'smu[1]', deviceModel: '2450' };
      main.removeBiasChannel(identity);
      main.removeStepChannel(identity);
      main.removeSweepChannel(identity);
      expect(submit.calls.allArgs()).toEqual([
        ['reallocation', 'remove,bias,smu[1],2450'],
        ['reallocation', 'remove,step,smu[1],2450'],
        ['reallocation', 'remove,sweep,smu[1],2450'],
      ]);
      expect(main.biasChannels.map((channel) => channel.common_chan_attributes.device_model)).toEqual(['2460']);
      expect(main.stepChannels.map((channel) => channel.start_stop_channel.common_chan_attributes.device_model)).toEqual(['2460']);
      expect(main.sweepChannels.map((channel) => channel.start_stop_channel.common_chan_attributes.device_model)).toEqual(['2460']);
    });

    it('sends both models over WebSocket and preserves the replacement assignment when removing the old model', () => {
      const float = { id: 'value', value: 1, unit: '' };
      const string = { id: 'value', value: 'Auto', range: ['Auto'] };
      main.sweepConfig = new SweepConfig({
        global_parameters: {
          line_frequency: 60, overhead_time: 0,
          sweep_timing_config: {
            measure_count: { id: 'count', value: 1 },
            smu_timing: {
              nplc: float, aperture: float, nplc_type: string,
              source_auto_delay: string, source_delay: float,
              measure_auto_delay: string, measure_delay: float,
            },
            psu_timing: { rate_normal: 1, rate_fast: 1, rate: string },
          },
        },
        bias_channels: main.biasChannels,
        step_channels: main.stepChannels,
        sweep_channels: main.sweepChannels,
        step_global_parameters: {
          step_points: { id: 'points', value: 2 },
          step_to_sweep_delay: float, list_step: false,
        },
        sweep_global_parameters: {
          sweep_points: { id: 'points', value: 2 }, list_sweep: false,
        },
        device_list: [device('2450', false, true), device('2460', true, true)],
      });
      main.updateAll();
      submit.and.callThrough();
      main.updateBiasChannelId({
        oldChanId: 'smu[1]', newChanId: 'smu[1]', oldModel: '2450', newModel: '2460',
      });
      const update = JSON.parse(socket.send.calls.mostRecent().args[0]);
      expect(update.request_type).toBe('reallocation');
      expect(update.additional_info).toBe('update,bias,smu[1],smu[1],2450,2460');
      main.removeBiasChannel({ deviceId: 'smu[1]', deviceModel: '2450' });
      const removal = JSON.parse(socket.send.calls.mostRecent().args[0]);
      expect(removal.additional_info).toBe('remove,bias,smu[1],2450');
      const config = JSON.parse(removal.json_value).sweep_config;
      expect(config.bias_channels.length).toBe(1);
      expect(config.bias_channels[0].common_chan_attributes.device_id).toBe('smu[1]');
      expect(config.bias_channels[0].common_chan_attributes.device_model).toBe('2460');
      expect(config.device_list.find((entry: Device) => entry.model === '2460').in_use).toBeTrue();
    });

    it('validates sweep lists against model instead of accepting another model at the same address', () => {
      main.deviceList = [device('2450')];
      main.updateSweepListsWithNames();
      expect(main.sweepListsWithNames.map((list) => list.isDeviceValid)).toEqual([true, false]);
    });
  });
});
