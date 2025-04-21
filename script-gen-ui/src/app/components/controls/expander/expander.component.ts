import { Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
  selector: 'expander',
  templateUrl: './expander.component.html',
  styleUrl: './expander.component.scss',
  standalone: false,
})
export class ExpanderComponent {
  @Input() title: string = '';
  @Input() buttonLabel: string = '';
  @Input() isExpanded: boolean = false;
  @Output() toggle = new EventEmitter<void>();
  @Output() buttonClick = new EventEmitter<void>();

  onToggle() {
    this.toggle.emit();
  }

  onButtonClick() {
    this.buttonClick.emit();
  }
}
