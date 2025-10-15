
import { Injectable } from '@angular/core';
import { Observable, Subject, BehaviorSubject } from 'rxjs';
import { IpcData } from './model/ipcData';
import { v4 as uuidv4 } from 'uuid'; // npm install uuid

export interface ProcessingStatus {
  type: 'processing_status';
  status: string;
  details?: string;
  timestamp: number;
}

export interface ServerResponse {
  error?: string;
  message?: string;
  status?: string;
  details?: string;
  type?: string;
  request_type?: string;
  additional_info?: string;
  json_value?: string;
  message_size_bytes?: number;
  max_size_mb?: number;
  received_size_mb?: number;
}

@Injectable({
  providedIn: 'root',
})
export class WebSocketService {
  private socket: WebSocket;
  private messageSubject: Subject<string> = new Subject<string>();
  private statusSubject: BehaviorSubject<ProcessingStatus | null> = new BehaviorSubject<ProcessingStatus | null>(null);
  private errorSubject: Subject<ServerResponse> = new Subject<ServerResponse>();
  private isProcessing = false;
  private requestQueue: string[] = [];

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
      this.handleMessage(event.data);
    };

    this.socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.isProcessing = false;
    };

    this.socket.onclose = () => {
      console.log('WebSocket connection closed');
      this.isProcessing = false;
      this.processQueue(); // Process any queued requests on reconnect
    };
  }

  private handleMessage(data: string): void {
    try {
      const response: ServerResponse = JSON.parse(data);
      
      // Handle different message types from the backend
      if (response.type === 'processing_status') {
        const statusData = response as ProcessingStatus;
        console.log(`Processing Status: ${statusData.status} - ${statusData.details || ''}`);
        this.statusSubject.next(statusData);
        
        // Update processing state based on status
        if (statusData.status === 'complete' || statusData.status === 'error') {
          this.isProcessing = false;
          this.processQueue(); // Process next queued request
        }
        return;
      }
      
      // Handle error responses
      if (response.error) {
        console.error('Server Error:', response.error, response.details || '');
        this.errorSubject.next(response);
        
        if (response.status === 'busy') {
          console.log('Server busy, will retry request...');
          // Don't reset processing flag for busy status, let it retry
          return;
        }
        
        this.isProcessing = false;
        this.processQueue();
        return;
      }
      
      // Handle normal responses
      this.isProcessing = false;
      this.messageSubject.next(data);
      this.processQueue(); // Process next queued request
      
    } catch (error) {
      console.error('Error parsing server response:', error);
      this.isProcessing = false;
      this.processQueue();
    }
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
      chunks[i] = str.substring(o, size);
    }
    return chunks;
  }

  send(message: string): void {
    if (this.isProcessing) {
      console.log('Server is processing, queuing request...');
      this.requestQueue.push(message);
      return;
    }

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
        this.isProcessing = true;
      } else {
        console.log('Sending message to server:', message.substring(0, 100) + (message.length > 100 ? '...' : ''));
        this.isProcessing = true;
        try {
          this.socket.send(message);
        } catch (error) {
          console.error('Failed to send WebSocket message:', error);
        }
      }
    } else {
      console.error('WebSocket is not open. ReadyState:', this.socket.readyState);
      this.requestQueue.push(message);
    }
  }

  private processQueue(): void {
    if (this.requestQueue.length > 0 && !this.isProcessing && this.socket.readyState === WebSocket.OPEN) {
      const nextMessage = this.requestQueue.shift();
      if (nextMessage) {
        console.log('Processing queued request...');
        this.send(nextMessage);
      }
    }
  }

  // Force send a message (bypassing queue) - use carefully
  public sendImmediate(message: string): void {
    if (this.socket.readyState === WebSocket.OPEN) {
      console.log('Force sending message to server');
      this.socket.send(message);
    } else {
      console.error('WebSocket is not open for immediate send. ReadyState:', this.socket.readyState);
    }
  }

  // Clear the request queue
  public clearQueue(): void {
    this.requestQueue = [];
    this.isProcessing = false;
  }

  getMessages(): Observable<string> {
    return this.messageSubject.asObservable();
  }

  // Get processing status updates
  getProcessingStatus(): Observable<ProcessingStatus | null> {
    return this.statusSubject.asObservable();
  }

  // Get error messages
  getErrors(): Observable<ServerResponse> {
    return this.errorSubject.asObservable();
  }

  // Get current processing state
  getProcessingState(): boolean {
    return this.isProcessing;
  }

  // Get queue length
  getQueueLength(): number {
    return this.requestQueue.length;
  }

  close(): void {
    this.clearQueue();
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
