#!/usr/bin/env pwsh
# Thin launcher for the Rust `cmt` CLI (comment-discuss skill).
# Resolution: $env:CMT_BIN -> <skill>/bin/cmt.exe -> PATH -> cargo build from <skill>/tools/comment_discuss.
$ErrorActionPreference = 'Stop'

$skillDir = Split-Path -Parent $PSScriptRoot
$exe = if ($env:CMT_BIN) { $env:CMT_BIN } else { Join-Path $skillDir 'bin\cmt.exe' }

if (-not (Test-Path -LiteralPath $exe)) {
    $onPath = Get-Command cmt -ErrorAction SilentlyContinue
    if ($onPath) { & $onPath.Source @args; exit $LASTEXITCODE }

    $manifest = Join-Path $skillDir 'tools\comment_discuss\Cargo.toml'
    if (-not (Test-Path -LiteralPath $manifest)) {
        Write-Error "cmt: no CMT_BIN, no bin/cmt.exe, no cmt on PATH, and no source at $manifest"
        exit 1
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "cmt: cargo not found; set CMT_BIN or put cmt on PATH"
        exit 1
    }
    Write-Host '[cmt] building release binary (first run)...'
    & cargo build --release --offline --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { Write-Error 'cmt: cargo build failed'; exit 1 }
    $built = Join-Path $skillDir 'tools\comment_discuss\target\release\cmt.exe'
    $cache = Join-Path $skillDir 'bin\cmt.exe'
    if (Test-Path -LiteralPath $built) {
        try {
            New-Item -ItemType Directory -Force (Split-Path -Parent $cache) | Out-Null
            Copy-Item -LiteralPath $built -Destination $cache -Force
        } catch { }
    }
    $exe = $built
}

& $exe @args
exit $LASTEXITCODE
