import { CommonModule } from '@angular/common';
import { Component, Input, OnInit, OnDestroy, Inject, ChangeDetectorRef } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { StatusMsg } from '../../../model/sweep_data/statusMsg';
import { StatusService } from '../../../services/status.service';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-banner-display',
  imports: [FormsModule, CommonModule, BrowserModule, MatIconModule],
  standalone: true,
  templateUrl: './banner-display.component.html',
  styleUrl: './banner-display.component.scss',
})
export class BannerDisplayComponent implements OnInit, OnDestroy {
  private _external?: StatusMsg;
  private _service?: StatusMsg;
  private _displayed?: StatusMsg;
  private _sub?: Subscription;

  @Input()
  set statusMsg(value: StatusMsg | undefined) {
    this._external = value;
    this.updateDisplayed();
  }
  get statusMsg(): StatusMsg | undefined {
    return this._displayed;
  }

  constructor(
    @Inject(StatusService) private statusService: StatusService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this._sub = this.statusService.status$.subscribe((msg: StatusMsg | undefined) => {
      this._service = msg;
      this.updateDisplayed();
      this.cdr.detectChanges(); // Trigger change detection
    });
  }

  ngOnDestroy(): void {
    this._sub?.unsubscribe();
  }

  clearServiceMessage(): void {
    this.statusService.clear();
  }

  private updateDisplayed(): void {
    this._displayed = this._external ?? this._service;
  }
}
