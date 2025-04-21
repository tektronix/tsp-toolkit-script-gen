import {
  Component,
  EventEmitter,
  input,
  Input,
  OnDestroy,
  OnInit,
  Output,
  SimpleChanges,
} from '@angular/core';
import {
  ParameterFloat,
  ParameterString,
  ParameterInt,
  TimingConfig,
} from '../../../model/sweep_data/TimingConfig';

@Component({
  selector: 'app-timing',
  standalone: false,
  templateUrl: './timing.component.html',
  styleUrl: './timing.component.scss',
})
export class TimingComponent implements OnDestroy {
  selectedWindow: 'window1' | 'window2' = 'window1';

  nplc: ParameterFloat | undefined;
  autoZero: ParameterString | undefined;
  sourceDelayType: ParameterString | undefined;
  sourceDelay: ParameterFloat | undefined;
  measureCount: ParameterInt | undefined;
  measureDelayType: ParameterString | undefined;
  measureDelay: ParameterFloat | undefined;
  measureDelayFactor: ParameterFloat | undefined;
  measureFilterEnable: ParameterString | undefined;
  measureFilterType: ParameterString | undefined;
  measureFilterCount: ParameterInt | undefined;
  measureAnalogFilter: ParameterString | undefined;
  msg1: number = 0.0;
  msg2: number | undefined;

  timingOption: 'sameTiming' | 'highSpeedSampling' = 'sameTiming';

  samplingInterval: ParameterFloat | undefined;
  samplingCount: ParameterInt | undefined;
  samplingDelayType: ParameterString | undefined;
  samplingDelay: ParameterFloat | undefined;
  samplingAnalogFilter: ParameterString | undefined;

  @Input() timingConfig: TimingConfig | undefined;
  @Output() ok = new EventEmitter<void>();
  @Output() emitTimingData = new EventEmitter<void>();

  constructor() {}

  // ngOnInit(): void {
  //   this.updateAll();
  // }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['timingConfig']) {
      // Handle the change in timingConfig here if needed
      this.updateAll();
      console.log('timingConfig updated:', this.timingConfig);
    }
  }

  updateAll() {
    if (this.timingConfig) {
      console.log('component initiated');
      this.nplc = this.timingConfig.nplc;
      this.autoZero = this.timingConfig.auto_zero;
      this.sourceDelayType = this.timingConfig.source_delay_type;
      this.sourceDelay = this.timingConfig.source_delay;
      this.measureCount = this.timingConfig.measure_count;
      this.measureDelayType = this.timingConfig.measure_delay_type;
      this.measureDelay = this.timingConfig.measure_delay;
      this.measureDelayFactor = this.timingConfig.measure_delay_factor;
      this.measureFilterEnable = this.timingConfig.measure_filter_enable;
      this.measureFilterType = this.timingConfig.measure_filter_type;
      this.measureFilterCount = this.timingConfig.measure_filter_count;
      this.measureAnalogFilter = this.timingConfig.measure_analog_filter;

      this.timingOption = this.timingConfig.high_speed_sampling
        ? 'highSpeedSampling'
        : 'sameTiming';

      this.samplingInterval = this.timingConfig.sampling_interval;
      this.samplingCount = this.timingConfig.sampling_count;
      this.samplingDelayType = this.timingConfig.sampling_delay_type;
      this.samplingDelay = this.timingConfig.sampling_delay;
      this.samplingAnalogFilter = this.timingConfig.sampling_analog_filter;
    }
  }

  okTiming() {
    //close the timing dialog
    this.ok.emit();
  }

  selectWindow(window: 'window1' | 'window2') {
    this.selectedWindow = window;
  }

  handleNplcChange(event: Event) {
    //
  }

  handleSourceDelayChange($event: number) {
    this.msg1 = $event;
    this.msg2 = this.sourceDelay?.value;
    this.submitTimingData();
  }

  submitTimingData() {
    this.emitTimingData.emit();
  }

  getTimingConfigFromComponent(): TimingConfig {
    return new TimingConfig({
      nplc: this.nplc,
      auto_zero: this.autoZero,
      source_delay_type: this.sourceDelayType,
      source_delay: this.sourceDelay,
      measure_count: this.measureCount,
      measure_delay_type: this.measureDelayType,
      measure_delay: this.measureDelay,
      measure_delay_factor: this.measureDelayFactor,
      measure_filter_enable: this.measureFilterEnable,
      measure_filter_type: this.measureFilterType,
      measure_filter_count: this.measureFilterCount,
      measure_analog_filter: this.measureAnalogFilter,
      high_speed_sampling: this.timingOption === 'highSpeedSampling',
      sampling_interval: this.samplingInterval,
      sampling_count: this.samplingCount,
      sampling_delay_type: this.samplingDelayType,
      sampling_delay: this.samplingDelay,
      sampling_analog_filter: this.samplingAnalogFilter,
    });
  }

  ngOnDestroy(): void {
    console.log('component destroyed');
  }
}
