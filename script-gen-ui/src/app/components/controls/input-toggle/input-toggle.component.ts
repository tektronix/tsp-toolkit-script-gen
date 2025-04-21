import {
  Component,
  Input,
  Output,
  EventEmitter,
  forwardRef,
} from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';

@Component({
  selector: 'app-input-toggle',
  templateUrl: './input-toggle.component.html',
  styleUrls: ['./input-toggle.component.scss'],
  standalone: false,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => InputToggleComponent),
      multi: true,
    },
  ],
})
export class InputToggleComponent implements ControlValueAccessor {
  @Input() label: string | undefined;
  @Input() options: string[] = [];
  @Input() selectedOption: string | undefined;
  @Output() toggleOptionChange = new EventEmitter<string>();

  private onChange: (value: any) => void = () => {};
  private onTouched: () => void = () => {};

  writeValue(value: any): void {
    this.selectedOption = value;
  }

  registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: any): void {
    this.onTouched = fn;
  }

  setDisabledState?(isDisabled: boolean): void {
    // Implement if needed
  }

  toggleOption(option: string): void {
    if (this.selectedOption !== option) {
      this.selectedOption = option;
      this.onChange(this.selectedOption);
      this.toggleOptionChange.emit(this.selectedOption);
    }
    this.onTouched();
  }
}
