import { Component, Input } from '@angular/core';
import { StepChannel } from '../../../../model/chan_data/stepChannel';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import { ParameterFloat, ParameterInt, ParameterString } from '../../../../model/sweep_data/TimingConfig';
import { StepGlobalParameters } from '../../../../model/sweep_data/stepSweepConfig';
import { CommonChanAttributes } from '../../../../model/chan_data/defaultChannel';
import * as Plotly from 'plotly.js-dist';

@Component({
  selector: 'plot-step',
  standalone: false,
  templateUrl: './plot-step.component.html',
  styleUrl: './plot-step.component.scss'
})
export class PlotStepComponent {
  @Input() stepChannel: StepChannel | undefined;
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;

  // @Input() plotData: any;
  // @Input() plotLayout: any;
  @Input() plotDataX: number[] = [];
  @Input() plotConfig: any;

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
  list: boolean = false;

  plotLayout =  {
    xaxis: {
      visible: true,
      ticksuffix: ' s',
      rangemode: 'nonnegative',
      separatethousands: false,
      tickfont: { family: 'Times New Roman', color: 'white', size: 9},
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
    },
    yaxis: {
      visible: true,
      ticksuffix: ' V',
      range: [0, 1],
      tickfont: { family: 'Times New Roman', color: 'white' },
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
      title: {
        // text: '0.25V/div',
        font: {color: 'white', size: 9, family: 'Times New Roman'}
      },
      tickfont: {family: 'Times New Roman', color: 'white', size: 9},
      anchor: 'x',
      overlaying: 'y',
      side: 'left',
      position: -3,
      showline: false,
      showlegend: false,
      showticklabels: true,
      zerolinewidth: 0,
      showgrid: true,
      gridcolor: 'lightgrey',
      gridwidth: 0.3,
      type: 'linear',
      griddash: 'dot',
      // position: 20,
      visible: true,
      ticksuffix: ' V',
      range: [0, 1], // Adjust the range as needed
      separatethousands: false,
      // tickfont: { family: 'Times New Roman', color: 'white' },
      dtick: 1,
      tick0: 0,
      showtickprefix: 'all',
      showticksuffix: 'all',
      tickwidth: 0,
      linecolor: 'black',
      linewidth: 1,
      layer: 'below traces',

    },
    // grid: {rows: 1, columns: 1, pattern: 'independent'},
    border_radius: 10,
    paper_bgcolor: 'black',
    plot_bgcolor: 'black',
    // borderradius: 10,
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
    },
    shapes: [
      {
        type: 'line',
        x0: 0, // Start of the line on the x-axis
        x1: 10, // End of the line on the x-axis
        y0: 1, // Y-axis value where the line starts
        y1: 1, // Y-axis value where the line ends (same as y0 for a horizontal line)
        line: {
          color: 'grey', // Color of the line
          width: 2, // Thickness of the line
          dash: 'solid', // Line style: 'solid', 'dot', 'dash', etc.
        },
      },
    ],
  };

  plotData1 = { x: [0],
    y: [0,0,0,0,0,0,0,0,0,0,0],
    mode: 'lines',
    line: {
      width: 2,
      color: '#7FBDC6',
      shape: 'hv'
    },
  };
  plotData2 = {  x: [],
    y: [0, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 2],
    yaxis: 'y2',
  };
  private plotData = [this.plotData1, this.plotData2];

  ngOnInit() {
    if (this.stepChannel && this.stepGlobalParameters) {
      this.commonChanAttributes =
        this.stepChannel.start_stop_channel.common_chan_attributes;

      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      this.sourceFunction = this.commonChanAttributes.source_function;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measRange = this.commonChanAttributes.meas_range;

      this.start = this.stepChannel.start_stop_channel.start;
      this.stop = this.stepChannel.start_stop_channel.stop;
      this.style = this.stepChannel.start_stop_channel.style;

      this.stepPoints = this.stepGlobalParameters.step_points;
      this.stepToSweepDelay = this.stepGlobalParameters.step_to_sweep_delay;

      // if (this.start?.value !== undefined && this.stop?.value !== undefined && this.stepPoints?.value !== undefined) {
      //   const stepSize = (this.stop.value - this.start.value) / (this.stepPoints.value - 1);
      //   this.plotDataX = Array.from({ length: this.stepPoints.value }, (_, i) => i).concat(this.stepPoints.value);
      //   this.plotData1.y = Array.from({ length: this.stepPoints?.value ?? 0 }, (_, i) => (this.start?.value ?? 0) + i * stepSize).concat(this.stop?.value ?? 0);
      // }
      // // this.generateStepData(this.stepPoints.value, this.start.value, this.stop.value, this.plotData1);
      // console.log("after",this.plotDataX, this.plotData, this.stepPoints);
    }
    this.plotData1.x = this.plotDataX;
    this.plotData1.y = [0, 0.111, 0.222, 0.333, 0.444, 0.555, 0.666, 0.777, 0.888, 0.999, 1];
    console.log('after loop', this.plotDataX, this.plotData, this.stepPoints);
    Plotly.newPlot('divStep', this.plotData, this.plotLayout, this.plotConfig);
  }

  generateStepData(num_steps: number, start: number, stop: number, plot_data: any){
    const stepSize = (stop - start) / (num_steps - 1);
    plot_data.x = Array.from({ length: num_steps }, (_, i) => i).concat(num_steps);
    plot_data.y = Array.from({ length: num_steps }, (_, i) => start + i * stepSize).concat(stop);
    console.log('generated step data:', plot_data);
    // Plotly.update(id, [plot_data], layout);
  }
}
