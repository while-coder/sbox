<script setup lang="ts">
import { RefreshCw } from 'lucide-vue-next'
import { useUpdater } from '../shell/useUpdater'
const { updateStatus, updateStatusText, checkForUpdate } = useUpdater()
async function onCheckUpdate() { await checkForUpdate({ silent: false }) }
</script>
<template>
<section class="about-panel" aria-labelledby="update-heading">
          <div class="about-panel-icon update-icon" aria-hidden="true"><RefreshCw :size="19" stroke-width="1.8" /></div>
          <div class="about-panel-copy">
            <h3 id="update-heading">应用更新</h3>
            <p :class="{ 'update-error': updateStatus === 'error' }">{{ updateStatusText }}</p>
          </div>
          <button
            type="button"
            class="btn btn-outline about-action"
            :disabled="updateStatus === 'checking' || updateStatus === 'downloading'"
            @click="onCheckUpdate"
          >{{ updateStatus === 'checking' ? '检查中…' : '检查更新' }}</button>
        </section>
</template>
