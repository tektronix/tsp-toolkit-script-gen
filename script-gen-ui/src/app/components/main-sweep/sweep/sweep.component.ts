import { Component, Input, Output, EventEmitter, SimpleChanges, OnChanges } from '@angular/core';
import { ChannelRange } from '../../../model/chan_data/channelRange';
import {
  ParameterFloat,
  ParameterInt,
  ParameterString,
} from '../../../model/sweep_data/SweepTimingConfig';
import { SweepChannel } from '../../../model/chan_data/sweepChannel';
import { CommonChanAttributes } from '../../../model/chan_data/defaultChannel';
import { Device } from '../../../model/device_data/device';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { DropdownComponent } from '../../controls/dropdown/dropdown.component';
import { InputPlainComponent } from '../../controls/input-plain/input-plain.component';
import { InputToggleComponent } from '../../controls/input-toggle/input-toggle.component';

@Component({
  selector: 'app-sweep',
  templateUrl: './sweep.component.html',
  styleUrls: ['./sweep.component.scss'],
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule, DropdownComponent, InputPlainComponent, InputToggleComponent],
})
export class SweepComponent implements OnChanges {
  commonChanAttributes: CommonChanAttributes | undefined;
  chanName = '';
  deviceID = '';
  uuid = '';
  dropdownDeviceList: string[] = [];

  sourceFunction: ParameterString | undefined;
  sourceRange: ChannelRange | undefined;
  measFunction: ParameterString | undefined;
  measRange: ChannelRange | undefined;
  sourceLimitI: ParameterFloat | undefined;
  sourceLimitV: ParameterFloat | undefined;
  senseMode: ParameterString | undefined;

  start: ParameterFloat | undefined;
  stop: ParameterFloat | undefined;
  style: ParameterString | undefined;

  //TODO: use this value directly from main-sweep.component.ts for plot computation
  sweepPoints = new ParameterInt({
    id: 'sweep_points',
    value: 10,
  });

  list = false;

  @Input() sweepChannel: SweepChannel | undefined;
  @Input() isSweepExpanded = false;
  @Input() deviceList: Device[] = [];
  @Output() emitSweepData = new EventEmitter<SweepChannel>();
  @Output() emitSweepChannelDelete = new EventEmitter<string>();
  @Output() emitSweepChannelIdChange = new EventEmitter<{
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

  expandedSweepChannels: Record<string, boolean> = {};

  // constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['sweepChannel']) {
      // Handle the change in biasChannel here if needed
      this.updateAll();
    }
  }

  updateAll() {
    if (this.sweepChannel) {
      this.commonChanAttributes =
        this.sweepChannel.start_stop_channel.common_chan_attributes;
      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      this.uuid = this.commonChanAttributes.uuid;

      this.sourceFunction = this.commonChanAttributes.source_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.measRange = this.commonChanAttributes.meas_range;
      this.sourceLimitI = this.commonChanAttributes.source_limiti;
      this.sourceLimitV = this.commonChanAttributes.source_limitv;
      this.senseMode = this.commonChanAttributes.sense_mode;

      this.start = this.sweepChannel.start_stop_channel.start;
      this.stop = this.sweepChannel.start_stop_channel.stop;
      this.style = this.sweepChannel.start_stop_channel.style;

      if (this.deviceList) {
        this.dropdownDeviceList = this.deviceList
          .filter((device) => !device.in_use) // Filter devices where in_use is false
          .map((device) => device._id);
        this.dropdownDeviceList.push(this.deviceID); // Add the current device ID to the list
      }
    }
  }

  toggleSweepChannel(deviceId: string) {
    this.expandedSweepChannels[deviceId] =
      !this.expandedSweepChannels[deviceId];
  }

  removeSweep() {
    this.emitSweepChannelDelete.emit(this.deviceID);
  }

  onToggle(selectedOption: string): void {
    console.log('Selected option:', selectedOption);
    this.submitSweepData();
    // this.sourceFunction?.value = selectedOption;
    // this.measFunction = selectedOption === 'Voltage' ? 'Current' : 'Voltage';
  }

  getSweepChanConfigFromComponent(): SweepChannel {
    return new SweepChannel({
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

  submitSweepData() {
    this.emitSweepData.emit(this.getSweepChanConfigFromComponent());
  }

  onEnter() {
    this.submitSweepData();
  }

  onDeviceIDChange($event: string) {
    if (this.sweepChannel) {
      this.emitSweepChannelIdChange.emit({
        oldChanId: this.sweepChannel.start_stop_channel.common_chan_attributes.device_id,
        newChanId: $event,
      });
    }
  }

  onChanNameChange(newName: string): void {
    console.log('Channel name changed to:', newName);
  }
  //TODO: Remove this function once plot is updated from main-sweep.component.ts
  // updatePlot() {}

  handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault(); // Prevent default scrolling for Space key
    }
  }
}
