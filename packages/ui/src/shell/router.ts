import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import { ALL_TOOLS, type ToolDef } from './registry'

export interface RouterOptions {
  tools?: ToolDef[]
  routes?: RouteRecordRaw[]
}
export function createSboxRouter(options: RouterOptions = {}) {
  const tools = options.tools ?? ALL_TOOLS
  const extraRoutes = options.routes ?? []
  const baseRoutes: RouteRecordRaw[] = [
    { path: '/', name: 'home', component: HomeView, props: { tools } },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  ]
  return createRouter({
    history: createWebHashHistory(),
    routes: [
      ...baseRoutes.filter(route => !extraRoutes.some(extra => extra.path === route.path)),
      ...tools.map(tool => ({ path: `/${tool.key}`, name: tool.key, component: tool.component })),
      ...extraRoutes,
      { path: '/:pathMatch(.*)*', redirect: '/' },
    ],
  })
}
