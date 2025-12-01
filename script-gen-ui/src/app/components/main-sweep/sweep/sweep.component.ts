import {
  Component,
  Input,
  Output,
  EventEmitter,
  SimpleChanges,
  OnChanges,
  ElementRef,
} from '@angular/core';
import { ChannelRange } from '../../../model/chan_data/channelRange';
import {
  ParameterFloat,
  ParameterString,
} from '../../../model/sweep_data/SweepTimingConfig';
import { SweepChannel } from '../../../model/chan_data/sweepChannel';
import { CommonChanAttributes } from '../../../model/chan_data/defaultChannel';
import { Device } from '../../../model/device_data/device';
import { Option } from '../../../model/chan_data/defaultChannel';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { DropdownComponent } from '../../controls/dropdown/dropdown.component';
import { ChannelDropdownComponent } from '../../controls/channel-dropdown/channel-dropdown.component';
import { InputPlainComponent } from '../../controls/input-plain/input-plain.component';
import { InputToggleComponent } from '../../controls/input-toggle/input-toggle.component';

@Component({
  selector: 'app-sweep',
  templateUrl: './sweep.component.html',
  styleUrls: ['./sweep.component.scss'],
  standalone: true,
  imports: [
    FormsModule,
    BrowserModule,
    CommonModule,
    MatIconModule,
    DropdownComponent,
    ChannelDropdownComponent,
    InputPlainComponent,
    InputToggleComponent,
  ],
})
export class SweepComponent implements OnChanges {
  commonChanAttributes: CommonChanAttributes | undefined;
  chanName = '';
  selectedDeviceOption: Option | undefined;
  uuid = '';
  deviceOptionList: Option[] = [];

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

  listSweep = false;
  list: ParameterFloat[] = [];

  @Input() sweepChannel: SweepChannel | undefined;
  @Input() isSweepExpanded = false;
  @Input() deviceList: Device[] = [];
  @Input() isListSweep = false;
  @Output() emitSweepExpanderState = new EventEmitter<{
    uuid: string;
    isExpanded: boolean;
  }>();
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

  constructor(public elementRef: ElementRef) {}

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

      this.list = this.sweepChannel.start_stop_channel.list;

      if (this.deviceList) {
        this.deviceOptionList = this.deviceList
          .filter(
            (device) =>
              !device.in_use ||
              device._id === this.commonChanAttributes?.device_id
          ) // Include current device even if in use
          .map((device) => new Option(device._id, device.is_valid)); // Create Option objects

        // Set the selected device option based on the sweep channel's device ID
        this.selectedDeviceOption = this.deviceOptionList.find(
          (option) => option.value === this.commonChanAttributes?.device_id // Match by device ID
        );
      }
    }
  }

  toggleSweepChannel(): void {
    this.isSweepExpanded = !this.isSweepExpanded;
    this.emitSweepExpanderState.emit({
      uuid: this.uuid,
      isExpanded: this.isSweepExpanded,
    });
  }

  removeSweep() {
    this.emitSweepChannelDelete.emit(this.selectedDeviceOption?.value || '');
  }

  getSweepChanConfigFromComponent(): SweepChannel {
    return new SweepChannel({
      start_stop_channel: {
        common_chan_attributes: {
          uuid: this.uuid,
          chan_name: this.chanName,
          device_id: this.selectedDeviceOption?.value || '',
          source_function: this.sourceFunction!,
          meas_function: this.measFunction!,
          source_range: this.sourceRange!,
          meas_range: this.measRange!,
          source_limiti: this.sourceLimitI!,
          source_limitv: this.sourceLimitV,
          sense_mode: this.senseMode,
        },
        start: this.start!,
        stop: this.stop!,
        style: this.style!,
        list: this.list,
      },
    });
  }

  submitSweepData() {
    this.emitSweepData.emit(this.getSweepChanConfigFromComponent());
  }

  onDeviceIDChange($event: Option) {
    if (this.sweepChannel) {
      this.emitSweepChannelIdChange.emit({
        oldChanId:
          this.sweepChannel.start_stop_channel.common_chan_attributes.device_id,
        newChanId: $event.value,
      });
    }
  }

  onChanNameChange(newName: string): void {
    console.log('Channel name changed to:', newName);
  }

  handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault(); // Prevent default scrolling for Space key
    }
  }
}
