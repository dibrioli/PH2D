# HANDOFF — Sprite Inspector v2 · FINALIZAÇÃO (fecho W0–W3 + roadmap W4–W8)

> **Self-contained.** Estado real do módulo em 2026-05-31, pós W3 Phase 6.
> Supersede o tracker W0-era [`HANDOFF_sprite_inspector_v2.md`](HANDOFF_sprite_inspector_v2.md)
> pra o estado atual. Autor: sessão solo Coord+Impl de finalização.

---

## §0 — Status executivo

**W0, W1, W2, W3 = COMPLETOS** (render + UI + gates + smoke do Enio). Tudo em
**commits LOCAIS** (≈30 ahead de `origin/main`), prontos pro ship desta jornada.
**W4–W8 = NÃO INICIADOS** (waves futuras, ratificadas uma a uma — NÃO
auto-implementadas nesta sessão por disciplina de ADR/audit por-wave).

Esta sessão de finalização fez: (1) W3 Phase 6 (ClipChildren stencil + §8
Visibility + Mask2D); (2) fechou o débito de ADRs (amendments 5/6/7 + 0074-amд-1
+ 0073-amд-1); (3) auditoria multiagêntica adversarial; (4) ship + babysit CI.

---

## §1 — O que está PRONTO (W0–W3)

| Wave | Escopo | Estado |
|---|---|---|
| **W0** | Spec freeze (17 docs, 6 ADRs 0069-0074 + 0025-amд-1, fixtures v3, postcard empírico) | ✅ Ratificado (5 lentes) |
| **W1** | Schema v3→v4 (`Sprite` 20 campos, `SpriteVersioned`, ABI v4, migrator, libm cross-OS) | ✅ |
| **W2** | Seções 1-6 (Identity·Transform+skew·Render Source·Sprite Sheet·Color&Tint OKLCH) + BulkSelect + GlobalTint cascade | ✅ Pushado (CI verde) |
| **W3** | Seções §7 Ordering · §8 Visibility · §9 Sampling + fundação ECS | ✅ Local |

### §1.1 — As 12 seções canônicas (spec §3)

| # | Seção | Estado | Wave |
|---|---|---|---|
| 1 | Identity (Name/Tags/Notes) | ✅ | W2 |
| 2 | Transform (+skew) | ✅ | W2 |
| 3 | Render Source | ✅ | W2 |
| 4 | Sprite Sheet | ✅ | W2 |
| 5 | **9-Slice** | ❌ NÃO FEITO | W2-resto/W4 |
| 6 | Color & Tint | ✅ | W2 |
| 7 | Ordering/Sorting (11 controles) | ✅ | W3 |
| 8 | Visibility (Visible + VisibilityLayer 4×8 + ClipChildren + MaskInteraction + Mask Source + OnScreenEnabler) | ✅ | W3 Phase 6 |
| 9 | Sampling (Filter/Repeat + UV tiling/scroll) | ✅ | W3 |
| 10 | **Material & Blend** | ❌ NÃO FEITO | W4 |
| 11 | **Animation** | ❌ NÃO FEITO | W4 |
| 12 | **Sockets/Slices (Named Anchors)** | ❌ NÃO FEITO | W5 |

`LIVE_SECTION_IDS` hoje = **8** (índice 1 = Visibility, agora com a seção §8
completa). Vira `[_;12]` ao adicionar 9-Slice/Material/Animation/Sockets.

### §1.2 — Render de stencil (W3 Phase 6, o mais novo)

- **ClipChildren** (Godot): recorta filhos pela silhueta do pai. 3 modos. Stencil.
- **Mask2D + MaskInteraction** (Unity SpriteMask): fonte mascara responders
  (Inside/Outside). Stencil compartilhado, escopo global.
- ABI `clip_group`+`clip_meta` CPU-tail (184 B; GPU 164/12 attrs intacto).
- `clip_pass.rs`: pass normal → clip pass → mask pass. Alocação lazy de stencil.
- Gates pixel-exatos em GPU: `clip_children_regression` + `mask_interaction_regression`.

---

## §2 — Ledger de ADRs (débito FECHADO)

| ADR | Conteúdo | Estado |
|---|---|---|
| 0070-amд-2 | `SpriteVersioned` back-compat | ✅ |
| 0070-amд-3 | `flip_uv` flags bitfield | ✅ |
| 0070-amд-4 | `rotation`→`basis` (skew) | ✅ |
| 0070-amд-5 | `sampling: u32` CPU-tail (TextureFilter/Repeat) | ✅ **escrito agora** |
| 0070-amд-6 | `UvTransform` + `uv_xform` @loc15 (tiling/scroll) | ✅ **escrito agora** |
| 0070-amд-7 | `clip_group`+`clip_meta` (clip+mask ABI) | ✅ |
| 0073-amд-1 | Z-before-YSort (reconcilia §5.1 vs §5.2) | ✅ **escrito agora** |
| 0074-amд-1 | Stencil (não back-buffer) + Mask2D Component | ✅ |
| 0070-amд-1 | Dual-buffer perf | ⏸️ RESERVADO (só se bench mostrar gargalo) |

---

## §3 — Carry-overs / limitações conhecidas (reais, deduped)

Nenhum BLOQUEIA ship. Todos documentados; viram trabalho de wave futura:

1. **z-order multi-pass:** clip/mask groups compõem POR CIMA do normal pass
   (não interleaved em z). Aceitável pros casos canônicos. Fix = ordenar
   passes por z ou OIT — wave futura.
2. **clip + mask no mesmo sprite:** clip tem precedência (stencil reusado por
   pass). Combinar = wave futura.
3. **MaskCustomRange** (scope por sorting-layer): não feito; máscara é global.
4. **OnScreenEnabler runtime:** o componente + UI existem; o SISTEMA que pausa/
   esconde off-screen não foi escrito (é dado + editor; runtime = W-futura).
5. **VisibilityLayer cull:** funciona no extract, mas sem UI de camera cull_mask
   (não há como o usuário ver o efeito sem editar a câmera).
6. **`snapshots.rs`** tem `// ph2d-loc-cap:` marker (acretou os produtores de
   snapshot W3; follow-up: extrair pros módulos `inspector_*`).
7. **Per-node sampling em texturas individuais** usa sampler global (atlas é
   per-sampling). Follow-up.
8. **Sorting layers customizadas** > 5 canônicas: sem UI de Project Settings.
9. **OrderDebugOverlay** (§7): omitido render-first (W4 phase-7).
10. **Smoke fixtures** `.scene` não existem (smoke é visual do Enio).

---

## §3.1 — Auditoria multiagêntica W3 Phase 6 (2026-05-31)

6 lentes adversariais (stencil/GPU · ABI/determinism · UI dispatch ·
cross-feature · test-coverage · resource/panic) + verificação adversarial
de cada finding. **Resultado: 1 HIGH + 1 MEDIUM + LOWs, todos sanados (commit
`05b16e9`):**

- **HIGH — contiguidade de clip-group:** o clip pass batcha por scan de runs
  consecutivos, mas o sort era só `(z_order,...)` → um membro com
  ZIndexOverride/YSort divergente, ou um sprite alheio interpolando por rank,
  quebrava o span → membros recortados **SUMIAM** (data loss, alcançável pela
  UI §7 Ordering já shipada). Fix: sort primário por clip-anchor
  (`clip_group-1` = rank do clip-parent) → grupo sempre contíguo. + teste e2e.
- **MEDIUM — stencil dimensionado pela winit size crua** podia divergir do
  game_rt num frame 0-dim transitório → panic de attachment-extent. Fix:
  dimensionar offscreen RTs pela `surface.size()` clampada.
- **LOWs:** AlphaCutoff RMW não fabrica mais MaskInteraction None em sibling
  sem máscara; docs `@location(16)`→`(5)`; prose de contagem 15→16; teste de
  mask-cutoff carving; semântica hidden/culled clip-parent + Mask2D source
  documentada (membros des-recortam / máscara some — itens §3.4/§3.5 abaixo).

Findings semânticos documentados (não-bugs, decisões de wave futura):
hidden/culled clip-parent un-clip dos filhos (Visibility é per-entity, sem
propagação); hidden Mask2D source = sem máscara; BulkSelect mixed flags
computadas mas não pintadas como indeterminate (consistente com §7/§9 — W7).

## §4 — Roadmap W4–W8 (waves futuras, NÃO feitas)

Cada wave = drop-in seguindo o padrão do módulo: spec → ADR(s) → render-first
→ UI stack (snapshot/event/build/apply mirror) → gate → audit → smoke → commit.

### W4 — Material & Blend (§10) + Animation (§11)
- **§10:** `Material` (asset slot) · `UseParentMaterial` (marker) ·
  `InstanceShaderParams` (KeyValue editor — **precisa VariantEditor, ver W6**) ·
  `BlendMode` (6: Mix/Add/Sub/Mul/Screen/PremultAlpha — GPU blend state por run).
- **§11:** `SpriteFrames` asset · `SpriteAnimator` runtime (elapsed_ticks u64
  fixed-point, já specado W0) · Current/Progress/Speed/Playing/Autoplay/
  Direction/Loop/Hold/RepeatDelay · botão OpenTimeline (stub).
- Entry: `SpriteAnimator` runtime é a peça grande (tick loop + frame select).

### W5 — Sockets/Slices · Named Anchors (§12)
- `NamedAnchor` schema (SortedSmallVec, já specado) · per-frame override ·
  NamedAnchorEditor (**precisa Rect2Editor + VariantEditor, W6**) · handles
  visuais no canvas (drag) · CameraFollowAnchor · import de slice do Aseprite.

### W6 — Foundational widgets (BLOQUEIA partes de W4/W5)
- **Rect2Editor** (4 NumberInputs + handles no canvas) — **§8 OnScreenEnabler
  já usa 4 NumberInputs soltos; W6 unifica.**
- **VariantEditor** (dropdown kind + sub-widget recursivo, cap depth ≤4) —
  needed por InstanceShaderParams (W4) + NamedAnchor user_data (W5).
- BlenderColorPicker→OKLCH extend · NumericInputWithUnit · BitmaskGrid32
  (o §8 já tem um bitmask 4×8 inline; W6 extrai pro widget reusável) ·
  KeyValueList · OrderDebugOverlay · Widget Gallery coverage.
- **Nota de scheduling:** W4/W5 referenciam widgets de W6. Ou puxa Rect2Editor/
  VariantEditor pra cima (definição inline na wave), ou faz W6 antes.

### W7 — Polish: i18n + a11y + bug bash
- Fluent i18n ~155 keys (en-US + pt-BR) + gate `sprite_inspector_i18n_keys_present`.
  **NOTA:** hoje as labels das seções (§7/§8/§9) são strings inglesas hardcoded
  (padrão de `sampling.rs`/`ordering.rs`/`visibility.rs`); W7 migra pro Fluent.
- WCAG 2.2 AA audit + AccessKit por-widget · 8 small items · bug bash · hotkeys
  · docs final · `LIVE_SECTION_IDS` → 12.

### W8 — Asset Cooker (wave separada, deferida)
- Aseprite full import + Linked Cels dedup-hash + PSD. Código NÃO existe em
  `tools/asset-cooker/src/` (só texture cooker). MCP Destructive Ops registry
  (§7.1.2) já specado pra os 7 destructive ops.

---

## §5 — Ship + CI (esta jornada)

Protocolo Coord (DIRETRIZ §8): `./scripts/ship.sh` (paridade-CI: fmt, clippy
`--all-targets`, machete, deny, audit, nextest `--cargo-profile ci-test`,
typos) → corrige todo `✗` → `git push origin main` → babysit CI até `success`.

**Anti-colisão:** `git add -- <só paths Sprite>`; foreign no working tree
(`.vscode/settings.json`, `docs/HANDOFF_imageio_*`, `docs/HANDOFF_ktx2_*`,
`docs/Painter_projeto/*`, `docs/UI_Fonts/`, `test_strip`) **NÃO entram**.

Link da run sempre fornecido: `https://github.com/dibrioli/PH2D/actions`.
