import { Component, OnInit, OnDestroy } from '@angular/core';
import { WebSocketService } from './websocket.service';
import { Subscription } from 'rxjs';
import { ServerData } from './model/serverData';
import { SweepConfig } from './model/sweep_data/sweepConfig';
import { SweepModel } from './model/sweep_data/sweepModel';
import { IpcData } from './model/ipcData';
import { MainSweepComponent } from './components/main-sweep/main-sweep.component';
import { EmptyConfigComponent } from './components/empty-config/empty-config.component';
import { BannerDisplayComponent } from './components/main-sweep/banner-display/banner-display.component';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatIconModule } from '@angular/material/icon';
import { BrowserModule } from '@angular/platform-browser';
import { StatusService } from './services/status.service';
import { StatusMsg } from './model/sweep_data/statusMsg';
import { StatusType } from './model/interface';


declare const acquireVsCodeApi: unknown;
// eslint-disable-next-line @typescript-eslint/no-empty-function
export const vscode = typeof acquireVsCodeApi === 'function' ? acquireVsCodeApi() : { postMessage: () => {} };

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
    BannerDisplayComponent
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

  constructor(private webSocketService: WebSocketService, private statusService: StatusService) {}

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
     this.statusService.show(new StatusMsg({ message: "Loading data...", status_type: StatusType.Info, time_stamp: Date.now().toString() }))
    try {
      // Parse the message as an IpcData object
      const ipcData: IpcData = JSON.parse(message);

      // Handle based on the request_type
      if (
        ipcData.request_type === 'initial_response' ||
        ipcData.request_type === 'evaluated_response' ||
        ipcData.request_type === 'reset_response'
      )
      {
        if (ipcData.request_type === 'initial_response') {
          vscode.postMessage({ command: 'create_new_session' , payload: message});
        }
        else if (ipcData.request_type === 'reset_response') {
          vscode.postMessage({ command: 'get_initial_configuration' });
        }
        else {
        vscode.postMessage({ command: 'update_session' , payload: message});
        }
        // Parse the json_value as the SweepModel
        const data = JSON.parse(ipcData.json_value);
        if (data.sweep_model) {
          this.sweepModel = new SweepModel(data.sweep_model);
          this.sweepConfig = this.sweepModel.sweep_config;
          if (this.sweepConfig.status_msg) {
            this.statusService.showTemporary(
              this.sweepConfig.status_msg,
              this.sweepConfig.status_msg.timeout_ms ?? 5000
            );
          } else {
            this.statusService.clear();
          }

          // Update visibility based on the device list
          if (this.sweepConfig.device_list.length > 0) {
            this.isMainSweepVisible = true;
            
          }
        } else {
          console.error('sweep_model property is missing in the data');
          this.statusService.clear();
        }
      } 
      
      else if (ipcData.request_type === 'open_script') {
        vscode.postMessage({ command: 'open_script' });
        const statusMsg = this.sweepConfig?.status_msg;
        if (statusMsg) {
          this.statusService.showTemporary(statusMsg, statusMsg.timeout_ms ?? 5000);
        } else {
          this.statusService.clear();
        }
      } 
      
      else if (ipcData.request_type === 'save_as_generated_script') {
        vscode.postMessage({ command: 'save_as_generated_script' });
        this.statusService.clear();
      }

      else if (ipcData.request_type === 'empty_system_config_error') {
        // handle empty system config error
        console.log('Empty system config error received');
        this.statusService.clear();
      } else {
        console.warn('Unhandled request_type:', ipcData.request_type);
        this.statusService.clear();
      }
    } catch (error) {
      console.log('Error parsing server data:', error);
      this.statusService.clear();
    }
  }

  ngOnDestroy(): void {
    this.wsSubscription?.unsubscribe();
    this.webSocketService.close();
  }
}
