# BubbleKeys v0.2.0 — Plan

**Created:** 2026-04-27
**Author:** Claude (subagent-driven-development mode)
**Branch strategy:** all phases on `main` (we don't have a release branch); tag `v0.2.0` at the end
**Predecessor:** v0.1.3 (commit `cbc4db7`, ship + user-confirmed working)

## Goals (from user)

1. **更多 pack 类型** — 当前只有 cherry-blue / red / brown 三个机械键盘 pack（且都是 BubbleKeys 自己写的 placeholder，见 deviation #8）。用户希望扩展（图片里的 mechanical switch 类型：red/black/silver/brown/purple/blue/white），并讨论：自己写包还是导入开源？
2. **Pack 列表 UX 修复**
   - (a) 已导入的 pack 增加删除按钮
   - (b) 多次导入不覆盖（当前 zip stem 同名会覆盖）
   - (c) 导入时支持自定义 pack 名字
   - (d) 当 pack 数量增多，列表用 `←/→` 翻页
3. **Settings UI 收紧** — 9 行内容超出窗口高度需要滚轮，要求重排布局让内容塞进 480px 窗口。

## Out of scope (顶到 v0.3.0+)

- Phase 7 menu-bar agent retry（独立大块工作，与本次 UI 调整无关）
- Lazy CJK font loading（deviation #20）
- Apple Developer Program 签名（用户明确 defer）
- Smoke-test framework

---

## Decision: pack 来源（goal #1）

**结论：两条路径并行 — 短期靠开源导入 + 中期升级默认 pack。**

| 路径 | 优点 | 代价 | 时机 |
|---|---|---|---|
| 用户自己 import Mechvibes 社区包 | 已经有完整 ecosystem（github.com/hainguyents13/mechvibes/tree/master 数百个 pack）；不增加 BubbleKeys 仓库体积；license 由用户自负 | 需要用户找 + 下载；UX 有摩擦 | **v0.2.0 已经支持**（Phase 10.2 commit `ad75b22` 起 import_pack 已 work），本次只优化导入流（goal #2） |
| BubbleKeys 自己 bundle 更多 default packs | 开箱即用；可以挑选高质量 pack 替换 deviation #8 的 placeholder | 必须做 license 审查（CC0 / CC-BY / MIT）；增加 .dmg 体积（每个真实 pack ~500KB-2MB）；需要去找 / 验证授权 | **Phase 5 单独处理**（asset 工作多于代码，需要用户协助挑选） |

**v0.2.0 不写"自己手写音效"路径** — 录音质量上限受设备限制，且没必要；社区已有大量 CC0 / 公共领域真录音。

**对 README 的影响**：在 `## Sound Packs` 章节加链接 → Mechvibes 社区仓库 + Marketplace。

---

## Phases overview

```
Phase 0: Pre-work (version bump, branch sanity)        — ~5 min
Phase 1: Settings UI compaction (independent, low-risk) — ~30 min
Phase 2: Backend — pack management (delete + dedupe + custom name) — ~45 min
Phase 3: Frontend — pack UI (delete button + import name prompt) — ~45 min
Phase 4: Frontend — pack pagination (←/→ within PACKS tab)        — ~40 min
Phase 5: Better default sound packs (asset/license work)          — needs user input first
Phase 6: i18n + verify + ship (tag v0.2.0)                        — ~20 min
```

执行顺序：**1 → 2 → 3 → 4 → (5 可选) → 6**。每个 phase 用一个 fresh subagent。

---

## Phase 0 — Pre-work

**Files touched:** `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, memory.

**Steps:**
1. Bump version `0.1.3` → `0.2.0` in 3 spots (package.json `version`, Cargo.toml `[package].version`, tauri.conf.json `version`).
2. `git status` clean check.
3. Append progress-memory entry: starting v0.2.0 cycle.

**Verify:** `npm run build` + `cargo check` clean.

**Commit:** `chore(v0.2.0): bump version` — single commit.

---

## Phase 1 — Settings UI compaction

**Goal:** 9 settings rows 塞进 320×480 窗口（screen 区可用高度 ≈ 400px）不出现滚轮。

### Current measurements
- 9 rows × (row padding 4px×2 + content ~16px + margin 4px) ≈ 32px each → 288px
- `<input type="time">` macOS WebKit 默认渲染 ~24px 高
- 实际超模因为 input 高度 + 字体 + 标签都用 default — 超过 400px 临界值

### Layout changes (`src/views/settings.ts`)

把 `night_silent.start` + `night_silent.end` 合并为 **一个 row** (label 居中 START–END, 2 个 time input 横排)：

```ts
${row(t("settings.night_silent.window"),
   `${input("night_silent.start", s.night_silent.start, "time")}
    <span class="row-sep">–</span>
    ${input("night_silent.end",   s.night_silent.end,   "time")}`)}
```

9 行 → 8 行。新增 i18n key `settings.night_silent.window`（5 locale 都翻一遍），删除 `settings.night_silent.start` / `.end`（或保留作为 placeholder/aria-label）。

### CSS tightening (`src/styles/pixel.css`)

```css
.settings-row { padding: 2px 6px; margin-bottom: 2px; min-height: 22px; }
.settings-list { font-size: 11px; }   /* 原 fz-body */
.set-val { padding: 1px 4px; }
.toggle span { width: 24px; height: 10px; }
.row-sep { padding: 0 4px; opacity: 0.6; }
```

预期：8 × ~26px = 208px，留出充足余量。

### Steps
1. Edit `src/views/settings.ts` — merge 2 time rows into 1.
2. Edit `src/styles/pixel.css` — tighten 4 selectors.
3. Edit `src/i18n/locales/{en,zh-CN,zh-TW,ja,ko}.json` — add `settings.night_silent.window`, remove `.start` + `.end`（也可以保留留作 aria-label，决定后再说；倾向于删，避免 unused key 触发 verify:i18n CI 报错）.
4. `npm run build` + `npm run verify:i18n`.
5. **Manual smoke**: `npm run tauri dev` → open SETTINGS tab → 确认无滚轮 + 内容完整。

**Commit:** `feat(v0.2.0): compact settings layout, fit 480px window without scroll`

---

## Phase 2 — Backend: pack management

**Goal:** support delete + multi-import (no overwrite) + custom name.

### 2.1 — Track bundled vs imported (`src-tauri/src/pack_store.rs`)

```rust
#[derive(Default)]
pub struct PackStore {
    packs: HashMap<String, LoadedPack>,
    bundled_ids: HashSet<String>,    // NEW
}

impl PackStore {
    pub fn mark_bundled(&mut self, ids: &[String]) {
        self.bundled_ids = ids.iter().cloned().collect();
    }
    pub fn is_bundled(&self, id: &str) -> bool {
        self.bundled_ids.contains(id)
    }
    // existing: load_dir / ids / get
}
```

`install_default_packs` 已经在 first-run 复制到 user dir。我们要在 `lib.rs::run().setup` 里：复制完默认 pack 后，先用一个临时 `PackStore` 加载 **resource_dir 里的** `packs/` 目录，把它的 ids 收下来作为 bundled set；然后再做正常 `pack_dir` (user dir) 加载并 `mark_bundled`。

> **替代方案（更简单）**：在 `install_default_packs` 返回 `Vec<String>` of ids it copied，setup 里直接传给 PackStore。本计划采用这种 — 比扫两次便宜。

### 2.2 — `delete_pack` IPC (`src-tauri/src/ipc.rs`)

```rust
#[tauri::command]
pub fn delete_pack(
    id: String,
    store: State<'_, Arc<RwLock<PackStore>>>,
) -> Result<(), String> {
    {
        let s = store.read().unwrap();
        if s.get(&id).is_none() { return Err(format!("unknown pack: {id}")); }
        if s.is_bundled(&id) { return Err("cannot delete bundled pack".into()); }
    }
    let user_pack_dir = crate::user_data_dir().join("packs");
    // pack 目录名 = manifest.id 还是 stem? 实际上不一定相等（深 import 后 stem 可能不同）
    // 当前 import_pack 用的是 stem 当目录名 → 我们需要在 LoadedPack 里记录 dir name
    // → 改 PackStore.load_dir 让 LoadedPack 多一个 `dir_name: String` 字段
    let dir = {
        let s = store.read().unwrap();
        s.get(&id).map(|p| p.dir_name.clone()).ok_or("not found")?
    };
    let target = user_pack_dir.join(&dir);
    std::fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
    store.write().unwrap().load_dir(&user_pack_dir).map_err(|e| e.to_string())?;
    Ok(())
}
```

需要 `LoadedPack` 加 `dir_name: String` 字段（当前是 `{ manifest, samples }`），在 `load_dir` 里赋值 `entry.file_name().to_string_lossy().to_string()`。

### 2.3 — `import_pack` 增强：no-overwrite + custom_name

修改 signature：
```rust
pub async fn import_pack(
    archive_path: String,
    custom_name: Option<String>,    // NEW
    store: State<'_, Arc<RwLock<PackStore>>>,
) -> Result<String, String>
```

**No-overwrite**：解压前 dst 目录冲突检测 — 如果 `user_pack_dir.join(stem).exists()`，自动尝试 `stem-2`, `stem-3`, ... 直到唯一。

```rust
let mut dst = user_pack_dir.join(&stem);
let mut suffix = 2;
while dst.exists() {
    dst = user_pack_dir.join(format!("{stem}-{suffix}"));
    suffix += 1;
}
```

**Manifest.id 冲突**：解压完成后，先 `load_manifest(&dst)` 读 id；如果 store 里已经有这个 id：rewrite manifest.id 为 `<old_id>-<suffix>` 并 fs::write 回 config.json。

**custom_name**：解压完成 + id 处理完后，若 `custom_name` 提供，rewrite manifest.name 写回 config.json。

```rust
if custom_name.is_some() || id_was_renamed {
    let cfg_path = dst.join("config.json");
    let bytes = std::fs::read(&cfg_path).map_err(|e| e.to_string())?;
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if let Some(n) = custom_name {
        json["name"] = serde_json::Value::String(n);
    }
    if let Some(new_id) = new_id_opt {
        json["id"] = serde_json::Value::String(new_id);
    }
    std::fs::write(&cfg_path, serde_json::to_vec_pretty(&json).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
}
```

### 2.4 — `list_packs` 返回 `bundled` 字段

```rust
#[derive(Serialize, Clone)]
pub struct PackSummary {
    pub id: String,
    pub name: String,
    pub bundled: bool,    // NEW
}
```

`list_packs` 在 map 里 `bundled: store.is_bundled(&id)`.

### 2.5 — Tests

- `pack_store::tests::imported_pack_marked_non_bundled` — load fixture 后 `is_bundled("test-single") == false`，`mark_bundled(&["test-single".into()])` 后 == true.
- `pack_store::tests::dir_name_persisted_on_load` — 确认 LoadedPack.dir_name 等于实际目录名.
- 现有 17 个测试不应回归.

### Steps
1. Modify `pack_store.rs`: add `bundled_ids` + `dir_name` field + `mark_bundled` / `is_bundled` 方法.
2. Modify `pack_format.rs`: 没必要改（manifest 不变）.
3. Modify `ipc.rs`: `delete_pack` 新增；`import_pack` 加 `custom_name` 参数 + dedupe + name/id rewrite；`PackSummary` 加 `bundled` 字段；`list_packs` 填 bundled.
4. Modify `lib.rs`: `install_default_packs` 改返回 `Vec<String>`；setup 里 mark_bundled.
5. Register `delete_pack` 在 `invoke_handler!`.
6. `cargo test` + `cargo check` 必须 clean，no warning.

**Commit:** `feat(v0.2.0): pack delete + multi-import + custom name (backend)`

---

## Phase 3 — Frontend: pack UI

### 3.1 — Pixel modal helper (`src/lib/modal.ts` — NEW)

简单的 pixel-style modal，导入 + delete confirm 复用：

```ts
export function pixelPrompt(opts: {
  title: string;
  defaultValue?: string;
  okLabel: string;
  cancelLabel: string;
}): Promise<string | null> {
  return new Promise(resolve => {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    overlay.innerHTML = `
      <div class="modal-box">
        <h3>${opts.title}</h3>
        <input class="set-val" value="${opts.defaultValue ?? ""}" />
        <div class="modal-btns">
          <button class="pixel-btn off" data-act="cancel">${opts.cancelLabel}</button>
          <button class="pixel-btn" data-act="ok">${opts.okLabel}</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const input = overlay.querySelector<HTMLInputElement>("input")!;
    input.focus(); input.select();
    overlay.addEventListener("click", e => {
      const t = e.target as HTMLElement;
      if (t.dataset.act === "ok") { resolve(input.value || null); cleanup(); }
      if (t.dataset.act === "cancel") { resolve(null); cleanup(); }
    });
    input.addEventListener("keydown", e => {
      if (e.key === "Enter") { resolve(input.value || null); cleanup(); }
      if (e.key === "Escape") { resolve(null); cleanup(); }
    });
    function cleanup() { overlay.remove(); }
  });
}

export function pixelConfirm(opts: { title: string; okLabel: string; cancelLabel: string }): Promise<boolean> {
  // 类似，没有 input，返回 boolean
}
```

CSS 加在 `pixel.css`：

```css
.modal-overlay { position: fixed; inset: 0; background: rgba(45,45,95,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-box { background: var(--c-white); border: 3px solid var(--c-ink);
  padding: 12px; min-width: 240px; display: flex; flex-direction: column; gap: 8px; }
.modal-box h3 { margin: 0; font-size: 12px; letter-spacing: 1px; }
.modal-btns { display: flex; gap: 8px; justify-content: flex-end; }
```

### 3.2 — packs.ts 改造

```ts
host.innerHTML = `
  <ul class="pack-list" role="listbox">
    ${packs.map(p => `
      <li class="pack-row ${p.id === state.active_pack ? 'sel' : ''}"
          data-id="${p.id}" data-bundled="${p.bundled}">
        <span class="pack-name-text">${p.name}</span>
        <span class="pack-row-actions">
          <span class="meta">${p.id === state.active_pack ? '♪' : ''}</span>
          ${p.bundled ? '' : `<button class="pack-del" data-act="delete">✕</button>`}
        </span>
      </li>
    `).join("")}
    <li class="pack-import" data-action="import">${t("packs.import")}</li>
  </ul>`;
```

CSS：
```css
.pack-row-actions { display: flex; gap: 6px; align-items: center; }
.pack-del { background: var(--c-pink); border: 1px solid var(--c-ink);
  width: 16px; height: 16px; cursor: pointer; padding: 0; line-height: 1;
  font: inherit; font-size: 10px; }
.pack-del:hover { background: #f59ec2; }
```

### 3.3 — Click handlers

- 点击 row（非 ✕ 按钮）：optimistic UI + setActivePack（保持现有逻辑）
- 点击 ✕：阻止冒泡 + `pixelConfirm({ title: t("packs.delete_confirm", { name }), ... })`，OK 时 `invoke("delete_pack", { id }) → renderPacks(host)`
- 点击 import：先 file picker → 然后 `pixelPrompt({ title: t("packs.name_prompt"), defaultValue: <stem from filename>, ok: OK, cancel: CANCEL })` → `invoke("import_pack", { archivePath: path, customName: name })` → `renderPacks`

> **Edge case**：用户在 import 后 cancel 命名 → 仍然导入，name 用 manifest 默认（pass `customName: null`）。**OR** cancel = 取消整个 import？倾向于后者更直觉，与 file picker cancel 一致。

### 3.4 — i18n keys（5 locale 都加）

- `packs.delete_confirm` → "Delete '{name}'?"
- `packs.name_prompt` → "Pack name"
- `common.ok` → "OK"
- `common.cancel` → "CANCEL"

### 3.5 — Frontend wrapper (`src/lib/ipc.ts`)

```ts
export const deletePack = (id: string) => invoke<void>("delete_pack", { id });
export const importPack = (archivePath: string, customName: string | null) =>
  invoke<string>("import_pack", { archivePath, customName });
export interface PackSummary { id: string; name: string; bundled: boolean; }
```

### Steps
1. Create `src/lib/modal.ts`.
2. Modify `src/views/packs.ts` (delete button + custom-name prompt).
3. Modify `src/lib/ipc.ts` (deletePack wrapper, PackSummary type, importPack typed).
4. Modify `src/styles/pixel.css` (modal styles, pack-del button).
5. Modify 5 locale JSONs.
6. `npm run verify:i18n` + `npm run build` clean.
7. **Manual smoke**: import 同一个 zip 两次，确认两个 pack 都在；删一个，剩一个；删 bundled 应该看不到 ✕ 按钮.

**Commit:** `feat(v0.2.0): pack delete + custom name UI`

---

## Phase 4 — Frontend: pack pagination

**Goal:** PACKS 页面 pack 数 > 8 时分页；plain `←/→` 翻页（不带 Cmd）。

### 4.1 — View lifecycle (router.ts)

当前 router 切 tab 时：`hostScreen.innerHTML = ""; await views[tab](hostScreen);` — 没有 cleanup hook。我们要给每个 view 一个机会在切走时 dispose 自己的全局 listener。

```ts
type ViewCleanup = () => void;
type ViewFn = (host: HTMLElement) => Promise<ViewCleanup | void>;

let _currentCleanup: ViewCleanup | null = null;

async function activate(tab: TabId) {
  if (_currentCleanup) { _currentCleanup(); _currentCleanup = null; }
  active = tab;
  paintTabs();
  hostScreen.innerHTML = "";
  const cleanup = await views[tab](hostScreen);
  if (typeof cleanup === "function") _currentCleanup = cleanup;
}
```

四个 view 函数都要改签名，但只有 `packs` 真的会返回 cleanup（其他三个 `return undefined` 兼容）。

### 4.2 — Pagination state (packs.ts)

```ts
const PAGE_SIZE = 8;
let currentPage = 0;

export async function renderPacks(host: HTMLElement): Promise<() => void> {
  const [packs, state] = await Promise.all([listPacks(), getState()]);
  const totalPages = Math.max(1, Math.ceil(packs.length / PAGE_SIZE));
  if (currentPage >= totalPages) currentPage = totalPages - 1;

  function paint() {
    const slice = packs.slice(currentPage * PAGE_SIZE, (currentPage + 1) * PAGE_SIZE);
    host.innerHTML = `
      <div class="pack-pager">
        <button class="pixel-btn off" data-pg="prev" ${currentPage === 0 ? 'disabled' : ''}>◀</button>
        <span>${t("packs.page", { n: currentPage + 1, total: totalPages })}</span>
        <button class="pixel-btn off" data-pg="next" ${currentPage >= totalPages - 1 ? 'disabled' : ''}>▶</button>
      </div>
      <ul class="pack-list">
        ${slice.map(...).join("")}
        ${currentPage === totalPages - 1 ? '<li class="pack-import">...</li>' : ''}
      </ul>`;
    bind();
  }

  function bind() { /* row click + delete + import + pager nav (handled below) */ }

  function onKey(e: KeyboardEvent) {
    if (e.metaKey || e.ctrlKey) return;  // 让 Cmd+←/→ 切 tab 走 router 那条
    if (e.key === "ArrowLeft" && currentPage > 0) { currentPage--; paint(); }
    if (e.key === "ArrowRight" && currentPage < totalPages - 1) { currentPage++; paint(); }
  }
  document.addEventListener("keydown", onKey);

  paint();
  return () => document.removeEventListener("keydown", onKey);
}
```

> **设计权衡**：`currentPage` 用模块级变量（不是 component state），所以切走再回来还在原页码 — 用户体验更好（切到 SETTINGS 看一眼再回来不会 reset）。但 `import_pack` 之后强制跳到最后一页（新 pack 在那），手动 `currentPage = totalPages - 1` 在 import 成功后调用。`delete_pack` 之后保持在同一页（如果还有内容）或回退一页（如果当前页空了）。

### 4.3 — pack-import 入口的处理

import 入口本来是列表最后一行。在分页里：
- 只在最后一页显示
- 或：永远在最后一页底部（保持当前行为，slice + import 行）— 选这个，更直觉

### 4.4 — i18n keys

- `packs.page` → "{n}/{total}"

### 4.5 — CSS

```css
.pack-pager { display: flex; align-items: center; justify-content: center;
  gap: 8px; margin-bottom: 6px; font-size: 11px; }
.pack-pager button { padding: 1px 8px; }
.pack-pager button[disabled] { opacity: 0.3; cursor: default; }
```

### Steps
1. Modify `src/lib/router.ts` — view fn 返回 `Promise<ViewCleanup | void>`，cleanup 调用.
2. Modify `src/main.ts` — view 函数签名同步更新（其他三个 return void OK）.
3. Modify `src/views/packs.ts` — page state + keyboard handler + cleanup.
4. Add `packs.page` to 5 locales.
5. CSS pager.
6. `npm run verify:i18n` + `npm run build`.
7. **Manual smoke**: import 多个 pack 凑够 9+ 个 → 翻页 OK；切 tab 出去再回来 page 保持；卸载/import 后 currentPage 不越界.

**Commit:** `feat(v0.2.0): pack list pagination with arrow-key nav`

---

## Phase 5 — Better default sound packs (asset 工作)

**Status: 需要用户确认要不要做这个 phase。**

如果做，proposed approach：

1. 用户协助在 https://mechvibes.com/sound-packs/ 或 mechvibes 仓库 issues 里挑 4-7 个真录音 + license 兼容（CC0 / CC-BY / 公有领域）的 pack：
   - cherry-mx-blue（clicky）
   - cherry-mx-red（linear）
   - cherry-mx-brown（tactile）
   - cherry-mx-black（heavy linear）
   - alps-orange / topre / model-m（如果想拉开音色差异）
2. 验证每个 pack 的 license + author + 提供 attribution
3. 替换 `src-tauri/packs/cherry-{blue,red,brown}/sound.ogg`，新增 `src-tauri/packs/cherry-black/`（或 model-m 等）
4. 每个 pack 的 `config.json`：
   - 加 `"license": "CC0"` / `"author": "Mechvibes/Username"`
   - 保持 `key_define_type: single` 形式（统一）
5. README 更新 `## Sound Packs included` 章节 + ATTRIBUTION
6. 可能需要 `packs/LICENSE.md` 列出每个 pack 的来源

**风险：**
- License 审查不能 100% 自动化 — 用户可能要花时间挑 pack
- .dmg 体积可能从 10.4MB 涨到 ~20-25MB（每个真录音 ~2-3MB）
- 每个 pack 我都得重新 cargo test（fixture 不变 OK，但 packs/ 变化会让 `loads_all_four_default_packs` 测试需要更新断言）

**建议：v0.2.0 不做 Phase 5，先 ship 改良后的 import UX，让用户自己装社区 pack。Phase 5 留给 v0.2.1 或 v0.3.0 — 那时候可以更专注地做 license curation。**

如果用户坚持 v0.2.0 做，我们至少需要：用户先选 packs（提供链接），我再做 import + manifest patching + tests + README。

---

## Phase 6 — i18n verify + ship

**Steps:**
1. `npm run verify:i18n` 确保 5 locale 一致.
2. `cargo test --all` 全绿.
3. `npm run build` 一次性 + `cargo check` 一次性 — no warning.
4. README 加 `## v0.2.0 changelog` 子章节（pack delete / multi-import / pagination / settings compact）.
5. `git tag v0.2.0 && git push origin v0.2.0` → 触发 release.yml workflow.
6. 监控 GH Actions：构建大约 3-5 min（warm cache，refs Phase 14.1 baseline 3m21s）.
7. Release 出来后 manual smoke：下载 dmg、xattr、安装、走一遍核心流程：import / delete / 翻页 / 改 pack 名 / 切 tab.
8. 用户验证后 update Cask formula（v0.1.0 同样流程）.

**Commit:** `chore(v0.2.0): tag release` (实际只是 git tag，不一定有 source 改动 — 这一步 README changelog 加完之后再 tag).

---

## Risks & open questions

### Open questions for user (回信确认时回答)

**Q1**：Phase 5（替换默认 packs 为真录音）要不要包含在 v0.2.0？还是 ship v0.2.0 = 1+2+3+4+6，Phase 5 留给 v0.2.1？
**我的建议**：留给 v0.2.1。v0.2.0 已经 4 phase 工作量。

**Q2**：custom-name prompt 在用户取消时，是 cancel 整个 import，还是仍然导入用 manifest 默认 name？
**我的建议**：cancel 整个 import（与 file-picker cancel 行为一致）。

**Q3**：删除 imported pack 时，要不要把当前 active pack 是它自己的情况处理掉（自动切回第一个 bundled pack）？
**我的建议**：要。否则 active_pack 指向不存在的 id，dispatcher 会 fail. 可以放在 `delete_pack` 后端逻辑里：检查 settings.active_pack == id → 改 active_pack 为 store.ids().first()，save_settings.

**Q4**：分页 pageSize 8 合适吗？还是要根据窗口高度算？
**我的建议**：硬编码 8 起步，根据用户测试调。

### Risks

- **R1**: `LoadedPack.dir_name` 改动会让 `pack_store::tests::*` 都要更新（构造 LoadedPack 时多一个字段）。已计入 Phase 2 工作量。
- **R2**: `list_packs` 返回新 field `bundled` 会让所有 frontend `PackSummary` 类型断言更新。grep `PackSummary` 现在只在 ipc.ts → 一处.
- **R3**: 修改 router.ts view-fn 签名（return cleanup）→ 4 个 view 文件全部改一行（`async function renderXXX(host) { ... }`）. 简单.
- **R4**: 1.5 行 settings UI（time start–end 同行）在某些 locale 下 label 太长会换行 — 需要测试 zh-CN/ja/ko 别破版.
- **R5**: pixel modal 在 transparent 窗口上的 backdrop 渲染 — 需要确认 `inset: 0` overlay 不会盖住 frame 外部.
- **R6**: 用户的 settings.json 已经存在（v0.1.x 保存的），加 `bundled_ids` 不影响 settings 文件 schema（bundled 是 PackStore in-memory 状态，不持久化）— 安全.

### Plan deviations expected

照预期会有：
- **#33**：bundle helper 函数命名（用 `pixelPrompt` / `pixelConfirm` vs alternative `showModal`）.
- **#34**：分页可能采用 page size = 8 还是基于高度计算（最简单的 8）.
- **#35**：i18n key naming convention `packs.page` → 用 `{n}/{total}` interpolation 还是 `Page {n} of {total}` 模板.

---

## Memory updates expected

执行每个 phase 完成后：
- 更新 `bubblekeys_progress.md`：last completed task / commit / 文件 diff stats
- v0.2.0 ship 后：状态切回"在 0.2.x patch 模式"

---

## Total estimate

- Phase 0：5 min
- Phase 1：30 min（settings UI 是最简单 + 立即可视化的 — 第一个做能 build confidence）
- Phase 2：45 min（Rust 改动最多的一块）
- Phase 3：45 min（modal helper + UI 重写）
- Phase 4：40 min（router lifecycle + page state）
- Phase 5：— skip for v0.2.0 unless user requests
- Phase 6：20 min

**v0.2.0 总：~3 小时**（不含 Phase 5）

---

## Next step (waiting for user)

1. 用户 review 这份 plan
2. 确认/修改回答 Q1-Q4
3. 用户给绿灯后，我从 Phase 0 开始执行（subagent-driven，每 phase 一个 fresh agent）
