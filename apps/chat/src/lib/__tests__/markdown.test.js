import { describe, it, expect } from 'vitest'
import { renderMarkdown } from '../markdown.js'

describe('markdown.js', () => {
  describe('inline formatting', () => {
    it('renders bold text', () => {
      expect(renderMarkdown('**bold**')).toContain('<strong>bold</strong>')
    })

    it('renders italic text', () => {
      expect(renderMarkdown('*italic*')).toContain('<em>italic</em>')
    })

    it('renders inline code', () => {
      expect(renderMarkdown('`code`')).toContain('<code>code</code>')
    })

    it('renders strikethrough', () => {
      expect(renderMarkdown('~~deleted~~')).toContain('<del>deleted</del>')
    })

    it('renders code blocks', () => {
      const result = renderMarkdown('```\nconst x = 1\n```')
      expect(result).toContain('<pre><code>')
      expect(result).toContain('const x = 1')
    })
  })

  describe('URL rendering', () => {
    it('renders https links as anchors', () => {
      const result = renderMarkdown('Visit https://example.com today')
      expect(result).toContain('<a href="https://example.com"')
      expect(result).toContain('target="_blank"')
      expect(result).toContain('rel="noopener noreferrer"')
    })

    it('renders http links as anchors', () => {
      const result = renderMarkdown('See http://example.com')
      expect(result).toContain('<a href="http://example.com"')
    })

    it('renders www links with https prefix', () => {
      const result = renderMarkdown('Go to www.example.com')
      expect(result).toContain('href="https://www.example.com"')
    })
  })

  describe('XSS prevention', () => {
    it('blocks javascript: URLs', () => {
      const result = renderMarkdown('javascript:alert(1)')
      expect(result).not.toContain('<a')
      expect(result).not.toContain('href')
    })

    it('blocks data: URLs', () => {
      const result = renderMarkdown('data:text/html,<script>alert(1)</script>')
      expect(result).not.toContain('<a')
      // HTML should be escaped
      expect(result).toContain('&lt;script&gt;')
    })

    it('escapes HTML tags', () => {
      const result = renderMarkdown('<script>alert("xss")</script>')
      expect(result).not.toContain('<script>')
      expect(result).toContain('&lt;script&gt;')
    })

    it('escapes HTML attributes in text', () => {
      const result = renderMarkdown('<img onerror="alert(1)">')
      expect(result).not.toContain('<img')
      expect(result).toContain('&lt;img')
    })

    it('only renders http/https links as clickable', () => {
      const result = renderMarkdown('ftp://evil.com')
      expect(result).not.toContain('<a')
    })
  })

  describe('edge cases', () => {
    it('returns empty string for null/undefined', () => {
      expect(renderMarkdown(null)).toBe('')
      expect(renderMarkdown(undefined)).toBe('')
      expect(renderMarkdown('')).toBe('')
    })

    it('preserves inline code from formatting', () => {
      const result = renderMarkdown('`**not bold**`')
      expect(result).toContain('<code>**not bold**</code>')
      expect(result).not.toContain('<strong>')
    })
  })
})
