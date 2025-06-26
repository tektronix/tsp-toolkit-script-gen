import { Component, EventEmitter, Input, Output, OnChanges } from '@angular/core';
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
  imports: [FormsModule, CommonModule, BrowserModule, MatIconModule, InputPlainComponent, InputNumericComponent, DragDropModule],
  templateUrl: './list.component.html',
  styleUrl: './list.component.scss'
})
export class ListComponent implements OnChanges {
  @Input() noOfPointsOrSteps: number | undefined;
  @Input() listsWithNames: { chanName: string, list: ParameterFloat[] }[] = [];
  @Output() closeList = new EventEmitter<void>();
  @Output() listOfPoints = new EventEmitter<{ chanName: string, list: ParameterFloat[] }[]>();
  @Output() updatedStepsOrPoints = new EventEmitter<number>();

  rowIndices: number[] = [];

  getRowIndices(): number[] {
    return this.listsWithNames.length > 0 ? Array.from({length: this.listsWithNames[0].list.length}, (_, i) => i) : [];
  }

  ngOnChanges() {
    console.log('listsWithNames:', this.listsWithNames);
    if (this.listsWithNames.length > 0) {
      this.rowIndices = Array.from({length: this.listsWithNames[0].list.length}, (_, i) => i);
    } else {
      this.rowIndices = [];
    }
  }

  onClose() {
    this.closeList.emit();
  }

  onEnter(){
    this.listOfPoints.emit(this.listsWithNames);
  }

  emitStepsOrPoints() {
    // emitting points or steps
    this.updatedStepsOrPoints.emit(this.noOfPointsOrSteps);
  }
}
