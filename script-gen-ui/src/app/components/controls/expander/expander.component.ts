import { Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
  selector: 'app-expander',
  templateUrl: './expander.component.html',
  styleUrl: './expander.component.scss',

})
export class ExpanderComponent {
  @Input() title = '';
  @Input() buttonLabel = '';
  @Input() isExpanded = false;
  @Output() ExpToggle = new EventEmitter<void>();
  @Output() buttonClick = new EventEmitter<void>();

  onToggle() {
    this.ExpToggle.emit();
  }

  onButtonClick() {
    this.buttonClick.emit();
  }
}
