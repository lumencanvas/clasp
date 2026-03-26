import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/flow',
  },
  {
    path: '/links',
    name: 'links',
    component: () => import('./components/panels/DirectLinksPanel.vue'),
  },
  {
    path: '/routes',
    name: 'routes',
    component: () => import('./components/panels/SignalRoutesPanel.vue'),
  },
  {
    path: '/flow',
    name: 'flow',
    component: () => import('./components/panels/FlowPanel.vue'),
  },
  {
    path: '/monitor',
    name: 'monitor',
    component: () => import('./components/panels/MonitorPanel.vue'),
  },
  {
    path: '/test',
    name: 'test',
    component: () => import('./components/panels/TestPanel.vue'),
  },
  {
    path: '/logs',
    name: 'logs',
    component: () => import('./components/panels/LogsPanel.vue'),
  },
  {
    path: '/rules',
    name: 'rules',
    component: () => import('./components/panels/RulesPanel.vue'),
  },
  {
    path: '/security',
    name: 'security',
    component: () => import('./components/panels/SecurityPanel.vue'),
  },
  {
    path: '/defra',
    name: 'defra',
    component: () => import('./components/panels/DefraPanel.vue'),
  },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})
