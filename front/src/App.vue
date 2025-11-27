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
</script>

<template lang="pug">
  nav
    span Source gist
    select(v-model="gist_url")
      option(value="")
      option(v-for="(g, i) in gists" :key="g.url ?? g.title ?? i" :value="g.url") {{ g.title }}
    button(@click="update_cache(true)") fetch force
  .main
    ul
      li.bookmark(v-for="[k, v] in bookmarks" :key="k")
        a(:href="k") {{ v }}
</template>

<style scoped lang="less">
ul {
  list-style: none;
}

li.bookmark {
  border: 1px solid grey;
  border-radius: 8px;
  padding: 8px;
  margin: 2px;
  background-color: #fff;
  display: inline-block;
  width: 200px;

  &:hover {
    background-color: #eff;
    border-color: blue;
    cursor: pointer;
  }
}
</style>
