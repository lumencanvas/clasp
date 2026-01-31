import { onMounted, onUnmounted } from 'vue'

export function useScrollAnimation(rootRef) {
  let observer = null

  onMounted(() => {
    const root = rootRef?.value?.$el || rootRef?.value || document
    const targets = root.querySelectorAll('.fade-in, .stagger')

    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('visible')
            observer.unobserve(entry.target)
          }
        })
      },
      { threshold: 0.2 }
    )

    targets.forEach((el) => observer.observe(el))
  })

  onUnmounted(() => {
    observer?.disconnect()
  })
}
