import {
  Component,
  EventEmitter,
  Input,
  Output,
  OnChanges,
  AfterViewInit,
  ViewChild,
  ElementRef,
} from '@angular/core';
import { DragDropModule } from '@angular/cdk/drag-drop';
import { ParameterFloat } from '../../../model/sweep_data/SweepTimingConfig';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { BrowserModule } from '@angular/platform-browser';
import { MatIconModule } from '@angular/material/icon';
import { InputPlainComponent } from '../../controls/input-plain/input-plain.component';
import { InputNumericComponent } from '../../controls/input-numeric/input-numeric.component';

@Component({
  selector: 'app-list',
  imports: [
    FormsModule,
    CommonModule,
    BrowserModule,
    MatIconModule,
    InputPlainComponent,
    InputNumericComponent,
    DragDropModule,
  ],
  templateUrl: './list.component.html',
  styleUrl: './list.component.scss',
})
export class ListComponent implements OnChanges, AfterViewInit {
  @ViewChild('listPopUp', { static: false }) listPopUp: ElementRef | undefined;
  @Input() noOfPointsOrSteps: number | undefined;
  @Input() isNoOfPointsOrSteps = true;
  @Input() listsWithNames: {
    chanName: string;
    list: ParameterFloat[];
    isDeviceValid: boolean;
  }[] = [];
  @Input() savedListPosition: { left: number; top: number } | null = null;
  @Output() closeList = new EventEmitter<void>();
  @Output() listOfPoints = new EventEmitter<
    { chanName: string; list: ParameterFloat[] }[]
  >();
  @Output() listPositionChange = new EventEmitter<{ left: number; top: number }>();
  @Output() updatedStepsOrPoints = new EventEmitter<number>();

  rowIndices: number[] = [];
  isDragDisabled = false;

  getRowIndices(): number[] {
    return this.listsWithNames.length > 0
      ? Array.from({ length: this.listsWithNames[0].list.length }, (_, i) => i)
      : [];
  }

  ngAfterViewInit(): void {
    if (this.listPopUp && this.savedListPosition) {
      this.listPopUp.nativeElement.style.left = `${this.savedListPosition.left}px`;
      this.listPopUp.nativeElement.style.top = `${this.savedListPosition.top}px`;
    }
  }

  ngOnChanges() {
    console.log('listsWithNames:', this.listsWithNames);
    if (this.listsWithNames.length > 0) {
      this.rowIndices = Array.from(
        { length: this.listsWithNames[0].list.length },
        (_, i) => i
      );
    } else {
      this.rowIndices = [];
    }
  }

  onClose() {
    this.captureListPosition();
    this.closeList.emit();
  }

  onChange() {
    this.captureListPosition();
    this.listOfPoints.emit(this.listsWithNames);
  }

  emitStepsOrPoints() {
    // emitting points or steps
    this.captureListPosition();
    this.updatedStepsOrPoints.emit(this.noOfPointsOrSteps);
  }

  captureListPosition() {
    if (this.listPopUp) {
      const rect = this.listPopUp.nativeElement.getBoundingClientRect();
      this.listPositionChange.emit({ left: rect.left, top: rect.top });
    }
  }

  onInputBlur(event: FocusEvent) {
    this.isDragDisabled = false;
  }

  onInputMouseDown(event: MouseEvent) {
    this.isDragDisabled = true;
    
    setTimeout(() => {
      if (this.isDragDisabled) {
        this.isDragDisabled = false;
      }
    }, 500);
  }

  onInputMouseUp(event: MouseEvent) {
    this.isDragDisabled = false;
  }
}
