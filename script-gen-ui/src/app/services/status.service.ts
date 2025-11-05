import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { StatusMsg } from '../model/sweep_data/statusMsg';

@Injectable({ providedIn: 'root' })
export class StatusService {
  private readonly _status = new BehaviorSubject<StatusMsg | undefined>(undefined);
  readonly status$: Observable<StatusMsg | undefined> = this._status.asObservable();

  show(msg: StatusMsg | undefined): void {
    this._status.next(msg);
  }

  showTemporary(msg: StatusMsg, timeoutMs = 5000): void {
    this.show(msg);
    if (timeoutMs > 0) {
      setTimeout(() => {
        // Only clear if the message is still the same instance
        if (this._status.getValue() === msg) {
          this.clear();
        }
      }, timeoutMs);
    }
  }

  clear(): void {
    this._status.next(undefined);
  }
}
