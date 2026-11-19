import { ComponentFixture, TestBed } from '@angular/core/testing';
import { BrowserModule } from '@angular/platform-browser';

import { ChannelDropdownComponent } from './channel-dropdown.component';
import { Option } from '../../../model/chan_data/defaultChannel';

describe('ChannelDropdownComponent', () => {
  let component: ChannelDropdownComponent;
  let fixture: ComponentFixture<ChannelDropdownComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ChannelDropdownComponent]
    })
    .overrideComponent(ChannelDropdownComponent, {
      remove: { imports: [BrowserModule] },
    })
    .compileComponents();

    fixture = TestBed.createComponent(ChannelDropdownComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('renders labels while emitting distinct model-aware option values', async () => {
    const address = 'node[2].slot[1].smu[1]';
    component.options = ['MSMU60-2', 'MSMU200-2'].map((model) => new Option(
      JSON.stringify([address, model]), true, address, model, `${address} (${model})`,
    ));
    component.selected = component.options[0];
    const emitted = spyOn(component.selectedChange, 'emit');
    fixture.detectChanges();

    const select: HTMLSelectElement = fixture.nativeElement.querySelector('select');
    expect(select.options[0].textContent?.trim()).toBe(`${address} (MSMU60-2)`);
    expect(select.options[1].textContent?.trim()).toBe(`${address} (MSMU200-2)`);
    select.value = component.options[1].value;
    select.dispatchEvent(new Event('change'));
    await fixture.whenStable();
    fixture.detectChanges();

    expect(emitted).toHaveBeenCalledWith(component.options[1]);
    expect(component.selected.deviceId).toBe(address);
    expect(component.selected.deviceModel).toBe('MSMU200-2');
  });

  it('updates immediately and accepts the authoritative selection afterward', async () => {
    component.options = [
      new Option('old', false, 'smu[1]', '2450', 'smu[1] (2450) — invalid', 'Unavailable'),
      new Option('new', true, 'smu[1]', '2460', 'smu[1] (2460)'),
    ];
    component.selected = component.options[0];
    const changed = jasmine.createSpy('modelChange');
    component.registerOnChange(changed);
    const emitted = spyOn(component.selectedChange, 'emit');
    fixture.detectChanges();

    const select: HTMLSelectElement = fixture.nativeElement.querySelector('select');
    select.value = 'new';
    select.dispatchEvent(new Event('change'));
    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    expect(emitted).toHaveBeenCalledWith(component.options[1]);
    expect(changed).toHaveBeenCalledWith(component.options[1]);
    expect(select.value).toBe('new');
    expect(component.selected).toBe(component.options[1]);
    component.selected = component.options[0];
    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();
    expect(select.value).toBe('old');
    expect(select.classList).toContain('invalid-select');
  });

  it('keeps the selected option when option objects are refreshed', async () => {
    const oldOptions = [
      new Option('first', true, 'smu[1]', '2450', 'smu[1] (2450)'),
      new Option('second', true, 'smu[2]', '2460', 'smu[2] (2460)'),
    ];
    component.options = oldOptions;
    component.selected = oldOptions[1];
    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    component.options = [
      new Option('first', true, 'smu[1]', '2450', 'smu[1] (2450)'),
      new Option('second', true, 'smu[2]', '2460', 'smu[2] (2460)'),
    ];
    component.selected = component.options[1];
    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    const select: HTMLSelectElement = fixture.nativeElement.querySelector('select');
    expect(select.value).toBe('second');
  });
});
