---
name: sprite-inspector-projeto
description: Spec do Inspector do Sprite v2 do PH2D — propriedades editáveis pelo artista no Inspector (12 seções) + schema do Sprite struct (v4) + princípio Sprite-vs-Component-ECS + arch gates. **W0 RATIFICADA 2026-05-28** via ADRs 0069..0074 + 0025-amendment-1 Accepted pós 5 lentes adversariais (147 findings; 31 CRITICALs fechados a erro-zero).
status: **ACCEPTED — W0 RATIFIED 2026-05-28**
data: 2026-05-28 (ratified)
decisor: Enio (aprovação explícita 2026-05-28)
---

# Sprite Inspector v2 — propriedades editáveis pelo artista no PH2D

## 0. Localização canônica e status

- **Doc canônico vivo:** este diretório. Spec **W0 RATIFICADA 2026-05-28** via ADRs 0069..0074 + 0025-amendment-1 Accepted.
- **Estado em 2026-05-28 (ratificada):** W0 fechada — **spec pós-pesquisa multi-engine** (Godot 2D · Unity 2D / URP · Unreal Paper 2D · Defold · GameMaker · Construct 3 · LÖVE · Phaser 3 · Aseprite) + **levantamento de comunidade** (Godot Proposals, Unity Discussions, Defold Forum, Reddit, GitHub Issues). 4 agentes adversariais paralelos retornaram 110+ features candidatas; pós-filtragem (escopo restrito ao Inspector, fora FX/Lighting/Physics/Onion/Camera) chegamos às **12 seções canônicas + ~70 propriedades**.
- **Família arquitetural:** subsistema foundational do renderer 2D — fronteira entre `Sprite` struct ([crates/ph2d-render/src/sprite.rs](../../crates/ph2d-render/src/sprite.rs)) + Components ECS ([ph2d-ecs](../../crates/ph2d-ecs/)) + Inspector panel ([ph2d-panel-inspector](../../crates/ph2d-panel-inspector/)). **Não** é Tool isolado (ADR-0040) nem Image Tool — é o **objeto canônico** que toda Image Tool e todo Tool editam.
- **Mandato:** padrão-ouro absoluto ([feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)). Sprite é (provavelmente) o objeto mais importante de jogos 2D — tem que ser **a melhor implementação já vista**. Tempo maior aceito como custo de superar Godot/Unity/Unreal simultaneamente.

## 1. Visão em uma frase

Um Inspector de Sprite que casa **profundidade-de-Godot** (skew, modulate hierárquico, clip_children, region_filter_clip, y_sort cascateado, parallax-por-objeto) com **per-vertex tint do Phaser** + **Self Tint independente** + **named anchors unificados** (socket Paper2D · slice Aseprite · image_point Construct) + **OKLCH color picker** + **order debug overlay** + **bulk-edit multi-select** — mantendo `Sprite` struct enxuto (POD) e tudo opcional como Component ECS anexável (ADR-0025 boundary).

## 2. Os quatro eixos

1. **Inspector como única fonte de verdade da edição.** Tudo que o artista mexe num sprite passa por uma das 12 seções. Sem propriedades escondidas em "Advanced ▶ Project Settings ▶ Sprite Defaults". Sem 3 caminhos pra mesma edição. Inspector = WYSIWYG do schema serializado.

2. **Sprite struct vs Component ECS = fronteira clara.** O `Sprite` carrega APENAS aparência intrínseca da imagem (POD, sempre presente, schema versionado). Tudo ortogonal (sorting, masking, anchors, animation, instance shader params) vai como Component ECS opcional. Ausência ≠ default. Princípio detalhado em [`02_components_ortogonais.md`](02_components_ortogonais.md).

3. **PH2D-native: serializável, determinístico opcional, LLM-first.** Schema do `Sprite` versionado com migrator obrigatório (HR-14, `Sprite::VERSION=4`). Per-corner tint cabe na ABI do `RenderInstance` sem quebrar `vertex_attr_offsets_match_struct`. Cada propriedade exposta via `#[lua_export]` → MCP toolset `sprite_*` (HR-10). Cores via `ph2d-tokens` OKLCH (HR-15).

4. **Coisas pequenas com impacto desproporcional.** Quatro pares complementares decisivos: `tint` + `self_tint` (herda vs não) · `z_index` + `z_as_relative` (absoluto vs hierárquico) · `centered` + `offset` (anchor logic vs scene logic) · `tint` flat + per-corner gradient. Quatro toggles minúsculos: Show Behind Parent · Top Level · Use Parent Material · Region Filter Clip. **A diferença entre "bom" e "padrão-ouro" mora nesses 8 itens.**

## 3. Escopo — IN (versão 1.0 do Inspector do Sprite)

**12 seções canônicas** (detalhe em [`03_inspector_secoes.md`](03_inspector_secoes.md)). Cap `inspector_section_count_canonical = 12` FROZEN (pre-W0 audit corrigiu inconsistência prévia "11"):

1. **Identity** — Name · Tags · Notes
2. **Transform** — Position · Rotation · Scale · **Skew X/Y** · Top Level · Reset · Look At
3. **Render Source** (existente, ampliada) — Strategy · Storage detail · Source W×H · **Region toggle + Rect** · **Region Filter Clip** · Pixel Format · Reimport
4. **Sprite Sheet** (inline) — **Centered** · **Offset (intrínseco)** · Flip H/V · **H Frames / V Frames** · Frame · Frame Coords
5. **9-Slice** — Draw Mode (Simple/Sliced/Tiled) · Borders L/T/R/B · Size · **Per-region tile mode (8 regiões)** · Tile Mode · Stretch Value · Fill Center
6. **Color & Tint** — **Tint** (modulate) · **Self Tint** (self_modulate) · **Per-corner tint** (4 cantos) · Tint Fill · Opacity
7. **Ordering / Sorting** — **Z Index** · **Z as Relative** · **Show Behind Parent** · Sorting Layer · Order in Layer · Y Sort Enabled · Sort Point · Custom Axis · Sorting Group · Sort At Root · Translucency Priority + Distance Offset · **Order Debug Overlay**
8. **Visibility** — Visible · Visibility Layer (bitmask) · **Clip Children mode** · **Mask Interaction** · Alpha Cutoff · On-Screen Enabler
9. **Sampling** — Texture Filter (per-node, hierarchical inherit) · Texture Repeat · Anti-halo / Edge Filtering
10. **Material & Blend** — Material slot · **Use Parent Material** · **Instance Shader Params** · **Blend Mode** (6 modos)
11. **Animation** (collapsible) — SpriteFrames · Current Animation (tag) · Frame · Frame Progress · Speed Scale · Playing · Autoplay · Direction · Loop · Hold ms · Repeat Delay
12. **Sockets / Slices** (Named Anchors) — lista com name + transform + bounds opc + center opc + user_data; per-frame override; visual handles no canvas

> Detalhe técnico de cada seção em arquivos numerados `01_*..09_*`. Schema versionamento em [`10_schema_versionamento.md`](10_schema_versionamento.md). Gates em [`11_arch_gates_e_caps.md`](11_arch_gates_e_caps.md).

## 4. Escopo — OUT (não-objetivos explícitos)

**Cada item abaixo é decisão consciente. Reverter exige ADR.** Detalhe em [`12_fora_de_escopo.md`](12_fora_de_escopo.md).

- ❌ **FX / shader chain por-sprite** (Outline, DropShadow, PaletteSwap, Dither, Glow, Blur, Distortion, ChromaticAberration, etc.). Vai para o **módulo Shader FX** dedicado, futuro. Inspector do Sprite NÃO duplica.
- ❌ **Sistema de luzes e sombras 2D** (Light2D, normal maps, LightMask, Shadow Caster 2D, Light Blend Styles, normal maps, secondary textures). Vai para o **módulo Lighting** dedicado, futuro.
- ❌ **Física / collision geometry** (Custom Physics Shape, Collider 2D, GameMaker collision masks). Vai para o **módulo Física** dedicado, futuro.
- ❌ **Onion skin no Inspector**. Onion skin vive na **timeline editor** (módulo Animation), não no Inspector de Sprite.
- ❌ **Pixel-perfect camera + sub-pixel snapping**. Propriedade da Camera2D / Viewport, não do Sprite. Vai para o **módulo Camera 2D** dedicado, futuro.
- ❌ **Frame events com payload tipado**. Pertencem à **timeline editor** (SpriteAnimator dispara, Inspector só mostra estado atual).
- ❌ **SpriteShape (terreno via spline)**. Subsistema próprio, não Inspector.
- ❌ **PSD importer / Aseprite importer** completo. Asset cooker territory.
- ❌ **Hot-reload de sprites** (file watcher). Propriedade global da app, não Inspector.

## 5. Filtro minimalista-Blender — princípios operacionais

Quatro regras em toda decisão (espelha Painter §5 e Vector Module §5):

### 5.1 Um caminho canônico por propriedade
Se Unity tem 3 jeitos de tintar (SpriteRenderer.color · Material._Color · MaterialPropertyBlock), o Inspector PH2D expõe **2 canais semânticos distintos** (Tint herdável + Self Tint local) **+ override por shader param em sub-seção**. Sem 3 botões na UI primária fazendo a mesma coisa.

### 5.2 Defaults excelentes; preferences escondidas
Defaults: `tint = WHITE`, `self_tint = WHITE`, `opacity = 1.0`, `z_as_relative = true`, `texture_filter = inherit`. O usuário típico nunca abre Material slot nem Instance Shader Params. Sub-seções (Sampling, Material&Blend) só interessam a power user.

### 5.3 Atalhos de teclado em desktop são primeira-classe
Hotkeys canônicos (registrados em `editor-core`): `F` flip horizontal, `Shift+F` flip vertical, `R` reset transform, `Ctrl+D` duplicate sprite, `Ctrl+[/]` z-index ±1, `Y` toggle Y Sort, `Shift+B` toggle Show Behind Parent (Lens D D16 — `B` reservado para Painter Brush activation; collision check obrigatório). Em iPad, gestos cobrem a mesma função via QuickMenu radial.

**Hotkey collision check vs sister specs** (Lens D D16):
| Hotkey | Sprite Inspector | Painter | Vector | Resolução |
|---|---|---|---|---|
| `B` | (era Show Behind Parent) | Brush activate | — | Sprite usa `Shift+B`; Painter mantém `B` |
| `F` | Flip H | (free) | (free) | OK |
| `R` | Reset Transform | (free) | Rotate tool | colisão pendente Vector — `R` em Sprite roda quando Inspector focused, Vector quando canvas focused |
| `S` | (free) | Smudge | Scale tool | OK |
| `Y` | Toggle Y Sort | (free) | (free) | OK |

Hotkeys são context-aware (Inspector focused vs canvas focused). DIRETRIZ §5.2 Widget Gallery enforce.

### 5.4 Power escondido atrás de sub-seções colapsáveis
12 seções, mas só 5 sempre abertas (Identity, Transform, Render Source, Sprite Sheet quando atlas, Color & Tint). Outras 7 colapsadas por default. A UI primária permanece minúscula.

## 6. Multi-plataforma desde W1

Inspector do Sprite é **chrome PH2D** existente — herda automaticamente desktop + iPad + iOS + Android + Web ([ph2d-host](../../crates/ph2d-host/) + 4 zonas ADR-0023). Não há feature-flag por plataforma; OKLCH color picker funciona em qualquer device (Vello vector renderer).

## 7. Mapping arquitetural ao PH2D

### 7.1 Onde cada coisa mora

| Camada | O que carrega | Crate |
|---|---|---|
| **`Sprite` struct (POD serde)** | Aparência intrínseca: source, size, tint, self_tint, per_corner_tint, tint_fill, opacity, flip_x/y, anchor, pivot_preset, centered, offset, hframes/vframes, frame, premultiplied | [`crates/ph2d-render/`](../../crates/ph2d-render/) (existente; expande) |
| **`RenderInstance` (POD Pod, GPU upload)** | Versão por-frame: world_pos, size, tint flat (collapsed do per-corner), anchor world-scaled, rotation, premultiplied, texture_id, z_order | [`crates/ph2d-render/`](../../crates/ph2d-render/) (existente; bump ABI) |
| **Components ECS opcionais** | Cada aspecto ortogonal — SortingGroup, YSort config, ZIndexOverride, ZAsRelative, ShowBehindParent, TopLevel, ClipChildren, MaskInteraction, AlphaCutoff, TextureFilter, TextureRepeat, ParentMaterial, InstanceShaderParams, BlendMode, OnScreenEnabler, VisibilityLayer, NamedAnchorList, SpriteAnimator | [`crates/ph2d-ecs/`](../../crates/ph2d-ecs/) + crates satélite |
| **Sub-asset companion** | SpriteFrames (tags + per-frame {duration_ms, pivot_override, image_points, slices}) · NamedAnchorList resource | [`crates/ph2d-asset/`](../../crates/ph2d-asset/) (existente; expande) |
| **Inspector panel (UI editora)** | Layout das 12 seções, populate, paint, event dispatch, snapshots por-frame | [`crates/ph2d-panel-inspector/`](../../crates/ph2d-panel-inspector/) (existente; expande seções) |
| **Foundational widgets** | OKLCH ColorPicker, NumericInputWithUnit, BulkSelectInspector, NamedAnchorEditor (visual handles), OrderDebugOverlay | [`crates/ph2d-editor-core/src/widget/`](../../crates/ph2d-editor-core/src/widget/) (expande Widget Gallery) |

### 7.1.1 Cross-feature integration map (Lens D D5)

Sprite Inspector v2 NÃO opera isolado. Mapping canônico:

| Sistema externo | Direção | Contract | Comentário |
|---|---|---|---|
| **Painter** (ADR-0043..0053) | Painter → Sprite | `EditorAction::SpritesCommittedFromPainter { entity, new_texture_id }` emitido quando Painter `take_pending_commit` Apply. Inspector subscribe + repaint Render Source section. | W2.T2.x integration test. Painter T1.9 active — Coord-A current. |
| **Vector Module** (ADR-0056..0068) | Vector → Sprite | `EditorAction::SpriteFromVectorRasterize { entity, vector_doc_id, ppm }` quando user invoca "Rasterize" no Vector Inspector. Sprite Inspector mostra "Source: Vector rasterized" hint em Render Source. | ADR-0062 painter-bridge bidirecional doc-side. W5+ cross-feature impl. |
| **5 RasterEditTool de produção** (BgRemoval, ColorEqualization, Padding, Upscale, EqualizeSizes) | Tool → Sprite (preview) | Cada Tool `current_preview()` produz texture; Inspector subscribe `EditorAction::SpriteTexturePreviewChanged { entity }` durante preview. Tool commit → `EditorAction::SpriteCommitted` final. | Wave 10 padrão existente. Inspector existente já lida — v2 expande. |
| **Asset cooker** (`tools/asset-cooker/`) | cooker → Sprite | Migrator v3→v4 (T0.12+W1.T1.4). **Aseprite import lossless** ([§7.9](07_named_anchors.md) + [§8.12](08_animation_inline.md)) e **Linked Cels dedup-hash** ([§12.20](12_fora_de_escopo.md)) prometidos pelo spec — **mas código cooker NÃO existe ainda** em [`tools/asset-cooker/src/`](../../tools/asset-cooker/src/). Lens D D5 finding crítico. | **Decisão pós-audit:** Aseprite/Linked Cels capabilities movidas para **wave separada `W8 Asset Cooker Integration`** (não bloqueia W0 ratificação) com ADR-0075 (futuro) cobrindo cooker contract. v1.0 Sprite Inspector ship sem Aseprite full import — apenas PNG/JPEG via existing imageio. |
| **`SpriteFrames` asset ↔ `ph2d-asset`** | cooker → asset | Schema v1 (vide [§8.3](08_animation_inline.md)) usa `ph2d-asset::AssetHandle<SpriteFrames>` quando asset crate amadurecer. W4 dependência. | Documentar em W4.T4.5 task — `ph2d-asset` deve aceitar schema. |
| **Luau / MCP** | runtime → Inspector | HR-10 `#[lua_export]` em cada Component novo (ZIndexOverride, SortingLayer, NamedAnchorList, etc.) + MCP toolset `sprite_*`. **HR-11 destructive ops canonical registry** abaixo (Lens E E3 fix). Sanitizer LLM token injection: `name`/`tag`/`anchor_name` strings clamp em UTF-8 64 bytes + character class `[a-zA-Z0-9_-]` (Lens E E4). | W2-W6 each Component impl exposes `#[lua_export]`. |

### 7.1.2 MCP Destructive Operations Canonical Registry (Lens E E3 fix)

Lista canônica de cada MCP op exposta pelo Sprite Inspector v2 com classificação destructive/mutative/read-only. HR-11 confirmation token obrigatório nas destructive. Sem este registry, gate é vazio.

| MCP op | Kind | requires_token (HR-11) | Justificativa |
|---|---|---|---|
| `sprite_get(entity)` | read-only | no | observation |
| `sprite_query(filter)` | read-only | no | enumeration |
| `sprite_anchor_get(entity, name)` | read-only | no | observation |
| `sprite_anchor_list(entity)` | read-only | no | enumeration |
| `sprite_anchor_set(entity, name, anchor)` | mutative (REJECT duplicate; vide [§7.4.1](07_named_anchors.md)) | no | criar/atualizar 1 anchor; idempotent semântica |
| `sprite_anchor_set_or_replace(entity, name, anchor)` | **destructive** | **yes** | sobrescreve anchor pre-existente; data loss potencial |
| `sprite_anchor_remove(entity, name)` | **destructive** | **yes** | deleta anchor; data loss |
| `sprite_clear_all_anchors(entity)` | **destructive** | **yes** | deleta TODOS anchors entity |
| `sprite_reset_v4_defaults(entity)` | **destructive** | **yes** | resetta TODOS campos pra default |
| `sprite_bulk_apply(filter, change)` | **destructive** (mass) | **yes** | aplicação em massa; rollback impossível |
| `sprite_attach_component(entity, name, payload)` | mutative | no | adiciona Component (ausência ≠ default; vide ADR-0074) |
| `sprite_remove_component(entity, name)` | **destructive** | **yes** | remove Component anexado; data loss se Component carregava state |
| `sprite_set_tint(entity, rgba)` | mutative | no | edita campo POD |
| `sprite_set_self_tint(entity, rgba)` | mutative | no | idem |
| `sprite_set_opacity(entity, f32)` | mutative (clamp [0,1]) | no | idem |
| `sprite_animator_play(entity, anim_name)` | mutative | no | inicia animação |
| `sprite_animator_clear_frames(entity)` | **destructive** | **yes** | deleta SpriteFrames atual |

**Gate `mcp_destructive_ops_registered_for_token`** (W2 cria): test itera registry + verifica cada destructive op em `MCP::destructive_op_ids()` exige token. Sem registry → gate vacuous.

**Contradição prévia resolvida (Lens E E3):** ADR-0072 §2.6 + §6 anteriormente diziam "HR-11 NÃO aplica" para `sprite_anchor_*` — afirmação **só vale para read-only (`get`/`list`/`set` non-replacing) e mutative (`set` com duplicate-reject)**; `sprite_anchor_set_or_replace`/`sprite_anchor_remove`/`sprite_clear_all_anchors` são destructive e exigem token.
| **Hierarchy panel** | bidirectional | Hierarchy drag-reorder altera DFS counter → Inspector da entity selecionada refletir. Inspector edita Z Index → Hierarchy visual order pode mudar (se Z dominante). Bidirectional via `EditorAction::EntityReorderedInHierarchy { entity, new_dfs }` + reverse `EditorAction::ZIndexChanged { entity }`. (Lens D D19) | Existing `ph2d-panel-hierarchy` already subscribes ECS changes; expand for Z reorderings. |

### 7.2 Contratos congelados que respeitam

- **`Tool` / `RasterEditTool` / `PanelEvent`** (ADR-0040 + 0041) — Inspector é **panel**, não tool. Não toca a superfície congelada.
- **`NodeOp` / `OpResolver` / `NodeManifest`** (ADR-0039) — Sprite **não é nó do grafo**. É entidade ECS. Não toca.
- **`Tool=10` / `RasterEditTool=5` / `PanelEvent=4`** caps — Inspector emite `PanelEvent::{Click, SetValue, Toggle, SelectOption}` (4 variants reais — vide [tool.rs:26](../../crates/ph2d-editor-core/src/tool.rs#L26)). Edição de texto (Name, Tags, Notes, NamedAnchor.name) usa **`WidgetEvent::TextChanged`** internamente no widget store, não `PanelEvent`. Sem cap-bump previsto em `PanelEvent`.
- **`Panel` trait + `PanelHostInternal`** (ADR-0029 Phase C.1) — Inspector segue padrão de `InspectorPanel` existente; novas seções adicionadas como sub-painters dentro de `sections.rs`.

### 7.3 Hard Rules aplicadas

| HR | Aplicação no Sprite Inspector |
|----|------------------------------|
| **HR-1** | Inspector é platform-agnostic; input via `ph2d-input`; OKLCH color picker via Vello (qualquer device com GPU). |
| **HR-3** | Inspector paint hot-path: zero `Box::new` / `Vec::push`-que-realoca. Thread-local snapshots padrão (vide [`crates/ph2d-panel-inspector/src/sync.rs`](../../crates/ph2d-panel-inspector/src/sync.rs) e `state.rs` Phase C.1 thread_local!{} pattern — Lens E E21 pointer fix). Per-corner tint cabe em 16 bytes extras na ABI sem alloc. **Gate `inspector_paint_no_alloc` em [11_arch_gates §11.2.6.1](11_arch_gates_e_caps.md)** (Lens E E8). |
| **HR-4** | Inspector paint <0.5ms baseline (medido em painéis atuais v1). v2 cap canonical por wave em [11_arch_gates §11.6.2.2](11_arch_gates_e_caps.md) (Lens E E9): W2 <0.5ms, W3 <0.7ms, W4 <1.0ms, W5 <1.2ms em M-series macOS; +50% em Linux. Per-corner tint é 4×4=16 bytes a mais, zero overhead de paint. **criterion benchmark p95 obrigatório por wave**. |
| **HR-5** | Sprite vive em `SimWorld` (canonical state). Determinismo aplica — schema serializado com ordem canônica de campos. |
| **HR-6** | Schema `Sprite::VERSION=4` content-addressed (blake3 implícito via postcard). |
| **HR-12** | Cada widget Inspector emite `accesskit::Node`. Color picker com role `Slider2D` + label "color picker OKLCH, L X, C Y, H Z". |
| **HR-14** | `Sprite::VERSION` bumpa de 3 → 4 com migrator obrigatório (back-compat via `#[serde(default)]` em campos novos). |
| **HR-15** | Strings via Fluent. `sprite.section.color_tint`, `sprite.field.self_tint`, etc. |
| **HR-18** | `sections.rs` decomposto se passar 500 LOC. Já tem ~574 LOC hoje — bump para v2 obriga split em `sections/{transform, color_tint, ordering, mask_clip, sampling, material_blend, animation, sockets}.rs`. |
| **Bevy ECS multi-thread** (Lens E E24) | Sprite + Components ortogonais (NamedAnchorList, ZIndexOverride, YSort, ClipChildren) mutated em multiple systems via Bevy scheduler default. Conflict resolution = `ResMut`/exclusive `Query`; Bevy detecta automaticamente conflicts (panic em runtime se sistema declarado mas tem write conflict). Inspector paint usa **snapshot thread-local** (não direct ECS access em hot path; vide HR-3). |

## 8. Roadmap por waves (proposta — ratifica em ADR-0069)

| Wave | Conteúdo | Critério de fechamento |
|------|----------|------------------------|
| **W0** | **Spec freeze** — este doc + 6 ADRs (0069..0074). | Todos os 6 ADRs Accepted + arch test `architecture_sprite_inspector_surface` passa vacuous. |
| **W1** | **Schema bump strategic-only** — `Sprite::VERSION=4` com novos campos (POD, `#[serde(default)]`), `RenderInstance` ABI nova passa em `vertex_attr_offsets_match_struct`, migrator v3→v4 fixtures. Zero feature visível ainda. | `cargo test -p ph2d-render` verde; legacy `.ph2d-scene` v3 carrega como v4 com defaults benignos. |
| **W2** | **Inspector v2 seções 1-6 (Identity até Color & Tint)** — refactor de `sections.rs` para módulos; adiciona Tint+SelfTint+PerCorner widgets; OKLCH picker integrado. | Smoke do Enio: cada um dos 4 canais de tint editável e visualmente correto. |
| **W3** | **Seções 7-9 (Sorting · Visibility · Sampling)** — Z Index + Z As Relative + Show Behind Parent + YSort variants + ClipChildren + Mask Interaction + TextureFilter/Repeat per-sprite. | Y-sort topdown funciona; Z absoluto vs relativo testado em hierarquia 3-níveis; ClipChildren funcional (gate de regressão). |
| **W4** | **Seções 10-11 (Material&Blend · Animation)** — Use Parent Material + Instance Shader Params + Blend Mode + Animation inline (SpriteAnimator component). | Material instance params variáveis per-sprite sem clone; blend modes (Mix/Add/Sub/Mul/Screen/PremultAlpha) testados. |
| **W5** | **Seção 12 (Named Anchors / Sockets / Slices)** — sistema unificado + visual handles no canvas + per-frame override. | Adicionar socket "Muzzle" num sprite, anexar particle filha pelo socket, ver socket mover com frame. |
| **W6** | **Foundational widgets novos** — OKLCH ColorPicker, NumericInputWithUnit, BulkSelectInspector, OrderDebugOverlay. | Widget Gallery completo; arch-gate `architecture_widget_showcase_coverage` verde. |
| **W7** | **Polish + i18n + a11y review + bug bash**. | WCAG 2.2 AA; strings 100% Fluent; per-widget AccessKit; smoke do Enio dos 8 itens "pequenos com impacto desproporcional". |

**Frequência de FREEZE:**
- **W0** congela schema v4 + ADRs.
- **W2** congela API pública dos canais de tint (matemática multiplicativa).
- **W5** congela formato de NamedAnchor.

## 9. Índice dos arquivos do spec

| Arquivo | Conteúdo |
|---------|----------|
| [00_visao.md](00_visao.md) | Visão executiva resumida (1 página). |
| [01_anatomia_canonica.md](01_anatomia_canonica.md) | `Sprite` struct campos v4 — POD, schema versionado, serde. |
| [02_components_ortogonais.md](02_components_ortogonais.md) | Princípio Sprite-vs-Component-ECS; matriz canônica; razão da fronteira. |
| [03_inspector_secoes.md](03_inspector_secoes.md) | Layout completo das 12 seções com cada widget, valor default, hotkey, AccessKit. |
| [04_color_tint_canais.md](04_color_tint_canais.md) | Matemática dos 4 canais (Tint herdável · Self Tint local · Per-corner 4 cantos · Opacity). Ordem de multiplicação. PremultAlpha. |
| [05_ordering_sorting.md](05_ordering_sorting.md) | Pipeline canônico de ordenação (Viewport → CanvasLayer → SortingLayer → YSort → Z + ZAsRelative → SortingGroup → ShowBehindParent → DFS counter). Gotchas. |
| [06_mask_clip.md](06_mask_clip.md) | ClipChildren (3 modos) + Mask Interaction + AlphaCutoff. Gate de regressão obrigatório (vs Godot 5 issues abertos). |
| [07_named_anchors.md](07_named_anchors.md) | Sistema unificado (socket Paper2D + slice Aseprite + image_point Construct) num único `NamedAnchor`. Per-frame override. Visual handles. |
| [08_animation_inline.md](08_animation_inline.md) | SpriteFrames com Tags (Aseprite) + per-frame duration_ms + per-frame pivot/anchors overrides. Hold + Repeat Delay (Phaser). |
| [09_sampling_e_material.md](09_sampling_e_material.md) | Texture Filter / Repeat per-node hierárquico (Godot) + Material slot + Use Parent Material + Instance Shader Params + Blend Mode. |
| [10_schema_versionamento.md](10_schema_versionamento.md) | `Sprite::VERSION` 3 → 4. Migrator obrigatório. Back-compat via `#[serde(default)]`. ABI `RenderInstance`. |
| [11_arch_gates_e_caps.md](11_arch_gates_e_caps.md) | Gates novos: `architecture_sprite_inspector_surface` · `vertex_attr_offsets_match_struct` (existente, ampliado) · cap counts. |
| [12_fora_de_escopo.md](12_fora_de_escopo.md) | FX, Lighting, Physics, Onion Skin, Pixel-Perfect Camera, Frame Events, Hot-Reload — todos OUT com razão concreta. |
| [13_referencias.md](13_referencias.md) | Pesquisa multi-engine + community sources. |
| [15_plano_de_implementacao.md](15_plano_de_implementacao.md) | Plano executável W0..W7 com tasks T-W.N granulares. |
| [16_i18n_catalog.md](16_i18n_catalog.md) | **Lens D D3 fix:** catálogo completo Fluent i18n strings (~155 keys) + bundle paths + naming convention + gate `sprite_inspector_i18n_keys_present`. |

## 10. Como contribuir com este spec

Spec é **vivo** até ADR-0069 Accepted. Sugestões:

1. Edite o arquivo específico (`NN_*.md`) com diff claro. Aponte a fonte da mudança (qual engine / qual feedback / qual issue da comunidade).
2. Spec contradiz código: spec ganha (pré-implementação). Pós-W1, código ganha e spec atualiza.
3. Decisão grande (adicionar canal de tint, mudar ordem de multiplicação, alterar `Sprite` struct): **exige ADR amendment** (`0069-amendment-N.md`).
4. LLM editando: leia [DIRETRIZ.md §0..§1.4](../IntegracaoMultiAgente/DIRETRIZ.md) antes de tocar em código.

## 11. ADRs a aprovar antes do W1 (6 ADRs novos + 1 amendment)

| ADR | Título | Conteúdo |
|-----|--------|----------|
| **ADR-0069** | Sprite Inspector v2 — decisão-mãe | Spec freeze; cita ADRs irmãs; mandato padrão-ouro; escopo IN/OUT congelado. |
| **ADR-0070** | Sprite schema v4 (`Sprite::VERSION` 3→4) | Novos campos POD; migrator v3→v4; `RenderInstance` ABI bump (per-corner tint cabe em 16B); `#[serde(default)]` em todo campo novo; gate `vertex_attr_offsets_match_struct`. |
| **ADR-0071** | Tint channels — matemática multiplicativa canônica | 4 canais: Tint herdável (cascateia) · Self Tint local (não cascateia) · Per-corner tint (vertex color) · Opacity (multiplicador final). Ordem: `final = (self_tint * Π(modulate_ancestors)) * per_corner * (1 - 0) * opacity`. PremultAlpha gotchas. |
| **ADR-0072** | Named Anchor unification (socket + slice + image_point) | `NamedAnchor { name, transform: ph2d_ecs::Transform, bounds: Option<[f32;4]>, center: Option<[f32;4]>, user_data }`. Per-frame override em `SpriteFrames`. Visual handles. Substitui 3 conceitos paralelos. Dict via `SmallVec<[(K,V); 8]>` (ADR-0022 no-HashMap). |
| **ADR-0073** | Sorting canonical order (Z + ZAsRelative + YSort + SortingGroup + ShowBehindParent + DFS) | Pipeline determinístico com ordem fixa de aplicação. Order Debug Overlay como gate visual contra regressões (Godot Issue #74265 padrão). |
| **ADR-0074** | Sprite-vs-Component boundary princípio | Regra: `Sprite` struct = aparência intrínseca da imagem (sempre presente, POD, schema versionado). Componentes ECS = ortogonal opcional. Anti-padrão god-struct documentado (vs GameMaker `image_*` instance variables). |
| **ADR-0025-amendment-1** | Skew em Transform (foundational cascade) | Bump `Transform::VERSION` 1→2 com `skew_x, skew_y: f32`. Migrator obrigatório. Ordem T·R·Sk·S canônica. Habilita Sprite Inspector v2 W2.T2.2. |

**Próximos passos concretos:**
1. **Auditoria adversarial multi-lente** (≥2 agentes em paralelo, lentes rotacionadas) sobre este spec antes de ratificar.
2. **Corrigir achados a erro-zero** (sem deferral, padrão-ouro).
3. **Ratificação Enio** dos 6 ADRs → W0 fechada → W1 abre.
