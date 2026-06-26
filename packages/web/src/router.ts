import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import { WEB_TOOLS, HomeView } from '@sbox/tools-core'

const toolRoutes: RouteRecordRaw[] = WEB_TOOLS.map(t => ({
  path: `/${t.key}`,
  name: t.key,
  component: t.component,
}))

export default createRouter({
  // hash 模式：GitHub Pages 静态托管下无需 404 回退配置
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView, props: { tools: WEB_TOOLS } },
    ...toolRoutes,
  ],
})
