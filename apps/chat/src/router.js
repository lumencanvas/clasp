import { createRouter, createWebHistory } from 'vue-router'
import JoinPage from './pages/JoinPage.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: JoinPage },
    {
      path: '/auth',
      component: () => import('./pages/AuthPage.vue'),
    },
    {
      path: '/chat',
      component: () => import('./pages/ChatPage.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach((to) => {
  const hasToken = !!localStorage.getItem('clasp-chat-token')

  if (to.meta.requiresAuth && !hasToken) {
    return '/auth'
  }

  // Don't block /auth — users may want to switch from guest to registered
})
