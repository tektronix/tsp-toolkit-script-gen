import { CommonModule } from '@angular/common';
import {
  Component,
  Input,
  Output,
  EventEmitter,
  forwardRef,
  OnInit,
} from '@angular/core';
import {
  ControlValueAccessor,
  FormsModule,
  NG_VALUE_ACCESSOR,
} from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { Option } from '../../../model/chan_data/defaultChannel';

@Component({
  selector: 'app-channel-dropdown',
  templateUrl: './channel-dropdown.component.html',
  styleUrls: ['./channel-dropdown.component.scss'],
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => ChannelDropdownComponent),
      multi: true,
    },
  ],
})
export class ChannelDropdownComponent implements ControlValueAccessor, OnInit {
  @Input() label: string | undefined;
  @Input() selected: Option | undefined;
  @Input() options: Option[] = [];
  // @Input() unit: string | undefined;
  @Output() selectedChange = new EventEmitter<Option>();

  private onChange: ((value: Option) => void) | undefined;

  ngOnInit(): void {
    // Ensure selected is initialized
    if (!this.selected && this.options.length > 0) {
      this.selected = this.options[0];
    }
  }

  writeValue(value: Option): void {
    this.selected = value;
  }

  registerOnChange(fn: ((value: Option) => void) | undefined): void {
    this.onChange = fn;
  }

  registerOnTouched(): void {
    // console.log('Dropdown touched');
  }

  setDisabledState?(): void {
    // Handle the disabled state if needed
  }

  onSelectionChanged(event: Event): void {
    const value = (event.target as HTMLSelectElement).value;
    const selectedOption = this.options.find((opt) => opt.value === value);
    if (selectedOption) {
      this.selected = selectedOption;
      if (this.onChange) {
        this.onChange(selectedOption);
      }
      this.selectedChange.emit(selectedOption);
    }
  }
}
