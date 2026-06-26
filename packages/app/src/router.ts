import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import { HomeView } from '@sbox/tools-core'
import { ALL_TOOLS } from './registry'

const toolRoutes: RouteRecordRaw[] = ALL_TOOLS.map(t => ({
  path: `/${t.key}`,
  name: t.key,
  component: t.component,
}))

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView, props: { tools: ALL_TOOLS } },
    { path: '/settings', name: 'settings', component: () => import('./views/SettingsView.vue') },
    { path: '/screenshot-overlay', name: 'screenshot-overlay', component: () => import('./tools/screenshot/ScreenshotOverlay.vue') },
    ...toolRoutes,
  ],
})
