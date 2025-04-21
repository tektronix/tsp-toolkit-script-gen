import {
  Component,
  EventEmitter,
  forwardRef,
  input,
  Input,
  Output,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

@Component({
  selector: 'input-numeric',
  templateUrl: './input-numeric.component.html',
  styleUrl: './input-numeric.component.scss',
  standalone: false,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => InputNumericComponent),
      multi: true,
    },
  ],
})
export class InputNumericComponent implements ControlValueAccessor {
  @Input() label: string | undefined;

  @Output() inputChange = new EventEmitter<number>();

  private _value: number | undefined;
  private _displayValue: string = '';
  private onChange: ((value: any) => void) | undefined;

  ngOnInit(): void {
    //
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

  writeValue(value: any): void {
    if (value !== undefined) {
      this._value = value;
    }
  }

  registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: any): void {}

  setDisabledState?(isDisabled: boolean): void {
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
