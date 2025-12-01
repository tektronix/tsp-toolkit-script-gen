import {
  AfterViewInit,
  Component,
  ElementRef,
  HostListener,
  Input,
  OnDestroy,
  OnInit,
  SimpleChanges,
  OnChanges,
} from '@angular/core';
import { StepChannel } from '../../../../model/chan_data/stepChannel';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import {
  ParameterFloat,
  ParameterInt,
  ParameterString,
} from '../../../../model/sweep_data/SweepTimingConfig';
import { StepGlobalParameters } from '../../../../model/sweep_data/stepSweepConfig';
import { CommonChanAttributes } from '../../../../model/chan_data/defaultChannel';
import * as Plotly from 'plotly.js-dist';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { PlotUtils } from '../plot-utils';

@Component({
  selector: 'app-plot-step',
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  templateUrl: './plot-step.component.html',
  styleUrl: './plot-step.component.scss',
})
export class PlotStepComponent
  implements AfterViewInit, OnInit, OnDestroy, OnChanges {
  @Input() stepChannel: StepChannel | undefined;
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;

  @Input() plotDataX: number[] = [];
  @Input() tickDifference: number | undefined;
  @Input() plotConfig: { staticPlot: boolean } | undefined;
  @Input() stepPointsList: ParameterFloat[][] = [];

  private _isActive = false;

  @Input()
  set isActive(value: boolean) {
    this._isActive = value;
    this.renderPlot(); // Re-render the plot when isActive changes
  }

  get isActive(): boolean {
    return this._isActive;
  }
  @Input() activeStyle: { backgroundColor: string; color: string } = {
    backgroundColor: '',
    color: '',
  };
  @Input() color = '';
  private mutationObserver: MutationObserver | undefined;
  private originalBackgroundColor = '';
  activeBackgroundColor = '';
  private tickColor = '';
  windowHeight = window.innerHeight;
  windowWidth = window.innerWidth;
  plotWidth = this.windowWidth * 0.58;

  commonChanAttributes: CommonChanAttributes | undefined;
  chanName = 'step1';
  deviceID = '';

  sourceRange: ChannelRange | undefined;
  sourceFunction: ParameterString | undefined;
  measRange: ChannelRange | undefined;
  measFunction: ParameterString | undefined;

  stepPoints: ParameterInt | undefined;
  stepToSweepDelay: ParameterFloat | undefined;
  start: ParameterFloat | undefined;
  stop: ParameterFloat | undefined;
  style: ParameterString | undefined;
  list = true;
  listStep: ParameterFloat[] = [];

  plotLayout = {
    xaxis: {
      visible: true,
      ticksuffix: ' s',
      rangemode: 'nonnegative',
      separatethousands: false,
      tickfont: {
        family: 'Roboto, "Helvetica Neue", sans-serif',
        color: this.tickColor,
        size: 9,
      },
      dtick: 1,
      range: [0, 10],
      // tick0: 0,
      showtickprefix: 'none',
      showticksuffix: 'all',
      tickwidth: 0,
      showline: true,
      layer: 'below traces',
      zeroline: false,
      zerolinecolor: 'gray',
      zerolinewidth: 1,
      showgrid: true,
      gridcolor: 'lightgrey',
      gridwidth: 0.5,
      griddash: 'dot',
      type: 'linear',
      position: 20,
      linewidth: 1,
    },
    xaxis2: {
      visible: true,
      rangemode: 'nonnegative',
      dtick: 1,
      // tick0: 0,
      showticklabels: false,
      showline: true,
      layer: 'below traces',
      zeroline: false,
      zerolinecolor: 'gray',
      zerolinewidth: 1,
      showgrid: false,
      type: 'linear',
      position: 1,
      overlaying: 'x',
      side: 'top',
      tickprefix: 'm',
      linewidth: 1,
    },
    yaxis: {
      visible: true,
      range: [0, 1],
      tickfont: {
        family: 'Roboto, "Helvetica Neue", sans-serif',
        color: this.tickColor,
        size: 9,
      },
      dtick: 0.25,
      tick0: 0,
      tickwidth: 0,
      linewidth: 1,
      layer: 'below traces',
      showline: false,
      showticklabels: false,
      showgrid: true,
      gridcolor: 'lightgrey',
      gridwidth: 0.3,
      type: 'linear',
      griddash: 'dot',
      zeroline: false,
    },
    yaxis2: {
      tickfont: { family: 'Times New Roman', color: this.tickColor, size: 9 },
      anchor: 'x',
      overlaying: 'y',
      side: 'left',
      position: -3,
      showticklabels: true,
      visible: true,
      ticksuffix: ' V',
      range: [0, 1],
      dtick: 1,
      tick0: 0,
      showtickprefix: 'all',
      showticksuffix: 'all',
      tickwidth: 0,
      linecolor: 'transparent',
      linewidth: 1,
      zeroline: false,
    },
    border_radius: 10,
    paper_bgcolor: 'black',
    plot_bgcolor: 'black',
    hovermode: 'closest',
    dragmode: false,
    autosize: false,
    height: 150,
    width: this.windowWidth * 0.58,
    margin: {
      l: 40,
      r: 20,
      b: 17,
      t: 10,
      pad: 4,
    },
  };

  plotData1 = {
    x: [0],
    y: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    mode: 'lines',
    line: {
      width: 2,
      color: this.color,
      shape: 'hv',
    },
  };
  plotData2 = {
    x: [],
    y: [0, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 2],
    yaxis: 'y2',
    xaxis: 'x2',
  };
  private plotData = [this.plotData1, this.plotData2];

  constructor(public elementRef: ElementRef) { }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['isActive'] && !changes['isActive'].isFirstChange()) {
      // console.log('isActive changed:', changes['isActive'].currentValue);
      this.renderPlot(); // Re-render the plot when isActive changes
    }
  }

  ngOnInit() {
    if (this.stepChannel && this.stepGlobalParameters) {
      this.commonChanAttributes =
        this.stepChannel.start_stop_channel.common_chan_attributes;

      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      console.log('device_id, channame', this.deviceID, this.chanName);
      this.sourceFunction = this.commonChanAttributes.source_function;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measRange = this.commonChanAttributes.meas_range;

      this.start = this.stepChannel.start_stop_channel.start;
      this.stop = this.stepChannel.start_stop_channel.stop;
      this.style = this.stepChannel.start_stop_channel.style;

      this.list = this.stepGlobalParameters?.list_step;
      this.listStep = this.stepChannel.start_stop_channel.list;

      this.stepPoints = this.stepGlobalParameters.step_points;
      this.stepToSweepDelay = this.stepGlobalParameters.step_to_sweep_delay;
    }
    this.plotData1.x = this.plotDataX;
    this.plotData1.line.color = this.color;
    this.updatePlotLayout();
    this.initializePlot();
    this.observeThemeChanges();
  }

  // the plots are rendered only after the DOM is created, so we need to render them after the DOM is loaded
  ngAfterViewInit(): void {
    if (this.style?.value == 'LIN' && this.list == false) {
      this.stepValues();
    } else if (this.list == true) {
      this.stepListPlot();
    } else if (this.style?.value == 'LOG' && this.list == false) {
      this.updatePlotStyle();
    }
    this.renderPlot();
  }

  @HostListener('window:resize')
  onResize(): void {
    this.windowHeight = window.innerHeight;
    this.windowWidth = window.innerWidth;
    // console.log('Window resized');
    this.plotLayout.width = (this.windowWidth * 58) / 100;
    this.renderPlot();
    // console.log('Plot resized to:', this.plotLayout.width);
  }

  private generatePlotData(yData: number[], type: string): void {
    if (this.stepPoints && this.stepToSweepDelay && this.tickDifference) {
      const delayTime = this.stepToSweepDelay?.value ?? 0;
      const targetLength = this.plotWidth;
      let processedYData = [...yData];
      // let processedXData: number[] = [];
      // const numberofSteps: number = this.stepPoints.value;
      // processedXData = Array.from({ length: processedYData.length }, (_, i) => (i / (processedYData.length - 1) * numberofSteps)).concat(numberofSteps).flat();

      // Adding delay first if its exists
      // if (delayTime > 0) {
      //   const { y } = this.generateDataWithDelay(processedYData, delayTime);
      //   processedYData = y;
      // }

      if (processedYData.length > targetLength) {
        if (type == 'LIN' || type == 'LOG' || type == 'LIST') {
          // If we have delay, we need to interpolate X,Y pairs together to maintain timing
          // This was suggested to be better than interpolation in this case so the graph is continous in all places and works well in our use case
          if (delayTime > 0) {
            // Create indices for interpolation based on target length
            const indices = Array.from({ length: targetLength }, (_, i) =>
              Math.floor((i / (targetLength - 1)) * (processedYData.length - 1))
            );

            // Interpolate by sampling the delay-generated data at calculated indices
            processedYData = indices.map(i => processedYData[i]);
            // processedXData = indices.map(i => processedXData[i]);
          } else {
            // Without delay, use original interpolation method
            const interpolated = PlotUtils.minMaxInterpolation(processedYData, targetLength);
            processedYData = interpolated.y;
            // processedXData = Array.from({ length: processedYData.length }, (_, i) => (i / (processedYData.length - 1) * numberofSteps)).concat(numberofSteps).flat();
          }
        }
      }

      this.plotData1.x = this.plotDataX;
      this.plotData1.y = processedYData;

      // const totalTime = this.stepPoints.value * (1 + delayTime); // Each step now takes (1 + delayTime) units

      this.plotLayout.xaxis.dtick = this.tickDifference;
      this.plotLayout.xaxis.range = [0, this.tickDifference * 10];
      console.log('plotdata', this.plotData1);
    }
  }

  private stepListPlot() {
    if (this.listStep && this.stepPoints && this.stop) {
      const stepValues = this.listStep
        .map((pf) => pf?.value ?? 0)
        .concat(this.listStep[this.listStep.length - 1]?.value ?? 0);
      this.generatePlotData(stepValues, 'LIST');
    }
    this.renderPlot();
  }

  private stepValues() {
    if (this.start && this.stop && this.stepPoints) {
      document.body.classList.add('wait-cursor'); // Force wait cursor everywhere
      const stepSize =
        (this.stop.value - this.start.value) / (this.stepPoints.value - 1);
      const yData = Array.from(
        { length: this.stepPoints.value },
        (_, i) => (this.start?.value ?? 0) + i * stepSize
      ).concat(this.stop?.value ?? 0);

      this.generatePlotData(yData, 'LIN');
    }
    document.body.classList.remove('wait-cursor'); // Restore cursor after rendering
  }

  private updatePlotLayout(): void {
    if (this.sourceFunction) {
      this.plotLayout.yaxis2.ticksuffix =
        this.sourceFunction.value === 'Voltage' ? ' V' : ' A';
    }
    if (this.start && this.stop && this.list == false && this.sourceRange) {
      const maxRange = PlotUtils.computeMaxRange(
        this.start.value,
        this.stop.value
      );
      const minRange = PlotUtils.computeMinRange(
        this.start.value,
        this.stop.value
      );

      if (typeof maxRange === 'number' && !isNaN(maxRange)) {
        this.plotLayout.yaxis.range = [minRange, maxRange];
        this.plotLayout.yaxis2.range = [minRange, maxRange];
        this.plotLayout.yaxis2.dtick = Math.abs(maxRange - minRange);
        this.plotLayout.yaxis2.tick0 = minRange;
        this.plotLayout.yaxis.tick0 = minRange;

        const dtick = Math.abs(maxRange - minRange) / 4; // Set vertical axis tick spacing to divide the range into 4 intervals
        this.plotLayout.yaxis.dtick = dtick;
      }
    }

    if (this.listStep && this.list == true) {
      const allValues = this.listStep.flat().map((pf) => pf.value);
      const max = Math.max(...allValues);
      const min = Math.min(...allValues);
      const maxRange = PlotUtils.computeMaxRange(min, max);
      const minRange = PlotUtils.computeMinRange(min, max);
      this.plotLayout.yaxis.range = [minRange, maxRange];
      this.plotLayout.yaxis2.range = [minRange, maxRange];
      this.plotLayout.yaxis2.dtick = Math.abs(maxRange - minRange);
      this.plotLayout.yaxis2.tick0 = minRange;
      this.plotLayout.yaxis.tick0 = minRange;

      const dtick = (maxRange - minRange) / 4; // Divide maxRange into 4 intervals
      this.plotLayout.yaxis.dtick = dtick;
    }
  }

  private updatePlotStyle(): void {
    if (this.style?.value === 'LOG') {
      // this.plotLayout.xaxis.type = 'log';
      // this.plotData1.line.shape = 'vh';

      if (this.start && this.stop && this.stepPoints) {
        const startValue = this.start.value > 0 ? this.start.value : 1e-12;
        const stopValue = this.stop.value > 0 ? this.stop.value : 1e-12;
        const numPoints = this.stepPoints.value;
        const stepFactor = Math.pow(
          stopValue / startValue,
          1 / (numPoints - 1)
        );

        const yData = Array.from(
          { length: numPoints },
          (_, i) => startValue * Math.pow(stepFactor, i)
        ).concat(this.stop.value);

        this.generatePlotData(yData, 'LOG');
      }
    } else {
      this.stepValues();
    }
    this.renderPlot();
  }

  private initializePlot(): void {
    const backgroundColor = this.getCssVariableValue(
      '--vscode-editor-background'
    );
    const backgroundColorHex = backgroundColor.startsWith('rgb')
      ? this.rgbToHex(backgroundColor)
      : backgroundColor;

    this.originalBackgroundColor = backgroundColorHex;
    this.plotLayout.paper_bgcolor = backgroundColorHex;
    this.plotLayout.plot_bgcolor = backgroundColorHex;

    // Fetch and store active background color
    const activeBg = this.getCssVariableValue(
      '--vscode-activityErrorBadge-foreground'
    );
    this.activeBackgroundColor = activeBg.startsWith('rgb')
      ? this.rgbToHex(activeBg)
      : activeBg;

    // Fetch and store tick color
    const tickColorRaw = this.getCssVariableValue('--vscode-editor-foreground');
    this.tickColor = tickColorRaw.startsWith('rgb')
      ? this.rgbToHex(tickColorRaw)
      : tickColorRaw;

    // Update tick colors in plot layout
    this.plotLayout.xaxis.tickfont.color = this.tickColor;
    this.plotLayout.yaxis.tickfont.color = this.tickColor;
    this.plotLayout.yaxis2.tickfont.color = this.tickColor;

    console.log('Initial background color:', backgroundColorHex);
    console.log('Initial active background color:', this.activeBackgroundColor);
    console.log('Initial tick color:', this.tickColor);
  }

  getCssVariableValue(variableName: string): string {
    const root = document.documentElement;
    const value = getComputedStyle(root).getPropertyValue(variableName).trim();
    console.log(`CSS variable ${variableName}:`, value);
    return value;
  }

  rgbToHex(rgb: string): string {
    const match = rgb.match(/\d+/g);
    if (!match) return rgb;

    const [r, g, b] = match.map(Number);
    return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
  }

  private renderPlot(): void {
    if (this.plotDataX && this.plotConfig) {
      if (this.isActive) {
        this.plotLayout.plot_bgcolor = this.activeBackgroundColor;
        this.plotLayout.paper_bgcolor = this.activeBackgroundColor;
      } else {
        this.plotLayout.plot_bgcolor = this.originalBackgroundColor;
        this.plotLayout.paper_bgcolor = this.originalBackgroundColor;
      }
      Plotly.newPlot(
        'divStep',
        this.plotData,
        this.plotLayout,
        this.plotConfig
      );
    }
  }

  private observeThemeChanges(): void {
    const root = document.documentElement;

    this.mutationObserver = new MutationObserver(() => {
      const backgroundColor = this.getCssVariableValue(
        '--vscode-editor-background'
      );
      const backgroundColorHex = backgroundColor.startsWith('rgb')
        ? this.rgbToHex(backgroundColor)
        : backgroundColor;

      this.plotLayout.paper_bgcolor = backgroundColorHex;
      this.plotLayout.plot_bgcolor = backgroundColorHex;

      // Update active background color on theme change
      const activeBg = this.getCssVariableValue(
        '--vscode-activityErrorBadge-foreground'
      );
      this.activeBackgroundColor = activeBg.startsWith('rgb')
        ? this.rgbToHex(activeBg)
        : activeBg;

      // Update tick color on theme change
      const tickColorRaw = this.getCssVariableValue(
        '--vscode-editor-foreground'
      );
      this.tickColor = tickColorRaw.startsWith('rgb')
        ? this.rgbToHex(tickColorRaw)
        : tickColorRaw;

      // Update tick colors in plot layout
      this.plotLayout.xaxis.tickfont.color = this.tickColor;
      this.plotLayout.yaxis.tickfont.color = this.tickColor;
      this.plotLayout.yaxis2.tickfont.color = this.tickColor;

      console.log('Theme changed, new background color:', backgroundColorHex);
      console.log(
        'Theme changed, new active background color:',
        this.activeBackgroundColor
      );
      console.log('Theme changed, new tick color:', this.tickColor);

      this.renderPlot();
    });

    this.mutationObserver.observe(root, {
      attributes: true,
      attributeFilter: ['style'],
    });
  }

  ngOnDestroy(): void {
    // Disconnect the MutationObserver when the component is destroyed
    if (this.mutationObserver) {
      this.mutationObserver.disconnect();
    }
  }
}
