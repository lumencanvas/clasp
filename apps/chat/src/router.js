import { createRouter, createWebHistory } from 'vue-router'
import JoinPage from './pages/JoinPage.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: JoinPage },
    {
      path: '/chat',
      component: () => import('./pages/ChatPage.vue'),
    },
  ],
})
