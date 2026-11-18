---
name: "comment-discuss"
description: "在文本文件中与 agent 做讨论式评审：`cmt` CLI 扫描 `·` / `··` 讨论评论及其上下文，按协议关闭（`·` 移入 sidecar 日志、`··` 行内标记 (closed)），并校验闭环。Rust 实现，单次读写、UTF-8/UTF-16 安全、默认带 3-5 行上下文、支持 JSON。Invoke when user asks to discuss a file in place, list or close comments, validate that all comments are addressed, or mentions `·` / `··` discussion markers — generic, upgraded replacement for review-comment and revise-findings."
metadata: { language: zh-CN }
---

# Comment Discuss — 文件内讨论与闭环

在**同一个文件里**与 agent 讨论、修订、留痕：用户或 agent 在正文旁写一行评论，agent 处理正文，然后按标记类型关闭评论，最后由工具校验"没有漏项"。

适用任何文本文件（`.md` / `.txt` / `.rst` …），不绑定具体报告类型。

## 一、两种标记（核心协议）

| 标记 | 名称 | 关闭动作 | 结果 |
|------|------|---------|------|
| `· 内容` | **L1 · 侧车日志** | 处理完后**整行移出**文档，追加到 `<doc>.discuss-log.md` | 讨论文件保持干净，决策留痕在日志 |
| `·· 内容` | **L2 · 行内关闭** | 处理完后在该行末尾追加 ` (closed)` | 讨论历史留在原文件里 |
| `···` 或空评论 | 非法标记 | `validate` 报 `malformed`，仅 `--force` 可删除 | 防止误删 |

```markdown
## 洞察 2

· 这个标题太抽象，改成"拒绝过度复杂"          ← L1：改完移入日志
·· 这里的访谈引用要补上被访者编号              ← L2：改完标 (closed)
```

选择规则：**讨论结论需要留档、又不想污染正文** → `·`；**这条讨论本身就该被读者看到**（例如对外评审记录、待确认清单）→ `··`。

## 二、工具

`cmt`（Rust，零依赖）。仓库不提交二进制；包装脚本按固定顺序解析：
`CMT_BIN` → `<skill>/bin/cmt[.exe]` → `PATH` → 从 `<skill>/tools/comment_discuss/` 本地编译（需要 cargo）。

```powershell
# Windows
pwsh .agents/skills/comment-discuss/scripts/cmt.ps1 scan <file> -C 4
```

```bash
# POSIX
bash .agents/skills/comment-discuss/scripts/cmt.sh scan <file> -C 4
```

`cmt --help` 是完整参数表；细节见 `references/cli.md`。

## 三、标准工作流

```
1. SCAN      cmt scan <file>                 # 评论 + 上下 4 行 + 所在章节 + hash
2. READ      上下文不够时，读取评论所在章节（不要猜）
3. EDIT      用编辑工具改正文；禁止手工删改评论行
4. CLOSE     cmt close <file> --line N -n "改了什么"   （或一次性 --all）
5. VALIDATE  cmt validate <file>             # 0 才算闭环
6. SUMMARY   汇报：处理了几条、改了哪些章节、日志落在哪
```

规则：

- **评论行只能由 `cmt close` 关闭**，不要用 Edit 删除或手写 `(closed)`——否则哈希日志与幂等失效。
- **关闭后行号会移动**：优先一次 `--all`；逐条处理时按行号**从大到小**关闭，或每次关闭后重新 `scan`。
- **改前确认文件版本**：人类刚发起新一轮讨论、或你刚跑过 `cmt close`（会重写文件）→ 先重新 `scan`/重读再动手；本轮双方都没动过文件 → 直接改。
- `--id` 也随文件变化而变，用之前先 `scan`。
- 处理不了的评论**不要关闭**，在汇报里点名，让用户决定。
- 只改评论指向的内容，不做顺手重构。

## 四、命令速查

| 命令 | 用途 | 关键参数 |
|------|------|---------|
| `cmt scan <path>…` | 列出评论与上下文 | `-C 4` 上下文行数、`-B/-A` 分别控制、`--level 1\|2`、`--open-only`、`--grep 文本`、`-s` 只要表格、`--json`、`--exit-code` |
| `cmt validate <path>…` | 校验闭环 | `--strict` 交叉核对侧车日志与重复评论、`--allow-open` 只看状态、`--json` |
| `cmt close <file>` | 关闭评论 | `--all`、`--line N`、`--id N`、`-n "resolution"`、`--dry-run`、`--log <path>`、`--force` |
| `cmt help` | 参数表 | |

`scan` 直接受目录：递归 `.md/.markdown/.mdx/.txt`，自动跳过 `.git`、`.venv`、`target`、`node_modules` 等，并跳过 `*.discuss-log.md`（`--include-logs` 可关掉）。

## 五、退出码

| 码 | `validate` 含义 | `scan --exit-code` 含义 |
|----|----------------|------------------------|
| 0 | 全部已关闭 / 已移出 | 无待办 |
| 1 | 用法或 IO 错误 | 同左 |
| 2 | 仍有 open 评论（该干活） | 仍有 open 评论 |
| 3 | 协议错误：非法标记、`--strict` 命中已记录或重复评论、非 UTF-8 | 含非法标记 |

`validate` 的问题码：`open-comment-l1`、`open-comment-l2`、`l1-inline-closed`、`malformed-marker`、`already-logged`、`duplicate-comment`、`lossy-encoding`。

## 六、侧车日志

`<doc>.discuss-log.md`，追加写入，最新在末尾：

```markdown
<!-- comment-discuss log v1 · sidecar for 报告.md -->

## 2026-10-27T06:30:01Z · line 24 · 二、核心洞察摘要

<!-- cmt:839c8757a9cb583b -->
> · 按照模块顺序，重新调一下洞察顺序
> resolution: 已按模块重排 1-4 章洞察
```

哈希由评论文本归一化后算出，`validate --strict` 用它判断"这条是不是早已记过"。同一评论重复关闭不会写第二条。

日志默认与文档同目录；`--log` 可改路径，`.comment-discuss.conf` 的 `log_suffix` 可改后缀。是否提交日志按项目约定，一般建议保留（它是评审附件）。

## 七、配置（可选）

在文档目录或更上层放 `.comment-discuss.conf`（最近的一层生效，向上找 6 层）：

```ini
log_suffix = .discuss-log.md
context = 4
closed_tokens = closed
skip_fences = true
extensions = md, markdown, mdx, txt
```

## 八、边界行为

- **代码块里的 `·` 不算评论**（默认跳过 ``` / ~~~ 围栏，`--no-fence-skip` 关闭该保护）。
- `> · 引用` 形式不被识别为评论（引用别人评论时不会误判）。
- 编码：读 UTF-8 / UTF-8-BOM / UTF-16LE/BE(BOM)；非 UTF-8 一律标 `lossy` 并**拒绝改写**（`--allow-lossy` 才强制）。
- 行尾：`\r\n` / `\n` 按原样逐行保留，不做全局换行符转换。
- 写文件走临时文件 + rename，不会中途截断文档。
- 删除 L1 评论行后，若上下都为空行会合并掉一个空行（避免留下双空行）；其余字节保持不变。

## 九、与旧 skill 的关系

`comment-discuss` 是**通用升级版**：

- 取代 `review-comment`（仅有 `·` 单一标记 + Python 扫描脚本）。
- 覆盖并泛化 `revise-findings`（原为研究发现报告专用，现为任意文本 + 双协议 + 校验）。
- 结论性差异：多了 L2 行内协议、`validate` 闭环校验、侧车日志哈希去重、编码/行尾安全、目录批量扫描与 JSON 输出。

研究类报告仍可沿用原有写作规范；讨论与闭环环节统一走本 skill。
