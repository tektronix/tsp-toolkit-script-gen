import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PlotSweepComponent } from './plot-sweep.component';

describe('PlotSweepComponent', () => {
  let component: PlotSweepComponent;
  let fixture: ComponentFixture<PlotSweepComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PlotSweepComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PlotSweepComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
