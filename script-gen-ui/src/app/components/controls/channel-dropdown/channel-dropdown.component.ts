import { CommonModule } from '@angular/common';
import {
  Component,
  ElementRef,
  HostListener,
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
  @Input() automationID: string | undefined;
  // @Input() unit: string | undefined;
  @Output() selectedChange = new EventEmitter<Option>();

  isOpen = false;
  activeIndex = 0;

  private onChange: ((value: Option) => void) | undefined;

  constructor(private readonly host: ElementRef<HTMLElement>) {}

  ngOnInit(): void {
    // Ensure selected is initialized
    if (!this.selected && this.options.length > 0) {
      this.selected = this.options[0];
    }
  }

  trackOptionByValue(_index: number, option: Option): string {
    return option.value;
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

  // Non-breaking spaces keep the wider id/model gap in the native tooltip.
  get tooltipText(): string | null {
    if (!this.selected) {
      return null;
    }
    return `${this.optionId(this.selected)}\u00A0\u00A0${this.optionDetail(this.selected)}`;
  }

  optionId(option: Option): string {
    const split = option.label.indexOf(' ');
    return split < 0 ? option.label : option.label.slice(0, split);
  }

  optionDetail(option: Option): string {
    const split = option.label.indexOf(' ');
    return split < 0 ? '' : option.label.slice(split + 1);
  }

  toggle(): void {
    if (this.isOpen) {
      this.close();
    } else {
      this.open();
    }
  }

  @HostListener('document:click', ['$event'])
  onDocumentClick(event: Event): void {
    const wrapper = this.host.nativeElement.querySelector('.select-wrapper');
    if (wrapper?.contains(event.target as Node)) {
      return;
    }
    this.close();
    const active = document.activeElement;
    if (active instanceof HTMLElement && this.host.nativeElement.contains(active)) {
      active.blur();
    }
  }

  onKeydown(event: KeyboardEvent): void {
    const last = this.options.length - 1;
    switch (event.key) {
      case 'ArrowDown':
      case 'ArrowUp': {
        event.preventDefault();
        if (!this.isOpen) {
          this.open();
          break;
        }
        const step = event.key === 'ArrowDown' ? 1 : -1;
        this.setActive(Math.min(last, Math.max(0, this.activeIndex + step)));
        break;
      }
      case 'Home':
      case 'End':
        if (this.isOpen) {
          event.preventDefault();
          this.setActive(event.key === 'Home' ? 0 : last);
        }
        break;
      case 'Enter':
      case ' ':
        event.preventDefault();
        if (this.isOpen) {
          const option = this.options[this.activeIndex];
          if (option) {
            this.selectOption(option);
          } else {
            this.close();
          }
        } else {
          this.open();
        }
        break;
      case 'Escape':
        if (this.isOpen) {
          event.preventDefault();
          this.close();
        }
        break;
      case 'Tab':
        this.close();
        break;
    }
  }

  selectOption(option: Option): void {
    this.close();
    if (option.value === this.selected?.value) {
      return;
    }
    this.selected = option;
    if (this.onChange) {
      this.onChange(option);
    }
    this.selectedChange.emit(option);
  }

  private open(): void {
    this.activeIndex = Math.max(
      0,
      this.options.findIndex((opt) => opt.value === this.selected?.value),
    );
    this.isOpen = true;
    this.scrollActiveIntoView();
  }

  private close(): void {
    this.isOpen = false;
  }

  private setActive(index: number): void {
    this.activeIndex = index;
    this.scrollActiveIntoView();
  }

  private scrollActiveIntoView(): void {
    setTimeout(() => {
      const item = this.host.nativeElement.querySelectorAll<HTMLElement>('.option')[this.activeIndex];
      item?.scrollIntoView?.({ block: 'nearest' });
    });
  }
}
