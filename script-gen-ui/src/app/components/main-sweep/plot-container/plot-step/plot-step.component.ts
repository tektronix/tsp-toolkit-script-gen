import { AfterViewInit, Component, ElementRef, Input, OnDestroy, OnInit } from '@angular/core';
import { StepChannel } from '../../../../model/chan_data/stepChannel';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import { ParameterFloat, ParameterInt, ParameterString } from '../../../../model/sweep_data/SweepTimingConfig';
import { StepGlobalParameters } from '../../../../model/sweep_data/stepSweepConfig';
import { CommonChanAttributes } from '../../../../model/chan_data/defaultChannel';
import * as Plotly from 'plotly.js-dist';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';

@Component({
  selector: 'app-plot-step',
  standalone: true,
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  templateUrl: './plot-step.component.html',
  styleUrl: './plot-step.component.scss'
})
export class PlotStepComponent implements AfterViewInit, OnInit, OnDestroy {
  @Input() stepChannel: StepChannel | undefined;
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;

  @Input() plotDataX: number[] = [];
  @Input() plotConfig: { staticPlot: boolean; } | undefined;
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
  @Input() activeStyle: {backgroundColor:string, color:string} = {
    backgroundColor: '',
    color: ''
  };
  @Input() color = '';
  private mutationObserver: MutationObserver | undefined;
  private originalBackgroundColor = '';
  activeBackgroundColor = '';

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
  list = false;

  plotLayout =  {
    xaxis: {
      visible: true,
      ticksuffix: ' s',
      rangemode: 'nonnegative',
      separatethousands: false,
      tickfont: { family: 'Roboto, "Helvetica Neue", sans-serif', color: 'white', size: 9 },
      dtick: 1,
      tick0: 0,
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
      linewidth: 1
    },
    xaxis2: {
      visible: true,
       rangemode: 'nonnegative',
       dtick: 1,
       tick0: 0,
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
      linewidth: 1
     },
    yaxis: {
      visible: true,
      range: [0, 1],
      tickfont: { family: 'Roboto, "Helvetica Neue", sans-serif', color: 'white', size: 9 },
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
    },
    yaxis2: {
      tickfont: {family: 'Times New Roman', color: 'white', size: 9},
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
    },
    border_radius: 10,
    paper_bgcolor: 'black',
    plot_bgcolor: 'black',
    hovermode: 'closest',
    dragmode: false,
    autosize: false,
    height: 150,
    margin: {
      l: 40,
      r: 20,
      b: 17,
      t: 10,
      pad: 4,
    }
  };

  plotData1 = { x: [0],
    y: [0,0,0,0,0,0,0,0,0,0,0],
    mode: 'lines',
    line: {
      width: 2,
      color: this.color,
      shape: 'vh'
    },
  };
  plotData2 = {  x: [],
    y: [0, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 2],
    yaxis: 'y2',
    xaxis: 'x2'
  };
  private plotData = [this.plotData1, this.plotData2];

  constructor(public elementRef: ElementRef){}

  ngOnInit() {
    if (this.stepChannel && this.stepGlobalParameters) {
      this.commonChanAttributes =
        this.stepChannel.start_stop_channel.common_chan_attributes;

      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      console.log("device_id, channame", this.deviceID, this.chanName);
      this.sourceFunction = this.commonChanAttributes.source_function;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measRange = this.commonChanAttributes.meas_range;

      this.start = this.stepChannel.start_stop_channel.start;
      this.stop = this.stepChannel.start_stop_channel.stop;
      this.style = this.stepChannel.start_stop_channel.style;

      this.stepPoints = this.stepGlobalParameters.step_points;
      this.stepToSweepDelay = this.stepGlobalParameters.step_to_sweep_delay;

    }
    // this.stepValues();
    console.log(" step points list", this.stepPointsList);
    this.plotData1.line.color = this.color;
    this.updatePlotLayout();
    this.initializePlot();
    this.observeThemeChanges();
    this.updatePlotStyle();
    console.log('after loop', this.plotDataX, this.plotData, this.stepPoints);
  }

  ngAfterViewInit(): void{
    if (this.plotDataX && this.plotConfig && this.style?.value == 'LIN') {
      this.stepValues();

      Plotly.newPlot('divStep', this.plotData, this.plotLayout, this.plotConfig);
      console.log('step data', this.plotData, this.plotLayout, this.plotConfig);
    }
    else {
      this.updatePlotStyle();
    }
    this.renderPlot();
  }

  stepValues(){
    if (this.start && this.stop && this.stepPoints) {
      this.plotData1.line.shape = 'hv';
      const stepSize = (this.stop.value - this.start.value) / (this.stepPoints.value - 1);
      this.plotData1.x = Array.from({ length: this.stepPoints.value }, (_, i) => i).concat(this.stepPoints.value).flat();
      this.plotData1.y = Array.from({ length: this.stepPoints?.value ?? 0 }, (_, i) => (this.start?.value ?? 0) + i * stepSize).concat(this.stop?.value ?? 0).flat();
      console.log('generated step data:', this.plotData1);
    }
  }

  private updatePlotLayout(): void {
    if (this.sourceFunction) {
      this.plotLayout.yaxis2.ticksuffix = this.sourceFunction.value === 'Voltage' ? ' V' : ' A';
    }
    if (this.start && this.stop) {
      const maxRange = Math.max(this.start.value, this.stop.value);
      this.plotLayout.yaxis.range = [0, maxRange];
      this.plotLayout.yaxis2.range = [0, maxRange];
      this.plotLayout.yaxis2.dtick = maxRange;

      const dtick = maxRange / 4; // Divide maxRange into 4 intervals
      this.plotLayout.yaxis.dtick = dtick;
    }
  }

  private updatePlotStyle(): void {
    if (this.style?.value === 'LOG') {
      // this.plotLayout.xaxis.type = 'log';
      // this.plotData1.line.shape = 'vh';

      if (this.start && this.stop && this.stepPoints) {
        console.log('start:', this.start.value, 'stop:', this.stop.value, 'stepPoints:', this.stepPoints.value);
        const startValue = this.start.value > 0 ? this.start.value : 1e-12;
        const stopValue = this.stop.value > 0 ? this.stop.value : 1e-12;
        const stepFactor = Math.pow(stopValue / startValue, 1 / (this.stepPoints.value - 1));
        console.log('startValue:', startValue, 'stopValue:', stopValue, 'stepFactor:', stepFactor);

        this.plotData1.x = Array.from({ length: this.stepPoints.value }, (_, i) => i).concat(this.stepPoints.value).flat();
        this.plotData1.y = Array.from({ length: this.stepPoints.value }, (_, i) =>
          startValue * Math.pow(stepFactor, i)
        );
        console.log("plotData1", this.plotData1);
      }
    } else {
      this.stepValues();
    }
    this.renderPlot();
  }

  private initializePlot(): void {
    const backgroundColor = this.getCssVariableValue('--vscode-editor-background');
    const backgroundColorHex = backgroundColor.startsWith('rgb')
      ? this.rgbToHex(backgroundColor)
      : backgroundColor;

    this.originalBackgroundColor = backgroundColorHex;
    this.plotLayout.paper_bgcolor = backgroundColorHex;
    this.plotLayout.plot_bgcolor = backgroundColorHex;

    // Fetch and store active background color
    const activeBg = this.getCssVariableValue('--vscode-activityErrorBadge-foreground');
    this.activeBackgroundColor = activeBg.startsWith('rgb')
      ? this.rgbToHex(activeBg)
      : activeBg;

    console.log('Initial background color:', backgroundColorHex);
    console.log('Initial active background color:', this.activeBackgroundColor);
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
      Plotly.newPlot('divStep', this.plotData, this.plotLayout, this.plotConfig);
    }
  }

  private observeThemeChanges(): void {
    const root = document.documentElement;

    this.mutationObserver = new MutationObserver(() => {
      const backgroundColor = this.getCssVariableValue('--vscode-editor-background');
      const backgroundColorHex = backgroundColor.startsWith('rgb')
        ? this.rgbToHex(backgroundColor)
        : backgroundColor;

      this.plotLayout.paper_bgcolor = backgroundColorHex;
      this.plotLayout.plot_bgcolor = backgroundColorHex;

      // Update active background color on theme change
      const activeBg = this.getCssVariableValue('--vscode-activityErrorBadge-foreground');
      this.activeBackgroundColor = activeBg.startsWith('rgb')
        ? this.rgbToHex(activeBg)
        : activeBg;

      console.log('Theme changed, new background color:', backgroundColorHex);
      console.log('Theme changed, new active background color:', this.activeBackgroundColor);

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
