import {
  AfterViewInit,
  Component,
  ElementRef,
  Input,
  OnDestroy,
  OnInit,
  OnChanges,
  SimpleChanges,
  HostListener,
} from '@angular/core';
import { SweepChannel } from '../../../../model/chan_data/sweepChannel';
import { CommonChanAttributes } from '../../../../model/chan_data/defaultChannel';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import {
  ParameterFloat,
  ParameterInt,
  ParameterString,
} from '../../../../model/sweep_data/SweepTimingConfig';
import * as Plotly from 'plotly.js-dist';
import {
  StepGlobalParameters,
  SweepGlobalParameters,
} from '../../../../model/sweep_data/stepSweepConfig';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { PlotUtils } from '../plot-utils';

@Component({
  selector: 'app-plot-sweep',
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  templateUrl: './plot-sweep.component.html',
  styleUrl: './plot-sweep.component.scss',
})
export class PlotSweepComponent
  implements AfterViewInit, OnInit, OnDestroy, OnChanges {
  @Input() sweepChannel: SweepChannel | undefined;
  @Input() sweepGlobalParameters: SweepGlobalParameters | undefined;
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;
  @Input() plotDataX: number[] = [];
  @Input() plotConfig: { staticPlot: boolean } | undefined;
  @Input() sweepPointsList: ParameterFloat[][] = [];

  @Input() isActive = false;
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

  sweepDivID = '';

  commonChanAttributes: CommonChanAttributes | undefined;
  chanName = 'Sweep1';
  deviceID = '';
  sourceFunction: ParameterString | undefined;
  sourceRange: ChannelRange | undefined;
  measFunction: ParameterString | undefined;
  measRange: ChannelRange | undefined;

  start: ParameterFloat | undefined;
  stop: ParameterFloat | undefined;
  style: ParameterString | undefined;

  numPoints: ParameterInt | undefined;
  list = true;
  listSweep: ParameterFloat[] = [];
  numSteps: number | undefined;
  stepToSweepDelay: ParameterFloat | undefined;

  plotUtils = new PlotUtils();

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
      tickfont: {
        family: 'Roboto, "Helvetica Neue", sans-serif',
        color: this.tickColor,
        size: 9,
      },
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
    if (
      this.sweepChannel &&
      this.sweepGlobalParameters &&
      this.stepGlobalParameters
    ) {
      this.commonChanAttributes =
        this.sweepChannel.start_stop_channel.common_chan_attributes;

      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      this.sourceFunction = this.commonChanAttributes.source_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.measRange = this.commonChanAttributes.meas_range;

      this.start = this.sweepChannel.start_stop_channel.start;
      this.stop = this.sweepChannel.start_stop_channel.stop;
      this.style = this.sweepChannel.start_stop_channel.style;

      this.numPoints = this.sweepGlobalParameters?.sweep_points;
      this.list = this.sweepGlobalParameters?.list_sweep;
      this.numSteps = this.stepGlobalParameters?.step_points.value;
      this.stepToSweepDelay = this.stepGlobalParameters?.step_to_sweep_delay;
      // this.list = this.sweepGlobalParameters?.list_sweep;
      this.listSweep = this.sweepChannel.start_stop_channel.list;

      this.sweepDivID = `plotDiv${this.sweepChannel.start_stop_channel.common_chan_attributes.uuid}`;
      console.log('sweepDivID', this.sweepDivID);
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
      this.sweepValues();
    } else if (this.list == true) {
      this.sweepListPlot();
    } else if (this.style?.value == 'LOG' && this.list == false) {
      this.updatePlotStyle();
    }
    this.renderPlot();
  }

  @HostListener('window:resize')
  onResize(): void {
    this.windowHeight = window.innerHeight;
    this.windowWidth = window.innerWidth;
    console.log('Window resized');
    this.plotLayout.width = (this.windowWidth * 58) / 100;
    this.renderPlot();
    console.log('Plot resized to:', this.plotLayout.width);
  }

  private sweepValues() {
    if (this.start && this.stop && this.numPoints) {
      const numberOfPoints = this.numPoints.value;
      const startValue = this.start.value;
      const stopValue = this.stop.value;

      const stepSize = (stopValue - startValue) / (numberOfPoints - 1);

      const sweepValues = Array.from(
        { length: numberOfPoints },
        (_, i) => startValue + i * stepSize
      ).flat();

      this.generatePlotData(sweepValues, 'LIN');
    }
  }

  private generatePlotDataxy(sweepValues: number[], xData?: number[]) {
    if (this.numPoints && this.numSteps) {
      const numSteps = this.numSteps;
      const numberOfPoints = this.numPoints?.value;
      const delayTime = this.stepToSweepDelay?.value ?? 0;
      
      if (delayTime > 0) {
        const { x, y } = this.generateDataWithDelay(sweepValues, numSteps, numberOfPoints, delayTime, xData);
        this.plotData1.x = x;
        this.plotData1.y = y;
      } else {
        this.generateDataWithoutDelay(sweepValues, numSteps, numberOfPoints, xData);
      }
    }
  }

  private generateDataWithDelay(
    sweepValues: number[], 
    numSteps: number, 
    numberOfPoints: number, 
    delayTime: number, 
    xData?: number[]
  ): { x: number[], y: number[] } {
    const finalX: number[] = [];
    const finalY: number[] = [];
    const delayPoints = Math.max(5, Math.floor(delayTime * 10));
    
    // Generate data for each step with delay
    for (let step = 0; step < numSteps; step++) {
      const stepStartTime = step * (1 + delayTime);
      
      // Add delay period (zeros) at the beginning of each step
      for (let d = 0; d < delayPoints; d++) {
        finalX.push(stepStartTime + (d * delayTime) / delayPoints);
        finalY.push(0);
      }
      
      // Add the actual sweep data for this step
      for (let j = 0; j < numberOfPoints; j++) {
        if (xData) {
          const originalIndex = step * numberOfPoints + j;
          if (originalIndex < xData.length) {
            finalX.push(stepStartTime + delayTime + (j / numberOfPoints));
          }
        } else {
          finalX.push(stepStartTime + delayTime + (j / numberOfPoints));
        }
        finalY.push(sweepValues[j]);
      }
    }
    
    // Add final point
    if (sweepValues.length > 0) {
      const finalStepTime = numSteps * (1 + delayTime);
      finalX.push(finalStepTime);
      finalY.push(sweepValues[sweepValues.length - 1]);
    }
    
    return { x: finalX, y: finalY };
  }

  private generateDataWithoutDelay(sweepValues: number[], numSteps: number, numberOfPoints: number, xData?: number[]) {
    this.plotData1.y = Array.from({ length: numSteps }, () => sweepValues)
      .flat()
      .concat(sweepValues[sweepValues.length - 1]);

    if (xData) {
      this.plotData1.x = xData;
    } else {
      this.plotData1.x = Array.from({ length: numSteps }, (_, i) =>
        Array.from({ length: numberOfPoints }, (_, j) => i + j / numberOfPoints)
      )
        .flat()
        .concat(numSteps);
    }
  }

  private generatePlotData(sweepValues: number[], type: string) {
    if (this.numPoints && this.numSteps && this.stepToSweepDelay) {
      const targetLength = this.plotWidth / this.numSteps;

      
      if (this.numPoints?.value > targetLength) {
        let xData: number[] = [];
        if (type == 'LIN' || type == 'LOG' || type == 'LIST') {
          const interpolated = PlotUtils.minMaxInterpolation(
            sweepValues,
            targetLength
          );
          sweepValues = interpolated.y;
        }
        xData = Array.from({ length: this.numSteps }, (_, i) =>
            Array.from({ length: sweepValues.length }, (_, j) => i + j / sweepValues.length)).flat().concat(this.numSteps)
        this.generatePlotDataxy(sweepValues, xData);
      } else {
        this.generatePlotDataxy(sweepValues);
      }
      
      // Update x-axis range to include delay time for each step
      const delayTime = this.stepToSweepDelay.value;
      const totalTime = this.numSteps * (1 + delayTime); // Each step now takes (1 + delayTime) units
      
      this.plotLayout.xaxis.dtick = totalTime / 10;
      this.plotLayout.xaxis.range = [0, totalTime];
    }
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

        const dtick = Math.abs(maxRange - minRange) / 4; // Divide vertical axis range into 4 intervals
        this.plotLayout.yaxis.dtick = dtick;
      }
    }

    if (this.listSweep && this.list == true) {
      const allValues = this.listSweep.flat().map((pf) => pf.value);
      const max = Math.max(...allValues);
      const min = Math.min(...allValues);
      const maxRange = PlotUtils.computeMaxRange(min, max);
      const minRange = PlotUtils.computeMinRange(min, max);
      this.plotLayout.yaxis.range = [minRange, maxRange];
      this.plotLayout.yaxis2.range = [minRange, maxRange];
      this.plotLayout.yaxis2.dtick = Math.abs(maxRange - minRange);
      this.plotLayout.yaxis2.tick0 = minRange;
      this.plotLayout.yaxis.tick0 = minRange;

      const dtick = Math.abs(maxRange - minRange) / 4; // Divide maxRange into 4 intervals
      this.plotLayout.yaxis.dtick = dtick;
    }
  }

  private updatePlotStyle(): void {
    if (this.style?.value === 'LOG') {
      if (this.start && this.stop && this.numSteps && this.numPoints) {
        const numberOfPoints = this.numPoints.value;
        const startValue = this.start.value > 0 ? this.start.value : 1e-12;
        const stopValue = this.stop.value > 0 ? this.stop.value : 1e-12;
        const stepFactor = Math.pow(
          stopValue / startValue,
          1 / (numberOfPoints - 1)
        );

        const sweepValues = Array.from(
          { length: numberOfPoints },
          (_, i) => startValue * Math.pow(stepFactor, i)
        );

        this.generatePlotData(sweepValues, 'LOG');
      }
    }
  }

  private sweepListPlot(): void {
    if (this.listSweep) {
      const sweepValues = this.listSweep.map((pf) => pf?.value ?? 0);
      this.generatePlotData(sweepValues, 'LIST');
    }
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

    const activeBg = this.getCssVariableValue(
      '--vscode-activityErrorBadge-foreground'
    );
    this.activeBackgroundColor = activeBg.startsWith('rgb')
      ? this.rgbToHex(activeBg)
      : activeBg;

    // Fetch and store tick color
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

    console.log('Initial background color:', backgroundColorHex);
    console.log('Initial active background color:', this.activeBackgroundColor);
    console.log('Initial tick color:', this.tickColor);
  }

  getCssVariableValue(variableName: string): string {
    const root = document.documentElement;
    const value = getComputedStyle(root).getPropertyValue(variableName).trim();
    // console.log(`CSS variable ${variableName}:`, value);
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
      const plotDiv = document.getElementById(this.sweepDivID);
      if (!plotDiv) {
        // Optionally, log or skip rendering if div is not present
        return;
      }
      if (this.isActive) {
        this.plotLayout.plot_bgcolor = this.activeBackgroundColor;
        this.plotLayout.paper_bgcolor = this.activeBackgroundColor;
      } else {
        this.plotLayout.paper_bgcolor = this.originalBackgroundColor;
        this.plotLayout.plot_bgcolor = this.originalBackgroundColor;
      }
      Plotly.newPlot(
        this.sweepDivID,
        this.plotData,
        this.plotLayout,
        this.plotConfig
      );
    }
  }

  // TODO:
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
      console.log('Theme changed, new active background color:', this.activeBackgroundColor);
      console.log('Theme changed, new tick color:', this.tickColor);

      this.renderPlot();
    });

    this.mutationObserver.observe(root, {
      attributes: true,
      attributeFilter: ['style'],
    });
  }

  ngOnDestroy(): void {
    if (this.mutationObserver) {
      this.mutationObserver.disconnect();
    }
  }
}
