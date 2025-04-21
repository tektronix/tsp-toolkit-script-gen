import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EmptyConfigComponent } from './empty-config.component';

describe('EmptyConfigComponent', () => {
  let component: EmptyConfigComponent;
  let fixture: ComponentFixture<EmptyConfigComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [EmptyConfigComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(EmptyConfigComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
