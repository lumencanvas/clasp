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
    // Forward ?join param through auth redirect
    const query = to.query.join ? { join: to.query.join } : undefined
    return { path: '/auth', query }
  }

  // Don't block /auth — users may want to switch from guest to registered
})
