import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TimingComponent } from './timing.component';

describe('TimingComponent', () => {
  let component: TimingComponent;
  let fixture: ComponentFixture<TimingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TimingComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(TimingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
