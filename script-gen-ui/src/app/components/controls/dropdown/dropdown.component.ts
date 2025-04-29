import { CommonModule } from '@angular/common';
import {
  Component,
  Input,
  Output,
  EventEmitter,
  forwardRef, OnInit,
} from '@angular/core';
import {
  ControlValueAccessor,
  FormsModule,
  NG_VALUE_ACCESSOR,
} from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';

@Component({
  selector: 'app-dropdown',
  templateUrl: './dropdown.component.html',
  styleUrls: ['./dropdown.component.scss'],
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => DropdownComponent),
      multi: true,
    },
  ],
})
export class DropdownComponent implements ControlValueAccessor, OnInit {
  @Input() label: string | undefined;
  @Input() selected: string | undefined;
  @Input() options: string[] = [];
  // @Input() unit: string | undefined;
  @Output() selectedChange = new EventEmitter<string>();

  private onChange: ((value: string) => void) | undefined;

  ngOnInit(): void {
    // Ensure selected is initialized
    if (!this.selected && this.options.length > 0) {
      this.selected = this.options[0];
    }
  }

  writeValue(value: string): void {
    this.selected = value;
  }

  registerOnChange(fn: ((value: string) => void) | undefined): void {
    this.onChange = fn;
  }

  registerOnTouched(): void {
    console.log('Dropdown touched');
  }

  setDisabledState?(): void {
    // Handle the disabled state if needed
  }

  onSelectionChanged(event: Event): void {
    const value = (event.target as HTMLSelectElement).value;
    this.selected = value;
    if (this.onChange) {
      this.onChange(value);
    }
    this.selectedChange.emit(value); // Emit the selectedChange event
  }
}
