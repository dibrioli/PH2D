# 01 — Anatomia canônica do `Sprite` struct (schema v4)

## 1.1 Princípio

`Sprite` carrega APENAS **aparência intrínseca da imagem** — propriedades que TODO sprite tem, sempre, com default benigno, POD, serializável, versionado. Tudo ortogonal vai como **Component ECS opcional** (vide [02_components_ortogonais.md](02_components_ortogonais.md)).

**Anti-padrão evitado:** GameMaker espalha `image_blend`/`image_angle`/`image_alpha`/`image_xscale` no objeto = sopa, zero type safety. Phaser acumula mixins (Alpha/Tint/Crop/Mask) = state-stuffed. PH2D resiste — `Sprite` permanece enxuto.

## 1.2 Schema v4 — campos canônicos

```rust
#[derive(Component, Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    /// Schema version (HR-14). v3 → v4 com novos campos.
    /// `#[serde(default = "default_version_4")]` quando deserializar v3.
    pub version: u32,                        // V4

    // ─── existentes em v3 ──────────────────────────────────────────
    pub source: SpriteSource,                // Atlas / Individual (existente)
    pub size: [f32; 2],                      // World meters (existente)
    pub tint: [f32; 4],                      // RGBA modulate (existente — semântica refinada em v4: HERDA pra filhos)
    #[serde(default)]
    pub anchor: [f32; 2],                    // Pivot offset (existente; #[serde(default)] em v3)
    #[serde(skip)]
    pub premultiplied: bool,                 // Runtime hint (existente; serde skip)

    // ─── NOVOS em v4 ──────────────────────────────────────────────
    /// Self tint — NÃO herda pra filhos (Godot self_modulate).
    /// Multiplica `tint` somente neste sprite, não cascateia.
    #[serde(default = "default_white")]
    pub self_tint: [f32; 4],

    /// Per-corner tint (gradient sem shader) — Phaser ⭐⭐⭐.
    /// Ordem: [TopLeft, TopRight, BottomLeft, BottomRight], cada RGBA.
    /// 16 floats = 64 bytes. Default = todos WHITE (zero overhead visual).
    /// `#[serde(default)]` retorna `[[1,1,1,1]; 4]`.
    #[serde(default = "default_per_corner_white")]
    pub per_corner_tint: [[f32; 4]; 4],

    /// Tint Fill (Phaser `setTintFill`) — quando true, RGB do sprite
    /// é IGNORADO e per_corner_tint/tint substituem (silhueta colorida).
    /// Damage flash de 1 toggle.
    #[serde(default)]
    pub tint_fill: bool,

    /// Opacity (multiplicador FINAL, separado de tint.a).
    /// `tint.a` = blend channel (cor com alpha); `opacity` = visibility multiplier.
    /// Animáveis independentemente.
    #[serde(default = "default_one")]
    pub opacity: f32,

    /// Flip horizontal lógico (≠ scale negativo). Universal.
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,

    /// Centered: se true, origem do sprite é o centro do quad.
    /// Se false, origem é top-left + `offset` aplica.
    /// Default true (legado v3).
    #[serde(default = "default_true")]
    pub centered: bool,

    /// Offset intrínseco da imagem (px). Aplicado DEPOIS de centered.
    /// Permite ancorar pivot no "pé" do char sem mexer transform.
    /// Default [0, 0].
    #[serde(default)]
    pub offset: [f32; 2],

    /// Sprite-sheet inline (sem precisar de SpriteFrames asset).
    /// hframes × vframes divide a textura em grid; `frame` indexa.
    /// Default hframes=vframes=1 (sprite único).
    #[serde(default = "default_one_u32")]
    pub hframes: u32,
    #[serde(default = "default_one_u32")]
    pub vframes: u32,
    #[serde(default)]
    pub frame: u32,

    /// Region toggle + rect (sub-area arbitrária da textura).
    /// Default region_enabled=false → usa textura inteira.
    #[serde(default)]
    pub region_enabled: bool,
    #[serde(default)]
    pub region_rect: [f32; 4],                // x, y, w, h em pixels

    /// Region filter clip — sampler trava no rect (anti-bleed atlas).
    /// Default true para Atlas, false para Individual.
    #[serde(default = "default_region_filter_clip")]
    pub region_filter_clip: bool,
}
```

Total **estimado** (W0 pin): ~170 bytes do struct POD com `#[repr(Rust)]` padding (`source` 8 + `size` 8 + `tint` 16 + `anchor` 8 + `premultiplied` skip + `version` 4 + `self_tint` 16 + `per_corner_tint` 64 + `tint_fill` 1+pad + `opacity` 4 + `flip_x` 1 + `flip_y` 1 + `centered` 1 + `offset` 8 + `hframes` 4 + `vframes` 4 + `frame` 4 + `region_enabled` 1 + `region_rect` 16 + `region_filter_clip` 1, com padding align-4). Postcard binário serializa ~80-120 bytes médios (sem padding; bools como 1 byte cada). **Cresceu ~3-4× vs v3** — aceito como custo aparência intrínseca completa. Bench W1.T1.x valida memory budget.

## 1.3 Campos NÃO no `Sprite` (vão para Components ECS)

Categoria | Vai para | Razão
---|---|---
**Skew X/Y** | `Transform` (ph2d-ecs) | Skew é decomposição de Transform 2D, não da imagem
**Z Index, Z As Relative** | `ZIndexOverride(i32)` + `ZAsRelative(bool)` Components | Ortogonais; ausência ≠ "Z=0 explícito" — vide DFS fallback
**Sorting Layer, Order in Layer** | `SortingLayer(LayerId)` Component | Ausência = "use DFS"
**Y Sort config** | `YSort { enabled, axis, sort_point }` Component | Cascateia pros descendentes — não é per-sprite intrínseco
**Sorting Group, Sort At Root** | `SortingGroup { sort_at_root }` Component | Sub-hierarquia como bloco; raro
**Show Behind Parent** | `ShowBehindParent` Component (zero-size marker) | Default = false; ausência ≠ "behind=false explícito"
**Top Level** | `TopLevel` Component (zero-size) | Idem
**Clip Children mode** | `ClipChildren(Mode)` Component | Default disabled
**Mask Interaction** | `MaskInteraction { mode, alpha_cutoff }` Component | Sprite responde a Mask2D irmão
**Visibility Layer** | `VisibilityLayer(u32 bitmask)` Component | Default = "todas as cameras veem"
**Visible** | `Visibility` Component (já existe) | Boolean opacity-style
**Texture Filter** | `TextureFilter(FilterMode)` Component | Hierárquico (inherit), per-node override
**Texture Repeat** | `TextureRepeat(RepeatMode)` Component | Idem
**Anti-halo / Edge Filtering** | Asset-level flag em `SpriteAtlas`, NÃO Component | Configura no atlas, propaga no cooker
**Material** | `Material(MaterialRef)` Component | Override opcional do default sprite material
**Use Parent Material** | `UseParentMaterial` Component (marker) | Default = false (usa próprio)
**Instance Shader Params** | `InstanceShaderParams(HashMap)` Component | Per-instance uniforms sem clone
**Blend Mode** | Vai dentro do `Material` (CanvasItemMaterial-like) OU `BlendMode(Mode)` Component | TBD em W4
**Animation state** | `SpriteAnimator { frames_ref, current_anim, frame, progress, ... }` Component | Estado runtime; muta a cada tick
**On-Screen Enabler** | `OnScreenEnabler { rect, mode }` Component | Culling automático opcional
**Named Anchors** | `NamedAnchorList(SmallVec<[NamedAnchor; 4]>)` Component | Per-sprite OU per-frame override em SpriteFrames asset

Razão da fronteira detalhada em [02_components_ortogonais.md](02_components_ortogonais.md).

## 1.4 Helpers de default (`#[serde(default = "fn")]`)

```rust
const fn default_version_4() -> u32 { 4 }
const fn default_white() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }
const fn default_per_corner_white() -> [[f32; 4]; 4] { [[1.0; 4]; 4] }
const fn default_one() -> f32 { 1.0 }
const fn default_true() -> bool { true }
const fn default_one_u32() -> u32 { 1 }
const fn default_region_filter_clip() -> bool { true }  // Atlas default (name matches §1.2 field attr)
```

> **Nota crítica (corrigida pós-audit):** `default_region_filter_clip` retorna `true` (default Atlas-style). Para sprites Individual cuja deserialização v3→v4 cai pelo path `#[serde(default)]` em vez do wrapper enum (vide [10_schema_versionamento.md §10.3](10_schema_versionamento.md)), o valor `true` é **incorreto** — Individual sprites devem ter `region_filter_clip=false`. **Solução canônica:** carregamento de v3 **DEVE** usar `SpriteVersioned` wrapper enum (ADR-0070 §2.3) que invoca `migrate_v3_to_v4` com lógica condicional explícita. `#[serde(default)]` é fallback de **defesa-em-profundidade**, não primary path para campos com lógica condicional.

## 1.5 Construtores canônicos (já existentes, atualizados v4)

```rust
impl Sprite {
    /// Atlas-backed sprite (caso dominante).
    pub fn atlas(key: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        Self {
            version: Self::VERSION,
            source: SpriteSource::Atlas { key },
            size,
            tint,
            self_tint: [1.0; 4],
            per_corner_tint: [[1.0; 4]; 4],
            tint_fill: false,
            opacity: 1.0,
            flip_x: false,
            flip_y: false,
            centered: true,
            offset: [0.0; 2],
            hframes: 1,
            vframes: 1,
            frame: 0,
            anchor: [0.0; 2],
            region_enabled: false,
            region_rect: [0.0; 4],
            region_filter_clip: true,                // Atlas default ON
            premultiplied: false,
        }
    }

    /// Individual-texture sprite.
    pub fn individual(texture_id: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        let mut s = Self::atlas(0, size, tint);
        s.source = SpriteSource::Individual { texture_id };
        s.region_filter_clip = false;                 // Individual default OFF
        s
    }
}
```

## 1.6 Invariantes (gateadas em testes)

| Invariante | Gate |
|---|---|
| `version >= 3 && version <= 4` | Migrator |
| `opacity ∈ [0.0, 1.0]` E `is_finite()` (Lens E E7) | clamp no setter + reject NaN/Inf |
| `tint[i] ∈ [0.0, +∞)` (allow HDR) E `is_finite()`, `tint[3] ∈ [0.0, 1.0]` E `is_finite()` | clamp parcial + **reject NaN/Inf** (Lens E E7) |
| `self_tint[i] ∈ [0.0, +∞)` E `is_finite()`, `self_tint[3] ∈ [0.0, 1.0]` E `is_finite()` | idem |
| `per_corner_tint[c][i] ∈ [0.0, +∞)` E `is_finite()`, `[c][3] ∈ [0.0, 1.0]` E `is_finite()` | idem |
| `hframes >= 1 && vframes >= 1` | clamp no setter |
| `frame < hframes * vframes` | clamp / wrap |
| `region_rect[2] >= 0 && region_rect[3] >= 0` | sanity |
| `size[0] > 0 && size[1] > 0` (em world meters) E `is_finite()` | sanity + reject NaN/Inf |

**Reject NaN/Inf rationale (Lens E E7):** NaN em `tint` cascade `extract_sprite_tint` propaga (NaN × x = NaN) para TODOS descendentes, poisons hierarchy inteira. GPU recebe NaN no vertex stage → bilinear interp gera NaN per fragment → undefined behavior em alguns drivers (GPU hang / pixel corruption widespread). Setters reject via `SpriteError::NonFiniteColor`. Gate `sprite_tint_finite_rejects_nan_and_inf` (W1.T1.x cria).

## 1.7 ABI do `RenderInstance` v4 (bytes)

Compatibilidade com `vertex_attr_offsets_match_struct` ([sprite.rs:343](../../crates/ph2d-render/src/sprite.rs#L343-L375)):

```rust
#[repr(C)]
pub struct RenderInstance {
    // Existentes v3 (12 attrs GPU + 2 CPU = 72 bytes):
    pub world_pos: [f32; 2],        // @location(2)
    pub size: [f32; 2],             // @location(3)
    pub atlas_uv: [f32; 4],         // @location(4)
    pub tint: [f32; 4],             // @location(5) — tint FLAT collapsed final
    pub rotation: f32,              // @location(6)
    pub premultiplied: f32,         // @location(7)
    pub anchor: [f32; 2],           // @location(8)
    pub texture_id: u32,            // CPU-only
    pub z_order: u32,               // CPU-only

    // NOVOS v4 (per-corner tint + opacity + flip flags):
    pub per_corner_tint: [[f32; 4]; 4],  // @location(9..12) — 64 bytes! 4 attrs novos
    pub opacity: f32,                     // @location(13)
    pub flip_uv: u32,                     // @location(14) — bitfield (bit0=flip_x, bit1=flip_y)
}
```

**Tamanho v4 estimado:** 72 + 64 + 4 + 4 = 144 bytes, 4-byte aligned. **Custo:** dobra o stride do instance buffer. Justificativa: per-corner tint é ⭐⭐⭐ pesquisa — sem isso, gradient tints exigiriam draw call por sprite (catastrófico).

**Mitigação opcional (W2 decision):** se 144 B revelar gargalo, colapsar per-corner num único `Color` quando os 4 cantos forem iguais (CPU pre-pass), e ativar 4 attrs novos só em batches "has-gradient". Trade-off: branch no extract. Decisão final em ADR-0070 com bench.

## 1.8 Backward compat (`Sprite::VERSION = 4`)

```rust
impl Sprite {
    /// Schema version. Bumpa de 3 → 4 com novos campos `#[serde(default)]`.
    /// Migrator obrigatório (HR-14): v3 → v4 = setar defaults benignos em todos os campos novos.
    pub const VERSION: u32 = 4;
}
```

Postcard de v3 carrega como v4 com:
- `self_tint = WHITE` (identidade)
- `per_corner_tint = [WHITE; 4]` (identidade)
- `tint_fill = false`
- `opacity = 1.0`
- `flip_x = flip_y = false`
- `centered = true`
- `offset = [0, 0]`
- `hframes = vframes = 1`, `frame = 0`
- `region_enabled = false`
- `region_filter_clip = true` (assume Atlas; se Individual, migrator detecta e seta false)

Gate: `tests/migrate_sprite_v3_to_v4.rs` carrega 5 fixtures v3 (gerados antes do bump) e asserta carga limpa.

## 1.8.0 Postcard scene file size cap (Lens E E6 fix)

Alinhamento com Vector Module ADR-0056 §2.6 (`MAX_ASSET_SIZE = 100 MB`). Sprite scene `.ph2d-scene.postcard` declara:

```rust
impl SpriteScene {
    /// Cap defensivo para load: 100 MB postcard binário. Rejeita asset bomb.
    pub const MAX_SCENE_BYTES: usize = 100 * 1024 * 1024;
    
    /// Cap defensivo: 1M entities/scene. Rejeita massive scene DoS.
    pub const MAX_ENTITIES_PER_SCENE: usize = 1_000_000;
}

pub fn load_sprite_scene(bytes: &[u8]) -> Result<SpriteScene, Error> {
    if bytes.len() > SpriteScene::MAX_SCENE_BYTES {
        return Err(Error::SceneTooLarge);
    }
    let scene: SpriteScene = postcard::from_bytes(bytes)?;
    if scene.entities.len() > SpriteScene::MAX_ENTITIES_PER_SCENE {
        return Err(Error::TooManyEntities);
    }
    Ok(scene)
}
```

Gate `sprite_scene_load_size_cap_enforced` em [11_arch_gates_e_caps.md §11.2](11_arch_gates_e_caps.md): fixture de 100MB+1 byte → `load_sprite_scene().is_err()`.

## 1.8.1 HR-13 MemoryBudget (Lens D D10 fix)

`Sprite Inspector v2` declara orçamento HR-13:

```rust
impl Sprite {
    /// Bytes per instance (POD + average optional Components).
    /// 170B Sprite POD + ~150B avg de Components opcionais (Sorting, YSort, etc.) = ~320B/entity.
    pub const MEMORY_BUDGET_PER_INSTANCE: usize = 320;
    
    /// RenderInstance v4 upload size (per frame, per sprite).
    pub const RENDER_INSTANCE_SIZE: usize = 144;
}

impl SpriteFrames {
    /// 4096 frames cap × ~100B per SpriteFrame entry = 400KB per asset.
    pub const MEMORY_BUDGET_PER_ASSET_MAX: usize = 4096 * 100;
}
```

**Orçamento totals (10k sprites scene, mobile tier ADR-0068):**
- RAM Sprite state: 10k × 320B = **3.2 MB**.
- VRAM RenderInstance upload/frame: 10k × 144B = **1.44 MB/frame** (90 MB/s @ 60Hz).
- SpriteFrames assets (50 unique × avg 100KB): **5 MB**.
- NamedAnchorList inline (10k × 480B inline): **4.8 MB** (heap fallback raro).
- **Total per-scene budget: ~14.4 MB RAM + 1.44 MB/frame GPU upload**.

**Components compondo "média 150B/entity" (Lens E E13 fix):**

| Component | Bytes médios | Frequência | Contribuição ponderada |
|---|---|---|---|
| `Transform v2` (5 fields) | 40 | 100% | 40.0 |
| `Visibility { visible: bool }` | 1 + pad → 4 | 100% | 4.0 |
| `Name(String)` ~25 chars + varint | 32 | 100% | 32.0 |
| `SortingLayer(u8)` | 1 + pad → 4 | 30% | 1.2 |
| `OrderInLayer(i32)` | 4 | 20% | 0.8 |
| `YSort { ... }` | 16 | 15% | 2.4 |
| `ZIndexOverride(i32)` + `ZAsRelative(bool)` | 8 | 25% | 2.0 |
| `ShowBehindParent` (marker zero-size) | 0 | 10% | 0 |
| `ClipChildren(Mode)` | 1 + pad → 4 | 5% | 0.2 |
| `MaskInteraction { mode, alpha_cutoff }` | 8 | 5% | 0.4 |
| `TextureFilter` + `TextureRepeat` | 2 | 10% | 0.2 |
| `Material(MaterialRef)` | 8 (handle) | 15% | 1.2 |
| `InstanceShaderParams` SmallVec[8] inline | 80 (8 × 10B avg) | 5% | 4.0 |
| `BlendMode(Mode)` | 1 + pad → 4 | 8% | 0.3 |
| `OnScreenEnabler { rect, mode }` | 24 | 5% | 1.2 |
| `SliceNine { ... }` | 64 | 3% | 1.9 |
| `SpriteAnimator { ... }` | 64 | 20% | 12.8 |
| `NamedAnchorList` inline | 480 (4 × ~120B) | 10% | 48.0 |
| `EntityNotes(String)` ~20 chars | 28 | 5% | 1.4 |
| `EntityTags` SmallVec[4] | 64 | 5% | 3.2 |
| **Total ponderado** | | | **~157 bytes/entity Components avg** |

Aproximado para `~150B` em §1.8.1; bench W1 valida real bytes via heap allocator stats.

Caps conferidos com Painter (`MemoryBudget` ADR-0046) + Vector (`DeviceTier` ADR-0068) — Sprite cabe em mobile tier (~50 MB total budget shared).

`Plugin::init` (W1.T1.1 task adicionada) declara budget canônico via `ph2d_host::MemoryBudget::sprite_inspector_v2()`.

## 1.9 Cap no número de campos do `Sprite`

Arch-gate `architecture_sprite_inspector_surface` (W1.T0.1 cria, vide [11_arch_gates_e_caps.md](11_arch_gates_e_caps.md)) força:

```
Sprite struct fields == 20 (v4 FROZEN)
```

Contagem: 5 v3 (source, size, tint, anchor, premultiplied) + 1 versão (version field serializável em v4) + 14 v4 (self_tint, per_corner_tint, tint_fill, opacity, flip_x, flip_y, centered, offset, hframes, vframes, frame, region_enabled, region_rect, region_filter_clip) = **20**.

Bump de cap → ADR-0070-amendment-N obrigatório. Razão: god-struct anti-pattern. Tudo novo vai como Component ECS, não como campo do `Sprite`.
