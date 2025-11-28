<script setup lang="ts">
import axios from 'axios'
import {onMounted, ref, computed, watch} from 'vue'
import type {Gist, Bookmark} from './types'
import {useLocalCache} from './composables/useLocalCache';

const gists = ref<Gist[]>([])

const {save_bookmarks, load_bookmarks} = useLocalCache();

onMounted(async () => {
  const cached_bookmarks = load_bookmarks();

  if (cached_bookmarks) {
    console.log('Loaded from cache.');
    bookmarks.value = JSON.parse(cached_bookmarks);
  } else {
    console.log('fetch from server');
  }

  try {
    const {data} = await axios.get<{ gists: Gist[] }>('/api/gists.json')
    // 受け取る配列の要素が string や別キーの場合に備えて正規化

    gists.value = data.gists || [];

    if (gists.value.length > 0) {
      gist_url.value = gists.value[0]!.url;
      update_cache(false);
    } else {
      alert('No gists found.');
    }
  } catch (err) {
    console.error('Failed to fetch gists:', err)
  }

})

const bookmarks = ref<Bookmark[]>([])

const update_cache = (force: boolean) => {
  axios.get('/api/bookmarks.json', {
    params: {
      force: force ? 'yes' : '',
      url: gist_url.value
    }
  }).then(({data}) => {
    bookmarks.value = data.items || [];
    save_bookmarks(JSON.stringify(bookmarks.value));
  });
}

const gist_url = ref('')

// ギア/タブ/UI 状態
const settingsOpen = ref(false)
const activeTab = ref<'apps' | 'edit'>('apps')

// 選択中Gistのオブジェクト
const selectedGist = computed(() => gists.value.find(g => g.url === gist_url.value))

// Gist選択変更時に自動取得
watch(gist_url, (nv, ov) => {
  if (nv && nv !== ov) update_cache(false)
})

// 表示名から先頭2文字を取り出して大文字化（日本語等はそのまま）
const getInitial2 = (name: string): string => {
  if (!name) return '?'
  const ch = name.trim().charAt(0)
  const ch2 = name.trim().charAt(1)
  // アルファベットの場合のみ大文字化
  return (/[a-z]/i.test(ch) ? ch.toUpperCase() : ch) + (ch2 ? ch2 : '')
}

// ーーーー 編集タブ用のロジック ーーーー
const isAllowedUrl = (url: string): boolean => {
  if (!url) return false
  if (url.startsWith('javascript:')) return true
  try {
    const u = new URL(url)
    return u.protocol === 'http:' || u.protocol === 'https:'
  } catch {
    return false
  }
}

const addRow = () => {
  bookmarks.value = [...bookmarks.value, ['', '']]
}

const removeRow = (idx: number) => {
  const next = bookmarks.value.slice()
  next.splice(idx, 1)
  bookmarks.value = next
}

const moveRow = (idx: number, dir: -1 | 1) => {
  const next = bookmarks.value.slice()
  const to = idx + dir
  if (to < 0 || to >= next.length) return
  const [item] = next.splice(idx, 1)
  next.splice(to, 0, item)
  bookmarks.value = next
}

const copyJsonToClipboard = async () => {
  // タイトルが空ならURLをタイトルとして補完
  const items = bookmarks.value.map(([url, title]) => [url, title && title.trim() !== '' ? title : url]) as Bookmark[]
  const text = JSON.stringify({items}, null, 2)
  await navigator.clipboard.writeText(text)
  // 軽いフィードバック
  console.log('JSON copied to clipboard')
}
</script>

<template lang="pug">
  nav.app-nav
    .nav-inner.container
      .left
        .brand Private Start Page
        ul.tabs
          li(:class="{active: activeTab==='apps'}")
            button.tab(@click="activeTab='apps'") アプリ
          li(:class="{active: activeTab==='edit'}")
            button.tab(@click="activeTab='edit'") 編集
      .right
        button.icon(@click="settingsOpen=!settingsOpen" aria-label="Toggle settings") ⚙️
    transition(name="panel")
      .settings-panel.container(v-if="settingsOpen")
        .row
          label Source gist
          select(v-model="gist_url")
            option(value="")
            option(v-for="(g, i) in gists" :key="g.url ?? g.title ?? i" :value="g.url") {{ g.title }}
        .row
          button(@click="update_cache(true)") Force fetch
          a.settings-link(v-if="selectedGist" :href="selectedGist?.html_url" target="_blank" rel="noopener noreferrer") 選択中Gistを開く
  main.container.app-main
    section(v-show="activeTab==='apps'")
      .link-grid
        a.link-item(v-for="[k, v] in bookmarks" :key="k" :href="k" target="_blank" rel="noopener noreferrer")
          span.initial {{ getInitial2(v) }}
          span.title {{ v || k }}
    section(v-show="activeTab==='edit'" class="edit-panel")
      .tools
        button.primary(@click="copyJsonToClipboard") JSONをクリップボードにコピー
        a(v-if="selectedGist" :href="`${selectedGist?.html_url}/edit`" target="_blank" rel="noopener noreferrer") このGistを編集
      .table
        .thead
          .th URL
          .th タイトル
          .th 操作
        .tbody
          .tr(v-for="([url, title], idx) in bookmarks" :key="idx")
            .td
              input(type="text" v-model="bookmarks[idx][0]" placeholder="https://... または javascript:...")
              span.err(v-if="bookmarks[idx][0] && !isAllowedUrl(bookmarks[idx][0])") URLが不正です
            .td
              input(type="text" v-model="bookmarks[idx][1]" placeholder="任意（空ならURLが使われます）")
            .td.ops
              button(@click="moveRow(idx,-1)" :disabled="idx===0") ↑
              button(@click="moveRow(idx,1)" :disabled="idx===bookmarks.length-1") ↓
              button.danger(@click="removeRow(idx)") 削除
        .tfoot
          button(@click="addRow") 行を追加
</template>

<style scoped lang="less">
/* Stickyな黒帯ナビ */
.app-nav {
  position: sticky;
  top: 0;
  width: 100%;
  background: #0f0f10;
  color: #fff;
  z-index: 1000;
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.06), 0 4px 12px rgba(0, 0, 0, 0.2);
}

.nav-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
}

.brand {
  font-weight: 700;
  margin-right: 1rem;
}

.left {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.tabs {
  display: flex;
  list-style: none;
  gap: 0.25rem;
  padding: 0;
  margin: 0;
}

.tab {
  background: transparent;
  color: #fff;
  border: none;
  padding: 0.25rem 0.75rem;
  border-radius: 8px;
  cursor: pointer;
}

.tabs li.active .tab {
  background: rgba(255, 255, 255, 0.12);
}

.icon {
  background: transparent;
  color: #fff;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  padding: 0.25rem 0.5rem;
  cursor: pointer;
}

.settings-panel {
  background: #0d0d0e;
  color: #fff;
  padding: 0.75rem 0;
}

.settings-panel .row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.25rem 0;
}

.settings-link {
  margin-left: 0.5rem;
}

.panel-enter-active, .panel-leave-active {
  transition: all .18s ease;
}

.panel-enter-from, .panel-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.app-main {
  padding-top: 1rem;
}

/* リンクをフレックスで並べ、2段構成（頭文字 + タイトル）のボタン風に */
.link-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.link-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  text-decoration: none;
  padding: 0.75rem 0.5rem;
  border-radius: 12px;
  border: 1px solid rgba(125, 125, 125, 0.25);
  background: rgba(127, 127, 127, 0.06);
  transition: background-color 120ms ease, border-color 120ms ease, transform 120ms ease;

  /* 幅を制御: 6〜10個/列程度を目安に */
  flex: 1 1 10rem; /* ベース ~160px */
  max-width: 14rem; /* 上限 ~224px */
  min-width: 8rem; /* 下限 ~128px */
}

.link-item:hover,
.link-item:focus-visible {
  border-color: rgba(125, 125, 125, 0.45);
  background: rgba(127, 127, 127, 0.12);
  transform: translateY(-1px);
}

.initial {
  font-size: 2rem;
  line-height: 1;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.title {
  margin-top: 0.25rem;
  font-size: 0.85rem;
  color: var(--muted-color, #6b7280);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 編集テーブル */
.edit-panel {
  margin-top: 1rem;
}

.tools {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin-bottom: 0.75rem;
}

.tools .primary {
  background: #2563eb;
  color: #fff;
  border: none;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  cursor: pointer;
}

.table {
  width: 100%;
  border: 1px solid rgba(125, 125, 125, 0.25);
  border-radius: 8px;
  overflow: hidden;
}

.thead, .tr {
  display: grid;
  grid-template-columns: 1.5fr 1fr auto;
  gap: 0.5rem;
  align-items: center;
}

.thead {
  background: rgba(127, 127, 127, 0.08);
  padding: 0.5rem;
  font-weight: 600;
}

.tbody {
  display: grid;
  gap: 0.25rem;
  padding: 0.5rem;
}

.td {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.td input {
  width: 100%;
  padding: 0.4rem 0.5rem;
  border-radius: 6px;
  border: 1px solid rgba(125, 125, 125, 0.35);
  background: #fff;
  color: #111;
}

.td .err {
  font-size: 0.75rem;
  color: #b91c1c;
}

.ops {
  display: flex;
  gap: 0.25rem;
  justify-content: flex-end;
}

.ops button {
  padding: 0.25rem 0.5rem;
  border-radius: 6px;
  border: 1px solid rgba(125, 125, 125, 0.35);
  background: #0043ca;
  cursor: pointer;
}

.ops .danger {
  background: #e63434;
  border-color: #fecaca;
}

.tfoot {
  padding: 0.5rem;
  border-top: 1px solid rgba(125, 125, 125, 0.25);
}

.tfoot button {
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(125, 125, 125, 0.35);
  background: #2d794b;
  cursor: pointer;
}
</style>
