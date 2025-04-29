import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';

@Component({
  selector: 'app-empty-config',
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  templateUrl: './empty-config.component.html',
  styleUrl: './empty-config.component.scss',
})
export class EmptyConfigComponent {}
