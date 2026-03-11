export interface BootstrapState {
  appVersion: string;
  receiverReady: boolean;
  receiverStatus: string;
  receiverListenAddr: string;
  receiverConfiguredListenAddr: string;
  receiverListenAddrSource: string;
}

export interface DetectedGame {
  path: string;
  version: string;
  versionLabel: string;
}

export interface InspectGameDirResult {
  path: string;
  exists: boolean;
  isValidGameDir: boolean;
  detectedVersion: string;
  detectedVersionLabel: string;
  installStatus: string;
  installStatusLabel: string;
}
