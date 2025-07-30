import { CommonModule } from '@angular/common';
import {
  Component,
  EventEmitter,
  forwardRef,
  Input,
  Output,
  OnInit,
} from '@angular/core';
import {
  ControlValueAccessor,
  FormsModule,
  NG_VALUE_ACCESSOR,
} from '@angular/forms';
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
  @Input() disabled = false;
  @Input() automationID: string | undefined;
  @Input() floatAllowed = false;
  @Output() inputChange = new EventEmitter<number>();

  private _value: number | undefined;
  private onChange: ((value: number) => void) | undefined;

  ngOnInit(): void {
    console.log('InputNumericComponent initialized with label:', this.label);
  }

  get displayValue(): number | undefined {
    return this._value;
  }

  set displayValue(val: number) {

    this._value = val;
    if (this.onChange) {
      this.onChange(this._value);
    }
    this.inputChange.emit(this._value);
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

    if (this.displayValue !== inputElement.valueAsNumber) {
      // Store previous value for fallback
      const previousValue = this.displayValue;

      let value = inputElement.valueAsNumber;

      // Only update if the value is valid (not NaN)
      if (!isNaN(value)) {
        if (!this.floatAllowed) {
          value = Math.floor(value);
          inputElement.value = `${value}`;
        }
        this.displayValue = value;
      } else {
        // Revert to the previous valid value
        inputElement.value = `${previousValue}`;
      }

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
