import {
  Component,
  OnInit,
  ViewChild,
  ElementRef,
  Input,
  Output,
  EventEmitter,
  OnChanges,
  SimpleChanges,
  AfterViewChecked,
  ViewChildren,
  QueryList,
} from '@angular/core';
import { BiasChannel } from '../../../model/chan_data/biasChannel';
import { StepChannel } from '../../../model/chan_data/stepChannel';
import { SweepChannel } from '../../../model/chan_data/sweepChannel';
import {
  StepGlobalParameters,
  SweepGlobalParameters,
} from '../../../model/sweep_data/stepSweepConfig';
import { PlotBiasComponent } from './plot-bias/plot-bias.component';
import { PlotStepComponent } from './plot-step/plot-step.component';
import { PlotSweepComponent } from './plot-sweep/plot-sweep.component';
import {
  ParameterInt,
  SweepTimingConfig,
} from '../../../model/sweep_data/SweepTimingConfig';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { TimingCalculation } from '../../utils/timing-calculation';
import { GlobalParameters } from '../../../model/sweep_data/sweepConfig';

@Component({
  selector: 'app-plot-container',
  standalone: true,
  imports: [
    FormsModule,
    BrowserModule,
    CommonModule,
    MatIconModule,
    PlotBiasComponent,
    PlotStepComponent,
    PlotSweepComponent,
  ],
  templateUrl: './plot-container.component.html',
  styleUrls: ['./plot-container.component.scss'],
})
export class PlotContainerComponent implements OnInit, OnChanges, AfterViewChecked {
  private needsScrollRestore = false;
  @ViewChild('plotContainer', { static: false }) plotContainer!: ElementRef;
  @Input() biasChannels: BiasChannel[] = [];
  @Input() stepChannels: StepChannel[] = [];
  @Input() sweepChannels: SweepChannel[] = [];
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;
  @Input() sweepGlobalParameters: SweepGlobalParameters | undefined;
  @Input() sweepTimingConfig: SweepTimingConfig | undefined;
  @Input() globalParameters: GlobalParameters | undefined;

  @Input() colorMap = new Map<string, string>(); // Accept colorMap from MainSweepComponent
  @Input() activeComponent: 'bias' | 'step' | 'sweep' | null = null; // Accept active component
  @Input() activeIndex: number | null = null; // Accept active index
  @Input() savedScrollPosition = 0; // Accept saved scroll position from parent

  @Output() scrollPositionChange = new EventEmitter<number>(); // Emit scroll position to parent

  @ViewChildren(PlotBiasComponent)
  plotBiasComponents!: QueryList<PlotBiasComponent>;
  @ViewChildren(PlotStepComponent)
  plotStepComponents!: QueryList<PlotStepComponent>;
  @ViewChildren(PlotSweepComponent)
  plotSweepComponents!: QueryList<PlotSweepComponent>;

  numberOfSteps: ParameterInt | undefined;
  plotDataX: number[] = [];
  plotConfig: { staticPlot: boolean; responsive: boolean } | undefined;

  totalTimePerStep: number | undefined;
  modeString: 'Aperture' | 'NPLC' = 'Aperture';
  modeValue = 0;
  ID = '';
  tickDifference = 0;

  constructor(public elementRef: ElementRef) { }

  ngOnInit(): void {
    this.plotConfig = { staticPlot: true, responsive: true };
    // this.calculateTimePerStep();
    this.calculateTime();
    // this.plotdataXCalculation();
    // console.log("plodataX", this.plotDataX);

    if (this.stepGlobalParameters) {
      this.numberOfSteps = this.stepGlobalParameters.step_points;
    }
  }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['sweepGlobalParameters'] || changes['stepGlobalParameters'] && changes['sweepTimingConfig']) {
      // this.calculateTimePerStep();
      this.calculateTime();
      // this.plotdataXCalculation();

      // Flag that scroll position needs to be restored
      this.needsScrollRestore = true;
    }
  }

  ngAfterViewChecked(): void {
    if (this.needsScrollRestore) {
      this.restoreScrollPosition();
      this.needsScrollRestore = false;
    }
  }

  onScroll(event: Event): void {
    const element = event.target as HTMLElement;
    this.scrollPositionChange.emit(element.scrollTop);
  }

  restoreScrollPosition(): void {
    const container = this.elementRef.nativeElement.parentElement;
    if (container && this.savedScrollPosition) {
      container.scrollTop = this.savedScrollPosition;
    }
  }

  calculateTime(): void {
    if (this.sweepChannels && this.sweepChannels.length > 0) {
      this.ID = this.sweepChannels[0].start_stop_channel.common_chan_attributes.device_id;
    }
    else if (this.stepChannels && this.stepChannels.length > 0) {
      this.ID = this.stepChannels[0].start_stop_channel.common_chan_attributes.device_id;
    }
    else if (this.biasChannels && this.biasChannels.length > 0) {
      this.ID = this.biasChannels[0].common_chan_attributes.device_id;
    }
    else {
      this.plotDataX = [0, 1e-6, 2e-6, 3e-6, 4e-6, 5e-6, 6e-6, 7e-6, 8e-6, 9e-6, 10e-6];
    }
    const result = this.checkDeviceType(this.ID);
    this.modeString = result.modeString;
    this.modeValue = result.modeValue;
    this.calculateTimePerStep();
  }

  checkDeviceType(ID: string): { modeString: 'Aperture' | 'NPLC'; modeValue: number } {
    let mode = this.sweepTimingConfig?.smu_timing.nplc_type.value;
    this.modeString = mode as 'Aperture' | 'NPLC';
    this.modeValue = 0;
    const rate = this.sweepTimingConfig?.psu_timing.rate.value;
    if (this.sweepTimingConfig) {
      if (ID.includes('smu')) {
        mode = this.sweepTimingConfig.smu_timing.nplc_type.value;
        if (mode === 'NPLC') {
          this.modeString = 'NPLC';
          this.modeValue = this.sweepTimingConfig.smu_timing.nplc.value;
        } else if (mode === 'Aperture') {
          this.modeString = 'Aperture';
          this.modeValue = this.sweepTimingConfig.smu_timing.aperture.value;
        }
      } else if (ID.includes('psu')) {
        mode = 'Aperture';
        this.modeString = 'Aperture';
        if (rate === 'Fast') {
          this.modeValue = this.sweepTimingConfig.psu_timing.rate_fast;
        } else if (rate === 'Normal') {
          this.modeValue = this.sweepTimingConfig.psu_timing.rate_normal;
        }
      }
    }
    return { modeString: this.modeString, modeValue: this.modeValue };
  }

  calculateTimePerStep(): void {
    if (this.sweepTimingConfig && this.stepGlobalParameters && this.globalParameters && this.sweepGlobalParameters) {
      const numMeas = this.sweepTimingConfig.measure_count.value;
      const lineFreq = this.globalParameters.line_frequency;
      const overhead = this.globalParameters.overhead_time;
      const sourceDelay = this.sweepTimingConfig.smu_timing.source_delay.value;
      const measDelay = this.sweepTimingConfig.smu_timing.measure_delay.value;
      const stepToSweepDelay = this.stepGlobalParameters.step_to_sweep_delay.value;
      const sweepPoints = this.sweepGlobalParameters.sweep_points.value;

      const timingCalc = new TimingCalculation();
      this.totalTimePerStep = timingCalc.calculateTotalTime(this.modeString, numMeas, overhead, lineFreq, this.modeValue, sourceDelay, measDelay, stepToSweepDelay, sweepPoints);
      this.plotdataXCalculation();
    }
  }

  plotdataXCalculation(): void {
    if (this.totalTimePerStep && this.stepGlobalParameters) {
      const points = this.stepGlobalParameters.step_points.value;
      const xData: number[] = [];
      for (let i = 0; i < points + 1; i++) {
        xData.push(i * this.totalTimePerStep);
      }
      this.plotDataX = xData;
      this.tickDifference = xData[points] / 10;
    }
  }

  getColor(uuid: string): string {
    return this.colorMap.get(uuid) || 'gray'; // Retrieve color from colorMap
  }

  getActiveStyle(
    uuid: string,
    componentType: 'bias' | 'step' | 'sweep',
    index: number
  ): { backgroundColor: string; color: string } {
    const isActive =
      this.activeComponent === componentType && this.activeIndex === index;
    const backgroundColor =
      this.colorMap.get(uuid) || 'var(--vscode-activityBar-foreground)';

    return {
      backgroundColor: isActive
        ? backgroundColor
        : 'var(--vscode-activityBar-border)',
      color: isActive ? 'black' : 'var(--vscode-badge-foreground)',
    };
  }

  scrollToPlot(componentType: 'bias' | 'step' | 'sweep', index: number): void {
    let plotComponent:
      | PlotBiasComponent
      | PlotStepComponent
      | PlotSweepComponent
      | undefined;

    if (componentType === 'bias') {
      plotComponent = this.plotBiasComponents.toArray()[index];
    } else if (componentType === 'step') {
      plotComponent = this.plotStepComponents.toArray()[index];
    } else if (componentType === 'sweep') {
      plotComponent = this.plotSweepComponents.toArray()[index];
    }

    if (plotComponent) {
      const element = plotComponent.elementRef.nativeElement; // Access the DOM element
      element.scrollIntoView({
        behavior: 'smooth', // Smooth scrolling
        block: 'center', // Align to the center of the viewport
      });
    }
  }

}
