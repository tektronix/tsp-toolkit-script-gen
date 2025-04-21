import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MainSweepComponent } from './main-sweep.component';

describe('MainSweepComponent', () => {
  let component: MainSweepComponent;
  let fixture: ComponentFixture<MainSweepComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [MainSweepComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(MainSweepComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
