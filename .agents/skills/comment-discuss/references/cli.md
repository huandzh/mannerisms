# `cmt` CLI reference

Binary: `<skill>/tools/comment_discuss/target/release/cmt` (`.exe` on Windows)，或包装脚本解析到的 `bin/cmt`。
构建：`cargo build --release --offline --manifest-path .agents/skills/comment-discuss/tools/comment_discuss/Cargo.toml`
测试：`cargo test --offline --manifest-path .agents/skills/comment-discuss/tools/comment_discuss/Cargo.toml`

`close -n` 的 resolution 只在移动 L1 行时写进侧车日志；纯 L2 的文件不产生日志，note 会被丢弃。

## 语法

```
cmt <scan|validate|close|help> [options] <path>...
```

`cmt <path>` 无子命令时等价于 `cmt scan <path>`。

## scan

列出评论与其上下文。

| 参数 | 说明 |
|------|------|
| `-C, --context <n>` | 前、后各 n 行上下文，默认 4（配置可改），上限 50 |
| `-B, --before <n>` / `-A, --after <n>` | 分别控制前后行数（覆盖 `-C`） |
| `--level <1\|2>` | 只看某一层 |
| `--open-only` / `--closed-only` | 按状态过滤 |
| `-g, --grep <text>` | 评论正文包含指定文本（忽略大小写） |
| `-s, --summary` | 只输出表格，不输出上下文块 |
| `--json` | 机器可读输出 |
| `--ext md,txt` | 目录递归时的扩展名白名单（空 = 全部） |
| `--no-fence-skip` | 不跳过 ``` / ~~~ 围栏内内容 |
| `--include-logs` | 不跳过 `*.discuss-log.md` |
| `--exit-code` | 有 open → 2，有 malformed → 3 |
| `-q, --quiet` | 只输出汇总行 |

## validate

校验闭环协议。

| 参数 | 说明 |
|------|------|
| `--strict` | 读侧车日志：命中 `already-logged`（日志里已有但文档里还留着）、`duplicate-comment`（同文重复） |
| `--allow-open` | 未关闭评论降级为 warning，退出码仍可为 0 |
| `--allow-lossy` | 接受非 UTF-8 文件（默认视为错误） |
| `--json` | 机器可读输出，含 `ok` / `exit_code` / `violations[]` |

问题码：

| code | severity | 含义 |
|------|----------|------|
| `open-comment-l1` | 2 | `·` 还没处理（处理完要移入日志） |
| `open-comment-l2` | 2 | `··` 还没处理（处理完要标 `(closed)`） |
| `l1-inline-closed` | 2 | `·` 被行内标了 `(closed)`，层级用错 |
| `malformed-marker` | 3 | `···` 或空评论 |
| `already-logged` | 3 | 侧车日志已记录同一条，文档里应删除该行 |
| `duplicate-comment` | 3 | 同一文件里归一化后完全相同的评论出现多次 |
| `lossy-encoding` | 3 | 非 UTF-8，改写不安全 |

## close

关闭评论。层级决定动作：`·` → 移入侧车日志；`··` → 行尾追加 ` (closed)`。

| 参数 | 说明 |
|------|------|
| `-a, --all` | 关闭所有 open 评论（配 `--force` 时含非法标记） |
| `-l, --line <n>` | 按行号关闭，可重复 |
| `--id <n>` | 按 `scan` 的 id 关闭，可重复（文件变化后 id 会变，先重新 scan） |
| `--level <1\|2>` | 限制层级 |
| `-n, --note <text>` | 写入日志的 `resolution:` 处理说明 |
| `--log <path>` | 自定义日志路径（仅单文件） |
| `--dry-run` | 只报动作，不落盘 |
| `--force` | 删除非法标记而不再报错 |
| `--allow-lossy` | 强制改写非 UTF-8 文件 |

一条评论只会被关闭一次；`··` 已带 `(closed)` 的会被跳过并报 `already-closed`。

## JSON 结构

`scan`：

```json
{
  "command": "scan",
  "version": "0.1.0",
  "files": [
    {
      "path": "docs/a.md",
      "encoding": "utf-8",
      "eol": "lf",
      "lossy": false,
      "comments": [
        {
          "id": 1,
          "line": 24,
          "level": 1,
          "mode": "sidecar",
          "state": "open",
          "hash": "839c8757a9cb583b",
          "section": "二、核心洞察摘要",
          "closed_token": null,
          "indent": 0,
          "issues": [],
          "content": "按照模块顺序，重新调一下洞察顺序",
          "context_before": [{"line": 22, "text": "## 二、核心洞察摘要"}],
          "context_after": [{"line": 26, "text": "### 【洞察1：…】"}]
        }
      ]
    }
  ],
  "summary": {"files": 1, "comments": 1, "open": 1, "closed": 0, "malformed": 0}
}
```

`validate`：`{command, version, ok, exit_code, summary, violations[]}`，每条 violation 含 `path/line/id/code/severity/message`。

`close`：`{command, version, dry_run, actions[], summary}`，每条 action 含 `path/line/level/mode/action/log/hash/content`，`action` ∈ `migrated | migrated-dedup | marked-closed | already-closed | removed`。

## 侧车日志格式

```markdown
<!-- comment-discuss log v1 · sidecar for 报告.md -->

# Discussion log · 报告.md

## 2026-10-27T06:30:01Z · line 24 · 二、核心洞察摘要

<!-- cmt:839c8757a9cb583b -->
> · 按照模块顺序，重新调一下洞察顺序
> resolution: 已按模块重排 1-4 章洞察
```

- 文件头 `<!-- comment-discuss log v1` 用于识别：目录扫描会自动跳过，`validate --strict` 据此读回。
- 评论文本以 `> · ` 引用，避免日志自身被识别为待办。
- 哈希 = 归一化（去空白/标点、统一小写）后 FNV-1a 64 位；`migrated-dedup` 表示该条已在日志中。
- 追加写：新条目在文件末尾，旧条目不改。

## 编码与行尾

| 输入 | 读取 | 回写 |
|------|------|------|
| UTF-8 / UTF-8 + BOM | ✅ | 保持 BOM 有无 |
| UTF-16 LE/BE（含 BOM） | ✅ | 保持原编码与 BOM |
| 其他（GBK 等） | 有损解码并标 `lossy` | 默认拒绝；`--allow-lossy` 才写 UTF-8 |

行尾逐行保留（`\r\n`、`\n` 混排也不改动）。写文件使用同目录临时文件 + rename，避免中途失败截断。

## 性能参考

单文件 8 MB / 20 万行 / 157 条评论：`scan --json` 约 18 ms（≈440 MB/s）。`docs/` 全树 444 个文件、1218 条评论约 0.5 s。

## 配置 `.comment-discuss.conf`

从目标文件目录向上最多查 6 层，取最近一份：

```ini
log_suffix = .discuss-log.md   # report.md -> report.discuss-log.md
context = 4                    # 默认上下文行数
closed_tokens = closed         # 行内关闭标记，逗号分隔别名
skip_fences = true             # 跳过代码围栏
extensions = md, markdown, mdx, txt
```

未知键忽略；解析失败整份回退默认值。

## 设计约束

- 评论必须行首（允许前导空白）出现 `·`；`- ·`、`> ·` 不算，避免误判列表与引用。
- 只有 `·` 与 `··` 合法，第三个点即 `malformed`。
- 上下文是"物理行"（含空行），便于 agent 精确定位；不做空白折叠。
- `scan` 的 `id` 按文件内出现顺序编号，`close --id` 依赖当前文件状态。
