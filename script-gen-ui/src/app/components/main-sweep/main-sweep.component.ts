import {
  Component,
  OnInit,
  ViewChildren,
  QueryList,
  Input,
  ViewChild,
  SimpleChanges,
} from '@angular/core';
import { BiasComponent } from './bias/bias.component';
import { StepComponent } from './step/step.component';
import { SweepComponent } from './sweep/sweep.component';
import { WebSocketService } from '../../websocket.service';
import {
  GlobalParameters,
  SweepConfig,
} from '../../model/sweep_data/sweepConfig';
import {
  ParameterFloat,
  ParameterInt,
  TimingConfig,
} from '../../model/sweep_data/TimingConfig';
import { BiasChannel } from '../../model/chan_data/biasChannel';
import { StepChannel } from '../../model/chan_data/stepChannel';
import { SweepChannel } from '../../model/chan_data/sweepChannel';
import {
  StepGlobalParameters,
  SweepGlobalParameters,
} from '../../model/sweep_data/stepSweepConfig';
import { TimingComponent } from './timing/timing.component';
import { SweepModel } from '../../model/sweep_data/sweepModel';
import { IpcData } from '../../model/ipcData';
import { Device } from '../../model/device_data/device';

@Component({
  selector: 'app-main-sweep',
  standalone: false,
  templateUrl: './main-sweep.component.html',
  styleUrls: ['./main-sweep.component.scss'],
})
export class MainSweepComponent {
  @ViewChildren(BiasComponent) biasComponents!: QueryList<BiasComponent>;
  @ViewChildren(StepComponent) stepComponents!: QueryList<StepComponent>;
  @ViewChildren(SweepComponent) sweepComponents!: QueryList<SweepComponent>;
  @ViewChild(TimingComponent) timingComponent!: TimingComponent;

  // console.log('biasComponents:', this.biasComponents);
  biasChannels: BiasChannel[] = [];
  stepChannels: StepChannel[] = [];
  sweepChannels: SweepChannel[] = [];

  activeComponent: 'bias' | 'step' | 'sweep' | null = null; // Tracks the active component
  activeIndex: number | null = null;

  colorIndex: number = 0;
  colors: string[] = ['#F6F07D', '#7FBDC6', '#C95B66', '#91CE32', '#FF9832', '#2626BF', '#E254A6', '#00E09B']
  colorMap: Map<string, string> = new Map();

  setActiveComponent(
    component: 'bias' | 'step' | 'sweep',
    index: number
  ): void {
    this.activeComponent = component;
    this.activeIndex = index;
    console.log(`Active Component: ${component}, Index: ${index}`);
  }

  // Called when an input box loses focus
  clearActiveComponent(): void {
    this.activeComponent = null;
    this.activeIndex = null;
  }

  showPopupBox = false;
  showTiming: boolean = false;
  timingConfig: TimingConfig | undefined;
  deviceList: Device[] = [];
  stepGlobalParameters: StepGlobalParameters | undefined;
  sweepGlobalParameters: SweepGlobalParameters | undefined;

  sweepPoints: ParameterInt | undefined;
  sweepTimePerPoint: ParameterFloat | undefined;
  list: boolean = false;

  @Input() sweepConfig: SweepConfig | undefined;

  isBiasExpanded: boolean = false;
  isStepExpanded: boolean = false;
  isSweepExpanded: boolean = false;
  channelsExpanderState: Map<string, boolean> = new Map<string, boolean>();

  constructor(private webSocketService: WebSocketService) {}

  // ngOnInit() {
  //   // this.updateAll();
  // }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['sweepConfig']) {
      // Handle the change in sweepConfig here if needed
      this.updateAll();
      console.log('sweepConfig updated:', this.sweepConfig);
    }
  }

  getBiasColor(biasChannel: BiasChannel): string {
    return this.colorMap.get(biasChannel.common_chan_attributes.uuid) || 'gray';
  }

  getStepColor(stepChannel: StepChannel): string {
    return this.colorMap.get(stepChannel.start_stop_channel.common_chan_attributes.uuid) || 'gray';
  }

  getSweepColor(sweepChannel: SweepChannel): string {
    return this.colorMap.get(sweepChannel.start_stop_channel.common_chan_attributes.uuid) || 'gray';
  }

  updateAll() {
    if (this.sweepConfig) {
      this.timingConfig = this.sweepConfig.global_parameters.timing_config;
      this.deviceList = this.sweepConfig.device_list;
      this.biasChannels = this.sweepConfig.bias_channels;
      this.stepChannels = this.sweepConfig.step_channels;
      this.sweepChannels = this.sweepConfig.sweep_channels;
      this.stepGlobalParameters = this.sweepConfig.step_global_parameters;
      this.sweepGlobalParameters = this.sweepConfig.sweep_global_parameters;

      this.sweepPoints = this.sweepGlobalParameters.sweep_points;
      this.sweepTimePerPoint = this.sweepGlobalParameters.sweep_time_per_point;

      this.biasChannels.forEach((biasChannel) => {
        const uuid = biasChannel.common_chan_attributes.uuid;
        if (!this.colorMap.has(uuid)) {
          const color = this.colors[this.colorIndex % this.colors.length];
          this.colorMap.set(uuid, color);
          this.colorIndex++;
        }
      });
      this.stepChannels.forEach((stepChannel) => {
        const uuid = stepChannel.start_stop_channel.common_chan_attributes.uuid;
        if (!this.colorMap.has(uuid)) {
          const color = this.colors[this.colorIndex % this.colors.length];
          this.colorMap.set(uuid, color);
          this.colorIndex++;
        }
      });
      this.sweepChannels.forEach((sweepChannel) => {
        const uuid = sweepChannel.start_stop_channel.common_chan_attributes.uuid;
        if (!this.colorMap.has(uuid)) {
          const color = this.colors[this.colorIndex % this.colors.length];
          this.colorMap.set(uuid, color);
          this.colorIndex++;
        }
      });
    }
  }

  enableTiming() {
    this.showTiming = true;
  }

  closeTimingDialog() {
    this.showTiming = false;
  }

  ngAfterViewInit(): void {
    this.biasComponents.changes.subscribe(() => {
      console.log('Updated biasComponents:', this.biasComponents.toArray());
    });
  }

  addBias() {
    this.submitSweepConfigAsJson('reallocation', 'add,bias');
  }

  addStep() {
    this.submitSweepConfigAsJson('reallocation', 'add,step');
  }

  addSweep() {
    this.submitSweepConfigAsJson('reallocation', 'add,sweep');
  }

  toggleBias() {
    this.isBiasExpanded = !this.isBiasExpanded;
  }

  toggleStep() {
    this.isStepExpanded = !this.isStepExpanded;
  }

  toggleSweep() {
    this.isSweepExpanded = !this.isSweepExpanded;
  }

  handleBiasChanExpanderStateChange(event: {
    uuid: string;
    isExpanded: boolean;
  }) {
    this.channelsExpanderState.set(event.uuid, event.isExpanded);
  }

  isBiasChannelExpanded(uuid: string): boolean {
    return this.channelsExpanderState.get(uuid) || false;
  }

  updateTimingConfig() {
    const timingConfig = this.timingComponent.getTimingConfigFromComponent();
    if (this.sweepConfig && this.sweepConfig.global_parameters) {
      this.sweepConfig.global_parameters.timing_config = timingConfig;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
  }

  updateBiasChannelsConfig(updatedBiasChannel: BiasChannel) {
    const index = this.biasChannels.findIndex(
      (channel) =>
        channel.common_chan_attributes.device_id ===
        updatedBiasChannel.common_chan_attributes.device_id
    );
    if (index !== -1) {
      this.biasChannels[index] = updatedBiasChannel;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
  }

  updateStepChannelsConfig(updatedStepChannel: StepChannel) {
    const index = this.stepChannels.findIndex(
      (channel) =>
        channel.start_stop_channel.common_chan_attributes.device_id ===
        updatedStepChannel.start_stop_channel.common_chan_attributes.device_id
    );
    if (index !== -1) {
      this.stepChannels[index] = updatedStepChannel;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
  }

  updateStepGlobalParameters(
    updatedStepGlobalParameters: StepGlobalParameters
  ) {
    this.stepGlobalParameters = updatedStepGlobalParameters;
    this.submitSweepConfigAsJson('evaluate_data', '');
  }

  updateSweepChannelsConfig(updatedSweepChannel: SweepChannel) {
    const index = this.sweepChannels.findIndex(
      (channel) =>
        channel.start_stop_channel.common_chan_attributes.device_id ===
        updatedSweepChannel.start_stop_channel.common_chan_attributes.device_id
    );
    if (index !== -1) {
      this.sweepChannels[index] = updatedSweepChannel;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
  }

  updateSweepGlobalParameters() {
    let sweepGlobalParams = new SweepGlobalParameters({
      sweep_points: this.sweepPoints,
      sweep_time_per_point: this.sweepTimePerPoint,
      list_sweep: this.list,
    });
    if (this.sweepConfig) {
      this.sweepConfig.sweep_global_parameters = sweepGlobalParams;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
  }

  removeBiasChannel(deviceID: string) {
    const index = this.biasChannels.findIndex(
      (channel) => channel.common_chan_attributes.device_id === deviceID
    );
    if (index !== -1) {
      this.biasChannels.splice(index, 1);
      this.submitSweepConfigAsJson('reallocation', 'remove,bias,' + deviceID);
    }
  }

  removeStepChannel(deviceID: string) {
    const index = this.stepChannels.findIndex(
      (channel) =>
        channel.start_stop_channel.common_chan_attributes.device_id ===
        deviceID
    );
    if (index !== -1) {
      this.stepChannels.splice(index, 1);
      this.submitSweepConfigAsJson('reallocation', 'remove,step,' + deviceID);
    }
  }

  removeSweepChannel(deviceID: string) {
    const index = this.sweepChannels.findIndex(
      (channel) =>
        channel.start_stop_channel.common_chan_attributes.device_id ===
        deviceID
    );
    if (index !== -1) {
      this.sweepChannels.splice(index, 1);
      this.submitSweepConfigAsJson('reallocation', 'remove,sweep,' + deviceID);
    }
  }

  updateBiasChannelId($event: { oldChanId: string; newChanId: string }) {
    this.submitSweepConfigAsJson(
      'reallocation',
      'update,bias,' + $event.oldChanId + ',' + $event.newChanId
    );
  }

  updateStepChannelId($event: { oldChanId: string; newChanId: string }) {
    this.submitSweepConfigAsJson(
      'reallocation',
      'update,step,' + $event.oldChanId + ',' + $event.newChanId
    );
  }

  updateSweepChannelId($event: { oldChanId: string; newChanId: string }) {
    this.submitSweepConfigAsJson(
      'reallocation',
      'update,sweep,' + $event.oldChanId + ',' + $event.newChanId
    );
  }

  submitSweepConfigAsJson(requestType: string, additionalInfo: string) {
    if (this.sweepConfig) {
      const sweepModel = new SweepModel({ sweep_config: this.sweepConfig });
      const sweepModelJson = JSON.stringify(sweepModel);

      const ipcData = new IpcData({
        request_type: requestType,
        additional_info: additionalInfo,
        json_value: sweepModelJson,
      });

      const ipcDataJson = JSON.stringify(ipcData);
      this.webSocketService.send(ipcDataJson);
      //console.log('Submitted combined data:', sweepModelJson);
    }
  }
}
