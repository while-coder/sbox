import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from './views/HomeView.vue'
import { TOOLS } from './tools/registry'

const toolRoutes: RouteRecordRaw[] = TOOLS.map(t => ({
  path: `/${t.key}`,
  name: t.key,
  component: t.component,
}))

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    ...toolRoutes,
  ],
})
