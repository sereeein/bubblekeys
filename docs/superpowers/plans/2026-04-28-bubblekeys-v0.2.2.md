# BubbleKeys v0.2.2 — Plan

**Created:** 2026-04-28
**Author:** Claude
**Branch strategy:** all phases on `main`; tag `v0.2.2` at the end
**Predecessor:** v0.2.1 (commit `aa279b8`, factory reset + about layout patch)

## Goals (from user)

扩充内置音色包，使开箱即可对比 8 大主流机械轴体声音。当前 4 个内置包覆盖
红轴 / 茶轴 / 青轴 + 自创 bubbles，缺以下 5 种：

- **黑轴**（线性，重弹簧，闷）
- **银轴 / 速度轴**（线性，行程短，尖脆）
- **静音红轴**（红轴 + 硅胶圈，办公友好）
- **紫轴**（段落，比茶轴更明显，层次感更强）
- **白轴**（点击，比青轴稍轻，收敛）

新包地位与现有 4 包**完全等同**：bundled / 受删除保护 / 首启动复制到 user pack dir。

## Decisions confirmed by user (2026-04-28)

1. **命名前缀**沿用 `cherry-*` 风格（与 `cherry-red/blue/brown` 一致）。
2. **静音红轴 id** = `cherry-red-silent`。
3. **去掉 `synth-placeholder` tag**（5 个新包不带该 tag；同步从现有 3 个
   cherry pack 移除该 tag，新老保持一致）。
4. **合成工具链**：Python + numpy + `oggenc`（与现有 fixture 同链路）。

## Out of scope

- 真实采样录音（受设备限制；社区已有 Mechvibes 资源，可由用户后期 import）。
- Mechvibes 包默认 bundle（license 审查工作量另起 PR）。
- v0.3 路线（菜单栏 agent / lazy CJK 字体 / Apple Developer 签名）。

---

## Phases overview

```
Phase 1: 合成脚本 scripts/synth-packs.py        — ~60 min
Phase 2: 试听 + 用户拍板 (HUMAN GATE)            — 用户主导，迭代调参
Phase 3: 写入 bundled 包 + 清理 placeholder tag  — ~25 min
Phase 4: 测试 + 版本号                          — ~20 min
Phase 5: 预发布 wipe + tag                      — 用户授权
```

执行顺序：**1 → 2 → 3 → 4 → 5**。Phase 2 是显式人工门，不会跳过。

---

## Phase 1 — 合成脚本

**Files touched:** `scripts/synth-packs.py` (new), `package.json` (新增 npm
script), `.gitignore` (忽略 preview 输出)。

**Deliverables:**

- `scripts/synth-packs.py` — 独立 CLI 工具，运行后在 `out/synth-packs/` 下
  为每个新轴产出一个 `.wav` (16-bit PCM, mono, 44.1 kHz, ~150 ms) 与一个
  `.ogg` (`oggenc -q 4`)。文件名 = pack id（如 `cherry-black.ogg`）。
- 每个轴一个合成函数，共享一个 `BaseHit` 原语（短指数衰减冲击 + 滤色噪声）：

  | id | 设计参数 |
  |---|---|
  | `cherry-black` | 基频 80–120 Hz，衰减 70 ms，峰值幅度 0.55（重弹簧"沉"） |
  | `cherry-silver` | 攻击 5 ms，衰减 35 ms，谱中心上移（高通 800 Hz），整体尖脆 |
  | `cherry-red-silent` | 复用 cherry-red 包络 × 强低通 (1.5 kHz) + 整体 −6 dB |
  | `cherry-purple` | 复用 cherry-brown 段落包络 + 第二共振峰增强 +3 dB（"层次感"） |
  | `cherry-white` | 复用 cherry-blue 双段点击包络 × −2 dB + 高频拖尾收紧 |

- 同时把 WAV 链接写到 `out/synth-packs/PREVIEW.md` 作为试听清单。
- `.gitignore` 添加 `/out/`。
- `package.json` 加 `"synth:packs": "python3 scripts/synth-packs.py"`。

**Dependencies:**
- `python3` + `numpy`（系统已有；如缺则 `pip3 install numpy`）。
- `oggenc`（`brew install vorbis-tools`）。

**Verification:**
- `npm run synth:packs` 退出 0。
- `out/synth-packs/` 下 5 个 `.wav` + 5 个 `.ogg`，每个 OGG ≤ 8 KB。
- `file out/synth-packs/cherry-black.ogg` 显示 `Ogg data, Vorbis audio`。
- 不修改 `src-tauri/packs/`。

---

## Phase 2 — 试听 + 用户拍板（HUMAN GATE）

**Process:**
1. 我跑 `npm run synth:packs`，把 5 个 `.wav` 路径列给用户（VSCode 里点开
   即可播放）。
2. 用户按任意子集打回（"黑轴太亮，沉一点" / "银轴还不够脆" 等）。
3. 我调 Phase 1 脚本里的对应函数参数，重新合成，再次提交试听。
4. **直到用户对全部 5 个轴明确通过**，才进入 Phase 3。

**Anti-pattern:** 不跳过，不"差不多就行"。声音是这版的核心交付，必须用户
亲耳确认。

---

## Phase 3 — 写入 bundled 包 + 清理 placeholder tag

**Files touched:** 5 × `src-tauri/packs/<id>/{config.json,sound.ogg}` (new),
3 × 现有 `src-tauri/packs/cherry-{red,blue,brown}/config.json` (tag 清理)。

**新建 5 个目录：**

```
src-tauri/packs/
├── cherry-black/
│   ├── config.json
│   └── sound.ogg
├── cherry-silver/
├── cherry-red-silent/
├── cherry-purple/
└── cherry-white/
```

**config.json 模板**（按现有 cherry-red 写法）：

```json
{
  "id": "cherry-black",
  "name": "Cherry Black",
  "key_define_type": "single",
  "sound": "sound.ogg",
  "includes_numpad": true,
  "license": "CC0-1.0",
  "author": "BubbleKeys",
  "tags": ["mechanical", "cherry", "black", "linear"]
}
```

各包字段：

| id | name | tags |
|---|---|---|
| `cherry-black` | Cherry Black | `["mechanical", "cherry", "black", "linear"]` |
| `cherry-silver` | Cherry Silver | `["mechanical", "cherry", "silver", "linear", "speed"]` |
| `cherry-red-silent` | Cherry Red Silent | `["mechanical", "cherry", "red", "linear", "silent"]` |
| `cherry-purple` | Cherry Purple | `["mechanical", "cherry", "purple", "tactile"]` |
| `cherry-white` | Cherry White | `["mechanical", "cherry", "white", "clicky"]` |

**OGG 来源**：直接从 `out/synth-packs/<id>.ogg` 复制到
`src-tauri/packs/<id>/sound.ogg`（Phase 2 已经过用户确认）。

**清理 placeholder tag：**

- `cherry-red/config.json` tags 从 `["mechanical","cherry","red","linear","synth-placeholder"]` 改为 `["mechanical","cherry","red","linear"]`。
- `cherry-blue/config.json` tags 从 `["mechanical","cherry","blue","tactile-clicky","synth-placeholder"]` 改为 `["mechanical","cherry","blue","clicky"]`（同时把 `tactile-clicky` 修正为标准的 `clicky`，与新 white 保持一致）。
- `cherry-brown/config.json` tags 从 `["mechanical","cherry","brown","tactile-bumpy","synth-placeholder"]` 改为 `["mechanical","cherry","brown","tactile"]`（同样把 `tactile-bumpy` 规范化）。
- `bubbles/config.json` 不动（自创风，tags 现状 `["bubble","soft","original"]` 没有 placeholder 词）。

**注意：**
- `tauri.conf.json` 的 `bundle.resources: ["packs/**/*"]` 是 glob，**无需改动**。
- `install_default_packs` 自动扫目录，**无需改 Rust 代码**。

---

## Phase 4 — 测试 + 版本号

**Files touched:** `src-tauri/src/pack_store.rs` (test 重命名 + 扩展),
`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`,
`README.md` (CHANGELOG)。

**Test 更新：**

`src-tauri/src/pack_store.rs::tests::loads_all_four_default_packs` →
重命名 `loads_all_default_packs`，断言 9 个 id：

```rust
for expected in [
    "cherry-blue", "cherry-red", "cherry-brown", "bubbles",
    "cherry-black", "cherry-silver", "cherry-red-silent",
    "cherry-purple", "cherry-white",
] {
    assert!(ids.contains(&expected.to_string()), "missing pack: {expected}");
}
```

**版本号：**
- `src-tauri/Cargo.toml` `version = "0.2.1"` → `"0.2.2"`。
- `src-tauri/tauri.conf.json` `"version": "0.2.1"` → `"0.2.2"`。
- `package.json` `"version"` 同步（如存在）。

**README CHANGELOG 段落：**

```markdown
### v0.2.2 — 2026-04-28
- Add 5 new bundled sound packs covering the rest of the
  mechanical-switch lineup: Cherry Black, Cherry Silver,
  Cherry Red Silent, Cherry Purple, Cherry White.
- Drop `synth-placeholder` tag from existing Cherry packs;
  normalize tactile/clicky tags across the bundled set.
```

**Verification:**
- `cargo test --all` 全绿（应 = 23 + 0 = 23 passed；test 数不变，只是名字
  换 + 断言扩展）。
- `npm run verify:i18n` ✓。
- `npm run build` clean。
- `cargo check` clean。

---

## Phase 5 — 预发布 wipe + tag

**强制规则**（来自记忆 `feedback_pre_release_wipe`）：tag 前 wipe
`~/Library/Application Support/BubbleKeys/`（含 dev/prod 双目录），让
fresh-install 验证有意义。

**Steps:**
1. `rm -rf "~/Library/Application Support/BubbleKeys"` (用户授权)。
2. `npm run tauri build`（约 5–10 min）。
3. 打开生成的 .app，手动验证：
   - PACKS 列表第 1 页（PAGE_SIZE=8）应显示 8 个包；翻第 2 页显示第 9 个 +
     `+ IMPORT MECHVIBES` 行。
   - 每个新包点击切换后能预览（▶ 按钮播放对应轴音）。
   - 每个新包**没有** × 删除按钮（bundled 保护）。
4. `git push origin main`（用户授权后执行）。
5. `git tag v0.2.2 && git push origin v0.2.2` —— 触发 release.yml
   universal build → draft release（用户授权后执行）。

**Stop & confirm before each `git push` / `git tag`.** Auto mode 不覆盖
"对外可见动作 / 不可逆动作"的确认门。

---

## Risks & open questions

- **静音红轴**与红轴是否听感差异够明显？低通滤波 + 音量降可能仍像红轴
  的低音版。Phase 2 试听若用户觉得"太像红轴"，调整方向：进一步缩短
  attack + 加更重低通（cutoff 800 Hz）+ 引入轻度谐波抑制。
- **银轴**的"尖脆"边界：太尖会刺耳。Phase 2 重点核对。
- **9 个包 = PAGE_SIZE 8 + 1**，正好让用户必须翻第 2 页才能看到
  `+ IMPORT MECHVIBES`。这是预期行为（Phase 4 v0.2.0 设计如此）但 UX 上
  值得在 README 提一句"导入按钮在最后一页"。
- **OGG 体积膨胀**：当前 .dmg ~10 MB，5 个新 OGG × ~5 KB ≈ 25 KB，可忽略。

---

## Memory updates after ship

- `bubblekeys_progress.md` 追加 v0.2.2 ship 记录（commit + tag + 用户确认
  时间）。
- 不新增 feedback memory（这次没新规则）。
