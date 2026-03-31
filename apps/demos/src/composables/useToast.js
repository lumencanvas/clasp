import { ref } from 'vue'

const toasts = ref([])

export function useToast() {
  function toast(msg, type) {
    const id = Date.now() + Math.random()
    toasts.value.push({ id, msg, type, out: false })
    setTimeout(() => {
      const t = toasts.value.find(x => x.id === id)
      if (t) t.out = true
      setTimeout(() => { toasts.value = toasts.value.filter(x => x.id !== id) }, 280)
    }, 3200)
  }

  return { toasts, toast }
}
