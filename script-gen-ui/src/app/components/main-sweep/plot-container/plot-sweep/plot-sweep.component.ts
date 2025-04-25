import { AfterViewInit, Component, Input } from '@angular/core';
import { SweepChannel } from '../../../../model/chan_data/sweepChannel';
import { CommonChanAttributes } from '../../../../model/chan_data/defaultChannel';
import { ChannelRange } from '../../../../model/chan_data/channelRange';
import { ParameterFloat, ParameterString } from '../../../../model/sweep_data/TimingConfig';
import * as Plotly from 'plotly.js-dist';

@Component({
  selector: 'plot-sweep',
  standalone: false,
  templateUrl: './plot-sweep.component.html',
  styleUrl: './plot-sweep.component.scss'
})
export class PlotSweepComponent implements AfterViewInit{
  @Input() sweepChannel: SweepChannel | undefined;
   // @Input() plotData: any;
  // @Input() plotLayout: any;
  @Input() plotDataX: number[] = [];
  @Input() plotConfig: any;

  @Input() isActive: boolean = false;
  @Input() activeStyle: any = {}; // Accept activeStyle as an input
  @Input() color: string = ''; // Accept color as an input

  sweepDivID:string = '';

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

  plotLayout = {
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
    if (this.sweepChannel) {
      this.commonChanAttributes =
        this.sweepChannel.start_stop_channel.common_chan_attributes;

      this.chanName = this.commonChanAttributes.chan_name;
      this.deviceID = this.commonChanAttributes.device_id;
      console.log("device_id, channame", this.deviceID, this.chanName);
      this.sourceFunction = this.commonChanAttributes.source_function;
      this.sourceRange = this.commonChanAttributes.source_range;
      this.measFunction = this.commonChanAttributes.meas_function;
      this.measRange = this.commonChanAttributes.meas_range;

      this.start = this.sweepChannel.start_stop_channel.start;
      this.stop = this.sweepChannel.start_stop_channel.stop;
      this.style = this.sweepChannel.start_stop_channel.style;

      this.sweepDivID = this.commonChanAttributes.device_id;
    }
    console.log(this.plotDataX, this.plotData);
    this.plotData1.x = this.plotDataX;
    const stepSize = (1 - 0) / (10 - 1);
    const sweepValues = Array.from({ length: 10 }, (_, i) => 0 + i * stepSize);
    this.plotData1.y = Array.from({ length: 10 }, () => sweepValues).flat();
    this.plotData1.x = Array.from({ length: 10 }, (_, i) => Array.from({ length: 10 }, (_, j) => i + j / 10)).flat();
    this.plotData1.line.color = this.color;
    this.updatePlotLayout();
  }

  ngAfterViewInit(): void{
    if (this.plotDataX && this.plotConfig) {
      Plotly.newPlot(this.sweepDivID, this.plotData, this.plotLayout, this.plotConfig);
      console.log('sweep data', this.plotData, this.plotLayout, this.plotConfig);
    }
  }

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
