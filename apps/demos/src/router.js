import { createRouter, createWebHistory } from 'vue-router'
import HubPage from './pages/HubPage.vue'

const routes = [
  { path: '/', component: HubPage },
  { path: '/social', component: () => import('./pages/SocialPage.vue') },
  { path: '/spaces', component: () => import('./pages/SpacesPage.vue') },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
