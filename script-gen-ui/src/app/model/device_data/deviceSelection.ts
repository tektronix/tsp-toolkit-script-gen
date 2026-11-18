import { CommonChanAttributes, Option } from '../chan_data/defaultChannel';
import { Device } from './device';

export interface DeviceIdentity {
  deviceId: string;
  deviceModel: string;
}

export interface DeviceIdentityChange {
  oldChanId: string;
  newChanId: string;
  oldModel: string;
  newModel: string;
}

export function matchesDevice(
  attributes: Pick<CommonChanAttributes, 'device_id' | 'device_model'>,
  identity: DeviceIdentity,
): boolean {
  return attributes.device_id === identity.deviceId &&
    attributes.device_model === identity.deviceModel;
}

export function resolveChannelDevice(
  attributes: Pick<CommonChanAttributes, 'device_id' | 'device_model'>,
  devices: Device[],
): { device?: Device; error: string } {
  if (!attributes.device_model) {
    return {
      error: `Device ${attributes.device_id}: model is missing. Select a model explicitly.`,
    };
  }
  const candidates = devices.filter((device) =>
    device._id === attributes.device_id && device.model === attributes.device_model,
  );
  if (candidates.length > 1) {
    return { error: `Ambiguous device ${attributes.device_id} (${attributes.device_model}). Select a unique device.` };
  }
  if (candidates.length === 1 && candidates[0].is_valid) {
    return { device: candidates[0], error: '' };
  }
  return { error: `Device ${attributes.device_id} (${attributes.device_model || 'model missing'}) is unavailable. Select a replacement.` };
}

export function channelDeviceOptions(
  attributes: CommonChanAttributes,
  devices: Device[],
): { options: Option[]; selected: Option } {
  const { device: current, error } = resolveChannelDevice(attributes, devices);
  const address = attributes.device_id;
  const model = attributes.device_model;
  const options = devices
    .filter((device) => device.is_valid && (!device.in_use || device === current) &&
      devices.filter((candidate) => candidate._id === device._id && candidate.model === device.model).length === 1)
    .map((device) => new Option(
      JSON.stringify([device._id, device.model]),
      true,
      device._id,
      device.model,
      `${device._id} (${device.model})`,
    ));
  let selected = current
    ? options.find((option) => option.deviceId === address && option.deviceModel === model)
    : undefined;
  if (!selected) {
    selected = new Option(
      JSON.stringify([address, model, 'invalid']),
      false,
      address,
      model,
      `${address} (${model || 'model missing'}) — invalid`,
      error,
    );
    options.unshift(selected);
  }
  return { options, selected };
}
