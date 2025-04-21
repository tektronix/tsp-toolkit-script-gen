import { Component, OnInit, AfterViewInit, ViewChild, ElementRef, Input, ViewChildren, QueryList } from '@angular/core';
import {
  ParameterFloat,
  ParameterString,
} from '../../../model/sweep_data/TimingConfig';
import { ChannelRange } from '../../../model/chan_data/channelRange';
import { PlotUtils } from './plot-utils';
import * as Plotly from 'plotly.js-dist';
import { BiasChannel } from '../../../model/chan_data/biasChannel';
import { StepChannel } from '../../../model/chan_data/stepChannel';
import { SweepChannel } from '../../../model/chan_data/sweepChannel';
import { StepGlobalParameters, SweepGlobalParameters } from '../../../model/sweep_data/stepSweepConfig';
import { PlotBiasComponent } from './plot-bias/plot-bias.component';
import { PlotStepComponent } from './plot-step/plot-step.component';
import { PlotSweepComponent } from './plot-sweep/plot-sweep.component';

@Component({
  selector: 'app-plot-container',
  templateUrl: './plot-container.component.html',
  styleUrls: ['./plot-container.component.scss'],
  standalone: false
})
export class PlotContainerComponent implements OnInit, AfterViewInit {
  @ViewChild('plotContainer', { static: false }) plotContainer!: ElementRef;
  @Input() biasChannels: BiasChannel[] = [];
  @Input() stepChannels: StepChannel[] = [];
  @Input() sweepChannels: SweepChannel[] = [];
  @Input() stepGlobalParameters: StepGlobalParameters | undefined;
  @Input() sweepGlobalParameters: SweepGlobalParameters | undefined;

  @Input() activeComponent: 'bias' | 'step' | 'sweep' | null = null; // Accept active component
  @Input() activeIndex: number | null = null; // Accept active index

  @ViewChildren(PlotBiasComponent) plotBiasComponents!: QueryList<PlotBiasComponent>;
  @ViewChildren(PlotStepComponent) plotStepComponents!: QueryList<PlotStepComponent>;
  @ViewChildren(PlotSweepComponent) plotSweepComponents!: QueryList<PlotSweepComponent>;

  plotDataX: number[] = [];
  plotLayout = {
    xaxis: {
      visible: true,
      ticksuffix: ' s',
      rangemode: 'nonnegative',
      separatethousands: false,
      tickfont: { family: 'Times New Roman', color: 'white' },
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
      tickfont: { family: 'Times New Roman', color: 'white' },
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
        font: {color: 'white'}
      },
      tickfont: {color: 'white'},
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
      ticksuffix: ' mV',
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
    paper_bgcolor: 'black',
    plot_bgcolor: 'black',
    hovermode: 'closest',
    dragmode: false,
    autosize: false,
    height: 150,
    margin: {
      l: 40,
      r: 20,
      b: 25,
      t: 20,
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
  plotConfig: { staticPlot: boolean; } | undefined;

  constructor() {}

  ngOnInit(): void {
    this.plotDataX = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    this.plotConfig = { staticPlot: true };
    console.log('Bias Channels:', this.biasChannels);
    console.log('Step Channels:', this.stepChannels);
    console.log('Sweep Channels:', this.sweepChannels);
    console.log('Step Global Parameters:', this.stepGlobalParameters);
    console.log('Sweep Global Parameters:', this.sweepGlobalParameters);
    console.log('plot bias list', this.plotBiasComponents);
  }

  ngAfterViewInit(): void {
    this.plotBiasComponents.changes.subscribe(() => {
      console.log('Updated plotBiasComponents:', this.plotBiasComponents.toArray());
    });
  }

}
