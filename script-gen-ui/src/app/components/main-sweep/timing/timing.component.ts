import {
  Component,
  EventEmitter,
  Input,
  OnDestroy,
  Output,
  SimpleChanges,
  OnChanges
} from '@angular/core';
import {
  ParameterFloat,
  ParameterString,
  ParameterInt,
  SweepTimingConfig,
  SmuTiming,
  PsuTiming,
} from '../../../model/sweep_data/SweepTimingConfig';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { DropdownComponent } from '../../controls/dropdown/dropdown.component';
import { InputNumericComponent } from '../../controls/input-numeric/input-numeric.component';
import { InputPlainComponent } from '../../controls/input-plain/input-plain.component';
import { InputToggleComponent } from '../../controls/input-toggle/input-toggle.component';
import { CdkDrag, CdkDragHandle } from '@angular/cdk/drag-drop';

@Component({
  selector: 'app-timing',
  standalone: true,
  imports: [
    FormsModule,
    BrowserModule,
    CommonModule,
    MatIconModule,
    DropdownComponent,
    InputNumericComponent,
    InputPlainComponent,
    InputToggleComponent,
    CdkDrag,
    CdkDragHandle
],
  templateUrl: './timing.component.html',
  styleUrl: './timing.component.scss',
})
export class TimingComponent implements OnDestroy, OnChanges {

  nplc: ParameterFloat | undefined;
  aperture: ParameterFloat | undefined;
  sourceAutoDelay: ParameterString | undefined;
  sourceDelay: ParameterFloat | undefined;
  measureAutoDelay: ParameterString | undefined;
  measureDelay: ParameterFloat | undefined;
  nplcType: ParameterString | undefined;

  rate: ParameterString | undefined;

  measureCount: ParameterInt | undefined;

  timingOption: 'smuTiming' | 'psuTiming' = 'smuTiming';

  boundaryElement = 'app-root';

  @Input() sweepTimingConfig: SweepTimingConfig | undefined;
  @Output() ok = new EventEmitter<void>();
  @Output() emitTimingData = new EventEmitter<void>();

  // constructor() {}

  // ngOnInit(): void {
  //   this.updateAll();
  // }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['sweepTimingConfig']) {
      // Handle the change in timingConfig here if needed
      this.updateAll();
      console.log('sweepTimingConfig updated:', this.sweepTimingConfig);
    }
  }

  updateAll() {
    if (this.sweepTimingConfig) {
      console.log('component initiated');
      this.nplc = this.sweepTimingConfig.smu_timing.nplc;
      this.aperture = this.sweepTimingConfig.smu_timing.aperture;
      this.sourceAutoDelay =
        this.sweepTimingConfig.smu_timing.source_auto_delay;
      this.sourceDelay = this.sweepTimingConfig.smu_timing.source_delay;
      this.measureAutoDelay =
        this.sweepTimingConfig.smu_timing.measure_auto_delay;
      this.measureDelay = this.sweepTimingConfig.smu_timing.measure_delay;
      this.nplcType = this.sweepTimingConfig.smu_timing.nplc_type;

      this.rate = this.sweepTimingConfig.psu_timing.rate;

      this.measureCount = this.sweepTimingConfig.measure_count;
    }
  }

  okTiming() {
    //close the timing dialog
    this.ok.emit();
  }

  submitTimingData() {
    this.emitTimingData.emit();
  }

  getSweepTimingConfigFromComponent(): SweepTimingConfig {
    const smu_timing = new SmuTiming({
      nplc: this.nplc!,
      aperture: this.aperture!,
      source_auto_delay: this.sourceAutoDelay!,
      source_delay: this.sourceDelay!,
      measure_auto_delay: this.measureAutoDelay!,
      measure_delay: this.measureDelay!,
      nplc_type: this.nplcType!,
    });
    const psu_timing = new PsuTiming({
      rate: this.rate!,
      aperture_value: this.sweepTimingConfig?.psu_timing.aperture_value || [],
    });
    return new SweepTimingConfig({
      measure_count: this.measureCount!,
      smu_timing: smu_timing,
      psu_timing: psu_timing,
    });
  }

  ngOnDestroy(): void {
    console.log('component destroyed');
  }
}
