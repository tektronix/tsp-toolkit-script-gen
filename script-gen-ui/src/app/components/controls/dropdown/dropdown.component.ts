import { CommonModule } from '@angular/common';
import {
  Component,
  Input,
  Output,
  EventEmitter,
  forwardRef,
} from '@angular/core';
import {
  FormsModule,
  ControlValueAccessor,
  NG_VALUE_ACCESSOR,
} from '@angular/forms';

@Component({
  selector: 'dropdown',
  templateUrl: './dropdown.component.html',
  styleUrls: ['./dropdown.component.scss'],
  standalone: false,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => DropdownComponent),
      multi: true,
    },
  ],
})
export class DropdownComponent implements ControlValueAccessor {
  @Input() label: string | undefined;
  @Input() selected: string | undefined;
  @Input() options: string[] = [];
  // @Input() unit: string | undefined;
  @Output() selectedChange = new EventEmitter<string>();

  private onChange: ((value: any) => void) | undefined;

  ngOnInit(): void {
    // Ensure selected is initialized
    if (!this.selected && this.options.length > 0) {
      this.selected = this.options[0];
    }
  }

  writeValue(value: any): void {
    this.selected = value;
  }

  registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: any): void {}

  setDisabledState?(isDisabled: boolean): void {
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
