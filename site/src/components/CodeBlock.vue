<script setup>
import { computed } from 'vue'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import python from 'highlight.js/lib/languages/python'
import rust from 'highlight.js/lib/languages/rust'
import c from 'highlight.js/lib/languages/c'
import json from 'highlight.js/lib/languages/json'
import plaintext from 'highlight.js/lib/languages/plaintext'
import 'highlight.js/styles/github.css'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('c', c)
hljs.registerLanguage('json', json)
hljs.registerLanguage('plaintext', plaintext)

const props = defineProps({
  code: String,
  language: {
    type: String,
    default: 'plaintext'
  }
})

const highlightedHtml = computed(() => {
  const lang = hljs.getLanguage(props.language) ? props.language : 'plaintext'
  return hljs.highlight(props.code, { language: lang }).value
})
</script>

<template>
  <pre><code class="hljs" :class="`language-${language}`" v-html="highlightedHtml"></code></pre>
</template>
