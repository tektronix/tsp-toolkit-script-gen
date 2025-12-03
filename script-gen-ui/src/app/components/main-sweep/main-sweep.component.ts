/* eslint-disable @typescript-eslint/consistent-generic-constructors */
import {
  Component,
  ViewChildren,
  QueryList,
  Input,
  ViewChild,
  SimpleChanges,
  OnChanges,
} from '@angular/core';
import { BiasComponent } from './bias/bias.component';
import { StepComponent } from './step/step.component';
import { SweepComponent } from './sweep/sweep.component';
import { WebSocketService } from '../../websocket.service';
import { GlobalParameters, SweepConfig } from '../../model/sweep_data/sweepConfig';
import {
  ParameterFloat,
  ParameterInt,
  SweepTimingConfig,
} from '../../model/sweep_data/SweepTimingConfig';
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
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { PlotContainerComponent } from './plot-container/plot-container.component';
import { InputNumericComponent } from '../controls/input-numeric/input-numeric.component';
import { ListComponent } from './list/list.component';
import { TooltipComponent } from './tooltip/tooltip.component';
import { StatusService } from '../../services/status.service';

@Component({
  selector: 'app-main-sweep',
  standalone: true,
  imports: [
    FormsModule,
    BrowserModule,
    CommonModule,
    MatIconModule,
    InputNumericComponent,
    BiasComponent,
    StepComponent,
    SweepComponent,
    TimingComponent,
    PlotContainerComponent,
    ListComponent,
    TooltipComponent
],
  templateUrl: './main-sweep.component.html',
  styleUrls: ['./main-sweep.component.scss'],
})
export class MainSweepComponent implements OnChanges {
  @ViewChildren(BiasComponent) biasComponents!: QueryList<BiasComponent>;
  @ViewChildren(StepComponent) stepComponents!: QueryList<StepComponent>;
  @ViewChildren(SweepComponent) sweepComponents!: QueryList<SweepComponent>;
  @ViewChild(TimingComponent) timingComponent!: TimingComponent;
  @ViewChild(PlotContainerComponent) plotContainer!: PlotContainerComponent;
  biasChannels: BiasChannel[] = [];
  stepChannels: StepChannel[] = [];
  sweepChannels: SweepChannel[] = [];

  activeComponent: 'bias' | 'step' | 'sweep' | null = null; // Tracks the active component
  activeIndex: number | null = null;
  isScrolled = false;

  colorIndex = 0;
  colors: string[] = [
    '#F6F07D',
    '#7FBDC6',
    '#C95B66',
    '#91CE32',
    '#FF9832',
    '#E254A6',
    '#00E09B',
  ];
  colorMap = new Map<string, string>();

  setActiveComponent(
    component: 'bias' | 'step' | 'sweep',
    index: number
  ): void {
    // Reset scroll flag when switching to a different component
    if (this.activeComponent !== component || this.activeIndex !== index) {
      this.isScrolled = false;
    }

    this.activeComponent = component;
    this.activeIndex = index;
    console.log(`Active Component: ${component}, Index: ${index}`);
  }

  // Called when an input box loses focus
  // clearActiveComponent(): void {
  //   this.activeComponent = null;
  //   this.activeIndex = null;
  //   this.isScrolled = false; // Reset scroll flag when clearing active component
  // }

  showPopupBox = false;
  showTiming = false;
  sweepTimingConfig: SweepTimingConfig | undefined;
  globalParameters: GlobalParameters | undefined;
  showList = false; //Sweep list popup box
  deviceList: Device[] = [];
  showBiasTooltip = false;
  showStepTooltip = false;
  showSweepTooltip = false;
  showBiasExpanderTooltip = false;
  showStepExpanderTooltip = false;
  showSweepExpanderTooltip = false;
  stepGlobalParameters: StepGlobalParameters | undefined;
  sweepGlobalParameters: SweepGlobalParameters | undefined;

  sweepPoints: ParameterInt | undefined;
  isListSweep = false; // list sweep checkbox
  sweepListsWithNames: {
    chanName: string;
    list: ParameterFloat[];
    isDeviceValid: boolean;
  }[] = [];

  showStepListStates: Record<string, boolean> = {}; // step list pop up box boolean tracking
  stepListPositions: Map<string, { left: number; top: number }> = new Map(); // step list positions
  @Input() sweepConfig: SweepConfig | undefined;

  isBiasExpanded = false;
  isStepExpanded = false;
  isSweepExpanded = false;
  channelsExpanderState: Map<string, boolean> = new Map<string, boolean>();

  constructor(private webSocketService: WebSocketService, private statusService: StatusService) {}

  // ngOnInit() {
  //   this.updateSweepListsWithNames();
  // }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['sweepConfig']) {
      // Handle the change in sweepConfig here if needed
      this.updateAll();
      console.log('sweepConfig updated:', this.sweepConfig);

      // Re-scroll to active plot after data updates if there's an active component
      if (this.activeComponent !== null && this.activeIndex !== null) {
        setTimeout(() => {
          this.scrollToPlotInPlotContainer(this.activeComponent!, this.activeIndex!);
        }, 10);
      }
    }
  }

  getBiasColor(biasChannel: BiasChannel): string {
    return this.colorMap.get(biasChannel.common_chan_attributes.uuid) || 'gray';
  }

  getStepColor(stepChannel: StepChannel): string {
    return (
      this.colorMap.get(
        stepChannel.start_stop_channel.common_chan_attributes.uuid
      ) || 'gray'
    );
  }

  getSweepColor(sweepChannel: SweepChannel): string {
    return (
      this.colorMap.get(
        sweepChannel.start_stop_channel.common_chan_attributes.uuid
      ) || 'gray'
    );
  }

  scrollToInstance(
    componentType: 'bias' | 'step' | 'sweep',
    index: number
  ): void {
    // Skip scrolling if this is already the active component (clicked again)
    if (this.isScrolled === true) {
      return;
    }

    let component: BiasComponent | StepComponent | SweepComponent | undefined;

    if (componentType === 'bias') {
      component = this.biasComponents.toArray()[index];
    } else if (componentType === 'step') {
      component = this.stepComponents.toArray()[index];
    } else if (componentType === 'sweep') {
      component = this.sweepComponents.toArray()[index];
    }

    if (component) {
      const element = component.elementRef.nativeElement; // Access the DOM element
      element.scrollIntoView({
        behavior: 'smooth', // Smooth scrolling
        block: 'center', // Align to the center of the viewport
      });
    }
    this.isScrolled = true;
  }

  scrollToPlotInPlotContainer(
    componentType: 'bias' | 'step' | 'sweep',
    index: number
  ): void {
    if (this.plotContainer) {
      // Use setTimeout to ensure scroll happens after any pending view updates
      setTimeout(() => {
        this.plotContainer.scrollToPlot(componentType, index);
      }, 100);
    }
  }

  updateAll() {
    if (this.sweepConfig) {
      this.sweepTimingConfig =
        this.sweepConfig.global_parameters.sweep_timing_config;

      this.deviceList = this.sweepConfig.device_list;
      this.biasChannels = this.sweepConfig.bias_channels;
      this.stepChannels = this.sweepConfig.step_channels;
      this.sweepChannels = this.sweepConfig.sweep_channels;
      this.stepGlobalParameters = this.sweepConfig.step_global_parameters;
      this.sweepGlobalParameters = this.sweepConfig.sweep_global_parameters;
      this.globalParameters = this.sweepConfig.global_parameters;

      this.sweepPoints = this.sweepGlobalParameters.sweep_points;
      this.isListSweep = this.sweepGlobalParameters.list_sweep;
      // this.list = this.sweepChannels[0].list;
      this.updateSweepListsWithNames();
      this.updateStatusMsg();

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
        const uuid =
          sweepChannel.start_stop_channel.common_chan_attributes.uuid;
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

  enableList() {
    this.showList = true;
  }

  closeList() {
    this.showList = false;
  }

  onShowStepListChange(stepId: string, value: boolean) {
    this.showStepListStates[stepId] = value;
  }

  onStepListPositionChange(data: { stepId: string; position: { left: number; top: number } }) {
    this.stepListPositions.set(data.stepId, data.position);
  }

  getStepListPosition(stepId: string): { left: number; top: number } | null {
    return this.stepListPositions.get(stepId) || null;
  }

  listOfSweepPointsUpdate(
    points: { chanName: string; list: ParameterFloat[] }[]
  ) {
    for (const incoming of points) {
      const sweep = this.sweepChannels.find(
        (s) =>
          s.start_stop_channel.common_chan_attributes.chan_name ===
          incoming.chanName
      );
      if (sweep) {
        sweep.start_stop_channel.list = incoming.list;
      }
    }
    this.sweepChannels.forEach((sweepChannel) =>
      this.updateSweepChannelsConfig(sweepChannel)
    );
  }

  updateSweepListsWithNames() {
    this.sweepListsWithNames = this.sweepChannels.map((sweep) => {
      const matchingDevice = this.deviceList.find(
        (device) =>
          device._id ===
          sweep.start_stop_channel.common_chan_attributes.device_id
      );
      return {
        chanName: sweep.start_stop_channel.common_chan_attributes.chan_name,
        list: sweep.start_stop_channel.list,
        isDeviceValid: matchingDevice ? matchingDevice.is_valid : false,
      };
    });
  }

  updateStatusMsg() {
    if (this.sweepConfig && this.sweepConfig.status_msg) {
      this.statusService.showTemporary(this.sweepConfig.status_msg, 5000);
    }
  }

  sweepPointsUpdatefromList(points: number) {
    if (this.sweepPoints) {
      this.sweepPoints.value = points;
      this.updateSweepGlobalParameters();
    }
  }

  addBias() {
    this.submitSweepConfigAsJson('reallocation', 'add,bias');
  }

  addStep() {
    this.submitSweepConfigAsJson('reallocation', 'add,step');
  }

  addSweep() {
    this.submitSweepConfigAsJson('reallocation', 'add,sweep');
    // this.buildListSweep();  //the sweepchannels is not updated yet, so the list will not be updated
  }

  hasAvailableChannels(): boolean {
    if (!this.deviceList) return false;
    return this.deviceList.some((device) => !device.in_use && device.is_valid);
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

  handleChannelExpanderStateChange(uuid: string, isExpanded: boolean): void {
    this.channelsExpanderState.set(uuid, isExpanded);
  }

  isChannelExpanded(uuid: string): boolean {
    return this.channelsExpanderState.get(uuid) || false;
  }

  hasBiasChannels(): boolean {
    return this.biasChannels.length > 0;
  }

  hasStepChannels(): boolean {
    return this.stepChannels.length > 0;
  }

  hasSweepChannels(): boolean {
    if (this.sweepChannels.length > 0) {
      return true;
    } else {
      this.isSweepExpanded = false; // if no sweep channels, close the expander
      return false;
    }
  }

  updateTimingConfig() {
    const sweepTimingConfig =
      this.timingComponent.getSweepTimingConfigFromComponent();
    if (this.sweepConfig && this.sweepConfig.global_parameters) {
      this.sweepConfig.global_parameters.sweep_timing_config =
        sweepTimingConfig;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
  }

  openScript() {
    // Create an IPC message requesting to fetch the script
    const ipcData = new IpcData({
      request_type: 'open_script',
      additional_info: '',
      json_value: '{}',
    });

    // Convert to JSON string
    const message = JSON.stringify(ipcData);

    // Send request via WebSocket service
    this.webSocketService.send(message);
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
    // this.stepGlobalParameters = updatedStepGlobalParameters;  // this updates other globals but not the boolean list_step
    if (this.sweepConfig) {
      this.sweepConfig.step_global_parameters = updatedStepGlobalParameters;
      this.submitSweepConfigAsJson('evaluate_data', '');
    }
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
    const sweepGlobalParams = new SweepGlobalParameters({
      sweep_points: this.sweepPoints!,
      list_sweep: this.isListSweep,
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
        channel.start_stop_channel.common_chan_attributes.device_id === deviceID
    );
    if (index !== -1) {
      this.stepChannels.splice(index, 1);
      this.submitSweepConfigAsJson('reallocation', 'remove,step,' + deviceID);
    }
  }

  removeSweepChannel(deviceID: string) {
    const index = this.sweepChannels.findIndex(
      (channel) =>
        channel.start_stop_channel.common_chan_attributes.device_id === deviceID
    );
    if (index !== -1) {
      this.sweepChannels.splice(index, 1);
      this.submitSweepConfigAsJson('reallocation', 'remove,sweep,' + deviceID);
    }
    this.updateSweepListsWithNames();
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
