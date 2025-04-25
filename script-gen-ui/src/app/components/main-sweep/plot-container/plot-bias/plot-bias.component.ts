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

  @Input() isActive: boolean = false;
  @Input() activeStyle: any = {}; // Accept activeStyle as an input
  @Input() color: string = ''; // Accept color as an input
  private mutationObserver: MutationObserver | undefined;

  defaultStyle = {
    backgroundColor: 'transparent',
    color: 'white',
    fontWeight: 'normal',
  };

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
      linewidth: 1
     },
    yaxis: {
      visible: true,
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
      tickfont: {family: 'Times New Roman', color: 'white', size: 9},
      anchor: 'x',
      overlaying: 'y',
      side: 'left',
      position: -3,
      showticklabels: true,
      visible: true,
      ticksuffix: ' V',
      range: [0, 2],
      dtick: 2,
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
    },
  };

  plotData2 = {  x: [],
    y: [0, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 2],
    yaxis: 'y2',
    xaxis: 'x2'
  };
  private plotData = [this.plotData1, this.plotData2];

  chanName = '';
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
    this.updatePlotLayout();
    this.plotData1.line.color = this.color;

    this.initializePlot();
    this.observeThemeChanges();

    // Plotly.newPlot(this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
    console.log('bias data', this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
  }

  ngAfterViewInit(): void{
    if (this.plotDataX && this.plotConfig) {
      Plotly.newPlot(this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
      console.log('bias data', this.plotData, this.plotLayout, this.plotConfig);
    }
    this.renderPlot();
  }

  ngOnDestroy(): void {
    // Disconnect the MutationObserver when the component is destroyed
    if (this.mutationObserver) {
      this.mutationObserver.disconnect();
    }
  }

  private updatePlotLayout(): void {
    if (this.sourceFunction) {
      this.plotLayout.yaxis2.ticksuffix = this.sourceFunction.value === 'Voltage' ? ' V' : ' A';
    }

    if (this.sourceRange) {
      const maxRange = parseFloat(this.sourceRange.value);
      this.plotLayout.yaxis.range = [0, maxRange];
      this.plotLayout.yaxis2.range = [0, maxRange];
      this.plotLayout.yaxis2.dtick = maxRange;

      const dtick = maxRange / 4;
      this.plotLayout.yaxis.dtick = dtick;
    }
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

  private initializePlot(): void {
    if (this.biasChannel) {
      this.plotDivID = `plotDiv${this.biasChannel.common_chan_attributes.chan_name}`;
    }

    const backgroundColor = this.getCssVariableValue('--vscode-editor-background');
    const backgroundColorHex = backgroundColor.startsWith('rgb')
      ? this.rgbToHex(backgroundColor)
      : backgroundColor;

    this.plotLayout.paper_bgcolor = backgroundColorHex;
    this.plotLayout.plot_bgcolor = backgroundColorHex;

    console.log('Initial background color:', backgroundColorHex);
  }

  private renderPlot(): void {
    if (this.plotDataX && this.plotConfig) {
      Plotly.newPlot(this.plotDivID, this.plotData, this.plotLayout, this.plotConfig);
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

      console.log('Theme changed, new background color:', backgroundColorHex);

      // Re-render the plot with the updated background color
      this.renderPlot();
    });

    this.mutationObserver.observe(root, {
      attributes: true,
      attributeFilter: ['style'], // Listen for changes to the "style" attribute
    });
  }
}
