import * as Plotly from 'plotly.js-dist';

export class PlotUtils {

  static generateBiasPlot(id: string, layout: any, config: any) {
    const plot_data = { x: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], y: [0], mode: 'lines', colorbar: 'green', name: 'Bias SMU(V)' };
    setTimeout(() => Plotly.newPlot(id, [plot_data], layout, config), 0); // Ensure the element is in the DOM
    return plot_data;
  }

  static generateStepPlot(id: string, layout: any, config: any) {
    const plot_data = { x: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], y: [0], mode: 'lines', line: { shape: 'hv' }, colorbar: 'darkblue', name: 'Step SMU(V)' };
    setTimeout(() => Plotly.newPlot(id, [plot_data], layout, config), 0); // Ensure the element is in the DOM
    return plot_data;
  }

  static generateSweepPlot(id: string, layout: any, config: any) {
    const plot_data = { x: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], y: [0], type: 'scatter', mode: 'lines', line: { shape: 'hv' }, colorbar: 'darkblue', name: 'Sweep SMU(V)' };
    setTimeout(() => Plotly.newPlot(id, [plot_data], layout, config), 0); // Ensure the element is in the DOM
    return plot_data;
  }

  static updateBiasPlot(id: string, value: number, plot_data: any, layout: any): void {
    plot_data.y = Array(11).fill(value);
    Plotly.update(id, [plot_data], layout);
  }

  static updateStepPlot(id: string, num_steps: number, start: number, stop: number, plot_data: any, layout: any): void {
    const stepSize = (stop - start) / (num_steps - 1);
    plot_data.x = Array.from({ length: num_steps }, (_, i) => i).concat(num_steps);
    plot_data.y = Array.from({ length: num_steps }, (_, i) => start + i * stepSize).concat(stop);
    Plotly.update(id, [plot_data], layout);
  }

  static updateSweepPlot(id: string, num_steps: number, num_points: number, sweep_start: number, sweep_stop: number, plot_data: any, layout: any): void {
    const stepSize = (sweep_stop - sweep_start) / (num_points - 1);
    const sweepValues = Array.from({ length: num_points }, (_, i) => sweep_start + i * stepSize);
    plot_data.y = Array.from({ length: num_steps }, () => sweepValues).flat();
    plot_data.x = Array.from({ length: num_steps }, (_, i) => Array.from({ length: num_points }, (_, j) => i + j / num_points)).flat();
    Plotly.update(id, [plot_data], layout);
  }

  static rangeMin(min: number): number {
    return min * 1.25;
  }
}
