import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PlotStepComponent } from './plot-step.component';

describe('PlotStepComponent', () => {
  let component: PlotStepComponent;
  let fixture: ComponentFixture<PlotStepComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PlotStepComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PlotStepComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
