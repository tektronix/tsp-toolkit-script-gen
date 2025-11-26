import {
  Component,
  OnInit,
  ViewChild,
  ElementRef,
  Input,
  OnChanges,
  SimpleChanges,
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
export class PlotContainerComponent implements OnInit, OnChanges {
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

  constructor(public elementRef: ElementRef) { }

  ngOnInit(): void {
    this.plotConfig = { staticPlot: true, responsive: true };
    this.calculateTimePerStep();
    this.plotdataXCalculation();
    // console.log("plodataX", this.plotDataX);

    if (this.stepGlobalParameters) {
      this.numberOfSteps = this.stepGlobalParameters.step_points;
    }
  }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['sweepGlobalParameters'] || changes['stepGlobalParameters']) {
      this.calculateTimePerStep();
      this.plotdataXCalculation();
    }
  }

  calculateTimePerStep(): void {
    if (this.sweepTimingConfig && this.stepGlobalParameters && this.globalParameters) {
      const numMeas = this.sweepTimingConfig.measure_count.value;
      const lineFreq = this.globalParameters.line_frequency;
      const overhead = this.globalParameters.overhead_time;
      const sourceDelay = this.sweepTimingConfig.smu_timing.source_delay.value;
      const measDelay = this.sweepTimingConfig.smu_timing.measure_delay.value;
      const stepToSweepDelay = this.stepGlobalParameters.step_to_sweep_delay.value;

      const mode = this.sweepTimingConfig.smu_timing.nplc_type.value;
      const modeString = mode as 'Aperture' | 'NPLC';   // Made same as JSON string
      let value: number;
      if (mode === 'NPLC') {
        value = this.sweepTimingConfig.smu_timing.nplc.value;
      } else {
        value = this.sweepTimingConfig.smu_timing.aperture.value;
      }

      const timingCalc = new TimingCalculation();
      this.totalTimePerStep = timingCalc.calculateTotalTime(modeString, numMeas, overhead, lineFreq, value, sourceDelay, measDelay, stepToSweepDelay);
    }
  }

  plotdataXCalculation(): void {
    if (this.totalTimePerStep) {
      const points = 11;
      const xData: number[] = [0];
      for (let i = 0; i < points; i++) {
        xData.push(i * this.totalTimePerStep);
      }
      this.plotDataX = xData;
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
