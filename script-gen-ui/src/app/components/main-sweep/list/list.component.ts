import { Component, EventEmitter, Input, Output, OnChanges } from '@angular/core';
import { DragDropModule } from '@angular/cdk/drag-drop';
import { ParameterFloat, ParameterInt } from '../../../model/sweep_data/SweepTimingConfig';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { BrowserModule } from '@angular/platform-browser';
import { MatIconModule } from '@angular/material/icon';
import { InputPlainComponent } from '../../controls/input-plain/input-plain.component';
import { SweepChannel } from '../../../model/chan_data/sweepChannel';
import { InputNumericComponent } from '../../controls/input-numeric/input-numeric.component';
import { StepChannel } from '../../../model/chan_data/stepChannel';

@Component({
  selector: 'app-list',
  imports: [FormsModule, CommonModule, BrowserModule, MatIconModule, InputPlainComponent, InputNumericComponent, DragDropModule],
  templateUrl: './list.component.html',
  styleUrl: './list.component.scss'
})
export class ListComponent implements OnChanges {
  @Input() sweepPoints: ParameterInt | undefined;
  @Input() sweepChannels: SweepChannel[] | undefined;
  @Input() stepChannels: StepChannel[] | undefined;
  @Output() closeList = new EventEmitter<void>();
  @Output() listOfPoints = new EventEmitter<ParameterFloat[][]>();

  values: ParameterFloat[][] = [];

  get pointsArray(): number[] {
    return Array(this.sweepPoints?.value || 0);
  }

  get channelCount(): number {
    return (this.sweepChannels?.length || this.stepChannels?.length || 0);
  }

  ngOnChanges() {
    const rows = this.sweepPoints?.value || 0;
    const cols = this.sweepChannels?.length ?? this.stepChannels?.length ?? 0;
    if (rows !== this.values.length || (this.values[0]?.length || 0) !== cols) {
      this.values = Array.from({ length: rows }, () =>
        Array.from({ length: cols }, () => ({ value: 0 } as ParameterFloat))
      );
    }
    console.log("values list", this.values)
  }

  onClose() {
    this.closeList.emit();
  }

  onEnter(){
    this.listOfPoints.emit(this.values);
  }
}
