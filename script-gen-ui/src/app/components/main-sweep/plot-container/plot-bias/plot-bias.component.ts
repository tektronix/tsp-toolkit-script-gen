import { Component, Input, AfterViewInit } from '@angular/core';
import { BiasChannel } from '../../../../model/chan_data/biasChannel';
import { ParameterFloat, ParameterString } from '../../../../model/sweep_data/TimingConfig';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import * as Plotly from 'plotly.js-dist';

@Component({
  selector: 'plot-bias',
  standalone: false,
  templateUrl: './plot-bias.component.html',
  styleUrl: './plot-bias.component.scss'
})
export class PlotBiasComponent implements AfterViewInit {
  @Input() biasChannel: BiasChannel | undefined;
  // @Input() plotData: any;
  // @Input() plotLayout: any;
  @Input() plotDataX: number[] = [];
  @Input() plotConfig: any;
  plotDivID: string = '';

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
    },
    yaxis: {
      visible: true,
      ticksuffix: ' V',
      range: [0, 2],
      tickfont: { family: 'Times New Roman', color: 'white', size: 9 },
      dtick: 0.5,
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
      range: [0, 2], // Adjust the range as needed
      separatethousands: false,
      // tickfont: { family: 'Times New Roman', color: 'white' },
      dtick: 2,
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
    // border_radius: 10,
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
        y0: 2, // Y-axis value where the line starts
        y1: 2, // Y-axis value where the line ends (same as y0 for a horizontal line)
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
      color: 'yellow',
    },
  };
  plotData2 = {  x: [],
    y: [0, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 2],
    yaxis: 'y2',
  };
  private plotData = [this.plotData1, this.plotData2];

  chanName = 'Bias1';
  deviceID = '';
  sourceFunction: ParameterString | undefined;
  sourceRange: ChannelRange | undefined;
  measFunction: ParameterString | undefined;
  measRange: ChannelRange | undefined;
  bias: ParameterFloat | undefined;

  ngOnInit(): void {
    if (this.biasChannel) {
      this.chanName = this.biasChannel.common_chan_attributes.chan_name;
      this.deviceID = this.biasChannel.common_chan_attributes.device_id;

      this.sourceFunction = this.biasChannel.common_chan_attributes.source_function;
      this.sourceRange = this.biasChannel.common_chan_attributes.source_range;
      this.measFunction = this.biasChannel.common_chan_attributes.meas_function;
      this.measRange = this.biasChannel.common_chan_attributes.meas_range;
      this.bias = this.biasChannel.bias;

      this.plotDivID = `plotDiv${this.biasChannel.common_chan_attributes.chan_name}`;
      for(let i:number =0; i<11; i++){
        this.plotData1.y[i] = this.bias?.value ?? 0;
      }
    }
    // console.log(this.plotDataX, this.plotData);
    this.plotData1.x = this.plotDataX;
    // Plotly.newPlot(this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
    console.log('bias data', this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
  }

  ngAfterViewInit(): void{
    if (this.plotDataX && this.plotConfig) {
      Plotly.newPlot(this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
      console.log('bias data', this.plotData, this.plotLayout, this.plotConfig);
    }
  }
}
