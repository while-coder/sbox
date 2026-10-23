/// <reference types="vite/client" />
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module '@sbox/ui/app' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent
  export default component
}

declare module '@sbox/ui/about' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent
  export default component
}
