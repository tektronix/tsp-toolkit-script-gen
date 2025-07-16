import { IStatusMsg, StatusType } from '../interface';

export class StatusMsg {
  status_type: StatusType;
  message: string;
  time_stamp: string;

  constructor(data: IStatusMsg) {
    this.status_type = data.status_type;
    this.message = data.message;
    this.time_stamp = data.time_stamp;
  }
}
