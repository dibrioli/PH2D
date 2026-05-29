# ADR-0070 — Sprite schema v4 (`Sprite::VERSION` 3→4) + RenderInstance ABI bump

**Status:** Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais.
**Decisor(es):** Enio + Claude (Coord-A sessão paralela docs-only, Sprite Inspector W0).
**Pré-requisitos:** [ADR-0069 — Sprite Inspector v2 decisão-mãe](0069-sprite-inspector-v2.md), [ADR-0022 — No HashMap in simulation](0022-no-hashmap-in-simulation.md).
**Spec normativa:** [`docs/Sprite_projeto/01_anatomia_canonica.md`](../../Sprite_projeto/01_anatomia_canonica.md) + [`10_schema_versionamento.md`](../../Sprite_projeto/10_schema_versionamento.md).
**Tags:** sprite, schema, abi, hr-14, foundational

---

## 1. Contexto

`Sprite` v3 ([crates/ph2d-render/src/sprite.rs](../../../crates/ph2d-render/src/sprite.rs)) tem 5 campos canônicos:
- `source: SpriteSource` (Atlas/Individual)
- `size: [f32; 2]`
- `tint: [f32; 4]`
- `anchor: [f32; 2]` (com `#[serde(default)]`, adicionado anteriormente)
- `premultiplied: bool` (`#[serde(skip)]`, runtime hint)

ADR-0069 define 12 seções do Inspector v2 com ~70 propriedades. Da pesquisa multi-engine, identificamos que **14 propriedades novas são aparência intrínseca da imagem** (devem morar no Sprite struct, não em Components):

- `self_tint` (Godot self_modulate — herda vs não)
- `per_corner_tint` (Phaser — gradient sem shader)
- `tint_fill` (Phaser setTintFill — silhueta colorida)
- `opacity` (universal — visibility multiplier separado de tint.a)
- `flip_x`, `flip_y` (universal — flip lógico ≠ scale negativo)
- `centered`, `offset` (Godot Sprite2D — origem da imagem)
- `hframes`, `vframes`, `frame` (Godot Sprite2D — sprite-sheet inline)
- `region_enabled`, `region_rect`, `region_filter_clip` (Godot — sub-rect + anti-bleed atlas)

Aspectos ortogonais (Z Index, Sorting, ClipChildren, etc.) vão como Components ECS opcionais (ADR-0074).

Bump v3 → v4 obrigatório. HR-14 exige migrator. `RenderInstance` ABI também precisa bumpar (per-corner tint, opacity, flip_uv adicionam attrs).

---

## 2. Decisão

### 2.1 `Sprite::VERSION = 4`

Schema completo em [`docs/Sprite_projeto/01_anatomia_canonica.md §1.2`](../../Sprite_projeto/01_anatomia_canonica.md). Resumo:

```rust
#[derive(Component, Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    pub version: u32,                                  // V4
    pub source: SpriteSource,                          // v3
    pub size: [f32; 2],                                // v3
    pub tint: [f32; 4],                                // v3 — semântica: HERDA pra filhos (cascateia)
    #[serde(default)]
    pub anchor: [f32; 2],                              // v3
    #[serde(skip)]
    pub premultiplied: bool,                           // v3 runtime hint
    
    // NOVOS v4 (13 campos):
    #[serde(default = "default_white")]
    pub self_tint: [f32; 4],                           // NÃO herda
    #[serde(default = "default_per_corner_white")]
    pub per_corner_tint: [[f32; 4]; 4],                // [TL, TR, BL, BR]
    #[serde(default)]
    pub tint_fill: bool,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default = "default_true")]
    pub centered: bool,
    #[serde(default)]
    pub offset: [f32; 2],
    #[serde(default = "default_one_u32")]
    pub hframes: u32,
    #[serde(default = "default_one_u32")]
    pub vframes: u32,
    #[serde(default)]
    pub frame: u32,
    #[serde(default)]
    pub region_enabled: bool,
    #[serde(default)]
    pub region_rect: [f32; 4],
    #[serde(default = "default_region_filter_clip")]
    pub region_filter_clip: bool,
}
```

**Total: 20 campos (5 v3 + 14 v4 + 1 `version` field). FROZEN. Bump exige amendment.**

Contagem: `source` + `size` + `tint` + `anchor` + `premultiplied` (5 v3) + `version` (novo serializable em v4) + `self_tint` + `per_corner_tint` + `tint_fill` + `opacity` + `flip_x` + `flip_y` + `centered` + `offset` + `hframes` + `vframes` + `frame` + `region_enabled` + `region_rect` + `region_filter_clip` (14 v4) = **20 campos**.

### 2.2 Caps congelados (arch-gate `architecture_sprite_inspector_surface`)

| Cap | Valor | Razão |
|---|---|---|
| `Sprite` struct fields | **20 (FROZEN v4)** | Anti-god-struct. Tudo novo vai como Component. |
| `Sprite::VERSION` (const) | **4** | Bump → ADR amendment. |
| `Sprite.version: u32` (serializable field) | **PRESENT em v4** | Habilita versioned dispatch (vide §2.3). |
| `RenderInstance` fields total | **12** (10 GPU-visible vertex attrs + 2 CPU-only) | 7 v3 + 3 novos v4 (per_corner_tint counta como 1 field Rust = `[[f32;4];4]` mas vira 4 vertex attrs @location(9..12)) + 2 (opacity, flip_uv). |
| `RenderInstance` vertex attrs (GPU) | **11** (locations 2..14, com per_corner_tint ocupando 9..12) | wgpu default `max_vertex_attributes = 16` → 5 slots livres |
| `RenderInstance` `size_of` | **144 bytes** | 72 v3 + 64 (per_corner) + 4 (opacity) + 4 (flip_uv). |

Arch-gate em `crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs` (W1.T1.12 cria).

### 2.3 Migrator v3 → v4 (HR-14 obrigatório) — wrapper enum versionado

**Problema do peek direto** (corrigido pós-audit): `Sprite` v3 atual ([crates/ph2d-render/src/sprite.rs:62](../../../crates/ph2d-render/src/sprite.rs#L62)) **NÃO tem campo `version` no struct serializado** — primeiro campo postcard é `source` (enum discriminant + payload). `postcard::from_bytes::<u32>(&bytes[0..4])` retornaria discriminant lixo, não version legível. Peek dispatch original era inviável.

**Decisão: wrapper enum versionado**. Adotar `SpriteVersioned` enum como wire-format canônico:

```rust
#[derive(Serialize, Deserialize)]
pub enum SpriteVersioned {
    V3(SpriteV3),                  // SpriteV3 struct preservada (5 campos legados)
    V4(Sprite),                    // Sprite v4 com 20 campos
}

pub fn load_sprite(bytes: &[u8]) -> Result<Sprite, Error> {
    let versioned: SpriteVersioned = postcard::from_bytes(bytes)?;
    match versioned {
        SpriteVersioned::V3(v3) => Ok(migrate_v3_to_v4(v3)),
        SpriteVersioned::V4(v4) => Ok(v4),
    }
}

pub fn save_sprite(sprite: &Sprite) -> Vec<u8> {
    // Always save as latest (V4).
    postcard::to_allocvec(&SpriteVersioned::V4(*sprite)).unwrap()
}

pub fn migrate_v3_to_v4(v3: SpriteV3) -> Sprite {
    let region_filter_clip = matches!(v3.source, SpriteSource::Atlas { .. });  // logic
    Sprite {
        version: 4,
        source: v3.source,
        size: v3.size,
        tint: v3.tint,
        anchor: v3.anchor,
        premultiplied: v3.premultiplied,
        // defaults benignos pros novos:
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
        region_enabled: false,
        region_rect: [0.0; 4],
        region_filter_clip,
    }
}
```

Postcard serializa enum como `(variant_discriminant: u32, payload: T)`. v3 = `(0u32, SpriteV3)`; v4 = `(1u32, Sprite)`. Dispatch automático, type-safe, sem peek manual.

**Fixtures v3 binárias** (vide §5 implementação): geradas em **W0.T0.12** (CRIADA pós-audit; ANTES do bump v3→v4 ser commitado no código). Sem isso, fixtures pós-bump viram tautologia.

**Sobre `#[serde(default = "fn")]` para campos novos:** ESTRATÉGIA SECUNDÁRIA, não primária. Postcard não garante back-fill silenciosa de campos trailing ausentes — docs postcard explicitamente alerta sobre `serde(default)` ([github.com/jamesmunns/postcard](https://github.com/jamesmunns/postcard)). Primary path = wrapper enum `SpriteVersioned`. `#[serde(default)]` continua nos campos novos como **defesa-em-profundidade** (caso versioned dispatch falhe em corner case, defaults benignos preenchem). **Empirical test obrigatório em W0.T0.13** valida ambos paths.

### 2.4 `RenderInstance` ABI v4 (144 bytes, 12 fields total, 11 GPU vertex attrs)

**wgpu backend cap check (Lens C M5):** 144 bytes < wgpu `max_vertex_buffer_array_stride` default (2048 em Vulkan/Metal/DX12); 13 attrs < `max_vertex_attributes` default (16). 3 slots livres pra futuras features. **WebGPU `downlevel_webgl2_defaults` = 255B stride** — fora-de-escopo v1 (mobile/WebGL2 não target). Verificado em [wgpu Limits](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html).


**WGSL pattern para `flip_uv` bitfield:**

```wgsl
// flip_uv: u32 (bit 0 = flip_x, bit 1 = flip_y)
let flip_x = (instance.flip_uv & 1u) != 0u;
let flip_y = (instance.flip_uv & 2u) != 0u;
let uv = vec2<f32>(
    select(quad_uv.x, 1.0 - quad_uv.x, flip_x),
    select(quad_uv.y, 1.0 - quad_uv.y, flip_y),
);
```

WGSL `&` (bitwise AND) suportado em u32 ([WGSL spec §8.18](https://www.w3.org/TR/WGSL/#bit-expr)). `select(a, b, cond)` é built-in conditional. Sem branch dinâmico; otimização compilador trivial.

```rust
#[repr(C)]
pub struct RenderInstance {
    pub world_pos: [f32; 2],         // @location(2), 8B
    pub size: [f32; 2],              // @location(3), 8B
    pub atlas_uv: [f32; 4],          // @location(4), 16B
    pub tint: [f32; 4],              // @location(5), 16B — collapsed final
    pub rotation: f32,               // @location(6), 4B
    pub premultiplied: f32,          // @location(7), 4B
    pub anchor: [f32; 2],            // @location(8), 8B
    pub per_corner_tint: [[f32; 4]; 4],  // @location(9..12), 64B
    pub opacity: f32,                // @location(13), 4B
    pub flip_uv: u32,                // @location(14), 4B (bitfield: bit0=flip_x, bit1=flip_y)
    pub texture_id: u32,             // CPU-only, 4B
    pub z_order: u32,                // CPU-only, 4B
}
// Total: 144 bytes (4-byte aligned).
```

`vertex_attr_offsets_match_struct` ([sprite.rs:343](../../../crates/ph2d-render/src/sprite.rs#L343)) expandido para 11 attrs.

### 2.5 Mitigação dual-buffer (opcional, decisão W1/W2 com bench)

Se 144 B virar gargalo:
- **Compact buffer (72B):** v3 attrs (world_pos, size, atlas_uv, tint, rotation, premultiplied, anchor).
- **Extended buffer (72B):** per_corner_tint, opacity, flip_uv. Ativado SÓ em batches com gradient detectado.

CPU pre-pass: se todos `per_corner_tint[c]` iguais (≈ flat tint), collapse em `tint` e pula extended buffer.

Trade-off: branch no extract. Bench em W1 decide.

### 2.6 Backward compat — postcard v3 carrega como v4

Cooker `tools/asset-cooker` aceita ambos:
- `load_sprite_asset_versioned()` peek version → despacha.
- `save_sprite_asset()` sempre salva v4 (latest).

Forward-compat: scene file v3 sem novos campos vira sprite v4 com defaults benignos (identidade visual: WHITE tints, opacity 1.0, flip false, etc.).

Backward-compat v4 → v3: **lossy explícito** (perde novos campos). NÃO objetivo.

### 2.7 Fixtures de teste

5 fixtures binárias v3 congeladas em `crates/ph2d-render/tests/fixtures/`:
- `sprite_v3_atlas.postcard` (atlas-backed, anchor=[0,0])
- `sprite_v3_atlas_with_anchor.postcard` (atlas, anchor=[8,-4])
- `sprite_v3_individual.postcard` (individual texture)
- `sprite_v3_premultiplied.postcard` (premultiplied=true, individual)
- `sprite_v3_max_size.postcard` (size=[100,100] world units)

Gerados ANTES do bump v3→v4 (snapshot do estado atual). Test `migrate_sprite_v3_to_v4` carrega cada um e asserta defaults benignos.

---

## 3. Consequências

### 3.1 Positivas

- **HR-14 cumprido** — migrator obrigatório; save files v3 carregam sem perda.
- **20 campos cobrem aparência intrínseca completa** (5 v3 + 1 version + 14 v4) — pesquisa multi-engine absorvida.
- **`#[serde(default)]` cobre 90% dos campos** — só `region_filter_clip` precisa migrator manual.
- **Forward-compat free** — scene file v3 vira v4 automaticamente.
- **ABI v4 expandida sem quebrar batching** — attrs adicionais são extensão, não rearrange.

### 3.2 Negativas

- **`Sprite` POD cresce de ~40B para ~108B** — ~2.7× maior. Aceito (intrínseco da imagem; sem cache miss em hot path porque Sprite acessado raramente comparado a Transform/RenderInstance).
- **`RenderInstance` cresce de 72B para 144B** — dobra upload bandwidth. Mitigação dual-buffer documentada.
- **5 fixtures binárias congeladas** — manutenção em mudanças de schema; aceito como gate de regressão.

### 3.3 Neutras

- **Cooker bump cobre v3 e v4** — single function de carga, automatic dispatch.
- **Componentes ECS opcionais** complementam: tudo NÃO-intrínseco vai pra Components (vide ADR-0074).

---

## 4. Alternativas consideradas

### 4.1 Sem versionamento (v3 muta in-place) — rejeitada

Adicionar campos diretamente sem `version: u32`. **Por que rejeitada:** HR-14 explicitamente exige; sem migrator, save files quebram silenciosamente.

### 4.2 Versionamento por blake3 hash de schema — rejeitada

Hash do schema como ID. **Por que rejeitada:** postcard simples não suporta hash dispatch; `version: u32` explícito é canon e estritamente mais legível.

### 4.3 Per-corner tint como Component opcional — rejeitada

Mover `per_corner_tint` para Component ECS, ausência = flat tint. **Por que rejeitada:** ABI `RenderInstance` SEMPRE precisa de vertex color (mesmo collapsed em flat); 95% dos sprites são flat (zero overhead com default WHITE × 4); Component duplicaria estado de attachment vs vertex format.

### 4.4 Migrator explícito para TODOS os campos novos — rejeitada

Não usar `#[serde(default)]`, escrever migrator manual para os 13 campos. **Por que rejeitada:** `#[serde(default)]` cobre defaults benignos simples; migrator explícito é necessário SÓ para `region_filter_clip` (lógica condicional baseada em source).

### 4.5 ABI v4 sem mitigação dual-buffer (sempre 144B) — aceita por enquanto

Aceitar 144B sempre. **Decisão W1:** bench em massas (10k sprites @ 60Hz) decide se mitigação dual-buffer vale. Por enquanto, manter simples (single buffer, 144B). Dual-buffer fica como ADR-0070-amendment se bench mostrar gargalo.

---

## 5. Implementação (Wave 1)

Tasks T-W.N materializando esta ADR vide [`docs/Sprite_projeto/15_plano_de_implementacao.md §15.2`](../../Sprite_projeto/15_plano_de_implementacao.md).

W1 fecha quando:
- Sprite v4 compila em `ph2d-render`.
- 5 fixtures v3 carregam como v4 (test verde).
- `vertex_attr_offsets_match_struct` verde com 11 attrs.
- `architecture_sprite_inspector_surface` verde (cap **20 fields**; Lens D D1 reconciliado).
- Smoke do Enio: cenário visual atual renderiza idêntico (zero regression).

---

## 6. Open questions

| Q | Resposta |
|---|----------|
| Bench W1 mostra 144B virou gargalo? | Decide em W1.T1.7 com criterion benchmark. Mitigação dual-buffer documentada. |
| Migrator chain v4 → v5 já existe? | Não. Bump v4 → v5 = ADR-0070-amendment quando necessário. Estrutura migrator preserva extensibilidade. |
| Backward-compat v4 → v3 necessária? | **Não objetivo.** Forward-compat OK; backward = lossy explícito. |

---

## 7. Referências

- Spec normativa: [`docs/Sprite_projeto/01_anatomia_canonica.md`](../../Sprite_projeto/01_anatomia_canonica.md) + [`10_schema_versionamento.md`](../../Sprite_projeto/10_schema_versionamento.md) + [`11_arch_gates_e_caps.md`](../../Sprite_projeto/11_arch_gates_e_caps.md).
- ADR pais: [ADR-0069](0069-sprite-inspector-v2.md).
- Código atual: [crates/ph2d-render/src/sprite.rs](../../../crates/ph2d-render/src/sprite.rs).
- HR-14: SKILL_Stack_PH2D_Definitiva.md §HR-14 (assets versionados com migrator).
