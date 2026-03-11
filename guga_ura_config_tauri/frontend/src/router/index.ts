import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/overview',
  },
  {
    path: '/overview',
    name: 'overview',
    component: () => import('@/features/overview/pages/OverviewPage.vue'),
    meta: {
      title: '总览',
      description: '查看应用状态、当前注入上下文与 Debug 开关。',
    },
  },
  {
    path: '/dll-injection',
    name: 'dll-injection',
    component: () => import('@/features/dll-injection/pages/DllInjectionPage.vue'),
    meta: {
      title: 'DLL 注入',
      description: '检测游戏目录、管理 DLL 安装状态并保存注入配置。',
    },
  },
  {
    path: '/receiver-config',
    name: 'receiver-config',
    component: () => import('@/features/receiver-config/pages/ReceiverConfigPage.vue'),
    meta: {
      title: '接收&转发配置',
      description: '管理 Receiver 监听、Relay 二次转发与 Fans 聚合设置。',
    },
  },
  {
    path: '/terminal',
    name: 'terminal',
    component: () => import('@/features/terminal/pages/TerminalPage.vue'),
    meta: {
      title: '终端',
      description: '查看 Receiver 状态、日志快照与刷新时间。',
    },
  },
  {
    path: '/game-settings',
    name: 'game-settings',
    component: () => import('@/features/game-settings/pages/GameSettingsPage.vue'),
    meta: {
      title: '游戏设置',
      description: '读取并保存 FPS 与垂直同步设置。',
    },
  },
  {
    path: '/deploy',
    redirect: '/dll-injection',
  },
  {
    path: '/forward-performance',
    redirect: '/dll-injection',
  },
  {
    path: '/debug',
    redirect: '/overview',
  },
  {
    path: '/console',
    redirect: '/terminal',
  },
  {
    path: '/bootstrap',
    redirect: '/overview',
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/overview',
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
