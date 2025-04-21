import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PlotBiasComponent } from './plot-bias.component';

describe('PlotBiasComponent', () => {
  let component: PlotBiasComponent;
  let fixture: ComponentFixture<PlotBiasComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PlotBiasComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PlotBiasComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
