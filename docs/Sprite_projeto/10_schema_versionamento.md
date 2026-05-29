# 10 — Schema versionamento (`Sprite::VERSION` 3 → 4)

## 10.1 Bump v3 → v4

`Sprite` schema bump:
- **v3 (atual):** 5 campos (`source`, `size`, `tint`, `anchor`, `premultiplied`).
- **v4 (esta spec):** **20 campos** (5 v3 + 14 v4 + 1 `version` field serializável — Lens C M1 reconciliação final). Wrapper enum `SpriteVersioned` (vide §10.3) é o caminho **ÚNICO** de back-compat; `#[serde(default = "fn")]` em campos novos é documentário/aspiracional apenas (dead sob postcard — vide §10.4 + [ADR-0070-amendment-2](../architecture/decisions/0070-amendment-2.md)).

> **Nota Lens C M2 sobre `version` field redundante com enum discriminant:**
> Wrapper enum `SpriteVersioned::V4(Sprite)` já carrega discriminant (4 bytes varint) na wire. `Sprite.version: u32` no struct é semanticamente **redundante** (`enum=V4` ↔ `sprite.version=4` devem coincidir). **Decisão:** manter `version: u32` no struct + adicionar `debug_assert!(sprite.version == 4)` no load post-deserialize wrapper. Razão: campo do struct preserva o invariant durante manipulation in-memory (mesmo sem o enum wrapper presente); custo 4 bytes/sprite aceito. Cap fica **20 fields total**.

Schema completo em [01_anatomia_canonica.md §1.2](01_anatomia_canonica.md).

## 10.2 Migrator v3 → v4 (HR-14 obrigatório)

```rust
// crates/ph2d-render/src/sprite_migrator.rs (a criar).

impl Sprite {
    /// Migrator chain v3 → v4. Setado em defaults benignos.
    /// Lê v3 postcard + reescreve como v4.
    pub fn migrate_v3_to_v4(v3: SpriteV3) -> Sprite {
        let region_filter_clip = matches!(v3.source, SpriteSource::Atlas { .. });
        Sprite {
            version: 4,
            source: v3.source,
            size: v3.size,
            tint: v3.tint,
            anchor: v3.anchor,
            premultiplied: v3.premultiplied,
            // novos v4: defaults benignos
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
}
```

## 10.3 Versioned postcard — wrapper enum (corrigido pós-audit)

**Problema do peek direto:** `Sprite` v3 atual ([crates/ph2d-render/src/sprite.rs:62](../../crates/ph2d-render/src/sprite.rs#L62)) **NÃO tem campo `version` serializado** — primeiro campo postcard é `source` (enum `SpriteSource`). `postcard::from_bytes::<u32>(&bytes[0..4])` retornaria discriminant lixo do enum source. Peek dispatch original inviável.

**Decisão canônica:** wrapper enum `SpriteVersioned` (postcard serializa enum como `(variant_discriminant: u32, payload: T)` — dispatch type-safe natural):

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub enum SpriteVersioned {
    V3(SpriteV3),                  // 5 campos legados
    V4(Sprite),                    // 20 campos v4
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
```

Postcard formato: v3 → bytes `[0u32, ...source bytes...]`; v4 → bytes `[1u32, ...sprite v4 bytes...]`. Dispatch automático.

**Atenção:** fixtures v3 binárias **precisam ser geradas no formato `SpriteVersioned::V3(SpriteV3)`** (com discriminant 0), NÃO no formato `SpriteV3` direto. Geração em W0.T0.12 (vide [15_plano_de_implementacao.md](15_plano_de_implementacao.md)).

## 10.4 Back-compat com `#[serde(default = "fn")]`

> **⚠️ SUPERSEDED por [ADR-0070-amendment-2](../architecture/decisions/0070-amendment-2.md) §3 (ratificado 2026-05-28 em W0).** A "Decisão híbrida pós-audit" abaixo está empiricamente FALSIFICADA: postcard 1.1.3 rejeita trailing-missing fields com `Error::DeserializeUnexpectedEnd` (test pin: `crates/ph2d-render/tests/sprite_versioned_postcard.rs::postcard_rejects_trailing_serde_default_on_short_payload`). `#[serde(default = "fn")]` é **dead** sob postcard (não-self-describing positional format). O wrapper enum (§10.3) é caminho ÚNICO de back-compat; o migrator W1.T1.6 é **mandatório** (não fallback). Mantenha as anotações `#[serde(default)]` em v4 fields como documentário/aspiracional para hypothetical self-describing format swap futuro (JSON debug bridge), mas NUNCA as use como tier de back-compat.

Alternativa LEVE ao migrator explícito (HISTÓRICO; não usar): cada campo novo em v4 tem `#[serde(default = "fn")]`. Postcard v3 deserializa em `SpriteV4` direto, com defaults benignos preenchendo os ausentes.

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    pub version: u32,                      // V4
    pub source: SpriteSource,              // v3
    pub size: [f32; 2],                    // v3
    pub tint: [f32; 4],                    // v3
    #[serde(default)]
    pub anchor: [f32; 2],                  // v3 — já tem default
    #[serde(skip)]
    pub premultiplied: bool,               // v3 — serde skip
    
    // novos v4:
    #[serde(default = "default_white")]
    pub self_tint: [f32; 4],
    #[serde(default = "default_per_corner_white")]
    pub per_corner_tint: [[f32; 4]; 4],
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

**Tradeoff:**
- `#[serde(default)]` LEVE — depende de comportamento postcard **não documentado como garantido** ([github.com/jamesmunns/postcard](https://github.com/jamesmunns/postcard) explícita lista atributos suportados; `serde(default)` em campos trailing não está nessa lista).
- Wrapper enum versionado (§10.3) é **PRIMARY PATH** — type-safe, dispatch explícito, sem dependência em postcard behavior não-documentado.

**Decisão híbrida pós-audit (HISTÓRICA — SUPERSEDED):**
- ~~**PRIMARY:** wrapper enum `SpriteVersioned` (§10.3) para carregamento de v3 legacy.~~ → agora **SOLE**.
- ~~**DEFESA-EM-PROFUNDIDADE:** `#[serde(default = "fn")]` em campos novos como fallback caso versioned path falhe em corner case.~~ → **EMPIRICAMENTE FALSO** sob postcard (W0.T0.13 pin); manter as anotações é documentário apenas, NÃO tier funcional.
- **EMPIRICAL VALIDATION (W0.T0.13) EXECUTADA 2026-05-28:** postcard retorna `Error::DeserializeUnexpectedEnd` em trailing-missing; vide [ADR-0070-amendment-2](../architecture/decisions/0070-amendment-2.md) §2 + `crates/ph2d-render/tests/sprite_versioned_postcard.rs`.

**Decisão pós-amendment-2 (LIVE):** wrapper enum `SpriteVersioned` (§10.3) é caminho único; migrator W1.T1.6 é mandatório (não fallback).

Migrator explícito é necessário para `region_filter_clip` (lógica condicional `source == Atlas`) — esse campo NUNCA usa só `#[serde(default)]` path.

## 10.5 RenderInstance ABI v4

[01_anatomia_canonica.md §1.7](01_anatomia_canonica.md) detalha o novo ABI:

```rust
#[repr(C)]
pub struct RenderInstance {
    // existentes v3 (72 bytes):
    pub world_pos: [f32; 2],
    pub size: [f32; 2],
    pub atlas_uv: [f32; 4],
    pub tint: [f32; 4],
    pub rotation: f32,
    pub premultiplied: f32,
    pub anchor: [f32; 2],
    pub texture_id: u32,
    pub z_order: u32,
    
    // novos v4 (+72 bytes):
    pub per_corner_tint: [[f32; 4]; 4],   // 64 bytes @location(9..12)
    pub opacity: f32,                      // 4 bytes @location(13)
    pub flip_uv: u32,                      // 4 bytes @location(14) — bitfield
}
```

Total v4: **144 bytes** (era 72). Alignment 4-byte preservado. Stride dobra; upload bandwidth dobra em massas (10k sprites = +720 KB/frame).

**Gate `vertex_attr_offsets_match_struct`** ([sprite.rs:343-375](../../crates/ph2d-render/src/sprite.rs#L343-L375)) ATIVO; bump cobre os novos 4 attrs:

```rust
let expect = [
    // existentes (passa):
    (2u32, offset_of!(RenderInstance, world_pos) as u64),
    (3, offset_of!(RenderInstance, size) as u64),
    (4, offset_of!(RenderInstance, atlas_uv) as u64),
    (5, offset_of!(RenderInstance, tint) as u64),
    (6, offset_of!(RenderInstance, rotation) as u64),
    (7, offset_of!(RenderInstance, premultiplied) as u64),
    (8, offset_of!(RenderInstance, anchor) as u64),
    // novos v4:
    (9, offset_of!(RenderInstance, per_corner_tint) as u64),  // 4 atts ocupando 9..12
    (13, offset_of!(RenderInstance, opacity) as u64),
    (14, offset_of!(RenderInstance, flip_uv) as u64),
];
```

`VERTEX_ATTRIBUTES` updated com 11 attrs (era 7).

**Mitigação opcional (decisão W2):** se 144 B revelar gargalo em bench (M5 perf gate), splitar em 2 buffers:
- **Compact buffer (72 B):** v3 attrs (mantém batching atual).
- **Extended buffer (72 B):** v4 attrs novos, ativado SÓ em batches com gradient tint detectado.

Trade-off: branch no extract phase. Bench em W2 decide. ADR-0070 documenta a decisão final.

## 10.6 Fixtures de teste v3 → v4

**Geração CRITICAL em W0.T0.12** (NÃO em W1) — antes de qualquer mudança no `Sprite` struct. Sem isso, fixtures pós-bump viram tautologia (v3 forjado pelo migrator reverso, não v3 original).

Em `crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs` (a criar em W0.T0.12, NÃO W1):

```rust
#[test]
fn deserialize_v3_postcard_loads_as_v4_with_defaults() {
    // Fixture binária gerada com SpriteV3 antes do bump.
    let v3_bytes: &[u8] = include_bytes!("fixtures/sprite_v3_atlas.postcard");
    let sprite: Sprite = deserialize_sprite_versioned(v3_bytes).unwrap();
    
    assert_eq!(sprite.version, 4);
    assert_eq!(sprite.self_tint, [1.0; 4]);          // default WHITE
    assert_eq!(sprite.per_corner_tint, [[1.0; 4]; 4]);
    assert_eq!(sprite.tint_fill, false);
    assert_eq!(sprite.opacity, 1.0);
    assert_eq!(sprite.flip_x, false);
    assert_eq!(sprite.centered, true);
    assert_eq!(sprite.hframes, 1);
    assert!(sprite.region_filter_clip);              // Atlas → true via migrator
}

#[test]
fn deserialize_v3_individual_sprite_loads_with_region_filter_clip_false() {
    let v3_bytes: &[u8] = include_bytes!("fixtures/sprite_v3_individual.postcard");
    let sprite: Sprite = deserialize_sprite_versioned(v3_bytes).unwrap();
    assert!(!sprite.region_filter_clip);             // Individual → false via migrator
}

#[test]
fn deserialize_v4_round_trip() {
    let original = Sprite {
        version: 4,
        per_corner_tint: [
            [1.0, 0.0, 0.0, 1.0],  // TL red
            [0.0, 1.0, 0.0, 1.0],  // TR green
            [0.0, 0.0, 1.0, 1.0],  // BL blue
            [1.0, 1.0, 0.0, 1.0],  // BR yellow
        ],
        // ... outros campos ...
    };
    let bytes = postcard::to_allocvec(&original).unwrap();
    let restored: Sprite = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(original, restored);
}
```

Fixtures binárias: **5 sprites v3 canônicos** (atlas, atlas_with_anchor, individual, premultiplied, max_size) gerados pelo cooker antes do bump, congelados em `fixtures/`. Lens D D25 reconciliou drift "3 vs 5".

## 10.7 SpriteFrames schema versionamento

`SpriteFrames` asset também tem `version: u32`. **Sem bump nesta spec** — `SpriteFrames` é asset NOVO (não existia em v3). Schema inicial = v1.

Quando módulo Animation/Timeline futuro adicionar campos (frame events com payload tipado, etc.), bump v1 → v2 com mesma mecânica.

## 10.8 Componentes ECS novos — sem versionamento explícito

Componentes anexáveis (ZIndexOverride, SortingLayer, NamedAnchorList, etc.) são **POD opcionais**; sem schema version. Razão: ausência = "Component não anexado" = comportamento default. Não precisa migrator porque não existem em scene files antigos.

Quando scene file v3 (com Sprite v3) carrega:
- Sprite migra para v4.
- Components novos = não anexados (entidade não tinha eles).
- Comportamento = idêntico ao v3 (porque default benigno).

Save em v4 → adiciona Components anexados ao snapshot. Forward-compat funciona; backward-compat (v4 → v3) NÃO é objetivo (lossy explícito, vide HR-14).

## 10.9 Cooker bump (asset cooker)

`tools/asset-cooker` precisa entender v3 + v4 (carrega qualquer; salva sempre v4). Branch nas:
- `load_sprite_asset_versioned()` — peek version, despacha.
- `save_sprite_asset()` — sempre v4 (latest).

Recook automático na primeira carga? **Não obrigatório** — fica lossy se v3 source ainda existir. Recook explícito via comando.

## 10.10 Gates de regressão

| Gate | O que verifica |
|---|---|
| `tests/migrate_sprite_v3_to_v4.rs` | 5 fixtures v3 carregam como v4 com defaults benignos. |
| `vertex_attr_offsets_match_struct` (existente) | ABI v4 com 11 attrs, offsets corretos. |
| `tests/render_instance_pod_size_v4.rs` | `size_of::<RenderInstance>() == 144`. |
| `tests/sprite_field_count_v4.rs` | Cap: Sprite **== 20 campos FROZEN** (Lens D D1). |

## 10.11 Cap arch-gate

`architecture_sprite_inspector_surface` ([11_arch_gates_e_caps.md](11_arch_gates_e_caps.md)):

```
Sprite struct fields: 20 (v4 FROZEN; Lens C M1 reconciliado em 6+ sites)
RenderInstance fields: 12 (v4 FROZEN; 10 GPU vertex attrs incluindo per_corner_tint×4 + 2 CPU-only)
RenderInstance size_of: 144 bytes (v4 FROZEN)
```

Bump exige ADR-0070-amendment-N. Adicionar 21º campo no Sprite = re-think (Component novo melhor caminho).

## 10.11.1 Amendments + version chain policy (Lens E E22 fix)

Sprite Inspector v2 segue precedente Painter/Vector amendments policy:

- **Bump caps numéricos** (`Sprite struct fields == 21`, `Inspector sections == 13`, etc.) → `ADR-XXXX-amendment-N.md` (incremental N).
- **Bump schema VERSION** (`Sprite::VERSION` 4→5) → nova ADR (não amendment); estende ADR-0070 via reference.
- **Cap-bump + schema-bump combinado** → amendment + nova ADR irmã.
- **Component novo ECS opcional** → vai como amendment de ADR-0074 (Sprite-vs-Component boundary princípio).
- **AnchorData variant novo** (e.g., `Quaternion`, `EntityRef`) → amendment de ADR-0072.

Bump unilateral sem ADR/amendment = bug arquitetural; rejeitado em revisão.

## 10.12 Anti-padrões evitados

1. **Bump sem migrator** ❌ — quebra save files; HR-14 exige migrator.
2. **`#[serde(skip)]` em campo novo** ❌ — só `premultiplied` é runtime hint (legado). Novos campos precisam serializar (per_corner_tint deve sobreviver save/load).
3. **Migrator one-shot sem fixture** ❌ — fixture binária congelada é único teste real; recriar v3 postcard ad-hoc é frágil.
4. **Bump silencioso da ABI sem rodar `vertex_attr_offsets_match_struct`** ❌ — gate existente; força respeitar.
5. **Versionamento por hash em vez de int monotônico** ❌ — postcard schema id confunde; `version: u32` explícito é canon.
