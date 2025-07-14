import { CommonModule } from '@angular/common';
import {
  Component,
  EventEmitter,
  forwardRef,
  Input,
  Output, OnInit,
} from '@angular/core';
import { ControlValueAccessor, FormsModule, NG_VALUE_ACCESSOR } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';

@Component({
  selector: 'app-input-numeric',
  templateUrl: './input-numeric.component.html',
  styleUrl: './input-numeric.component.scss',
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => InputNumericComponent),
      multi: true,
    },
  ],
})
export class InputNumericComponent implements ControlValueAccessor, OnInit {
  @Input() label: string | undefined;

  @Input() automationID: string | undefined;

  @Output() inputChange = new EventEmitter<number>();

  private _value: number | undefined;
  private onChange: ((value: number) => void) | undefined;

  ngOnInit(): void {
    console.log('InputNumericComponent initialized with label:', this.label);
  }

  get displayValue(): string {
    // if(this._value){
    //   this._value = Math.floor(this._value);
    //   console.log('displayvalue : ', this._value);
    //   return `${this._value}`;
    // }
    return this._value !== undefined ? `${this._value}` : '';
  }

  set displayValue(val: string) {
    const parsedValue = parseFloat(val);
    if (!isNaN(parsedValue)) {
      this._value = Math.floor(parsedValue);
      if (this.onChange) {
        this.onChange(this._value);
      }
      this.inputChange.emit(this._value);
    } else {
      // this._value = undefined;
    }
  }

  writeValue(value: number): void {
    if (value !== undefined) {
      this._value = value;
    }
  }

  registerOnChange(fn: ((value: number) => void) | undefined): void {
    this.onChange = fn;
  }

  registerOnTouched(): void {
    // console.log('InputNumericComponent touched');
  }

  setDisabledState?(): void {
    // Handle the disabled state if needed
  }

  onInputChange(event: Event): void {
    const inputElement = event.target as HTMLInputElement;

    if (this.displayValue !== inputElement.value) {
      let value = parseFloat(inputElement.value);
      value = Math.floor(value);

      // Ensure the value is not negative
      if (isNaN(value) || value < 0) {
        value = 1; // Set to 1 if the value is invalid or negative
      }

      this.displayValue = `${value}`;
      inputElement.value = `${value}`; // Update the input field with the validated value
    }
  }

  onBlur(event: Event): void {
    this.onInputChange(event);
  }

  onKeyDown(event: KeyboardEvent): void {
    if (event.key === 'Enter') {
      this.onInputChange(event);
    }
  }
}
