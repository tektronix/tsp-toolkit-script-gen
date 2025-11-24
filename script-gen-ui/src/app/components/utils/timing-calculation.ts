export class TimingCalculation {
  numMeas: number = 0;
  lineFreq: number = 0;
  overhead: number = 0;
  sourceDelay: number = 0;
  measDelay: number = 0;
  measTime: number = 0; // measurement time

  constructor(params: {
    numMeas?: number,
    lineFreq?: number,
    overhead?: number,
    sourceDelay?: number,
    measDelay?: number
  }) {
    if (params.numMeas !== undefined) this.numMeas = params.numMeas;
    if (params.lineFreq !== undefined) this.lineFreq = params.lineFreq;
    if (params.overhead !== undefined) this.overhead = params.overhead;
    if (params.sourceDelay !== undefined) this.sourceDelay = params.sourceDelay;
    if (params.measDelay !== undefined) this.measDelay = params.measDelay;
  }

  /**
   * Calculates total time 
   *
   * @param mode 'aperture' or 'nplc'
   * @param value aperture (number) or nplc (number)
   * @param overhead overhead time (s)
   * @param lineFreq line frequency (Hz)
   * @param sourceDelay source delay (s)
   * @param measDelay measure delay (s)
   * @param stepToSweepDelay step to sweep delay (s)
   */
  calculateTotalTime(mode: 'aperture' | 'nplc', overhead: number, lineFreq: number, value: number, sourceDelay: number, measDelay: number, stepToSweepDelay: number): number {
    this.overhead = overhead;
    this.lineFreq = lineFreq;
    if (mode === 'aperture') {
      this.measTime = this.numMeas * (value + measDelay) + sourceDelay;
    } else if (mode === 'nplc') {
      this.measTime = (this.numMeas * ((1 / this.lineFreq) * value) + measDelay) + sourceDelay;
    }
    return stepToSweepDelay + this.overhead + this.measTime;
  }
}