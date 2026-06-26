<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { toolsByCategory, searchTools, CATEGORIES, type ToolDef } from '../registry'

/** 由宿主传入完整工具列表（web 为 9 个，桌面为 13 个）。 */
const props = defineProps<{ tools: ToolDef[] }>()

const router = useRouter()
const query = ref('')
const searchEl = ref<HTMLInputElement | null>(null)

const groups = computed(() => toolsByCategory(props.tools))
const categoryLabel = (key: string) => CATEGORIES.find(c => c.key === key)?.label ?? ''

const filtered = computed<ToolDef[]>(() => searchTools(props.tools, query.value))
const searching = computed(() => query.value.trim().length > 0)

function open(t: ToolDef) { router.push(`/${t.key}`) }

function onEnter() {
  if (searching.value && filtered.value.length >= 1) open(filtered.value[0])
}

onMounted(() => searchEl.value?.focus())
</script>

<template>
  <div class="home">
    <div class="search-bar">
      <span class="search-icon">🔍</span>
      <input
        ref="searchEl"
        v-model="query"
        class="search-input"
        type="text"
        placeholder="搜索工具…（名称、关键词，如 base64、jwt、密码）"
        @keydown.enter="onEnter"
        @keydown.esc="query = ''"
      />
      <button v-if="query" class="clear-btn" @click="query = ''">✕</button>
    </div>

    <!-- 搜索结果（扁平） -->
    <template v-if="searching">
      <p class="result-meta">{{ filtered.length }} 个结果</p>
      <div v-if="filtered.length" class="grid">
        <article v-for="t in filtered" :key="t.key" class="card" @click="open(t)">
          <div class="card-head">
            <h2>{{ t.label }}</h2>
            <span class="tag">{{ categoryLabel(t.category) }}</span>
          </div>
          <p>{{ t.description }}</p>
        </article>
      </div>
      <p v-else class="empty">没有匹配的工具。换个关键词试试？</p>
    </template>

    <!-- 默认：按分类分组 -->
    <template v-else>
      <section v-for="g in groups" :key="g.key" class="cat">
        <h3 class="cat-title">{{ g.label }}<span class="cat-count">{{ g.tools.length }}</span></h3>
        <div class="grid">
          <article v-for="t in g.tools" :key="t.key" class="card" @click="open(t)">
            <h2>{{ t.label }}</h2>
            <p>{{ t.description }}</p>
          </article>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.home { max-width: 960px; margin: 0 auto; }

/* 搜索栏 */
.search-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 0 12px;
  margin-bottom: 24px;
  transition: border-color 0.15s;
}
.search-bar:focus-within { border-color: var(--primary); }
.search-icon { font-size: 14px; opacity: 0.6; }
.search-input {
  flex: 1 1 auto;
  border: none;
  background: none;
  outline: none;
  color: var(--fg);
  font-size: 14px;
  padding: 11px 0;
}
.search-input::placeholder { color: var(--fg-muted); }
.clear-btn {
  border: none; background: none; cursor: pointer;
  color: var(--fg-muted); font-size: 14px; padding: 4px 6px;
}
.clear-btn:hover { color: var(--fg); }

/* 分类 */
.cat { margin-bottom: 28px; }
.cat-title {
  font-size: 12px; font-weight: 600; color: var(--fg-muted);
  text-transform: uppercase; letter-spacing: 0.5px;
  margin: 0 0 12px; display: flex; align-items: center; gap: 8px;
}
.cat-count {
  font-size: 11px; font-weight: 600; color: var(--fg-muted);
  background: var(--bg); border: 1px solid var(--border);
  border-radius: 10px; padding: 0 7px; line-height: 17px;
}

.result-meta { color: var(--fg-muted); font-size: 13px; margin: 0 0 16px; }

.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; }
.card {
  background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 16px; cursor: pointer; transition: border-color 0.15s, transform 0.15s;
}
.card:hover { border-color: var(--primary); transform: translateY(-1px); }
.card-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 6px; }
.card h2 { font-size: 15px; font-weight: 600; margin: 0 0 6px; }
.card-head h2 { margin: 0; }
.card p { font-size: 13px; color: var(--fg-muted); margin: 0; }
.tag {
  flex: 0 0 auto; font-size: 11px; color: var(--fg-muted);
  background: var(--bg); border: 1px solid var(--border);
  border-radius: 4px; padding: 1px 7px;
}

.empty { color: var(--fg-muted); text-align: center; padding: 48px 0; }
</style>
