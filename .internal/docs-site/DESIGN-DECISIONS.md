# CLASP Docs Site — Design Decisions

## Framework: Custom Vue 3 + Vite (not VitePress)

**Rationale:** The CLASP design language (paper/ink/grain texture, Archivo Black + Space Mono + JetBrains Mono, specific accent colors) is very specific. A custom app reuses the exact CSS from `site/src/style.css` without fighting VitePress's theming system. The project already has Vue 3 + Vite patterns in `site/` and `apps/chat/`.

## Content Processing: Vite Plugin with Virtual Modules

**Approach:** Custom `vite-plugin-clasp-docs.js` scans `../../docs/**/*.md` at build time, processes with gray-matter + markdown-it + highlight.js, and exports as two virtual modules:
- `virtual:docs-manifest` — metadata only (path, title, section, order, headings)
- `virtual:docs-content` — full rendered HTML + metadata

**Why virtual modules:** Clean integration with Vite's module system. HMR works by invalidating virtual modules when source .md files change.

**Why single bundle (not code-split):** With 150 docs totaling ~398KB gzipped, the full content is small enough to serve as a single bundle. Text compresses extremely well. This avoids the complexity of dynamic imports with virtual modules, which Vite can't statically analyze.

## Theme: Light Default with Dark Toggle

- Light mode: `site/` palette (paper #fefcf6, ink #1a1a1a, grain at 0.5 opacity)
- Dark mode: `apps/chat/` palette (#1a1a1a bg, #e0e0e0 text, grain at 0.15 opacity)
- CSS custom properties swapped via `[data-theme="dark"]` on `<html>`
- Persisted to `localStorage` key `clasp-docs-theme`
- Defaults to `prefers-color-scheme` media query if no stored preference

## Sidebar Navigation: Hardcoded Section Order, Auto-populated Items

The sidebar uses a `SECTION_CONFIG` array that maps doc directories to labeled sections with specific ordering. Items within each section are auto-populated from the docs manifest. This approach:
- Matches the planned information architecture immediately
- Works without frontmatter (Phase 4 will add ordering)
- Handles subgroups (e.g., Reference -> Protocol, API - Rust, etc.)

## Link Transformation: Build-time Rewriting

Cross-document markdown links (e.g., `../explanation/why-clasp.md`) are transformed at build time by the markdown-it renderer to proper SPA routes (e.g., `/explanation/why-clasp`). Additionally, `DocPage.vue` intercepts clicks on rendered links and uses `vue-router.push()` for client-side navigation.

## Deployment Target: Docker (node build + nginx alpine)

Matches the `apps/chat/` deployment pattern. Will be implemented in Phase 3.
