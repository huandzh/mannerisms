#!/usr/bin/env node
// Validate every skill bundle under .agents/skills against docs/skill-anatomy.md.
// Zero dependencies, deterministic output, --json for machines.
import { spawnSync } from 'node:child_process'
import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs'
import { join, dirname, resolve, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const SKILLS_DIR = join(ROOT, '.agents', 'skills')
const README = join(ROOT, 'README.md')
const PROVENANCE = join(ROOT, 'docs', 'provenance.md')

const ALLOWED_KEYS = new Set(['name', 'description', 'license', 'compatibility', 'metadata', 'allowed-tools'])
const ALLOWED_DIRS = new Set(['references', 'scripts', 'assets', 'tools', 'bin', 'examples'])
const BINARY_EXT = new Set(['.exe', '.dll', '.pdb'])
const MAX_BODY_LINES = 500
const MAX_FILE_BYTES = 5 * 1024 * 1024
const NAME_RE = /^[a-z0-9]+(-[a-z0-9]+)*$/

const errors = []
const warnings = []
const skills = []

const json = process.argv.includes('--json')

function fail(skill, message) {
  errors.push(skill ? `${skill}: ${message}` : message)
}
function warn(skill, message) {
  warnings.push(skill ? `${skill}: ${message}` : message)
}

// Build output (target/, bin/*.exe, .temp/) is gitignored: it can never be committed,
// so the bundle rules must not judge it. One git call answers for every path.
function ignoredSet(paths) {
  if (paths.length === 0) return new Set()
  const rel = paths.map((p) => relative(ROOT, p).split(sep).join('/'))
  const r = spawnSync('git', ['-C', ROOT, 'check-ignore', '--stdin'], { input: rel.join('\n') })
  if (r.error || (r.status !== 0 && r.status !== 1)) return new Set()
  const out = (r.stdout ?? Buffer.alloc(0)).toString('utf8')
  return new Set(out.split('\n').filter(Boolean).map((p) => resolve(ROOT, p)))
}

function parseFrontmatter(text) {
  if (!text.startsWith('---\n')) return null
  const end = text.indexOf('\n---', 3)
  if (end === -1) return null
  const block = text.slice(4, end)
  const fields = {}
  let current = null
  for (const raw of block.split('\n')) {
    if (!raw.trim()) continue
    if (/^\s/.test(raw) && current) {
      fields[current] += ' ' + raw.trim()
      continue
    }
    const m = raw.match(/^([A-Za-z0-9_-]+):\s*(.*)$/)
    if (!m) continue
    current = m[1]
    fields[current] = m[2].trim()
  }
  for (const key of Object.keys(fields)) {
    const v = fields[key]
    if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) {
      fields[key] = v.slice(1, -1)
    }
  }
  return fields
}

function walk(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name)
    if (entry.isDirectory()) walk(full, out)
    else out.push(full)
  }
  return out
}

function checkBundle(name, dir) {
  const skillFile = join(dir, 'SKILL.md')
  const buf = readFileSync(skillFile)

  if (buf.length >= 3 && buf[0] === 0xef && buf[1] === 0xbb && buf[2] === 0xbf) {
    fail(name, 'SKILL.md starts with a UTF-8 BOM')
  }
  if (buf.includes(0x0d)) fail(name, 'SKILL.md contains CR; this repo is LF-only')

  const text = buf.toString('utf8')
  const fields = parseFrontmatter(text)
  if (!fields) {
    fail(name, 'SKILL.md has no YAML frontmatter block (--- ... ---)')
    return
  }

  for (const key of Object.keys(fields)) {
    if (!ALLOWED_KEYS.has(key)) warn(name, `unknown frontmatter key "${key}"`)
  }
  if (!fields.name) fail(name, 'frontmatter has no name')
  if (!fields.description) fail(name, 'frontmatter has no description')

  if (fields.name) {
    if (fields.name !== name) fail(name, `name "${fields.name}" does not match the directory`)
    if (!NAME_RE.test(fields.name)) fail(name, `name "${fields.name}" is not lowercase-hyphen`)
    if (fields.name.length > 64) fail(name, 'name is longer than 64 characters')
  }
  if (fields.description) {
    const n = [...fields.description].length
    if (n === 0) fail(name, 'description is empty')
    if (n > 1024) fail(name, `description is ${n} characters (max 1024)`)
  }

  const body = text.slice(text.indexOf('\n---', 3) + 4)
  const bodyLines = body.replace(/^\n/, '').split('\n').length
  if (bodyLines > MAX_BODY_LINES) fail(name, `body is ${bodyLines} lines (max ${MAX_BODY_LINES})`)

  for (const m of text.matchAll(/\]\(([^)\s]+)\)/g)) {
    const target = m[1]
    if (/^(https?:|mailto:)/.test(target) || target.startsWith('#')) continue
    const clean = target.split('#')[0]
    if (!clean) continue
    const resolved = resolve(dir, clean)
    const rel = relative(dir, resolved)
    if (rel.startsWith('..') || rel.startsWith(sep)) {
      fail(name, `link escapes the bundle: ${target}`)
    } else if (!existsSync(resolved)) {
      fail(name, `link target does not exist: ${target}`)
    }
  }

  const entries = readdirSync(dir, { withFileTypes: true })
  const ignored = ignoredSet([...walk(dir), ...entries.map((e) => join(dir, e.name))])

  for (const entry of entries) {
    if (entry.isDirectory() && !ALLOWED_DIRS.has(entry.name) && !ignored.has(join(dir, entry.name))) {
      warn(name, `unexpected top-level directory "${entry.name}"`)
    }
  }

  for (const file of walk(dir)) {
    if (ignored.has(file)) continue
    const size = statSync(file).size
    if (size > MAX_FILE_BYTES) fail(name, `${relative(dir, file)} is larger than 5 MB`)
    const lower = file.toLowerCase()
    for (const ext of BINARY_EXT) {
      if (lower.endsWith(ext)) fail(name, `${relative(dir, file)} is a committed binary`)
    }
  }

  skills.push({ name, bodyLines, descriptionLength: fields.description ? [...fields.description].length : 0 })
}

function main() {
  if (!existsSync(SKILLS_DIR)) {
    console.error(`no skills directory at ${SKILLS_DIR}`)
    process.exit(1)
  }

  const dirs = readdirSync(SKILLS_DIR, { withFileTypes: true }).filter((e) => e.isDirectory())
  if (dirs.length === 0) fail(null, 'no skills found under .agents/skills')

  for (const entry of dirs) {
    const dir = join(SKILLS_DIR, entry.name)
    if (!existsSync(join(dir, 'SKILL.md'))) {
      fail(entry.name, 'missing SKILL.md')
      continue
    }
    checkBundle(entry.name, dir)
  }

  const readme = existsSync(README) ? readFileSync(README, 'utf8') : ''
  const provenance = existsSync(PROVENANCE) ? readFileSync(PROVENANCE, 'utf8') : ''
  for (const { name } of skills) {
    if (!readme.includes(`.agents/skills/${name}/`)) fail(name, 'missing from the README catalog')
    if (!provenance.includes('`' + name + '`')) fail(name, 'missing from docs/provenance.md')
  }

  if (json) {
    console.log(JSON.stringify({ ok: errors.length === 0, skills, errors, warnings }, null, 2))
  } else {
    for (const w of warnings) console.warn(`warn  ${w}`)
    for (const e of errors) console.error(`error ${e}`)
    const summary = `${skills.length} skill(s), ${errors.length} error(s), ${warnings.length} warning(s)`
    console.log(errors.length === 0 ? `PASS  ${summary}` : `FAIL  ${summary}`)
  }
  process.exit(errors.length === 0 ? 0 : 1)
}

main()
