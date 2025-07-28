export class PlotUtils {
  static computeMaxRange(start: number, stop: number): number {
    const max = Math.max(start, stop);
    const diff = Math.abs(stop - start);
    return max + diff * 0.1;
  }

  static computeMinRange(start: number, stop: number): number {
    const min = Math.min(start, stop);
    const diff = Math.abs(stop - start);
    return min - diff * 0.1;
  }
}
