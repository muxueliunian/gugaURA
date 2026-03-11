export interface ReceiverRuntimeSettings {
  receiverListenAddr: string;
  relayEnabled: boolean;
  relayTargetHost: string;
  fansEnabled: boolean;
  fansOutputDir: string;
}

export interface SaveReceiverRuntimeSettingsInput {
  receiverListenAddr: string;
  relayEnabled: boolean;
  relayTargetHost: string;
  fansEnabled: boolean;
  fansOutputDir: string;
}

export interface ReceiverRuntimeSettingsActionResult {
  settings: ReceiverRuntimeSettings;
  notice: string;
}
