#!/usr/bin/env node
// The local gate: validator -> vendored crate tests -> release build -> whitespace.
// Runs offline; no network, no GitHub Actions. Exit code is the gate.
import { spawnSync } from 'node:child_process'
import { join, dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const MANIFEST = join(ROOT, '.agents', 'skills', 'comment-discuss', 'tools', 'comment_discuss', 'Cargo.toml')

const steps = [
  { name: 'skill validator', cmd: process.execPath, args: [join(ROOT, 'scripts', 'validate-skills.mjs')] },
  { name: 'cmt tests', cmd: 'cargo', args: ['test', '--offline', '--manifest-path', MANIFEST] },
  { name: 'cmt release build', cmd: 'cargo', args: ['build', '--release', '--offline', '--manifest-path', MANIFEST] },
  { name: 'whitespace', cmd: 'git', args: ['diff', '--check'] },
  { name: 'whitespace (staged)', cmd: 'git', args: ['diff', '--cached', '--check'] },
]

for (const step of steps) {
  console.log(`\n=== ${step.name} ===`)
  const result = spawnSync(step.cmd, step.args, { cwd: ROOT, stdio: 'inherit' })
  if (result.error) {
    console.error(`gate: ${step.name} could not run: ${result.error.message}`)
    process.exit(1)
  }
  if (result.status !== 0) {
    console.error(`gate: FAILED at ${step.name} (exit ${result.status})`)
    process.exit(result.status ?? 1)
  }
}

console.log('\nGATE PASS')
