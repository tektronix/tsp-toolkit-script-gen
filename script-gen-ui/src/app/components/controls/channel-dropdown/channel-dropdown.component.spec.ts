import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ChannelDropdownComponent } from './channel-dropdown.component';

describe('ChannelDropdownComponent', () => {
  let component: ChannelDropdownComponent;
  let fixture: ComponentFixture<ChannelDropdownComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ChannelDropdownComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ChannelDropdownComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
