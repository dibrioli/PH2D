# 02 — Components ECS ortogonais (princípio organizador)

## 2.1 A regra de decisão (3 lugares)

Toda propriedade candidata de Sprite passa por este teste:

```
┌─ É aparência intrínseca da imagem, universal,
│  todo sprite tem, default benigno, POD pequeno?
│  └─ SIM → Campo do Sprite struct (versionado, serde)
│
├─ É aspecto ortogonal, opcional, nem todo sprite carrega,
│  ausência ≠ default explícito, pode ter UI/dispatch própria?
│  └─ SIM → Component ECS anexável (presença = override)
│
└─ É derivado por algoritmo per-frame, produto de avaliador?
   └─ SIM → Output de Sistema ECS (extract) ou Nó do grafo (Motion/Shader)
```

**Caso ambíguo: testa "ausência ≠ default explícito"?**
- "Sprite sem `ZIndexOverride`" → use DFS counter (não "Z=0 explícito"). → Component opcional.
- "Sprite sem `tint`" → impossível, tint sempre tem valor. → Campo do struct.
- "Sprite sem `ShowBehindParent`" → "default = false comportamento". → Component marker opcional.

Se você consegue dizer "este sprite não tem essa propriedade" e isso ter significado distinto de "tem com valor X" → **Component**.

## 2.2 Matriz canônica completa

### Vai para `Sprite` struct (POD, schema versionado)

| Campo | Tipo | Default | Por quê intrínseco |
|---|---|---|---|
| `source` | `SpriteSource` | — | TODO sprite tem fonte de pixel |
| `size` | `[f32; 2]` | da textura | TODO sprite tem tamanho |
| `tint` | `[f32; 4]` | WHITE | TODA aparência tem cor (cascateia) |
| `self_tint` | `[f32; 4]` | WHITE | aparência local (não cascateia) — sempre presente |
| `per_corner_tint` | `[[f32; 4]; 4]` | [WHITE; 4] | gradient é aparência; default não-cor zero overhead |
| `tint_fill` | `bool` | false | toggle de semântica de tint, sempre presente |
| `opacity` | `f32` | 1.0 | visibility multiplier, sempre presente |
| `anchor` | `[f32; 2]` | [0, 0] | pivot da imagem, intrínseco |
| `flip_x`, `flip_y` | `bool` | false | espelhamento da imagem (não scale!) |
| `centered`, `offset` | `bool`, `[f32; 2]` | true, [0,0] | origem da imagem |
| `hframes`, `vframes`, `frame` | `u32` × 3 | 1, 1, 0 | sprite-sheet inline (substitui SpriteFrames asset quando único) |
| `region_enabled`, `region_rect`, `region_filter_clip` | bool, Rect, bool | false, _, true | sub-área da textura |
| `premultiplied` | `bool` | false | runtime hint (serde skip) |
| `version` | `u32` | 4 | schema version (HR-14) |

### Vai para Component ECS (presença = override)

| Component | Quando anexar | Origem do design |
|---|---|---|
| `Transform { translation, rotation, scale, skew_x, skew_y }` | sempre (ADR-0025 + amendment-1; bump `Transform::VERSION` 1→2) | Godot Node2D |
| `Visibility { visible }` | sempre (já existe via default) | universal |
| `Name(String)` | sempre via default | universal |
| `ZIndexOverride(i32)` | sprite com ordem manual contradiz hierarquia | Godot z_index |
| `ZAsRelative(bool)` | quando hierarquia precisa ser relativa OU absoluta | Godot z_as_relative |
| `SortingLayer(LayerId)` | macro-camada nominal (BG/Player/UI) | Unity ⭐⭐ |
| `OrderInLayer(i32)` | micro-ordering dentro do SortingLayer | Unity |
| `YSort { enabled: bool, axis: Vec2, sort_point: SortPoint }` | topdown/iso | Godot y_sort + Unity Custom Axis |
| `SortingGroup { sort_at_root: bool }` | multi-piece char sortado como bloco | Unity Sorting Group |
| `ShowBehindParent` (marker, zero-size) | filho desenha atrás do pai | Godot show_behind_parent |
| `TopLevel` (marker) | quebra cascata de transform/modulate | Godot top_level |
| `ClipChildren(Mode)` | recorta filhos pela silhueta | Godot |
| `MaskInteraction { mode: MaskMode, alpha_cutoff: f32 }` | sprite responde a Mask2D irmão | Unity SpriteMask |
| `VisibilityLayer(u32 bitmask)` | culling per-camera | Godot |
| `TextureFilter(FilterMode)` | override hierárquico (pixel-art mundo + UI vetorial) | Godot per-node filter ⭐⭐ |
| `TextureRepeat(RepeatMode)` | tile sem TileMap | Godot |
| `Material(MaterialRef)` | shader custom per-sprite | universal |
| `UseParentMaterial` (marker) | filhos compartilham material do pai (batching) | Godot use_parent_material ⭐⭐ |
| `InstanceShaderParams(SmallVec<[(Box<str>, InstanceParamValue); 8]>)` | per-instance uniforms sem clone | Godot set_instance_shader_parameter |
| `BlendMode(Mode)` | Add/Sub/Mul/Screen/PremultAlpha override | Godot CanvasItemMaterial + Defold |
| `OnScreenEnabler { rect: Rect2, mode: EnableMode }` | culling automático de processamento | Godot VisibleOnScreenEnabler2D |
| `SliceNine { borders: [f32; 4], size: [f32; 2], draw_mode: DrawMode, tile_modes: [TileMode; 8], stretch_value: f32, fill_center: bool }` | 9-slice quando ativo | Unity + Defold + GameMaker |
| `SpriteAnimator { frames_ref, current_anim, frame, progress, speed_scale, playing, autoplay, direction, hold_ms, repeat_delay_ms }` | sprite anima ao longo do tempo | Godot AnimatedSprite2D + Phaser anim |
| `NamedAnchorList(SmallVec<[NamedAnchor; 4]>)` | sockets/slices/image_points por-sprite | unificado (vide [07](07_named_anchors.md)) |
| `OrderDebugOverlay(bool)` | gate visual contra regressões de sorting | 🆕 PH2D |

### Vai para Sistema ECS / extract / nó do grafo (derivado por-frame)

| Output | Quando | Origem |
|---|---|---|
| `RenderInstance.world_pos` | propagate_transforms (ADR-0025) | sempre |
| `RenderInstance.z_order` | DFS counter fallback OU ZIndexOverride se presente | extract |
| `RenderInstance.rotation` | decomposto de Transform global | extract |
| `RenderInstance.anchor` (world-scaled) | `Sprite.anchor * GlobalTransform.scale` | extract |
| `RenderInstance.tint` (collapsed) | `tint * self_tint * Π(modulate_ancestors) * opacity` | extract / shader |
| `RenderInstance.per_corner_tint` | cópia direta do `Sprite.per_corner_tint` | extract |

## 2.3 Razão da fronteira (3 motivos pra resistir ao god-struct)

### Motivo 1: Schema bump custa
Cada novo campo no `Sprite` = bump `VERSION` + migrator obrigatório (HR-14) + adapter para asset cooker + bytes na ABI do `RenderInstance`. Multiplicar por N features ortogonais = sopa serializada + migrators encadeados quebráveis.

**Component:** adicionar `ClipChildren` Component novo = zero impacto no schema do `Sprite`. Cooker só serializa quando presente. Fan-out drop-crate (ADR-0040).

### Motivo 2: Default explícito ≠ ausência semântica
`ZIndex = 0` significa "Z absoluto = zero" OU "use DFS"? Sem Component, é ambíguo. Com `ZIndexOverride(i32)` Component:
- Ausência → DFS counter (auto-ordered pela hierarquia)
- Presença com valor `0` → "Z forçado = zero" (override explícito)

Mesma lógica em Unity (`SortingGroup` component opcional), Godot (`Visibility` herdada vs explícita), Unreal (`Tags` component-style).

### Motivo 3: Fan-out paralelo só funciona se features novas não tocam estruturas centrais
DIRETRIZ §3.A (drop-crate fan-out): cada feature é uma sessão Implementador-só que NÃO toca arquivos centrais. Adicionar `ClipChildren(Mode)` Component em `crates/ph2d-component-clip-children/` (ou similar) — zero edit em `ph2d-render`, zero risco de colidir com outras sessões.

Se `ClipChildren` fosse campo do `Sprite`, cada feature ortogonal viraria PR no `Sprite` struct = serial. ADR-0039 (nodegraph FREEZE) e ADR-0040 (tool FREEZE) existem **exatamente** para impedir esse anti-padrão.

## 2.4 Casos-limite (o que ficou borderline)

### `flip_x`, `flip_y` no `Sprite`, não Component
**Decisão:** campo do `Sprite`. Razão: 99% dos sprites de jogos 2D viram pra esquerda/direita em algum momento (char animation), e flip lógico (≠ scale negativo) é **propriedade da imagem**, não do transform. Default `false` é benigno; 2 bytes a mais no struct.

### `opacity` no `Sprite`, não Component
**Decisão:** campo do `Sprite`. Razão: TODO sprite tem visibilidade contínua (fade in/out é caso comum). Não fazer "Sprite sem opacity" significar "100% sempre" porque queremos animar via tween — campo simples é estritamente mais expressivo.

### `tint` per-corner no `Sprite`, não Component
**Decisão:** campo do `Sprite`. Razão: cabe em 16 bytes do struct (zero overhead quando default WHITE × 4); o shader sempre precisa de input vertex color (ABI `RenderInstance`); colocar em Component duplicaria o estado de attachment vs vertex format.

**Trade-off aceito:** v3 → v4 inflate o `RenderInstance` em ~70 bytes. Mitigação opcional descrita em [01_anatomia_canonica.md §1.7](01_anatomia_canonica.md).

### `material` Component, não `Sprite`
**Decisão:** Component opcional. Razão: 95% dos sprites usam o default material (sprite-default WGSL); só power user troca. Forçar `material: Option<MaterialRef>` no struct = serialização + bytes a mais sem ganho semântico.

### `Z Index` Component, não `Sprite`
**Decisão:** Component opcional (`ZIndexOverride(i32)` + `ZAsRelative(bool)`). Razão crítica: ausência ≠ default. "Sprite sem Z override" → DFS counter (deterministic, hierarchy-aware). "Sprite com `ZIndexOverride(0)`" → Z absoluto forçado a zero. Sem Component, esse distinção semântica se perde.

### `SkewX/Y` no `Transform`, não `Sprite`
**Decisão:** vai para `Transform` Component via **[ADR-0025 amendment-1](../architecture/decisions/0025-amendment-1.md)** formal (cascata foundational — `Transform::VERSION` bump 1→2 + migrator obrigatório + ABI bump + `compose()` matemática T·R·Sk·S). Razão: skew é decomposição da matriz 2D (igual rotação, scale); não é propriedade da imagem. Godot Node2D acerta colocando skew no transform; LÖVE expõe `kx`/`ky` no draw call.

## 2.5 Anti-padrões observados (do que NÃO fazer)

1. **GameMaker `image_*` instance variables** — sopa de mutáveis. ❌
2. **Phaser mixins acumulados (Alpha + Tint + Crop + Mask + Flip + Origin + Pipeline)** — Inspector vira state-stuffed; difícil distinguir o que é canônico. ❌
3. **Unity `SpriteRenderer.color` herda sem `self_modulate`** — força artista a duplicar tint em material override. ❌
4. **`Sprite.material: Option<Material>` no struct** — campo Optional poluindo POD; melhor Component opcional. ❌
5. **`Sprite.z_index: i32` no struct (Godot CanvasItem coloca lá)** — não distingue "DFS" de "z=0 explícito". ❌
6. **9-Slice como objeto separado de Sprite (Construct 3)** — duplica registry; Defold faz certo: Component opcional. ✅

## 2.6 Implicação para o asset cooker

Sprite POD + Components ECS opcionais = serialização HÍBRIDA:

```
.ph2d-scene.postcard
└── Entity { 
      Sprite { ... POD v4 ... },                  // sempre presente
      Transform { ... },                          // sempre presente
      Visibility { ... },                         // sempre presente (default)
      Name(String),                                // sempre presente (default)
      ZIndexOverride(5),                           // opcional — só serializa se presente
      MaskInteraction { mode: VisibleInside, ... }, // opcional
      // ...
    }
```

Cada Component opcional é serializado **só se anexado**. Sprite serializado ocupa ~80 bytes; com 5 Components opcionais médios, ~150 bytes/entity total. Comparável a Godot (que serializa o nó CanvasItem inteiro independente de campos terem valor default).

## 2.7 Gate arquitetural

Arch-gate `architecture_sprite_inspector_surface` ([11_arch_gates_e_caps.md](11_arch_gates_e_caps.md)) força:

| Cap | Valor | Razão |
|---|---|---|
| `Sprite struct fields == 20` | **20 (v4 FROZEN)** | Anti-god-struct. Reconciliado em Lens D D1 (drift 18 vs 20). |
| `Total Components opcionais relacionados ao Sprite Inspector` | ≤ 32 | Cobertura das **12 seções** sem explosão |
| `NamedAnchorList inline SmallVec` | **4 (FROZEN)** | shape comum (~3-4 sockets); >4 vai pra heap. **Decisão pós-audit:** consolidado em 4 (não 8) — drift inter-arquivo corrigido |
| `InstanceShaderParams inline SmallVec` | **8 pares** | shape comum (1-3 params); >8 vai heap |
| `InstanceShaderParams` key | `Box<str>` (≤ **32 bytes UTF-8** ENFORCED on setter) | sem dep externa (SmolStr/CompactString); cap reconciliado pós-Lens-E E2; rejeita oversize via `try_insert` retornar `Err(SpriteError::ShaderParamKeyTooLong)` |
| `InstanceShaderParams` value | `InstanceParamValue` enum (vide [09 §9.6](09_sampling_e_material.md)) | tipo canônico (não `Value` genérico) |

Bump exige ADR amendment.
