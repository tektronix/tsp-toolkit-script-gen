import { AfterViewInit, Component, Input, OnInit } from '@angular/core';
import { StepChannel } from '../../../../model/chan_data/stepChannel';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import { ParameterFloat, ParameterInt, ParameterString } from '../../../../model/sweep_data/TimingConfig';
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
export class PlotStepComponent implements AfterViewInit, OnInit{
  @Input() stepChannel: StepChannel | undefined;
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;

  // @Input() plotData: any;
  // @Input() plotLayout: any;
  @Input() plotDataX: number[] = [];
  @Input() plotConfig: { staticPlot: boolean; } | undefined;

  @Input() isActive = false;
  @Input() activeStyle: {backgroundColor:string, color:string} = {
    backgroundColor: '',
    color: ''
  };
  @Input() color = '';

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
      tickfont: { family: 'Times New Roman', color: 'white', size: 9 },
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
      tickfont: { family: 'Times New Roman', color: 'white', size: 9 },
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
      linecolor: 'black',
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
      shape: 'hv'
    },
  };
  plotData2 = {  x: [],
    y: [0, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 2],
    yaxis: 'y2',
    xaxis: 'x2'
  };
  private plotData = [this.plotData1, this.plotData2];

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
    this.stepValues();
    // this.plotData1.x = this.plotDataX;
    // this.plotData1.y = [0, 0.111, 0.222, 0.333, 0.444, 0.555, 0.666, 0.777, 0.888, 0.999, 1];
    this.plotData1.line.color = this.color;
    this.updatePlotLayout();
    console.log('after loop', this.plotDataX, this.plotData, this.stepPoints);
    // Plotly.newPlot('divStep', this.plotData, this.plotLayout, this.plotConfig);
  }

  ngAfterViewInit(): void{
    if (this.plotDataX && this.plotConfig) {
      this.stepValues();
      Plotly.newPlot('divStep', this.plotData, this.plotLayout, this.plotConfig);
      console.log('step data', this.plotData, this.plotLayout, this.plotConfig);
    }
  }

  stepValues(){
    if (this.start && this.stop && this.stepPoints) {
      const stepSize = (this.stop.value - this.start.value) / (this.stepPoints.value - 1);
      this.plotData1.x = Array.from({ length: this.stepPoints.value }, (_, i) => i).concat(this.stepPoints.value).flat();
      this.plotData1.y = Array.from({ length: this.stepPoints?.value ?? 0 }, (_, i) => (this.start?.value ?? 0) + i * stepSize).concat(this.stop?.value ?? 0).flat();
      console.log('generated step data:', this.plotData1);
      // Plotly.update('divStep', [this.plotData1], this.plotLayout);
    }
  }
  // generateStepData(num_steps: number, start: number, stop: number, plot_data: any){
  //   const stepSize = (stop - start) / (num_steps - 1);
  //   plot_data.x = Array.from({ length: num_steps }, (_, i) => i).concat(num_steps);
  //   plot_data.y = Array.from({ length: num_steps }, (_, i) => start + i * stepSize).concat(stop);
  //   console.log('generated step data:', plot_data);
  //   // Plotly.update(id, [plot_data], layout);
  // }

  private updatePlotLayout(): void {
    if (this.sourceFunction) {
      this.plotLayout.yaxis2.ticksuffix = this.sourceFunction.value === 'Voltage' ? ' V' : ' A';
    }

    // if (this.sourceRange) {
    //   const maxRange = parseFloat(this.sourceRange.value);
    //   this.plotLayout.yaxis.range = [0, maxRange];
    //   this.plotLayout.yaxis2.range = [0, maxRange];
    //   this.plotLayout.yaxis2.dtick = maxRange;

    //   const dtick = maxRange / 4;
    //   this.plotLayout.yaxis.dtick = dtick;
    // }
  }
}
