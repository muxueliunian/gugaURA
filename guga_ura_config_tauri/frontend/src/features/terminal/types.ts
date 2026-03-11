export interface TerminalSnapshot {
  receiverReady: boolean;
  receiverStatus: string;
  receiverListenAddr: string;
  receiverConfiguredListenAddr: string;
  receiverListenAddrSource: string;
  logs: string[];
}
