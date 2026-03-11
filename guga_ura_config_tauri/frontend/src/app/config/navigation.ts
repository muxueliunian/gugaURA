export interface AppNavigationItem {
  path: string;
  title: string;
  description: string;
  badge?: string;
}

export const APP_NAVIGATION_ITEMS: AppNavigationItem[] = [
  {
    path: '/overview',
    title: '总览',
    description: '查看应用状态、当前注入上下文与 Debug 开关。',
  },
  {
    path: '/dll-injection',
    title: 'DLL 注入',
    description: '检测游戏目录、管理 DLL 安装状态并保存注入配置。',
  },
  {
    path: '/receiver-config',
    title: '接收&转发配置',
    description: '管理 Receiver 监听、Relay 二次转发与社团Fans设置。',
  },
  {
    path: '/terminal',
    title: '终端',
    description: '查看 Receiver 状态、日志快照与刷新时间。',
  },
  {
    path: '/game-settings',
    title: '游戏设置',
    description: '读取并保存 FPS 与垂直同步设置。',
  },
];
