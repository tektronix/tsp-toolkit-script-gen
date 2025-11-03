import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { IpcData } from './model/ipcData';
import { v4 as uuidv4 } from 'uuid';

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  timeout: ReturnType<typeof setTimeout>;
  timestamp: number;
}

@Injectable({
  providedIn: 'root',
})
export class WebSocketService {
  private socket: WebSocket;
  private messageSubject: Subject<string> = new Subject<string>();
  private readonly CHUNK_SIZE = 30 * 1024; // 30 KB per chunk
  private readonly REQUEST_TIMEOUT = 30000; // 30 seconds timeout
  private pendingRequests: Map<string, PendingRequest> = new Map<string, PendingRequest>();

  constructor() {
    this.socket = new WebSocket('ws://localhost:27950/ws');
  }

  connect(): void {
    this.socket.onopen = () => {
      console.log('WebSocket connection established');
      this.sendInitialDataRequest();
    };

    this.socket.onmessage = (event) => {
      this.handleResponse(event.data);
      this.messageSubject.next(event.data);
    };

    this.socket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.socket.onclose = () => {
      console.log('WebSocket connection closed');
      this.rejectAllPendingRequests('WebSocket connection closed');
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

  private handleResponse(data: string): void {
    try {
      const response = JSON.parse(data);
      if (response.request_id && this.pendingRequests.has(response.request_id)) {
        const pendingRequest = this.pendingRequests.get(response.request_id);
        if (pendingRequest) {
          clearTimeout(pendingRequest.timeout);
          this.pendingRequests.delete(response.request_id);
          pendingRequest.resolve(response);
        }
      }
    } catch (error) {
      // Not a JSON response or doesn't have request_id, handle normally
      console.log('Response is not JSON or missing request_id:', error);
    }
  }

  private rejectAllPendingRequests(reason: string): void {
    this.pendingRequests.forEach((pendingRequest) => {
      clearTimeout(pendingRequest.timeout);
      pendingRequest.reject(new Error(reason));
    });
    this.pendingRequests.clear();
  }

  sendWithResponse(message: string, timeoutMs: number): Promise<unknown> {
    return new Promise((resolve, reject) => {
      if (this.socket.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket is not open'));
        return;
      }

      const requestId = uuidv4();
      const timeout = timeoutMs;

      // Add request_id to the message
      let messageWithId: Record<string, unknown>;
      try {
        messageWithId = JSON.parse(message);
        messageWithId['request_id'] = requestId;
      } catch {
        // If message is not JSON, create a wrapper
        messageWithId = {
          request_id: requestId,
          data: message
        };
      }

      const timeoutHandle = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(new Error(`Request timeout after ${timeout}ms`));
      }, timeout);

      this.pendingRequests.set(requestId, {
        resolve,
        reject,
        timeout: timeoutHandle,
        timestamp: Date.now()
      });

      this.send(JSON.stringify(messageWithId));
    });
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

  // Get pending requests count for monitoring
  getPendingRequestsCount(): number {
    return this.pendingRequests.size;
  }

  // Cancel a specific request
  cancelRequest(requestId: string): boolean {
    const pendingRequest = this.pendingRequests.get(requestId);
    if (pendingRequest) {
      clearTimeout(pendingRequest.timeout);
      this.pendingRequests.delete(requestId);
      pendingRequest.reject(new Error('Request cancelled'));
      return true;
    }
    return false;
  }

  // Cancel all pending requests
  cancelAllRequests(): number {
    const count = this.pendingRequests.size;
    this.rejectAllPendingRequests('All requests cancelled');
    return count;
  }

  close(): void {
    this.rejectAllPendingRequests('WebSocket service closed');
    this.socket.close();
  }
}