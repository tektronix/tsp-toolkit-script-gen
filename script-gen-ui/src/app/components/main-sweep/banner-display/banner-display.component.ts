import { CommonModule } from '@angular/common';
import { Component, Input } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { StatusMsg } from '../../../model/sweep_data/statusMsg';

@Component({
  selector: 'app-banner-display',
  imports: [FormsModule, CommonModule, BrowserModule, MatIconModule],
  standalone: true,
  templateUrl: './banner-display.component.html',
  styleUrl: './banner-display.component.scss',
})
export class BannerDisplayComponent {
  @Input() statusMsg: StatusMsg | undefined;
}
