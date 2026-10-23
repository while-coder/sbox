import { native } from '../platform/native'
import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import { ALL_TOOLS } from './registry'

const toolRoutes: RouteRecordRaw[] = ALL_TOOLS.map(t => ({
  path: `/${t.key}`,
  name: t.key,
  component: t.component,
}))

export function createSboxRouter() { return createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView, props: { tools: ALL_TOOLS } },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
    ...(native.kind === 'rust' ? [{ path: '/screenshot-overlay', name: 'screenshot-overlay', component: () => import('../tools/media/screenshot/ScreenshotOverlay.vue') }] : []),
    { path: '/:pathMatch(.*)*', redirect: '/' },
    ...toolRoutes,
  ],
}) }
