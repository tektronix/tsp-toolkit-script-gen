import { CommonModule } from '@angular/common';
import { Component, ElementRef, HostListener, Input, Renderer2 } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';

@Component({
  selector: 'app-tooltip',
  imports: [FormsModule,
      BrowserModule,
      CommonModule,
      MatIconModule],
  templateUrl: './tooltip.component.html',
  styleUrl: './tooltip.component.scss'
})
export class TooltipComponent {
  @Input() text = '';
  @Input() position: 'top' | 'right' | 'bottom' | 'left' = 'top';

  @Input()visible = false;
  top = 0;
  left = 0;

  constructor(private el: ElementRef, private renderer: Renderer2) {}

  @HostListener('mouseenter')
  onMouseEnter() {
    this.visible = true;
    setTimeout(() => this.setPosition(), 0);
  }

  @HostListener('mouseleave')
  onMouseLeave() {
    this.visible = false;
  }

  private setPosition() {
    const host = this.el.nativeElement;
    const tooltip = host.querySelector('.tooltip-content');
    if (!tooltip) return;

    const hostRect = host.getBoundingClientRect();
    const tooltipRect = tooltip.getBoundingClientRect();

    switch (this.position) {
      case 'top':
        this.top = -tooltipRect.height - 8;
        this.left = (hostRect.width - tooltipRect.width) / 2;
        break;
      case 'bottom':
        this.top = hostRect.height + 8;
        this.left = (hostRect.width - tooltipRect.width) / 2;
        break;
      case 'left':
        this.top = (hostRect.height - tooltipRect.height) / 2;
        this.left = -tooltipRect.width - 8;
        break;
      case 'right':
        this.top = (hostRect.height - tooltipRect.height) / 2;
        this.left = hostRect.width + 8;
        break;
    }
  }
}
