import { onMounted, ref } from 'vue'
import { checkJava } from '../tools/genverify/keystore-gen/tauri'

/**
 * 依赖 keytool（JDK）的工具共用：打开页面时检测 keytool 是否存在。
 * javaAvailable 为 null 表示检测中；false 时视图层展示下载引导。
 */
export function useKeytoolCheck() {
  const javaAvailable = ref<boolean | null>(null)
  const javaPath = ref('')
  onMounted(async () => {
    try {
      const r = await checkJava()
      javaAvailable.value = r.available
      javaPath.value = r.path ?? ''
    } catch {
      javaAvailable.value = false
    }
  })
  return { javaAvailable, javaPath }
}
