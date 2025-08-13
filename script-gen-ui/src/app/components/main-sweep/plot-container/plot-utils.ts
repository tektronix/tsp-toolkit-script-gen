import numeric from 'numeric';

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

  static downSampling(
    data: number[],
    targetLength: number
  ): { x: number[]; y: number[] } {
    if (targetLength >= data.length) {
      return {
        x: Array.from({ length: data.length }, (_, i) => i),
        y: data.slice(),
      };
    }
    const ratio = data.length / targetLength;
    const x: number[] = [];
    const y: number[] = [];
    for (let i = 0; i < targetLength; i++) {
      const idx = Math.floor(i * ratio);
      x.push(idx);
      y.push(data[idx]);
    }
    return { x, y };
  }

  static linearInterpolation(
    data: number[],
    targetLength: number
  ): { x: number[]; y: number[] } {
    const n = data.length;
    if (targetLength >= n) {
      return {
        x: Array.from({ length: n }, (_, i) => i),
        y: data.slice(),
      };
    }
    const x: number[] = [];
    const y: number[] = [];
    const scale = (n - 1) / (targetLength - 1);
    for (let i = 0; i < targetLength; i++) {
      const pos = i * scale;
      const idx = Math.floor(pos);
      const frac = pos - idx;
      if (idx + 1 < n) {
        y.push(data[idx] * (1 - frac) + data[idx + 1] * frac);
      } else {
        y.push(data[n - 1]);
      }
      x.push(pos);
    }
    return { x, y };
  }

  static cubicSplineInterpolation(
    data: number[],
    targetLength: number
  ): { x: number[]; y: number[] } {
    const n = data.length;
    if (targetLength >= n) {
      return {
        x: Array.from({ length: n }, (_, i) => i),
        y: data.slice(),
      };
    }
    const x: number[] = [];
    const y: number[] = [];
    const originalX = Array.from({ length: n }, (_, i) => i);
    const spline = numeric.spline(originalX, data);
    const scale = (n - 1) / (targetLength - 1);
    for (let i = 0; i < targetLength; i++) {
      const pos = i * scale;
      x.push(pos);
      const val = spline.at(pos);
      y.push(
        typeof val === 'number'
          ? val
          : Array.isArray(val) && val.length > 0
          ? val[0]
          : NaN
      );
    }
    return { x, y };
  }

  static minMaxInterpolation(
    data: number[],
    targetLength: number
  ): { x: number[]; y: number[] } {
    const n = data.length;
    if (targetLength >= n) {
      return {
        x: Array.from({ length: n }, (_, i) => i),
        y: data.slice(),
      };
    }
    const x: number[] = [];
    const y: number[] = [];
    const binSize = n / targetLength;
    for (let i = 0; i < targetLength; i++) {
      const start = Math.floor(i * binSize);
      const end = Math.min(Math.floor((i + 1) * binSize), n);
      const bin = data.slice(start, end);
      if (bin.length > 0) {
        y.push(Math.min(...bin));
        y.push(Math.max(...bin));
        x.push(i);
        x.push(i);
      }
    }
    return { x, y };
  }
}
