<script setup lang="ts">
import { ref } from 'vue'
import AboutPanel from './AboutPanel.vue'
import STabs from '@qingfeng346/ui-kit/components/tabs/STabs.vue'
import STabPane from '@qingfeng346/ui-kit/components/tabs/STabPane.vue'
const activeTab = ref('about')
</script>
<template>
  <div class="settings">
    <h2 class="page-title">设置</h2>
    <STabs v-model="activeTab" type="card" class="settings-stabs">
      <STabPane name="about" tab="关于"><AboutPanel /></STabPane>
    </STabs>
  </div>
</template>
<style scoped>
.settings { max-width: 720px; margin: 0 auto; }
.page-title { font-size: 18px; font-weight: 600; margin: 0 0 20px; }
.settings-stabs { margin-bottom: 16px; }
</style>
