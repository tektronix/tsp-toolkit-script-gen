export class TimingCalculation {
  numMeas = 0;
  lineFreq = 0;
  overhead = 0;
  sourceDelay = 0;
  measDelay = 0;
  measTime = 0; // measurement time

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
  calculateTotalTime(mode: 'Aperture' | 'NPLC', numMeas: number, overhead: number, lineFreq: number, value: number, sourceDelay: number, measDelay: number, stepToSweepDelay: number, sweepPoints: number): number {
    this.overhead = overhead;
    this.lineFreq = lineFreq;
    this.numMeas = numMeas;
    if (mode === 'Aperture') {
      this.measTime = this.numMeas * (value + measDelay) + sourceDelay;
    } else if (mode === 'NPLC') {
      this.measTime = (this.numMeas * ((1 / this.lineFreq) * value) + measDelay) + sourceDelay;
    }
    return stepToSweepDelay + sweepPoints*(this.overhead + this.measTime);
  }
}