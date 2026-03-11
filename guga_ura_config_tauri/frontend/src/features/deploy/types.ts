export interface DeployChecklistItem {
  key: string;
  title: string;
  description: string;
  status: 'planned' | 'ready';
}
