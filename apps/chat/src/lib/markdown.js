/**
 * Light markdown renderer for chat messages.
 * Supports: **bold**, *italic*, `inline code`, ```code blocks```, ~~strikethrough~~
 * Escapes HTML to prevent XSS.
 */

const ESCAPE_MAP = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }

function escapeHtml(str) {
  return str.replace(/[&<>"']/g, c => ESCAPE_MAP[c])
}

export function renderMarkdown(text) {
  if (!text) return ''

  // Extract code blocks first to protect them from inline processing
  const codeBlocks = []
  let result = text.replace(/```([\s\S]*?)```/g, (_, code) => {
    const idx = codeBlocks.length
    codeBlocks.push(`<pre><code>${escapeHtml(code.trim())}</code></pre>`)
    return `\x00CB${idx}\x00`
  })

  // Extract inline code to protect from further processing
  const inlineCode = []
  result = result.replace(/`([^`]+)`/g, (_, code) => {
    const idx = inlineCode.length
    inlineCode.push(`<code>${escapeHtml(code)}</code>`)
    return `\x00IC${idx}\x00`
  })

  // Escape remaining HTML
  result = escapeHtml(result)

  // URLs: detect and wrap in anchor tags (before inline formatting)
  // Only http(s) and www. are matched — rejects javascript: and other protocols
  const urls = []
  result = result.replace(/(https?:\/\/[^\s<]+|www\.[^\s<]+)/g, (url) => {
    const href = url.startsWith('www.') ? `https://${url}` : url
    // Validate protocol (defense-in-depth)
    if (!/^https?:\/\//i.test(href)) return url
    const idx = urls.length
    // Strip trailing punctuation that's likely not part of the URL
    const clean = href.replace(/[.,;:!?)]+$/, '')
    const display = url.replace(/[.,;:!?)]+$/, '')
    const trailing = url.slice(display.length)
    urls.push(`<a href="${clean}" target="_blank" rel="noopener noreferrer">${display}</a>${trailing}`)
    return `\x00URL${idx}\x00`
  })

  // Bold: **text**
  result = result.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')

  // Italic: *text* (but not **)
  result = result.replace(/(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)/g, '<em>$1</em>')

  // Strikethrough: ~~text~~
  result = result.replace(/~~(.+?)~~/g, '<del>$1</del>')

  // Restore inline code
  result = result.replace(/\x00IC(\d+)\x00/g, (_, idx) => inlineCode[idx])

  // Restore URLs
  result = result.replace(/\x00URL(\d+)\x00/g, (_, idx) => urls[idx])

  // Restore code blocks
  result = result.replace(/\x00CB(\d+)\x00/g, (_, idx) => codeBlocks[idx])

  // Convert newlines to <br> (outside code blocks)
  result = result.replace(/\n/g, '<br>')

  return result
}
