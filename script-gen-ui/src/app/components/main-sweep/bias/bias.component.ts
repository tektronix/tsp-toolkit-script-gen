import {
  Component,
  Input,
  Output,
  EventEmitter,
  SimpleChanges, OnChanges,
} from '@angular/core';
import { BiasChannel } from '../../../model/chan_data/biasChannel';
import {
  ParameterFloat,
  ParameterString,
} from '../../../model/sweep_data/TimingConfig';
import { ChannelRange } from '../../../model/chan_data/channelRange';
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
  selector: 'app-bias',
  templateUrl: './bias.component.html',
  styleUrls: ['./bias.component.scss'],
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule, DropdownComponent, InputPlainComponent, InputToggleComponent],
})
export class BiasComponent implements OnChanges {
  commonChanAttributes: CommonChanAttributes | undefined;
  chanName = '';
  deviceID = '';
  uuid = '';
  dropdownDeviceList: string[] = [];
  @Input() isActive = false;

  toggleActive() {
    this.isActive = !this.isActive;
  }

  sourceFunction: ParameterString | undefined;
  sourceRange: ChannelRange | undefined;
  measFunction: ParameterString | undefined;
  measRange: ChannelRange | undefined;
  bias: ParameterFloat | undefined;
  sourceLimitI: ParameterFloat | undefined;
  sourceLimitV: ParameterFloat | undefined;
  senseMode: ParameterString | undefined;

  @Input() biasChannel: BiasChannel | undefined;
  @Input() isBiasChanExpanded = false;
  @Input() deviceList: Device[] = [];
  @Output() emitBiasChannelData = new EventEmitter<BiasChannel>();
  @Output() emitBiasExpanderState = new EventEmitter<{
    uuid: string;
    isExpanded: boolean;
  }>();
  @Output() emitBiasChannelDelete = new EventEmitter<string>();
  @Output() emitBiasChannelIdChange = new EventEmitter<{
    oldChanId: string;
    newChanId: string;
  }>();

  @Input() color = '';
  isFocused = false;

  toggleFocus(state: boolean): void {
    this.isFocused = state;
  }

  // constructor() {}

  // ngOnInit(): void {
  //   this.updateAll();
  // }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['biasChannel']) {
      // Handle the change in biasChannel here if needed
      this.updateAll();
    }
  }

  updateAll() {
    if (this.biasChannel) {
      this.commonChanAttributes = this.biasChannel.common_chan_attributes;
      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      this.uuid = this.commonChanAttributes.uuid;

      this.sourceFunction = this.commonChanAttributes.source_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.measRange = this.commonChanAttributes.meas_range;
      this.bias = this.biasChannel.bias;
      this.sourceLimitI = this.commonChanAttributes.source_limiti;
      this.sourceLimitV = this.commonChanAttributes.source_limitv;
      this.senseMode = this.commonChanAttributes.sense_mode;

      if (this.deviceList) {
        this.dropdownDeviceList = this.deviceList
          .filter((device) => !device.in_use) // Filter devices where in_use is false
          .map((device) => device._id);
        this.dropdownDeviceList.push(this.deviceID); // Add the current device ID to the list
      }
    }
  }

  toggleBiasChannel() {
    this.isBiasChanExpanded = !this.isBiasChanExpanded;
    this.emitBiasExpanderState.emit({
      uuid: this.uuid,
      isExpanded: this.isBiasChanExpanded,
    });
  }

  removeBias() {
    this.emitBiasChannelDelete.emit(this.deviceID);
  }

  getBiasChanConfigFromComponent(): BiasChannel {
    //TODO: for all values that can be undefined throw an error as it's not a valid scenario
    return new BiasChannel({
      common_chan_attributes: {
        uuid: this.uuid,
        chan_name: this.chanName,
        device_id: this.deviceID,
        source_function: this.sourceFunction!,
        source_range: this.sourceRange!,
        meas_function: this.measFunction!,
        meas_range: this.measRange!,
        source_limiti: this.sourceLimitI!,
        source_limitv: this.sourceLimitV!,
        sense_mode: this.senseMode,
      },
      bias: this.bias!,
    });
  }

  submitBiasData() {
    const biasChannel = this.getBiasChanConfigFromComponent();
    this.emitBiasChannelData.emit(biasChannel);
  }

  onBiasValueChange() {
    this.submitBiasData();
  }

  onRangeChange(event: Event): void {
    const target = event.target as HTMLSelectElement;
    const selectedRange = target.value;
    console.log('Selected range:', selectedRange);
    this.submitBiasData();
  }

  onToggle(selectedOption: string) {
    console.log('Selected option:', selectedOption);
    this.submitBiasData();
  }

  onDeviceIDChange($event: string) {
    if (this.biasChannel) {
      this.emitBiasChannelIdChange.emit({
        oldChanId: this.biasChannel.common_chan_attributes.device_id,
        newChanId: $event,
      });
    }
  }

  handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault(); // Prevent default scrolling for Space key
    }
  }
}
