import {
  Component,
  Input,
  Output,
  EventEmitter,
  forwardRef,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';
import { parseScientificInput, parseToDecimal } from '../input-parser.util'; // Import the parser
import { ParameterFloat } from '../../../model/sweep_data/TimingConfig';

@Component({
  selector: 'input-plain',
  templateUrl: './input-plain.component.html',
  styleUrls: ['./input-plain.component.scss'],
  standalone: false,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => InputPlainComponent),
      multi: true,
    },
  ],
})
export class InputPlainComponent implements ControlValueAccessor {
  @Input() label: string | undefined;
  @Input() unit: string | undefined;
  @Output() inputChange = new EventEmitter<number>();

  private _value: number | undefined;
  private _displayValue: string = '';
  private onChange: ((value: any) => void) | undefined;

  ngOnInit(): void {
    //
  }

  // TODO: have to make the rounding off based on the resolution defined for each range

  get displayValue(): string {
    const parsedValue = parseScientificInput(`${this._value} ${this.unit}`);
    // console.log('parsed value : ', parsedValue);

    // If parseScientificInput returns a valid value, update _displayValue
    if (parsedValue && parsedValue !== 'Invalid input') {
      this._displayValue = parsedValue;
    }
    // console.log('displayvalue : ', this._displayValue);
    // Return the cached _displayValue or fallback to default
    return this._displayValue || (this._value !== undefined ? `${this._value} ${this.unit}` : '');
  }

  set displayValue(val: string) {
    const decimalValue = parseToDecimal(val);
    // console.log('displayvalue',this.displayValue);
    // console.log('decimal value :',decimalValue);
    if (decimalValue !== null) {
      this._value = decimalValue;

      if (this.onChange) {
        this.onChange(this._value);
      }
      this.inputChange.emit(this._value);         // emitting the parsed value to rust
    } else {
      // this._value = this._value;
      // console.log("value: ", this._value);
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
  }

  onInputChange(event: Event): void {
    const inputElement = event.target as HTMLInputElement;

    if (this.displayValue !== inputElement.value) {
      const previousValue = this.displayValue;
      this.displayValue = inputElement.value;

      // If the input is invalid, revert to the previous value
      if (parseToDecimal(inputElement.value) === null) {
        // console.log('Reverting to previous valid value:', previousValue);
        inputElement.value = previousValue;
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
