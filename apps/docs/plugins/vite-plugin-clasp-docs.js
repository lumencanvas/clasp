import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import matter from 'gray-matter'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const DOCS_DIR = path.resolve(__dirname, '../../../docs')

const VIRTUAL_MANIFEST = 'virtual:docs-manifest'
const VIRTUAL_CONTENT = 'virtual:docs-content'
const RESOLVED_MANIFEST = '\0' + VIRTUAL_MANIFEST
const RESOLVED_CONTENT = '\0' + VIRTUAL_CONTENT

function slugify(text) {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function walkDir(dir, base = '') {
  if (!fs.existsSync(dir)) return []
  const entries = fs.readdirSync(dir, { withFileTypes: true })
  let files = []
  for (const entry of entries) {
    const rel = base ? `${base}/${entry.name}` : entry.name
    if (entry.isDirectory()) {
      files = files.concat(walkDir(path.join(dir, entry.name), rel))
    } else if (entry.name.endsWith('.md')) {
      files.push(rel)
    }
  }
  return files
}

function extractHeadings(content) {
  const headings = []
  const regex = /^(#{1,6})\s+(.+)$/gm
  let match
  while ((match = regex.exec(content))) {
    const level = match[1].length
    const text = match[2]
      .replace(/\*\*(.+?)\*\*/g, '$1')
      .replace(/\*(.+?)\*/g, '$1')
      .replace(/`(.+?)`/g, '$1')
      .replace(/\[(.+?)\]\(.+?\)/g, '$1')
      .trim()
    headings.push({ level, text, id: slugify(text) })
  }
  return headings
}

function extractTitle(content) {
  const match = content.match(/^#\s+(.+)$/m)
  if (!match) return null
  return match[1]
    .replace(/\*\*(.+?)\*\*/g, '$1')
    .replace(/\*(.+?)\*/g, '$1')
    .replace(/`(.+?)`/g, '$1')
    .replace(/\[(.+?)\]\(.+?\)/g, '$1')
    .trim()
}

function processCallouts(html) {
  // Transform GitHub-style alerts: > [!NOTE], > [!TIP], > [!WARNING], > [!IMPORTANT], > [!CAUTION]
  return html.replace(
    /<blockquote>\s*<p>\[!(NOTE|TIP|WARNING|IMPORTANT|CAUTION)\]\s*/gi,
    (_, type) => {
      const t = type.toLowerCase()
      const label = t.charAt(0).toUpperCase() + t.slice(1)
      return `<blockquote class="callout callout-${t}"><p><span class="callout-title">${label}</span> `
    }
  )
}

function createMarkdownRenderer() {
  const md = new MarkdownIt({
    html: true,
    linkify: true,
    typographer: true,
    highlight(str, lang) {
      if (lang && hljs.getLanguage(lang)) {
        try {
          return `<pre class="hljs"><code class="language-${lang}">${hljs.highlight(str, { language: lang }).value}</code></pre>`
        } catch (_) { /* fall through */ }
      }
      return `<pre class="hljs"><code>${md.utils.escapeHtml(str)}</code></pre>`
    }
  })

  // Transform .md links to site routes
  const defaultLinkOpen = md.renderer.rules.link_open || function (tokens, idx, options, _env, self) {
    return self.renderToken(tokens, idx, options)
  }

  md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
    const hrefIdx = tokens[idx].attrIndex('href')
    if (hrefIdx >= 0) {
      let href = tokens[idx].attrs[hrefIdx][1]
      if (!href.startsWith('http') && !href.startsWith('#') && !href.startsWith('mailto:')) {
        const [linkPath, anchor] = href.split('#')
        if (linkPath.endsWith('.md') || (linkPath.includes('/') && !linkPath.startsWith('/'))) {
          let resolved = linkPath.replace(/\.md$/, '').replace(/\/README$/, '')
          // Resolve any relative path (including bare filenames) against the current file's directory
          if (!resolved.startsWith('/')) {
            const currentDir = path.posix.dirname(env.filePath || env.docPath || '')
            resolved = path.posix.normalize(path.posix.join(currentDir, resolved))
          }
          if (!resolved.startsWith('/')) resolved = '/' + resolved
          resolved = resolved.replace(/\/+/g, '/')
          if (anchor) resolved += '#' + anchor
          tokens[idx].attrs[hrefIdx][1] = resolved
        }
      }
    }
    return defaultLinkOpen(tokens, idx, options, env, self)
  }

  // Add id attributes to headings
  md.renderer.rules.heading_open = function (tokens, idx) {
    const inline = tokens[idx + 1]
    const text = inline?.children?.map(t => t.content).join('') || ''
    const id = slugify(text)
    return `<${tokens[idx].tag} id="${id}">`
  }

  return md
}

export default function claspDocs() {
  const md = createMarkdownRenderer()
  let cache = null

  function scanDocs() {
    if (!fs.existsSync(DOCS_DIR)) {
      console.warn('[clasp-docs] docs directory not found:', DOCS_DIR)
      return {}
    }

    const files = walkDir(DOCS_DIR)
    const result = {}

    for (const file of files) {
      const fullPath = path.join(DOCS_DIR, file)
      const raw = fs.readFileSync(fullPath, 'utf-8')
      const { data: frontmatter, content } = matter(raw)

      let docPath = file.replace(/\.md$/, '')
      if (docPath.endsWith('/README') || docPath === 'README') {
        docPath = docPath.replace(/\/?README$/, '') || 'index'
      }
      if (docPath === 'index' && file !== 'index.md' && !file.endsWith('/README.md')) {
        // Don't override index for non-root README files
      }

      const headings = extractHeadings(content)
      const title = frontmatter.title || extractTitle(content) || formatPathAsTitle(docPath)
      const section = frontmatter.section || docPath.split('/')[0] || 'root'
      const rawHtml = md.render(content, { docPath, filePath: file })
      const html = processCallouts(rawHtml)

      result[docPath] = {
        path: docPath,
        file,
        title,
        section,
        order: frontmatter.order ?? 999,
        description: frontmatter.description || '',
        headings,
        html
      }
    }

    return result
  }

  function formatPathAsTitle(docPath) {
    const last = docPath.split('/').pop()
    return last
      .replace(/-/g, ' ')
      .replace(/\b\w/g, c => c.toUpperCase())
  }

  return {
    name: 'vite-plugin-clasp-docs',

    resolveId(id) {
      if (id === VIRTUAL_MANIFEST) return RESOLVED_MANIFEST
      if (id === VIRTUAL_CONTENT) return RESOLVED_CONTENT
    },

    load(id) {
      if (id === RESOLVED_MANIFEST || id === RESOLVED_CONTENT) {
        if (!cache) cache = scanDocs()
      }

      if (id === RESOLVED_MANIFEST) {
        const manifest = Object.values(cache).map(({ html, ...meta }) => meta)
        return `export default ${JSON.stringify(manifest)}`
      }

      if (id === RESOLVED_CONTENT) {
        return `export default ${JSON.stringify(cache)}`
      }
    },

    buildStart() {
      if (!cache) cache = scanDocs()

      const allPaths = new Set(Object.keys(cache))
      const warnings = []
      let missingFrontmatter = 0

      for (const doc of Object.values(cache)) {
        // Check for missing frontmatter title
        if (!doc.description) {
          missingFrontmatter++
        }

        // Extract internal links from HTML and validate
        const linkRegex = /href="(\/[^"#]*)/g
        let match
        while ((match = linkRegex.exec(doc.html))) {
          let linkPath = match[1].replace(/^\//, '').replace(/\/$/, '')
          if (!linkPath) continue
          // Check: exact path, directory index, or section prefix (any child page exists)
          const isValid = allPaths.has(linkPath) ||
            allPaths.has(linkPath + '/index') ||
            [...allPaths].some(p => p.startsWith(linkPath + '/'))
          if (!isValid) {
            warnings.push(`[${doc.file}] broken link: /${linkPath}`)
          }
        }
      }

      if (warnings.length > 0) {
        console.warn(`\n[clasp-docs] ${warnings.length} broken link(s):`)
        warnings.forEach(w => console.warn(`  ${w}`))
      }
      if (missingFrontmatter > 0) {
        console.warn(`[clasp-docs] ${missingFrontmatter}/${Object.keys(cache).length} docs missing description frontmatter`)
      }
      if (warnings.length === 0 && missingFrontmatter === 0) {
        console.log(`[clasp-docs] ${Object.keys(cache).length} docs validated, 0 issues`)
      }
      console.log('')
    },

    configureServer(server) {
      server.watcher.add(DOCS_DIR)
      server.watcher.on('change', (file) => {
        if (file.startsWith(DOCS_DIR) && file.endsWith('.md')) {
          cache = null
          const manifestMod = server.moduleGraph.getModuleById(RESOLVED_MANIFEST)
          if (manifestMod) server.moduleGraph.invalidateModule(manifestMod)
          const contentMod = server.moduleGraph.getModuleById(RESOLVED_CONTENT)
          if (contentMod) server.moduleGraph.invalidateModule(contentMod)
          server.ws.send({ type: 'full-reload' })
        }
      })
    },

    generateBundle() {
      if (!cache) cache = scanDocs()

      const baseUrl = 'https://docs.clasp.to'
      const today = new Date().toISOString().split('T')[0]
      const urls = Object.values(cache).map(doc => {
        const loc = doc.path === 'index' ? '/' : '/' + doc.path
        return `  <url><loc>${baseUrl}${loc}</loc><lastmod>${today}</lastmod></url>`
      })

      const sitemap = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
        `  <url><loc>${baseUrl}/</loc><lastmod>${today}</lastmod><priority>1.0</priority></url>`,
        ...urls,
        '</urlset>'
      ].join('\n')

      this.emitFile({
        type: 'asset',
        fileName: 'sitemap.xml',
        source: sitemap
      })
    }
  }
}
