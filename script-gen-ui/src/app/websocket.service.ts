import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { IpcData } from './model/ipcData';
import { v4 as uuidv4 } from 'uuid';

@Injectable({
  providedIn: 'root',
})
export class WebSocketService {
  private socket: WebSocket;
  private messageSubject: Subject<string> = new Subject<string>();
  private readonly CHUNK_SIZE = 30 * 1024; // 30 KB per chunk

  constructor() {
    this.socket = new WebSocket('ws://localhost:27950/ws');
  }

  connect(): void {
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

  private chunkString(str: string, size: number): string[] {
    const numChunks = Math.ceil(str.length / size);
    const chunks = new Array(numChunks);
    for (let i = 0, o = 0; i < numChunks; ++i, o += size) {
      chunks[i] = str.substring(o, o + size);
    }
    return chunks;
  }
  send(message: string): void {
    if (this.socket.readyState === WebSocket.OPEN) {
      console.log(`Input size is: ${(message.length / (1024 * 1024)).toFixed(2)} MB)`);
      if (message.length > this.CHUNK_SIZE) {
        const msg_id = uuidv4();
        const chunks = this.chunkString(message, this.CHUNK_SIZE);
        const total_chunks = chunks.length;
        console.log(`Sending large message in ${total_chunks} chunks`);
        for (let i = 0; i < total_chunks; i++) {
          const chunkMsg = {
            type: 'chunk',
            msg_id,
            chunk_index: i,
            total_chunks,
            data: chunks[i],
          };
          this.socket.send(JSON.stringify(chunkMsg));
        }
      }
      else {
        this.socket.send(message);
      }

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