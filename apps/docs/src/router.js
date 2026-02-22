import { createRouter, createWebHistory } from 'vue-router'
import docsManifest from 'virtual:docs-manifest'
import DocsHome from './pages/DocsHome.vue'
import DocPage from './pages/DocPage.vue'

export const router = createRouter({
  history: createWebHistory(),
  scrollBehavior(to, _from, savedPosition) {
    if (to.hash) {
      return { el: to.hash, behavior: 'smooth' }
    }
    if (savedPosition) return savedPosition
    return { top: 0 }
  },
  routes: [
    { path: '/', component: DocsHome },
    {
      path: '/tools/relay-configurator',
      component: () => import('./pages/RelayConfigurator.vue')
    },
    { path: '/:pathMatch(.*)*', component: DocPage }
  ]
})

router.afterEach((to) => {
  const path = to.params.pathMatch
  const docPath = Array.isArray(path) ? path.join('/') : path || 'index'
  const doc = docsManifest.find(d => d.path === docPath)
  document.title = doc ? `${doc.title} - CLASP Docs` : 'CLASP Documentation'
})
