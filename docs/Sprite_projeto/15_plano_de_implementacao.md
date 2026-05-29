# 15 — Plano de implementação (W0 → W7)

## 15.0 Princípios

- **Padrão-ouro absoluto.** Nada de "v1 que dá pro gasto" ([feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)). Cada wave fecha a erro-zero após auditoria adversarial rotacionada.
- **Forma: funnel** (como Painter, Vector, Node). Neck serial (Coord-A only, foundational) → FREEZE → fan-out paralelo (Implementadores).
- **Smoke do Enio em cada wave.** Critério visual concreto, não verbal.
- **Sem deferral.** Gaps conhecidos viram trabalho na sessão atual antes de ratificar.

## 15.1 W0 — Spec freeze

**Objetivo:** spec completa + 6 ADRs novos + 1 ADR amendment Accepted + arch tests vacuous-pass + fixtures v3 binárias geradas.

### Tasks (W0)

| Task | Descrição | Critério |
|---|---|---|
| **T0.1** | Spec completa em `docs/Sprite_projeto/` (16 arquivos) | Este doc + 15 irmãos shipados |
| **T0.2** | ADR-0069 — Sprite Inspector v2 (decisão-mãe) | Accepted |
| **T0.3** | ADR-0070 — Sprite schema v4 (`Sprite::VERSION` 3→4) | Accepted |
| **T0.4** | ADR-0071 — Tint channels multiplicativos canônicos | Accepted |
| **T0.5** | ADR-0072 — Named Anchor unification | Accepted |
| **T0.6** | ADR-0073 — Sorting canonical order | Accepted |
| **T0.7** | ADR-0074 — Sprite-vs-Component boundary princípio | Accepted |
| **T0.8** | Auditoria adversarial multi-lente (≥2 agentes paralelos, lentes rotacionadas) | Findings classificados (Crítico/Alto/Médio/Baixo) |
| **T0.9** | Correção a erro-zero | Todos findings Crítico+Alto fechados |
| **T0.10** | Re-auditoria pós-correção (3ª lente: Determinism + Multi-OS) | Sem novos findings Crítico+Alto |
| **T0.11** | **ADR-0025-amendment-1 — Skew em Transform (formal cascade)** ✨ POST-AUDIT | Accepted |
| **T0.12** | **Gerar 5 fixtures v3 binárias `crates/ph2d-render/tests/fixtures/`** ANTES do bump v3→v4 ✨ POST-AUDIT | atlas.postcard + atlas_with_anchor + individual + premultiplied + max_size geradas e commited |
| **T0.13** | **Empirical test: postcard + `SpriteVersioned` wrapper enum** ✨ POST-AUDIT | Test em `crates/ph2d-render/tests/sprite_versioned_postcard.rs` carrega v3 fixture + asserta `migrate_v3_to_v4` produz defaults benignos. Valida postcard semantics ANTES de W1. |
| **T0.14** | Ratificação Enio dos 6 ADRs + 1 amendment | W0 fechada |

### Critério de fechamento W0
- 6 ADRs novos + 1 amendment (0025-amendment-1) Accepted.
- Spec (16 arquivos) sem TODO ou "TBD".
- Audit lens diversity ≥ 2; rotacionadas entre rounds.
- **5 fixtures v3 binárias geradas e commited** (T0.12 — pre-bump v3→v4).
- **Empirical postcard test verde** (T0.13 — valida `SpriteVersioned` wrapper enum).
- Memória atualizada (`project_sprite_w0_ratified_<data>.md`).
- Anúncio em SESSION_ACTIVE.md como Coord-A pausado/concluído.

### Audit lenses sugeridas (W0)
1. **Lens A — Escopo + gaps:** o spec é completo? Hi findings na pesquisa multi-engine não incluídos? Algo OUT que deveria ser IN?
2. **Lens B — ABI + back-compat:** Sprite v3→v4 migrator é byte-correct? RenderInstance ABI passa em todos os gates? `#[serde(default)]` cobre TODO campo novo?
3. **Lens C — UI canon + a11y:** Inspector layout respeita Widget Gallery? Cada widget novo tem entry no showcase? HR-12 AccessKit em TODO widget novo?
4. **Lens D — Determinismo + multi-plataforma:** Ordering pipeline (Seção 7) é byte-identical cross-OS? Per-corner tint vertex color é determinístico?

## 15.2 W1 — Schema bump strategic-only

**Objetivo:** `Sprite::VERSION = 4` no código + RenderInstance ABI nova + migrator + zero feature visível.

### Tasks (W1)

| Task | Descrição | Critério |
|---|---|---|
| **T1.1** | Adicionar campos novos ao `Sprite` struct (15 campos a mais) | Compila em `ph2d-render` |
| **T1.2** | Helpers de default (`default_white`, `default_one`, etc.) | Funções const |
| **T1.3** | Constructors `atlas()` / `individual()` atualizados | Retornam v4 com defaults benignos |
| **T1.3.5** | **`libm` crate dep em `ph2d-ecs/Cargo.toml`** ✨ POST-LENS-C C1 | `libm = "0.2"` + substituir `f32::sin_cos` por `libm::sincosf` em `Transform::compose` v1 atual; gate `transform_determinism` re-validado cross-OS pre-amendment-1 |
| **T1.4** | Migrator v3 → v4 (`migrate_v3_to_v4`) | Função + testes |
| **T1.5** | ~~5 fixtures binárias v3~~ → **MOVIDO PARA T0.12** (post-audit; fixtures geradas pre-bump) | já feito em W0 |
| **T1.7a** | `RenderInstance` v4 ABI compile-time check (11 attrs, 144 bytes) | `vertex_attr_offsets_match_struct` verde |
| **T1.7b** | **Criterion bench bandwidth** (Lens E E26 fix — split de T1.7 anterior): `sprites_upload_144b_vs_72b` — 10k sprites @ 60Hz; assert frame budget <8ms M-series tier; trigger condition pra dual-buffer mitigation (ADR-0070 §2.5). | Criterion p95 < 8ms M-series; > triggers ADR-0070-amendment-1 dual-buffer |
| **T1.6** | Test `migrate_sprite_v3_to_v4` + `sprite_versioned_postcard` (já parcial em T0.13) | Carrega fixtures → asserta defaults benignos |
| **T1.7** | `RenderInstance` v4 ABI (11 attrs, 144 bytes) | `vertex_attr_offsets_match_struct` verde |
| **T1.8** | Extract phase: collapse tint cascade (`tint * self_tint * Π(ancestors)`) | Sistema ECS |
| **T1.9** | Extract phase: per-corner tint cópia direta | Vai pra @location(9..12) |
| **T1.10** | Extract phase: flip_uv bitfield encoding | Encoded em `RenderInstance.flip_uv` |
| **T1.11** | Sprite shader (WGSL) atualizado para v4 | Lê per_corner_tint interpolado + tint_fill + opacity |
| **T1.12** | Arch-gate `architecture_sprite_inspector_surface` | Cap **20 fields** enforçado (Lens D D1 reconciliado em 8 sites) |
| **T1.13** | Auditoria adversarial pós-impl | ≥2 lentes rotacionadas |
| **T1.14** | Correção a erro-zero + commit local | Commit |

### Critério de fechamento W1
- `cargo test -p ph2d-render` verde.
- `cargo clippy -p ph2d-render --all-targets -- -D warnings` verde.
- Fixtures v3 → v4 carregam sem perda.
- Smoke do Enio: cenário visual existente continua renderizando IDÊNTICO (zero regression visual).
- **Nenhuma feature nova visível ainda** — só fundação ABI/schema.

## 15.3 W2 — Inspector v2 seções 1-6 + OKLCH ColorPicker

**Objetivo:** Inspector com Identity · Transform (com skew) · Render Source ampliada · Sprite Sheet · 9-Slice · Color & Tint completos.

### Tasks (W2)

| Task | Descrição | Critério |
|---|---|---|
| **T2.1** | Refactor `sections.rs` (574 LOC atual) → `sections/{transform,color_tint,ordering,mask_clip,sampling,material_blend,animation,sockets,...}.rs` módulos | HR-18 LOC cap; gate `inspector_section_loc_cap` removido `#[ignore]` |
| **T2.2** | Adicionar `Transform.skew_x/y` (vide [ADR-0025 amendment-1](../architecture/decisions/0025-amendment-1.md) já Accepted em T0.11) — bump `Transform::VERSION` 1→2 + migrator wrapper enum `TransformVersioned` + `compose()` math T·R·Sk·S via **libm::tanf + sincosf** + GlobalTransform skew propagation + reescrever comment em `transform_determinism.rs` (não claim cross-OS dentro do mesmo processo — Lens C M3) | Compila em `ph2d-ecs` + `propagate_transforms` aplica skew cascade + 3 fixtures Transform v1 binárias + gate `transform_compose_with_skew_determinism` cross-OS verde via libm |
| **T2.3** | Seção 2 Inspector — Transform com Skew | Sliders editáveis |
| **T2.4** | Seção 3 Inspector — Render Source ampliada (Region toggle + Region Filter Clip) | Funciona |
| **T2.5** | Seção 4 Inspector — Sprite Sheet inline (Centered + Offset + Flip + HFrames/VFrames + Frame) | Editável |
| **T2.6** | Component `SliceNine` + Seção 5 Inspector — 9-Slice (toggle add/remove) | Sliced/Tiled mode visíveis |
| **T2.7** | Widget novo: `OKLCH ColorPicker` | Showcase + AccessKit + arch-tests verdes |
| **T2.8** | Seção 6 Inspector — Color & Tint (Tint + Self Tint + Per-corner + Tint Fill + Opacity) | Editável |
| **T2.0** | **Widget `BulkSelectInspector` precondição (movido de W6.T6.3 pós-Lens-D D14)** — primitivo "Mixed" placeholder antes de Color & Tint section usar. | Widget compila + AccessKit `Indeterminate` state + showcase |
| **T2.9** | Bulk-edit multi-select (Color & Tint section em W2) usando T2.0 widget canônico | Aplica a N sprites sem ad-hoc "Mixed" |
| **T2.10** | Auditoria adversarial pós-impl | ≥2 lentes rotacionadas |
| **T2.11** | Correção a erro-zero + commit local | Commit |

### Critério de fechamento W2
- Smoke do Enio (visual checklist):
  - [ ] Tint cascateia pra filhos (selecionar pai, mexer Tint, filhos tingem juntos).
  - [ ] Self Tint NÃO cascateia (filhos imunes).
  - [ ] Per-corner tint produz gradient diagonal (TL=red, BR=blue).
  - [ ] Tint Fill ativa = silhueta colorida; desativa = volta normal.
  - [ ] Opacity slider preserva RGB.
  - [ ] Skew X/Y editável; transform aplica.
  - [ ] Region toggle + Rect + Filter Clip funcionam (atlas sem bleeding).
  - [ ] 9-Slice toggle funcional; corners não distorcem.

## 15.4 W3 — Seções 7-9 (Sorting · Visibility · Sampling)

**Objetivo:** Inspector com Ordering · Visibility · Sampling completos + gate de regressão ClipChildren.

### Tasks (W3)

| Task | Descrição | Critério |
|---|---|---|
| **T3.1** | Component `ZIndexOverride(i32)` + `ZAsRelative(bool)` | Compila |
| **T3.2** | Component `SortingLayer(LayerId)` + Project Settings registrar layers | Funciona |
| **T3.3** | Component `OrderInLayer(i32)` | Funciona |
| **T3.4** | Component `YSort { enabled, axis, sort_point }` | Cascateia |
| **T3.5** | Component `SortingGroup { sort_at_root }` | Multi-piece char unit-sort |
| **T3.6** | Component `ShowBehindParent` (marker) | Filho renderiza antes do pai |
| **T3.7** | Component `TopLevel` (marker) | Quebra cascata |
| **T3.8** | Pipeline canônico de ordenação (extract phase) | 7 estágios respeitados |
| **T3.9** | Component `ClipChildren(Mode)` + 3 modos funcionais | Test fixtures pixel-comparison |
| **T3.10** | Component `MaskInteraction { mode, alpha_cutoff }` | Funciona (stub Mask2D em W3) |
| **T3.11** | Component `TextureFilter` + `TextureRepeat` hierárquico | Per-node override funciona |
| **T3.12** | Component `VisibilityLayer(u32 bitmask)` + Camera2D cull_mask | Bitmask aplica |
| **T3.13** | Component `OnScreenEnabler { rect, mode }` | Auto-disable processing |
| **T3.14** | Seção 7 Inspector — Ordering / Sorting (todos os widgets) | Editável |
| **T3.15** | Seção 8 Inspector — Visibility | Editável |
| **T3.16** | Seção 9 Inspector — Sampling | Editável |
| **T3.17** | `OrderDebugOverlay` widget + Component toggle | Overlay visual no canvas |
| **T3.18** | Test `clip_children_regression` (3 fixtures) | Verde, gate ativo |
| **T3.19** | Test `sorting_pipeline_determinism` cross-OS | Hash identical |
| **T3.20** | Auditoria adversarial pós-impl | ≥2 lentes |
| **T3.21** | Correção a erro-zero + commit local | Commit |

### Critério de fechamento W3
- Smoke do Enio:
  - [ ] Z Index = 5 com ZAsRelative=true cascada pro filho.
  - [ ] Show Behind Parent: filho desenha atrás do pai.
  - [ ] YSort em hierarchy: char mais "embaixo" desenha na frente.
  - [ ] ClipChildren ClipOnly: filhos recortados, pai invisível.
  - [ ] Mask Interaction VisibleInside: sprite só dentro do mask.
  - [ ] Texture Filter Nearest no mundo + Linear na UI lado-a-lado funcionam.
  - [ ] Order Debug Overlay: ativando, vejo cor da SortingLayer + Z label em cada sprite.

## 15.5 W4 — Seções 10-11 (Material&Blend · Animation)

**Objetivo:** Material slot + Use Parent Material + Instance Shader Params + Blend Mode + Animation inline.

### Tasks (W4)

| Task | Descrição | Critério |
|---|---|---|
| **T4.1** | Component `Material(MaterialRef)` | Default sprite material works |
| **T4.2** | Component `UseParentMaterial` (marker) | Filhos compartilham material instance |
| **T4.3** | Component `InstanceShaderParams(SmallVec<[(StringKey, Value); 8]>)` | Per-instance uniforms sem clone |
| **T4.4** | Component `BlendMode(Mode)` (6 modos) | Pipeline state muda por sprite |
| **T4.5** | Asset `SpriteFrames` (schema v1) com Tags | Cooker integration |
| **T4.6** | Component `SpriteAnimator` runtime | Sistema ECS avança frame |
| **T4.7** | Sistema `animate_sprites` (tick) | duration_ms per-frame respeitado |
| **T4.8** | Direction modes (Forward/Reverse/PingPong/PingPongReverse) | Funcionam |
| **T4.9** | Hold ms + Repeat Delay ms (Phaser) | Pausas no loop respeitadas |
| **T4.10** | Signals (FrameChanged, AnimationFinished, Looped) | Emite via ActionBus |
| **T4.11** | Seção 10 Inspector — Material & Blend | Editável |
| **T4.12** | Seção 11 Inspector — Animation (SpriteAnimator) | Editável |
| **T4.13** | Botão "Open in Timeline" (stub — abre dialog "coming soon") | Funcional |
| **T4.14** | Auditoria adversarial pós-impl | ≥2 lentes |
| **T4.15** | Correção a erro-zero + commit local | Commit |

### Critério de fechamento W4
- Smoke do Enio:
  - [ ] Material instance shader params variáveis per-sprite SEM clone (CPU profile mostra 1 material handle).
  - [ ] 6 Blend Modes funcionam (Mix/Add/Sub/Mul/Screen/PremultAlpha).
  - [ ] Animation toca com duration_ms per-frame respeitado.
  - [ ] PingPong + Hold + Repeat Delay funcionam visualmente.

## 15.6 W5 — Seção 12 (Named Anchors)

**Objetivo:** sistema unificado NamedAnchor + visual handles + per-frame override.

### Tasks (W5)

| Task | Descrição | Critério |
|---|---|---|
| **T5.1** | `NamedAnchor` struct + `AnchorData` enum | Compila |
| **T5.2** | Component `NamedAnchorList(SmallVec<[NamedAnchor; 4]>)` | Anexável |
| **T5.3** | `SpriteFrame.named_anchors` per-frame override | Lookup resolve fallback |
| **T5.4** | Widget `NamedAnchorEditor` (lista + add/remove) | Showcase + AccessKit |
| **T5.5** | Widget visual handles no canvas (drag socket/slice) | Drag handle gizmo |
| **T5.6** | Seção 12 Inspector — Sockets/Slices | Editável |
| **T5.7** | API runtime `entity.named_anchor("muzzle")` | Returns NamedAnchor |
| **T5.8** | Aseprite slice → NamedAnchor import (lossless) | Cooker bridge |
| **T5.9** | Auditoria adversarial pós-impl | ≥2 lentes |
| **T5.X.1** | `CameraFollowAnchor` Component + sistema de update (Lens E E15) | Compila + smoke "câmera segue muzzle do sprite player" |
| **T5.X.2** | `named_anchor_lookup_perf` bench (Lens E E19) — 100 entities × 32 anchors × 60Hz lookup; assert < 100μs/frame | Criterion verde |
| **T5.10** | Correção a erro-zero + commit local | Commit |

### Critério de fechamento W5
- Smoke do Enio:
  - [ ] Adiciono socket "muzzle" no Inspector; visual handle aparece no canvas; drag move.
  - [ ] Anexo particle emitter filha pelo socket; emite no socket point.
  - [ ] Sprite anima 4 frames de attack; muzzle se move automaticamente (per-frame override).
  - [ ] Slice nomeada "face_box" com bounds editáveis via handles.
  - [ ] 9-slice region nomeada "dialog_bg" com bounds + center editáveis.

## 15.7 W6 — Foundational widgets novos

**Objetivo:** Widget Gallery completo com widgets novos do Inspector v2.

### Tasks (W6)

| Task | Descrição | Critério |
|---|---|---|
| **T6.1** | **Estender `BlenderColorPicker` existente** (1790 LOC em [widget/blender_color_picker/](../../crates/ph2d-editor-core/src/widget/blender_color_picker/)) para expor OKLCH como modo selecionável + emit `OklchColor`. NÃO reinventar (Lens D D9). | Showcase polished + AccessKit role `ColorWell` + `ph2d-color::OklchColor` output |
| **T6.2** | Widget `NumericInputWithUnit` (`px`/`m`/`deg`/`rad`/`%`) | Suffix parse |
| **T6.3** | ~~Widget `BulkSelectInspector`~~ → **MOVIDO PARA W2.T2.0** (Lens D D14: era precondição de W2.T2.9). | já feito em W2 |
| **T6.4** | Widget `BitmaskGrid32` (4×8 checkbox) | Visibility Layer + VisibilityCullMask use |
| **T6.4.1** | Widget `Rect2Editor` (Lens D D17) — 4 NumberInputs (x/y/w/h) + canvas handles para drag bounds/center. Reaproveitável em OnScreenEnabler, NamedAnchorEditor bounds/center, Region Rect §3.3. | Showcase + AccessKit + arch-test |
| **T6.4.2** | Widget `VariantEditor` (Lens D D18) — dropdown kind + sub-widget per variant (None/Str/Int/Float/Color/Dict). Recursivo (Dict contém VariantEditor anêcs). Cap depth ≤ 4 (§7.12). | Showcase + AccessKit `Role::Group` + variant-kind ComboBox + recursive children |
| **T6.5** | Widget `NamedAnchorEditor` (refinement pós-W5; consume Rect2Editor + VariantEditor de T6.4.1+T6.4.2) | Polished |
| **T6.6** | Widget `OrderDebugOverlay` (refinement pós-W3) | Polished |
| **T6.7** | Widget `SegmentedAdaptive` (8-region 9-slice modes) | Funcional |
| **T6.8** | Widget `KeyValueList` (Instance Shader Params editor) | Funcional |
| **T6.9** | Widget Gallery showcase entries para TODOS os widgets novos | Arch gate verde |
| **T6.10** | Auditoria adversarial pós-impl | ≥2 lentes |
| **T6.11** | Correção a erro-zero + commit local | Commit |

### Critério de fechamento W6
- `architecture_widget_showcase_coverage` verde.
- `hr12_widgets_a11y` verde para todos os widgets novos.
- `no_literal_color` verde.
- Widget Gallery panel mostra cada widget novo com sample interativo.

## 15.8 W7 — Polish + i18n + a11y + bug bash

**Objetivo:** v1.0 declarável.

### Tasks (W7)

| Task | Descrição | Critério |
|---|---|---|
| **T7.1** | Fluent strings i18n para todas as labels (`sprite.section.*`) | en-US + pt-BR bundles |
| **T7.2** | WCAG 2.2 AA audit completo | Verde |
| **T7.3** | AccessKit per-widget review | Roles + labels corretos |
| **T7.4** | Smoke completo dos 8 itens "pequenos com impacto desproporcional" | Visual checklist |
| **T7.5** | Bug bash do Enio + correções | Lista resolvida |
| **T7.6** | Performance gates (HR-3 + HR-4) | Bench cap |
| **T7.7** | Hotkeys canônicos registrados (F, Shift+F, R, Y, B, etc.) | Funcionam |
| **T7.8** | Documentation update final | README + ADRs atualizados se mudou |
| **T7.9** | Memória atualizada (`project_sprite_inspector_v2_complete_<data>.md`) | Saved |
| **T7.10** | Commit + push (após autorização Enio) | CI verde |

### Critério de fechamento W7
- Sprite Inspector v2 v1.0 declarado.
- CI matrix verde cross-OS.
- 8 "itens pequenos com impacto desproporcional" checados visualmente.
- Memory atualizada com estado pós-v1.0.

## 15.8.1 CI feat/** matrix override (Lens C H4 fix)

Wave 10 + Etapa 0.4 fizeram CI determinism job rodar SÓ em `push main` / `pull_request to main` ([`.github/workflows/spike.yml:144`](../../.github/workflows/spike.yml#L144)). Sprite Inspector v2 W3-W6 vai produzir N feat/** branches em fan-out paralelo; Implementadores **não veem cross-OS gates** até o PR final.

**Decisão pós-audit:**

Tasks que tocam determinism (especificamente W3.T3.8 pipeline canônico + W3.T3.19 cross-OS hash test + W2.T2.2 transform skew determinism + W4.T4.7 sprite_animator determinism) DEVEM ser empurradas para branches com naming convention `feat/sprite-determinism-*`. Coord adiciona override em `spike.yml`:

```yaml
on:
  push:
    branches: [main, 'feat/sprite-determinism-*']
  pull_request:
    branches: [main]
```

OU Coord abre PR draft `main ← feat/sprite-determinism-*` cedo (antes de feature completa) — qualquer push para a branch dispara matrix completo via PR rule.

**Documentado em CLAUDE.md** quando W3 abrir.

## 15.8.2 Smoke fixture canônica (Lens D D6)

Smoke checklists em W2-W5 atuais têm itens não-pixel-identifiable ("Tint cascateia pra filhos" sem hierarquia fixa). Pós-Lens-D: **scene fixtures + screenshots golden obrigatórios.**

**Fixtures path canônico:** `assets/smoke_fixtures/sprite_inspector_v2/`

| Fixture | Wave | Conteúdo | Screenshots golden em `docs/Sprite_projeto/smoke_goldens/` |
|---|---|---|---|
| `smoke_w2_color_tint.scene` | W2 | 5 sprites em hierarquia 3-níveis: root (tint=cyan) → child A (self_tint=red) + child B (per_corner gradient TL=red BR=blue) + grandchild C (opacity=0.5) + sibling D (tint_fill=true, fill=green) | `w2_tint_cascade.png`, `w2_self_tint_local.png`, `w2_per_corner_gradient.png`, `w2_opacity_independent.png`, `w2_tint_fill_silhouette.png` |
| `smoke_w3_sorting.scene` | W3 | 10 sprites em hierarquia 4-níveis com mix: SortingLayer named (BG/Player/UI), Z Index ±5, YSort top→bottom, SortingGroup multi-piece char, ShowBehindParent shadow filho | `w3_y_sort_topdown.png`, `w3_z_index_relative.png`, `w3_show_behind_parent.png`, `w3_sorting_group_block.png`, `w3_clip_children_3_modes.png` |
| `smoke_w4_material_animation.scene` | W4 | 6 sprites: 1 com Material custom; 2 com UseParentMaterial; 3 com InstanceShaderParams variando hue_shift; 1 SpriteAnimator com 4-frame walk anim (durations 100/100/100/200ms; pingpong) | `w4_use_parent_material_batching.png`, `w4_instance_params_variation.png`, `w4_anim_pingpong_60fps.png` |
| `smoke_w5_named_anchors.scene` | W5 | 3 sprites: 1 com socket "muzzle" + particle filho anexado; 1 com slice "face_box" + bounds editor; 1 com 9-slice region "dialog_bg" + center editor | `w5_socket_attach.png`, `w5_slice_bounds_drag.png`, `w5_9slice_region_drag.png`, `w5_per_frame_anchor_override.png` |

**Smoke protocol** (Lens D D6 fix):

```
1. Enio: `./play.command --load smoke_fixtures/sprite_inspector_v2/<wave>.scene`
2. Cada checklist item refere screenshot golden com pixel-identifiable reference:
   - W2.S1: "Pixel (200, 100) deve ser #00FFFF8E (cyan tint cascateado de root)"
   - W2.S2: "Pixel (300, 100) deve ser #FF0000FF (red self_tint local, sem cyan cascade)"
   - W2.S3: "Pixel (50, 50) TL = #FF0000FF; Pixel (250, 50) TR = #FFFFFFFF; gradient diagonal visível"
3. Enio confirma ✓ (golden idêntico ±epsilon) ou ⚠ (diff > epsilon → bug)
4. Diff > epsilon → Coord diagnostica + fix + re-smoke
```

**Gate `smoke_fixture_renderable`** (W0.T0.X cria): cada fixture carrega no `./play.command` sem panic; goldens existem em `smoke_goldens/`. Sem isso, smoke é teatral.

## 15.9 Frequência de FREEZE

- **W0** congela schema v4 + 6 ADRs + caps numéricos.
- **W2** congela API pública dos canais de tint (matemática multiplicativa).
- **W3** congela pipeline canônico de sorting + ClipChildren regression gate.
- **W5** congela `NamedAnchor` schema.
- **W7** congela Inspector v2 v1.0 inteiro.

Mudanças pós-FREEZE = ADR-amendment (custo deliberado).

## 15.10 Risk register

| Risco | Probabilidade | Mitigação |
|---|---|---|
| RenderInstance v4 144 bytes virar gargalo de bandwidth | Médio | Bench em W1; mitigação dual-buffer documentada em ADR-0070 |
| ClipChildren regredindo entre waves (padrão Godot) | Alto | Gate `clip_children_regression` summary-stats cross-OS desde W3; pixel-comparison Linux-only; smoke do Enio em cada wave |
| Skew em Transform quebrar `propagate_transforms` existente | Médio | Test fixtures W2.T2 antes de wire; cross-test com hierarchy 3-níveis + `libm` para cross-OS determinism (Lens C C1) |
| `f32::tan` / `sin_cos` cross-OS divergence (Lens C C1) | **Alto** (originalmente Baixo — sub-estimado) | `libm` crate pure-Rust em `ph2d-ecs` Cargo.toml; sweep policy "todos transcendentals em SimWorld via libm::*" |
| OKLCH ColorPicker complexidade UI | Médio | Showcase iterativo (não one-shot); pode usar fallback HSV em casos extreme |
| Per-corner tint não-determinístico cross-OS (FP precision) | Baixo (CPU collapse) / Exempt (GPU varying) | Test `tint_math_multiplicative_canonical` cross-OS + epsilon comparison + traversal order gate (H3) |
| **SpriteAnimator `f32` time accumulator divergir cross-OS (Lens C C4)** | **Alto** | Fixed-point `elapsed_ticks: u64` desde dia 1 + `speed_scale_q16_16: i32` (não f32) |
| **Skew additive em cascade aproximação (Lens C C5)** | Médio | Convenção leaf-only documentada em ADR-0025-amendment-1 §2.2.1; sem enforcement (custo runtime); doc UX no slider hint |
| **Postcard back-compat assumption (Lens C C3)** | **Alto** | Wrapper enum `*Versioned` PRIMARY path; `#[serde(default)]` defesa-em-profundidade; W0.T0.13 empirical test obrigatório |
| Bulk-edit explodir em complexidade Inspector | Médio | Restringir W2 a Color & Tint section; expandir gradualmente W3-W6 |
| Audit lenses convergindo (mesma lente N rounds) | Médio | Rotação canônica documentada em [feedback-audit-lens-diversity](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md); Lens C aplicada com 82% findings originais |

## 15.11 Coord assignments (após W0)

- **W1 (foundational schema)** — Coord-A only (mexe em `ph2d-render` `ph2d-ecs`).
- **W2-W6 (Inspector seções + Components)** — Mix: Coord-A para Component novos em `ph2d-ecs` (caminho C); Coord-B para sections em `ph2d-panel-inspector` (caminho B scaffold); Implementadores em paralelo (caminho A drop-crate quando aplicável — ex: cada Component novo poderia ser crate satélite se complexidade justificar).
- **W7 (polish + i18n + a11y)** — qualquer agente.

## 15.12 Smoke do Enio — protocolo

Em cada wave, **antes do commit final**, Coord pede:

> "Enio, rode `./play.command` e verifica:
> 1. [item 1 com descrição visual concreta]
> 2. [item 2]
> ...
> N. Sem regressão visual em outras features."

Enio responde:
- ✅ "Todos OK" → wave fecha.
- ⚠️ "Item N quebrado" → Coord diagnostica + fix + re-smoke.

Memória [feedback-smoke-at-end](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_smoke_at_end.md): smoke é UMA vez no fim da wave, não a cada commit.

## 15.13 Anti-padrões observados em planos anteriores (Painter, Vector)

1. **17 waves no Painter** (escopo gigante) — Sprite Inspector v2 limita a 7 waves para preservar momentum.
2. **W0 espalhado em "design + impl"** — Sprite Inspector W0 é PURO docs/ADRs (zero código).
3. **Smoke verbal sem checklist** — cada wave aqui tem checklist explícita.
4. **Audit lens repetida em rounds** — rotacionamos.
5. **ADRs criados sem auditoria adversarial** — W0 obriga ≥2 lentes antes de Accept.
