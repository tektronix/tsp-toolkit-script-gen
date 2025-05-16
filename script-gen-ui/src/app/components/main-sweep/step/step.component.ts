import {
  Component,
  Input,
  Output,
  EventEmitter,
  SimpleChanges, OnChanges,
  ElementRef,
} from '@angular/core';
import {
  ParameterInt,
  ParameterFloat,
  ParameterString,
} from '../../../model/sweep_data/SweepTimingConfig';
import { ChannelRange } from '../../../model/chan_data/channelRange';
import { StepChannel } from '../../../model/chan_data/stepChannel';
import { CommonChanAttributes } from '../../../model/chan_data/defaultChannel';
import { StepGlobalParameters } from '../../../model/sweep_data/stepSweepConfig';
import { Device } from '../../../model/device_data/device';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { DropdownComponent } from '../../controls/dropdown/dropdown.component';
import { InputNumericComponent } from '../../controls/input-numeric/input-numeric.component';
import { InputPlainComponent } from '../../controls/input-plain/input-plain.component';
import { InputToggleComponent } from '../../controls/input-toggle/input-toggle.component';

@Component({
  selector: 'app-step',
  templateUrl: './step.component.html',
  styleUrls: ['./step.component.scss'],
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule, DropdownComponent, InputNumericComponent, InputPlainComponent, InputToggleComponent],
})
export class StepComponent implements OnChanges {
  commonChanAttributes: CommonChanAttributes | undefined;
  chanName = '';
  deviceID = '';
  uuid = '';
  dropdownDeviceList: string[] = [];

  sourceRange: ChannelRange | undefined;
  sourceFunction: ParameterString | undefined;
  measRange: ChannelRange | undefined;
  measFunction: ParameterString | undefined;
  sourceLimitI: ParameterFloat | undefined;
  sourceLimitV: ParameterFloat | undefined;
  senseMode: ParameterString | undefined;

  stepPoints: ParameterInt | undefined;
  stepToSweepDelay: ParameterFloat | undefined;
  start: ParameterFloat | undefined;
  stop: ParameterFloat | undefined;
  style: ParameterString | undefined;
  list = false;

  @Input() stepChannel: StepChannel | undefined;
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;
  @Input() isStepExpanded = false;
  @Input() deviceList: Device[] = [];
  @Output() emitStepData = new EventEmitter<StepChannel>();
  @Output() emitStepGlobalParameters = new EventEmitter<StepGlobalParameters>();
  @Output() emitStepChannelDelete = new EventEmitter<string>();
  @Output() emitStepChannelIdChange = new EventEmitter<{
    oldChanId: string;
    newChanId: string;
  }>();

  @Input() isActive = false;
  @Input() color = '';
  isFocused = false;

  toggleFocus(state: boolean): void {
    this.isFocused = state;
  }

  toggleActive() {
    this.isActive = !this.isActive;
  }

  expandedStepChannels: Record<string, boolean> = {};

  constructor(public elementRef: ElementRef) {}

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['stepChannel']) {
      // Handle the change in biasChannel here if needed
      this.updateAll();
    }
  }

  updateAll() {
    if (this.stepChannel && this.stepGlobalParameters) {
      this.commonChanAttributes =
        this.stepChannel.start_stop_channel.common_chan_attributes;
      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      this.uuid = this.commonChanAttributes.uuid;

      this.sourceFunction = this.commonChanAttributes.source_function;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measRange = this.commonChanAttributes.meas_range;
      this.sourceLimitI = this.commonChanAttributes.source_limiti;
      this.sourceLimitV = this.commonChanAttributes.source_limitv;
      this.senseMode = this.commonChanAttributes.sense_mode;

      this.start = this.stepChannel.start_stop_channel.start;
      this.stop = this.stepChannel.start_stop_channel.stop;
      this.style = this.stepChannel.start_stop_channel.style;

      this.stepPoints = this.stepGlobalParameters.step_points;
      this.stepToSweepDelay = this.stepGlobalParameters.step_to_sweep_delay;

      if (this.deviceList) {
        this.dropdownDeviceList = this.deviceList
          .filter((device) => !device.in_use) // Filter devices where in_use is false
          .map((device) => device._id);
        this.dropdownDeviceList.push(this.deviceID); // Add the current device ID to the list
      }
    }
  }

  toggleStepChannel(stepName: string): void {
    this.expandedStepChannels[stepName] = !this.expandedStepChannels[stepName];
  }

  removeStep() {
    this.emitStepChannelDelete.emit(this.deviceID);
  }

  getStepChanConfigFromComponent(): StepChannel {
    return new StepChannel({
      start_stop_channel: {
        common_chan_attributes: {
          uuid: this.uuid,
          chan_name: this.chanName,
          device_id: this.deviceID,
          source_function: this.sourceFunction!,
          meas_function: this.measFunction!,
          source_range: this.sourceRange!,
          meas_range: this.measRange!,
          source_limiti: this.sourceLimitI!,
          source_limitv: this.sourceLimitV!,
          sense_mode: this.senseMode,
        },
        start: this.start!,
        stop: this.stop!,
        style: this.style!,
      },
    });
  }

  getStepGlobalParamsFromComponent(): StepGlobalParameters {
    return new StepGlobalParameters({
      step_points: this.stepPoints!,
      step_to_sweep_delay: this.stepToSweepDelay!,
      list_step: this.list,
    });
  }

  submitStepData() {
    this.emitStepData.emit(this.getStepChanConfigFromComponent());
  }

  onEnter() {
    this.submitStepData();
  }

  onToggle(selectedOption: string) {
    console.log('Selected option:', selectedOption);
    this.submitStepData();
  }

  onDeviceIDChange($event: string) {
    if (this.stepChannel) {
      this.emitStepChannelIdChange.emit({
        oldChanId: this.stepChannel.start_stop_channel.common_chan_attributes.device_id,
        newChanId: $event,
      });
    }
  }

  submitStepGlobalParamsData() {
    // this.updatePlot();
    this.emitStepGlobalParameters.emit(this.getStepGlobalParamsFromComponent());
  }

  // TODO: Remove this function once plot is updated from main-sweep.component.ts

}
