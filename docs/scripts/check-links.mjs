import { readdirSync, readFileSync, existsSync, statSync } from 'node:fs'
import { resolve, relative } from 'node:path'

const root = resolve('.vitepress/dist')
const base = process.env.DOCS_BASE || '/youki/'
if (!base.startsWith('/') || !base.endsWith('/')) throw new Error('DOCS_BASE must start and end with /')

function walk(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = resolve(directory, entry.name)
    return entry.isDirectory() ? walk(path) : [path]
  })
}

const html = walk(root).filter(path => path.endsWith('.html'))
const ids = new Map(html.map(path => [path, new Set(
  [...readFileSync(path, 'utf8').matchAll(/\bid=["']([^"']+)["']/g)].map(match => match[1])
)]))
const failures = []
for (const path of html) {
  const content = readFileSync(path, 'utf8')
  const current = new URL(base + relative(root, path), 'https://docs.invalid')
  for (const match of content.matchAll(/\b(href|src)=["']([^"']+)["']/g)) {
    const raw = match[2].replaceAll('&amp;', '&')
    if (!raw || /^(?:data:|mailto:|tel:|javascript:)/.test(raw)) continue
    const url = new URL(raw, current)
    if (url.origin !== current.origin) continue
    if (!url.pathname.startsWith(base)) {
      failures.push(relative(root, path) + ': URL outside base: ' + raw)
      continue
    }
    const pathname = decodeURIComponent(url.pathname.slice(base.length))
    let target = resolve(root, pathname || 'index.html')
    if (existsSync(target) && statSync(target).isDirectory()) target = resolve(target, 'index.html')
    if (!existsSync(target) && existsSync(target + '.html')) target += '.html'
    if (!existsSync(target)) {
      failures.push(relative(root, path) + ': missing target: ' + raw)
      continue
    }
    const fragment = decodeURIComponent(url.hash.slice(1))
    if (fragment && ids.has(target) && !ids.get(target).has(fragment)) {
      failures.push(relative(root, path) + ': missing anchor: ' + raw)
    }
  }
}
if (failures.length) {
  console.error(failures.join('\n'))
  process.exit(1)
}
console.log('Checked local links, assets and anchors in ' + html.length + ' HTML pages.')
