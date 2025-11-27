<script setup lang="ts">
import axios from 'axios'
import {onMounted, ref} from 'vue'
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

// 表示名から先頭1文字を取り出して大文字化（日本語等はそのまま）
const getInitial2 = (name: string): string => {
  if (!name) return '?'
  const ch = name.trim().charAt(0)
  const ch2 = name.trim().charAt(1)
  // アルファベットの場合のみ大文字化
  return (/[a-z]/i.test(ch) ? ch.toUpperCase() : ch) + (ch2 ? ch2 : '')
}
</script>

<template lang="pug">
  nav.container
    ul
      li
        strong Source gist
      li
        select(v-model="gist_url")
          option(value="")
          option(v-for="(g, i) in gists" :key="g.url ?? g.title ?? i" :value="g.url") {{ g.title }}
    ul
      li
        button(@click="update_cache(true)") Force fetch
  main.container
    .link-grid
      a.link-item(v-for="[k, v] in bookmarks" :key="k" :href="k" target="_blank" rel="noopener noreferrer")
        span.initial {{ getInitial2(v) }}
        span.title {{ v }}
</template>

<style scoped lang="less">
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
  min-width: 8rem;  /* 下限 ~128px */
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
</style>
