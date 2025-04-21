import { ComponentFixture, TestBed } from '@angular/core/testing';

import { InputPlainComponent } from './input-plain.component';

describe('InputPlainComponent', () => {
  let component: InputPlainComponent;
  let fixture: ComponentFixture<InputPlainComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InputPlainComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(InputPlainComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
