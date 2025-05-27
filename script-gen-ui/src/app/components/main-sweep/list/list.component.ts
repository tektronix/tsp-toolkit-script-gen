import { Component, EventEmitter, Input, Output } from '@angular/core';
import { ParameterInt } from '../../../model/sweep_data/SweepTimingConfig';
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
  imports: [FormsModule, CommonModule, BrowserModule, MatIconModule, InputPlainComponent, InputNumericComponent],
  templateUrl: './list.component.html',
  styleUrl: './list.component.scss'
})
export class ListComponent {
  @Input() sweepPoints: ParameterInt | undefined;
  @Input() sweepChannels: SweepChannel[] | undefined;
  @Input() stepChannels: StepChannel[] | undefined;
  @Output() closeList = new EventEmitter<void>();

  get pointsArray(): number[] {
    return Array(this.sweepPoints?.value || 0);
  }

  onClose() {
    this.closeList.emit();
  }

}
