import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { IpcData } from './model/ipcData';

@Injectable({
  providedIn: 'root',
})
export class WebSocketService {
  private socket: WebSocket;
  private messageSubject: Subject<string> = new Subject<string>();

  constructor() {
    this.socket = new WebSocket('ws://localhost:8080/ws');
  }

  connect(): void {
    // this.socket = new WebSocket('ws://localhost:8080/ws');

    this.socket.onopen = () => {
      console.log('WebSocket connection established');
      this.sendInitialDataRequest();
    };

    this.socket.onmessage = (event) => {
      console.log('Message received from server:', event.data);
      this.messageSubject.next(event.data);
    };

    this.socket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.socket.onclose = () => {
      console.log('WebSocket connection closed');
    };
  }

  sendInitialDataRequest(): void {
    const ipcData = new IpcData({
      request_type: 'get_data',
      additional_info: '',
      json_value: '{}',
    });
    this.send(JSON.stringify(ipcData));
  }

  send(message: string): void {
    if (this.socket.readyState === WebSocket.OPEN) {
      console.log('sendind message from socket: ', message);
      this.socket.send(message);
    } else {
      console.error(
        'WebSocket is not open. ReadyState:',
        this.socket.readyState
      );
    }
  }

  getMessages(): Observable<string> {
    return this.messageSubject.asObservable();
  }

  close(): void {
    this.socket.close();
  }
}

// export class WebSocketService {
//   private socket: WebSocket | undefined;
//   private subject: Subject<MessageEvent>;

//   constructor() {
//     this.subject = new Subject<MessageEvent>();
//   }

//   public connect(url: string): Observable<MessageEvent> {
//     this.socket = new WebSocket(url);

//     this.socket.onmessage = (event) => {
//       this.subject.next(event);
//     };

//     this.socket.onerror = (event) => {
//       this.subject.error(event);
//     };

//     this.socket.onclose = (event) => {
//       this.subject.complete();
//     };

//     return this.subject.asObservable();
//   }

//   public send(data: any): void {
//     this.socket?.send(JSON.stringify(data));
//   }

//   public close(): void {
//     this.socket?.close();
//   }
// }
