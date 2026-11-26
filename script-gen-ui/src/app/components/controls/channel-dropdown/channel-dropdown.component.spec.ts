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

  const address = 'node[2].slot[1].smu[1]';

  function setOptions(models: string[], selectedIndex = 0): void {
    component.options = models.map((model) => new Option(
      JSON.stringify([address, model]), true, address, model, `${address} (${model})`,
    ));
    component.selected = component.options[selectedIndex];
    fixture.detectChanges();
  }

  const trigger = (): HTMLButtonElement => fixture.nativeElement.querySelector('.select-trigger');
  const rows = (): HTMLElement[] => Array.from(fixture.nativeElement.querySelectorAll('.option'));

  it('shows only the id when closed, with the full text in a tooltip', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    expect(trigger().querySelector('.option-id')?.textContent).toBe(address);
    expect(trigger().querySelector('.option-detail')?.textContent).toBe('(MSMU60-2)');
    expect(trigger().getAttribute('title')).toBe(`${address}\u00A0\u00A0(MSMU60-2)`);
    expect(rows().length).toBe(0);
  });

  it('removes the tooltip while the list is open and lists id and model', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    trigger().click();
    fixture.detectChanges();

    expect(trigger().hasAttribute('title')).toBeFalse();
    expect(rows().map((row) => row.querySelector('.option-id')?.textContent)).toEqual([address, address]);
    expect(rows()[1].querySelector('.option-detail')?.textContent).toBe('(MSMU200-2)');

    trigger().click();
    fixture.detectChanges();
    expect(rows().length).toBe(0);
    expect(trigger().hasAttribute('title')).toBeTrue();
  });

  it('emits model-aware option values when an option is clicked', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    const emitted = spyOn(component.selectedChange, 'emit');
    const changed = jasmine.createSpy('modelChange');
    component.registerOnChange(changed);
    trigger().click();
    fixture.detectChanges();
    rows()[1].click();
    fixture.detectChanges();

    expect(emitted).toHaveBeenCalledWith(component.options[1]);
    expect(changed).toHaveBeenCalledWith(component.options[1]);
    expect(component.selected?.deviceModel).toBe('MSMU200-2');
    expect(rows().length).toBe(0);
  });

  it('does not emit when the current option is chosen again', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    const emitted = spyOn(component.selectedChange, 'emit');
    trigger().click();
    fixture.detectChanges();
    rows()[0].click();
    expect(emitted).not.toHaveBeenCalled();
  });

  it('marks invalid selections and options and keeps the full invalid text in the list', () => {
    component.options = [
      new Option('old', false, 'smu[1]', '2450', 'smu[1] (2450) — invalid', 'Unavailable'),
      new Option('new', true, 'smu[1]', '2460', 'smu[1] (2460)'),
    ];
    component.selected = component.options[0];
    fixture.detectChanges();

    expect(trigger().classList).toContain('invalid-select');
    expect(trigger().querySelector('.option-id')?.textContent).toBe('smu[1]');
    expect(trigger().querySelector('.option-detail')?.textContent).toBe('(2450) — invalid');
    trigger().click();
    fixture.detectChanges();
    expect(rows()[0].classList).toContain('invalid-option');
    expect(rows()[0].querySelector('.option-detail')?.textContent).toBe('(2450) — invalid');
    expect(rows()[1].classList).not.toContain('invalid-option');
  });

  it('caps the open list at five rows and scrolls beyond that', () => {
    setOptions(['A', 'B', 'C', 'D', 'E', 'F', 'G']);
    trigger().click();
    fixture.detectChanges();
    const list: HTMLElement = fixture.nativeElement.querySelector('.options-list');
    const rowHeight = rows()[0].getBoundingClientRect().height;

    expect(rows().length).toBe(7);
    expect(getComputedStyle(list).overflowY).toBe('auto');
    expect(list.getBoundingClientRect().height).toBeLessThanOrEqual(rowHeight * 5 + 2);
    expect(list.scrollHeight).toBeGreaterThan(list.clientHeight);
  });

  it('does not scroll when five or fewer rows exist', () => {
    setOptions(['A', 'B', 'C', 'D', 'E']);
    trigger().click();
    fixture.detectChanges();
    const list: HTMLElement = fixture.nativeElement.querySelector('.options-list');
    expect(list.scrollHeight).toBeLessThanOrEqual(list.clientHeight);
  });

  it('supports keyboard selection and Escape', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    const emitted = spyOn(component.selectedChange, 'emit');
    const key = (k: string) => {
      trigger().dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true }));
      fixture.detectChanges();
    };
    key('ArrowDown');
    expect(rows().length).toBe(2);
    key('Escape');
    expect(rows().length).toBe(0);
    key('ArrowDown');
    key('ArrowDown');
    key('Enter');
    expect(emitted).toHaveBeenCalledWith(component.options[1]);
    expect(rows().length).toBe(0);
  });

  it('closes when clicking outside', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    trigger().click();
    fixture.detectChanges();
    document.body.click();
    fixture.detectChanges();
    expect(rows().length).toBe(0);
  });

  it('closes when clicking the label above the control', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    trigger().click();
    fixture.detectChanges();
    expect(rows().length).toBe(2);
    fixture.nativeElement.querySelector('label').click();
    fixture.detectChanges();
    expect(rows().length).toBe(0);
  });

  it('accepts the authoritative selection afterward', () => {
    setOptions(['MSMU60-2', 'MSMU200-2']);
    component.selected = component.options[1];
    fixture.detectChanges();
    expect(trigger().getAttribute('title')).toContain('(MSMU200-2)');
  });
});

