import { Component, OnInit, OnDestroy } from '@angular/core';
import { WebSocketService } from './websocket.service';
import { Subscription } from 'rxjs';
import { ServerData } from './model/serverData';
import { SweepConfig } from './model/sweep_data/sweepConfig';
import { SweepModel } from './model/sweep_data/sweepModel';
import { IpcData } from './model/ipcData';
import { MainSweepComponent } from './components/main-sweep/main-sweep.component';
import { EmptyConfigComponent } from './components/empty-config/empty-config.component';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    FormsModule,
    BrowserModule,
    CommonModule,
    MatIconModule,
    MainSweepComponent,
    EmptyConfigComponent,
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent implements OnInit, OnDestroy {
  title = 'script-gen-ui';
  private wsSubscription: Subscription | undefined;
  parsedData: ServerData | undefined;
  sweepModel: SweepModel | undefined;
  sweepConfig: SweepConfig | undefined;
  isMainSweepVisible = false;

  constructor(private webSocketService: WebSocketService) {}

  ngOnInit() {
    this.webSocketService.connect();

    this.wsSubscription = this.webSocketService.getMessages().subscribe(
      (message) => {
        this.processServerData(message);
      },
      (error) => {
        console.error('WebSocket error:', error);
      },
      () => {
        console.log('WebSocket connection closed');
      }
    );
  }

  processServerData(message: string): void {
    try {
      // Parse the message as an IpcData object
      const ipcData: IpcData = JSON.parse(message);

      // Handle based on the request_type
      if (
        ipcData.request_type === 'initial_response' ||
        ipcData.request_type === 'evaluated_response' ||
        ipcData.request_type === 'reset_response'
      ) {
        // Parse the json_value as the SweepModel
        const data = JSON.parse(ipcData.json_value);
        if (data.sweep_model) {
          this.sweepModel = new SweepModel(data.sweep_model);
          this.sweepConfig = this.sweepModel.sweep_config;

          // Update visibility based on the device list
          if (this.sweepConfig.device_list.length > 0) {
            this.isMainSweepVisible = true;
          }
        } else {
          console.error('sweep_model property is missing in the data');
        }
      } else if (ipcData.request_type === 'empty_system_config_error') {
        // handle empty system config error
        console.log('Empty system config error received');
      } else {
        console.warn('Unhandled request_type:', ipcData.request_type);
      }
    } catch (error) {
      console.log('Error parsing server data:', error);
    }
  }

  ngOnDestroy(): void {
    this.wsSubscription?.unsubscribe();
    this.webSocketService.close();
  }
}
