import { NgModule } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { BrowserModule } from '@angular/platform-browser';
import { CommonModule } from '@angular/common';
import { AppComponent } from './app.component';
// import { MainSweepComponent } from './components/main-sweep/main-sweep.component';
// import { BiasComponent } from './components/main-sweep/bias/bias.component';
// import { StepComponent } from './components/main-sweep/step/step.component';
// import { SweepComponent } from './components/main-sweep/sweep/sweep.component';
// import { DropdownComponent } from './components/controls/dropdown/dropdown.component';
// import { InputPlainComponent } from './components/controls/input-plain/input-plain.component';
// import { EmptyConfigComponent } from './components/empty-config/empty-config.component';
// import { TimingComponent } from './components/main-sweep/timing/timing.component';
// import { PlotContainerComponent } from './components/main-sweep/plot-container/plot-container.component';
// import { InputToggleComponent } from './components/controls/input-toggle/input-toggle.component';
// import { InputNumericComponent } from './components/controls/input-numeric/input-numeric.component';
import { MatIconModule } from '@angular/material/icon';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
// import { PlotBiasComponent } from './components/main-sweep/plot-container/plot-bias/plot-bias.component';
// import { PlotStepComponent } from './components/main-sweep/plot-container/plot-step/plot-step.component';
// import { PlotSweepComponent } from './components/main-sweep/plot-container/plot-sweep/plot-sweep.component';

@NgModule({
  declarations: [
    // AppComponent,
    // MainSweepComponent,
    // EmptyConfigComponent,
    // TimingComponent,
    // BiasComponent,
    // StepComponent,
    // SweepComponent,
    // DropdownComponent,
    // InputPlainComponent,
    // InputNumericComponent,
    // PlotContainerComponent,
    // InputToggleComponent,
    // PlotBiasComponent,
    // PlotStepComponent,
    // PlotSweepComponent
  ],
  imports: [FormsModule, BrowserModule, CommonModule, MatIconModule],
  providers: [
    provideAnimationsAsync()
  ],
  bootstrap: [AppComponent],
})
export class AppModule {}
