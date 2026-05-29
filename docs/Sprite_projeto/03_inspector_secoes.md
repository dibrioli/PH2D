# 03 — Layout das 12 seções do Inspector

## 3.0 Princípios de layout

1. **12 seções colapsáveis canônicas** (FROZEN; cap `inspector_section_count_canonical == 12`).
2. **3 sempre abertas por default** (revisão pós-Lens-D D11 layout density budget): Identity · Transform · Color & Tint (sub-tabs).
3. **9 colapsadas por default:** Render Source · Sprite Sheet · 9-Slice · Sorting · Visibility · Sampling · Material&Blend · Animation · Sockets/Slices (Named Anchors).
4. **Color & Tint em sub-tabs** (D11 fix): `[Tint] [Self Tint] [Per-corner] [Effects]` — só 1 sub-tab visível por vez; seção ~120px em vez de ~342px.
5. **Gate `inspector_default_open_total_height_max_768px`** (W2 cria): mede paint height em scene fixture canônica; falha se excede viewport laptop.
4. **Header de seção** (ALL-CAPS + color dot + collapse chevron) consistente com Painter/Vector ([widget panel_chrome](../../crates/ph2d-editor-core/src/widget/panel_chrome.rs)).
5. **Hotkeys canônicos** registrados em `editor-core`.

## 3.1 Seção 1: Identity (sempre topo, não colapsa)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Name | `TextInput` | "" (entidade ID se vazio) | F2 (rename) | já existe |
| Tags | `ChipList` (multi-select) | [] | — | Unity tags + GameMaker |
| Notes | `TextArea` (multiline, ~3 linhas) | "" | — | 🆕 |

**Razão Notes:** anota propósito do sprite no level ("PORTA, abre quando flag.door_opened==true"). Facilita produção 2D; gap universal.

**Onde mora (Lens D D26):** `Notes` vive em **Component opcional `EntityNotes(String)`** (não em `Name { name, notes }`). Razão: ausência ≠ "empty notes explicit" — vasta maioria das entities NÃO tem notes; Component opcional preserva memory (sem string vazia serializada). Bevy ECS Component padrão.

**`Tags` ChipList multi-select** idem: vive em `EntityTags(SmallVec<[Box<str>; 4]>)` Component opcional; ausência = "no tags".

**EntityTags sanitization (Lens E E20 fix):**
- Cada tag ≤ **32 bytes UTF-8** ENFORCED on setter.
- Character class `[a-zA-Z0-9_-]` (ASCII letters/digits + underscore/hyphen); rejeita control chars + `\0` + `\x1b` ANSI escape.
- Max 16 tags por entity (cap defensivo).
- Gate `entity_tags_sanitized` (W2 cria): test inserção de tag oversize/control-char retorna `Err(SpriteError::TagInvalid)`.

## 3.2 Seção 2: Transform

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Position X/Y | `NumberInput` (unit `m` ou `px` via Display Unit toggle) | [0, 0] | — | já existe |
| Rotation | `NumberInput` (unit `deg` ou `rad`) | 0 | R | já existe |
| Scale X/Y | `NumberInput` (sem unit) | [1, 1] | S | já existe |
| **Skew X (kx)** ⭐⭐ | `NumberInput` (unit `deg`) | 0 | — | Godot |
| **Skew Y (ky)** ⭐⭐ | `NumberInput` (unit `deg`) | 0 | — | Godot (extensão) |
| Top Level | `bool toggle` | false | — | Godot |
| **Reset** | `Button` (icon-only) | — | — | Godot reset |
| **Look At target** | `Button + EntityPicker` | — | — | Godot look_at |

**Decisão Skew:** vai dentro de `Transform` Component (ADR-0025 amendment via ADR-0070). Razão: skew é decomposição da matriz 2D, não da imagem. Godot Node2D acerta. Implementação: 2 floats novos em `Transform`; `propagate_transforms` aplica `Transform2D::from_skew(kx, ky)` na composição.

**Decisão Top Level:** Component marker zero-size (`TopLevel`). Presença = quebra cascata de transform + modulate (HUD-in-world fixo a inimigo).

**Reset button:** zera position, rotation, scale para identidade. Skew preservado (intencional — skew costuma ser propriedade fixa do sprite, não animação).

**Look At target:** popup com EntityPicker filtra entidades visíveis na cena. Aplica `look_at()` (orienta +X local para a posição global do alvo). Aciona quando clicado, não persistente.

## 3.3 Seção 3: Render Source (existente, ampliada)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Strategy | `Segmented` (Atlas / Individual / Hand-packed) | Atlas | — | já existe |
| Storage detail (key/texture_id) | `read-only label` | — | — | já existe |
| Source (W × H px) | `read-only label` | — | — | já existe |
| **Region toggle** | `bool toggle` | false | — | Godot region_enabled |
| **Region Rect (x, y, w, h)** | 4 × `NumberInput` (px) | [0, 0, w, h] | — | Godot region_rect |
| **Region Filter Clip** | `bool toggle` (default true para Atlas) | true (Atlas) / false (Individual) | — | Godot region_filter_clip_enabled |
| Pixel Format | `Segmented` (RGBA8 / RGBA16) | RGBA8 | — | já existe |
| Reimport at current px/m | `Button` | — | — | já existe |

Quando Strategy = Hand-packed: Region campos ficam ocultos (Hand-packed traz seu rect do asset).

## 3.4 Seção 4: Sprite Sheet (inline, sem precisar de SpriteFrames asset)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| **Centered** | `bool toggle` | true | — | Godot Sprite2D |
| **Offset X/Y** | 2 × `NumberInput` (px intrínsecos) | [0, 0] | — | Godot |
| Flip H | `bool toggle` | false | F | universal |
| Flip V | `bool toggle` | false | Shift+F | universal |
| **H Frames** | `NumberInput` (int ≥ 1) | 1 | — | Godot hframes |
| **V Frames** | `NumberInput` (int ≥ 1) | 1 | — | Godot vframes |
| **Frame** | `NumberInput` (int) + scrubber slider | 0 | — | Godot frame |
| **Frame Coords (col, row)** | 2 × `NumberInput` (int) | [0, 0] | — | Godot frame_coords |

**Razão "sprite-sheet inline":** muitos casos comuns (char com 4 frames de andar, tile com 8 estados) NÃO precisam de SpriteFrames asset separado. Inline em `Sprite` struct é leve. Quando ganha complexidade (animação, tags, per-frame durations), promove para SpriteFrames via botão "Promote to SpriteFrames asset".

**Frame ↔ Frame Coords sync bidirecional:** mudar frame_coords atualiza frame (`frame = row * hframes + col`) e vice-versa. Visual no canvas mostra grid de hframes×vframes overlay quando seção expandida.

## 3.5 Seção 5: 9-Slice (collapsible — só ativa quando `SliceNine` Component anexado)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Draw Mode | `Segmented` (Simple / Sliced / Tiled) | Simple | — | Unity |
| Slice Borders L | `NumberInput` (px) | 0 | — | Unity + Defold |
| Slice Borders T | `NumberInput` (px) | 0 | — | idem |
| Slice Borders R | `NumberInput` (px) | 0 | — | idem |
| Slice Borders B | `NumberInput` (px) | 0 | — | idem |
| Size X/Y (Sliced/Tiled) | 2 × `NumberInput` (m) | da textura | — | Unity SpriteRenderer.size |
| **Per-region tile mode** | 8 × `Dropdown` (Stretch/Repeat/Mirror/BlankRepeat — **4 modos canônicos** GameMaker) — 4 cantos + 4 bordas | Stretch | — | GameMaker (verificado em docs oficiais) |
| Tile Mode (global Tiled) | `Segmented` (Continuous / Adaptive) | Continuous | — | Unity |
| Stretch Value (Adaptive) | `Slider 0..1` | 0.5 | — | Unity |
| Fill Center | `bool toggle` | true | — | Unity |

**Razão per-region tile mode:** GameMaker é único em ter por-região; Unity/Godot só global. Diferença: pode ter cantos fixos + lateral repete + topo stretcha + bottom hide. UI completa de dialog box / button bar.

**Adicionar SliceNine ao sprite:** botão "+ Add 9-Slice" no top da seção. Cria Component `SliceNine` com defaults. Botão "× Remove 9-Slice" remove.

## 3.6 Seção 6: Color & Tint ⭐⭐⭐ (a seção crítica)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| **Tint (modulate)** ⭐⭐ | `ColorPicker OKLCH` | WHITE | — | Godot modulate |
| **Self Tint (self_modulate)** ⭐⭐⭐ | `ColorPicker OKLCH` | WHITE | — | Godot self_modulate |
| **Per-corner Tint (TL)** ⭐⭐⭐ | `ColorPicker OKLCH` | WHITE | — | Phaser |
| **Per-corner Tint (TR)** ⭐⭐⭐ | `ColorPicker OKLCH` | WHITE | — | Phaser |
| **Per-corner Tint (BL)** ⭐⭐⭐ | `ColorPicker OKLCH` | WHITE | — | Phaser |
| **Per-corner Tint (BR)** ⭐⭐⭐ | `ColorPicker OKLCH` | WHITE | — | Phaser |
| **Tint Fill** | `bool toggle` | false | — | Phaser setTintFill |
| **Opacity** | `Slider 0..1` (com chip 0..100%) | 1.0 | — | universal |

**Layout em pares:** linha "Tint | Self Tint" lado a lado; linha "Per-corner: TL | TR" + linha "BL | BR" formando grid 2×2. Inspector mostra **preview do gradiente** num quad pequeno (~40×40px) na direita do grid de cantos.

**Hotkey rápido "Equalize corners"** (não acessível por teclado, button só): copia o TL para os outros 3 cantos. Útil quando o artista quer gradient desabilitado temporariamente sem perder a configuração.

**Matemática multiplicativa canônica** documentada em [04_color_tint_canais.md](04_color_tint_canais.md).

**Bulk-edit:** Inspector com N sprites selecionados (multi-select) mostra "Mixed" placeholder no campo se valores divergem; editar aplica a TODOS os sprites selecionados (gap universal — Unity proposal aberto há anos).

## 3.7 Seção 7: Ordering / Sorting ⭐⭐

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| **Z Index** ⭐⭐⭐ | `NumberInput` (int −1_073_741_823..1_073_741_823 = ±i32::MAX/2) | (ausente / DFS) | Ctrl+] / Ctrl+[ | Godot (PH2D usa range maior — vide [§5.9](05_ordering_sorting.md)) |
| **Z as Relative** ⭐⭐⭐ | `bool toggle` (só visível se Z Index presente) | true | — | Godot |
| **Show Behind Parent** ⭐⭐⭐ | `bool toggle` | false | B | Godot |
| Sorting Layer | `Dropdown` (named, Project Settings) | "Default" | — | Unity |
| Order in Layer | `NumberInput` (int signed) | 0 | — | Unity |
| Y Sort Enabled | `bool toggle` | false | Y | Godot |
| Y Sort: Sort Point | `Segmented` (Center / Pivot / Custom) | Pivot | — | Unity |
| Y Sort: Custom Axis | `Vec2` (só visível se Sort Point=Custom) | [0, 1] | — | Unity |
| Sorting Group | `bool toggle` | false | — | Unity |
| Sort At Root | `bool toggle` (dentro de Sorting Group) | false | — | Unity |
| Translucency Sort Priority | `NumberInput` (int) | 0 | — | Paper2D |
| Translucency Distance Offset | `NumberInput` (float, world m) | 0 | — | Paper2D |
| **Order Debug Overlay** | `bool toggle` | false | — | 🆕 |

**Razão Z Index ausente vs explícito:** quando Component `ZIndexOverride` não anexado, sprite usa DFS counter (auto-ordered pela hierarquia). Quando anexado com valor 0, sprite força Z=0 absoluto. UI mostra "Z Index: —" (ausente) com botão "+ Set Z Index manually" para anexar Component.

**Order Debug Overlay:** quando ativo, sprite renderiza overlay no canvas mostrando: cor da sorting layer (codigo por dicionário), número Z, índice DFS, marcador de Y-sort axis. Resolve "por que esse sprite tá atrás daquele?" em 1s. 🆕 ninguém tem.

**Pipeline canônico de ordering** detalhado em [05_ordering_sorting.md](05_ordering_sorting.md).

## 3.8 Seção 8: Visibility

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Visible | `bool toggle` | true | — | universal |
| Visibility Layer | `Bitmask 32-bit` (checkbox grid 4×8) | bit 0 ativo | — | Godot |
| **Clip Children mode** ⭐⭐ | `Segmented` (Disabled / ClipOnly / ClipAndDraw) | Disabled | — | Godot |
| **Mask Interaction** ⭐⭐ | `Segmented` (None / VisibleInside / VisibleOutside) | None | — | Unity SpriteMask |
| Alpha Cutoff (Mask Interaction != None) | `Slider 0..1` | 0.5 | — | Unity |
| On-Screen Enabler | `bool toggle + Rect2 editor` (collapsible inner) | false | — | Godot VisibleOnScreenEnabler2D |

**ClipChildren mode:**
- **Disabled** — comportamento normal (default).
- **Clip Only** — filhos recortados pela silhueta do sprite, mas sprite NÃO desenha (template/molde).
- **Clip And Draw** — sprite desenha + filhos recortados pela silhueta.

**Razão gate de regressão obrigatório:** Godot teve 5 issues abertos sucessivos (#79885, #102190, #102224, #91068, #90793). PH2D adiciona `tests/clip_children_regression.rs` com fixtures visuais 4-pixel comparison para cada modo. Sem isso, PH2D regride como Godot regrediu.

**Mask Interaction + Custom Range** vai para sub-seção avançada (não default), porque é raro precisar de range específico de sorting layer.

## 3.9 Seção 9: Sampling

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| **Texture Filter** ⭐⭐ | `Dropdown` (Inherit / Nearest / Linear / NearestMipmap / LinearMipmap / NearestAniso / LinearAniso) | Inherit | — | Godot per-node ⭐⭐ |
| **Texture Repeat** ⭐ | `Dropdown` (Inherit / Disabled / Enabled / Mirror) | Inherit | — | Godot per-node |
| Anti-halo / Edge Filtering | `read-only label` (vem do asset cooker, não editável aqui) | — | — | GameMaker — flag do asset |

**Razão per-node hierárquico:** Godot permite pixel-art mundo (Nearest) + UI vetorial (Linear) coexistindo sem hack global. Inspector mostra "Inherit" como default; texto secundário mostra qual filter está sendo herdado.

## 3.10 Seção 10: Material & Blend

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Material | `AssetSlot` (link a Material asset) | (default sprite material) | — | universal |
| **Use Parent Material** ⭐⭐ | `bool toggle` | false | — | Godot use_parent_material |
| **Instance Shader Params** ⭐⭐ | `KeyValueList` (name: String + value: f32/Vec2/Vec3/Color/TextureRef) | [] | — | Godot set_instance_shader_parameter + Unity MPB |
| **Blend Mode** ⭐ | `Dropdown` (Mix / Add / Sub / Mul / Screen / PremultAlpha) | Mix | — | Godot CanvasItemMaterial + Defold |

**Razão Use Parent Material:** 10k sprites filhos compartilham 1 material instance = 1 draw call. Performance brutal em batch grandes.

**Razão Instance Shader Params:** quando shader declara `instance uniform float foo;`, cada sprite pode setar `foo` distinto SEM clonar Material. Crítico para variar hue/intensity/etc per-instance em massas (tilemap, partículas, fogo).

**Blend Mode default Mix** (alpha-blend padrão). Modos especiais:
- **Add** — magias, fogo, glow.
- **Sub** — sombras, "drain".
- **Mul** — máscaras de iluminação, lentes.
- **Screen** — flash, lens flare.
- **PremultAlpha** — pré-multiplica RGB×alpha (necessário pra composite chains corretas matematicamente).

## 3.11 Seção 11: Animation (collapsible — só ativa se entidade tem `SpriteAnimator` Component)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| SpriteFrames asset | `AssetSlot` | — | — | Godot AnimatedSprite2D |
| Current Animation (tag) | `Dropdown` (tags do asset) | "default" | — | Aseprite tags |
| Frame | `NumberInput` (int) + scrubber | 0 | — | Godot |
| Frame Progress | `Slider 0..1` (animável) | 0.0 | — | Godot |
| Speed Scale | `NumberInput` (float, negativo = reverse, 0 = pausa) | 1.0 | — | Godot |
| Playing | `bool toggle` | true | Space (toggle) | Godot |
| Autoplay | `bool toggle` | false | — | Godot |
| Direction override | `Dropdown` (Forward / Reverse / PingPong / PingPongReverse) | (do tag) | — | Aseprite |
| Loop override | `bool toggle` (3-state: Inherit / On / Off) | Inherit | — | Godot |
| Hold ms (pausa antes do loop) | `NumberInput` (ms) | 0 | — | Phaser |
| Repeat Delay ms | `NumberInput` (ms) | 0 | — | Phaser |

**Sub-asset SpriteFrames** detalhado em [08_animation_inline.md](08_animation_inline.md).

**Onion skin** NÃO é Inspector — vive na timeline editor (módulo Animation futuro). Inspector só mostra ESTADO atual.

## 3.12 Seção 12: Sockets / Slices (Named Anchors)

| Campo | Widget | Default | Hotkey | Origem |
|---|---|---|---|---|
| Lista de NamedAnchor | `RepeatList` (collapsible inner items) | [] | — | unificado |
| Per-anchor: name | `TextInput` | "anchor_N" | — | universal |
| Per-anchor: transform (pos+rot+scale 2D) | `inline TransformEditor` | identity | — | Paper2D socket |
| Per-anchor: bounds (opcional) | `bool toggle + Rect2 editor` | None | — | Aseprite slice |
| Per-anchor: center (opcional, dentro de bounds) | `bool toggle + Rect2 editor` | None | — | Aseprite 9-slice region |
| Per-anchor: user_data | `Variant editor` (string/dict/num) | — | — | Aseprite slice data |
| Per-frame override toggle | `bool` | false | — | Construct ImagePoint |
| Visual handles no canvas | (sem widget no Inspector — gizmo visual) | — | — | universal |

Sistema unificado detalhado em [07_named_anchors.md](07_named_anchors.md).

## 3.13 Padrões de widget canônicos

Cada widget no Inspector segue regras de [DIRETRIZ §5.2 Widget Gallery](../IntegracaoMultiAgente/DIRETRIZ.md#52-widget-gallery-é-a-fonte-de-verdade):

- **Slider + chip pareados** → SEMPRE `store.link_slider_number(slider_id, chip_id)`.
- **Chip pill sem stepper** → `store.mark_chip_no_stepper(chip_id)`.
- **Storage chip e slider no mesmo espaço (`0..1`)** → unidade natural via `display_override` no paint.
- **Bridge `<tool>_bridge.rs`** se altera pixels (canvas live update).
- **AccessKit `Node` emitido por TODO widget** (HR-12).

## 3.14 Bulk-edit (multi-select) — categorização Lens D D21

Inspector com N sprites selecionados:
- Campos com valores idênticos → mostram o valor.
- Campos com valores divergentes → mostram placeholder "—" (Mixed via `BulkSelectInspector` widget T2.0).
- **Live apply (default)** em campos cheap: tint, self_tint, opacity, flip_x/y, centered, offset, z_index — edit em qualquer campo aplica imediatamente a TODOS selecionados sem confirmation.
- **Confirm apply** em campos heavy (custo proporcional a N): SortingLayer change, Material/Shader Params batch edit, BlendMode change, FrameRef/SpriteFrames swap, NamedAnchorList replace. Inspector mostra "Apply to {$count} sprites?" modal antes.

**Gate `bulk_edit_confirmation_required_fields`** (W6 cria): lista canônica de campos heavy enforce confirmation modal. Sem isso, click acidental em SortingLayer de 10k sprites freezes editor.

Gap universal nos engines existentes (Unity proposal aberto há anos). 🆕 PH2D first-class desde **W2.T2.0** (BulkSelectInspector widget precondition; Lens D D14 movido de W6).

## 3.15 Visibility Layer bitmask — semântica Lens D D23

`Visibility Layer` Component (32-bit bitmask) controla **culling per-Camera2D**. Cada bit representa uma named layer no registry global (similar SortingLayer mas para culling, não ordering).

- **Bit 0 (default ATIVO):** "Default" — ANY camera com `cull_mask & 1 != 0` renderiza este sprite.
- **Bits 1-31:** user-defined em Project Settings → "Visibility Layer Registry" (~32 named entries max).

Convenção esperada:
| Bit | Nome típico | Uso |
|---|---|---|
| 0 | Default | Game world (all cameras render) |
| 1 | Minimap | Só camera minimap renderiza |
| 2 | HUD | Só camera HUD renderiza |
| 3 | Spy mode | Só camera spy mode revela |
| 4-31 | User-defined | Custom uses (per-puzzle, per-cutscene, etc) |

`Camera2D.cull_mask: u32` é simetria — render sprites onde `sprite.visibility_layer & camera.cull_mask != 0`. Gate `visibility_layer_camera_simmetric_cull` enforce.

## 3.16 Sync Inspector ↔ Hierarchy panel (Lens D D19)

Inspector + Hierarchy panel sincronizam bidirectionally via `EditorAction`:

- **Hierarchy → Inspector:** Hierarchy drag-reorder emite `EditorAction::EntityReorderedInHierarchy { entity, new_dfs_index }`. Inspector da entity selecionada refletir mudança (RenderInstance.z_order update; se Z Index manual ausente, DFS counter atualizado).
- **Inspector → Hierarchy:** Inspector edita Z Index/SortingLayer/SortingGroup → `EditorAction::SpriteOrderingChanged { entity, key }`. Hierarchy repaint label color (cor da SortingLayer) + reordering visual.
- **Hierarchy → Inspector (selection):** clicar entity em Hierarchy → Inspector mostra essa entity. Existing pattern; no change.

Gate `inspector_hierarchy_sync_bidirectional` (W3.T3.X cria): scene fixture com 5 sprites; drag-reorder em Hierarchy → asserta Inspector reflect; edit Z em Inspector → asserta Hierarchy repaint.

> **Nota Lens E E25:** Os EditorAction variants `EntityReorderedInHierarchy`, `SpriteOrderingChanged`, `SpritesCommittedFromPainter`, `SpriteFromVectorRasterize` referenciados em §3.16 + §7.1.1 README **AINDA NÃO existem** no enum `EditorAction` em [crates/ph2d-editor-core/src/action_bus.rs](../../crates/ph2d-editor-core/src/action_bus.rs). `EditorAction` é `#[non_exhaustive]` — adicionar variants é safe sem ADR-amendment de ADR-0040 (cap PanelEvent=4 frozen NÃO aplica a EditorAction). **Tasks W2-W3 adicionam variants** (plano §15.3-§15.4). `PanelEvent` cap intacto.

## 3.17 Visual canvas handles — undo granularity (Lens D D22)

NamedAnchor handles + Region Rect handles + 9-Slice borders no canvas são editáveis via drag.

**Undo granularity canônica:**
- **1 drag (mouse-down → drop) = 1 `EditorAction` entry no history.** Não per-frame intermediário.
- Drag in-flight: cursor preview overlay; estado real só committed em drop.
- ESC durante drag: cancel sem commit.
- `EditorAction::AnchorTransformDragged { entity, anchor_id, start_transform, end_transform }` (1 entry per drag).
- Idem para `RegionRectDragged`, `SliceBorderDragged`, `NinePatchCenterDragged`.

**NÃO é Tool isolation (ADR-0040).** Handles são **Inspector-side gizmos**, não Tools. Sem `impl Tool`. Inspector dispatch direto na zona Right + canvas overlay paint conditional ("Inspector section X expanded → paint handle Y").

Memory [`project-image-tools-audit-close`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_image_tools_audit_close_2026_05_23.md): cross-sprite undo é CRITICAL data-loss class — Inspector handles cuidam não emitir per-frame entries.
