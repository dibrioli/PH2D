# ADR-0046 Amendment 1 — `.ph2d-painter` v2: layer-stack persistível (`LayerStackEntry::Node`)

**Status:** Accepted (2026-05-31)
**Amenda:** [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md) §2.7.1.
**Decisor(es):** Enio + Claude (Coord, sessão Painter W3).
**Triggered by:** Painter W3 (layers) — [02_layers.md §2.1/§2.7](../../Painter_projeto/02_layers.md) + a divergência de modelo runtime/savefile ([HANDOFF_painter_w3_layerstack_divergence_RATIFIED.md](../../HANDOFF_painter_w3_layerstack_divergence_RATIFIED.md)).
**Tags:** amendment, painter, persistence, layers, schema-bump, contract

---

## 1. Contexto

ADR-0046 §2.7.1 congelou o `PaintProject` v1 com `layer_stack: LayerStack` onde
`LayerStackEntry` é um `enum` com **um único variant `Reserved(Vec<u8>) = 0`** —
um stub deliberado, projetado (doc do tipo + audit T1.8 L1-F11) para W3 preencher
com variants reais APÓS o discriminant 0 (forward-compat: files v1 carregam
`Reserved` e migram).

W3 (layers) tem o modelo runtime canônico em `ph2d_tool_painter::layers`
(`LayerStack { arena, root, active, next_id }` + `Layer` + `LayerKind`), com
`LayerId(u64)`. O savefile precisa **persistir esse stack losslessly**. A
ratificação da divergência (Opção A) fixou: runtime = fonte de verdade em
memória; o savefile **serializa** o runtime preenchendo o stub.

---

## 2. Amendment

### 2.1 `SCHEMA_VERSION` 1 → 2

`ph2d_painter_stroke::SCHEMA_VERSION = 2`. Files v1 (stub `Reserved`) migram via
`persistence::migrate_v1_to_v2`. Writer sempre emite v2.

### 2.2 `LayerStackEntry` ganha `Node = 1` (Reserved=0 preservado)

```rust
#[repr(u8)]
pub enum LayerStackEntry {
    Reserved(Vec<u8>) = 0,   // v1 FROZEN — nunca emitido por writer v2
    Node(LayerNode) = 1,     // NEW — uma layer top-level (grupos aninham filhos)
}
```

A `Vec<LayerStackEntry>` é armazenada em **z-order top-first** (índice 0 = topo,
= `root` do runtime); grupos aninham filhos recursivamente; a layer ativa é
marcada por `LAYER_FLAG_ACTIVE`. `next_id` NÃO é serializado — deriva de
`max(id)+1` no load.

```rust
pub struct LayerNode {            // 7 fields (cap gate co-locado)
    pub id: LayerId,              // u32 — ver §2.3
    pub name: String,             // cap MAX_LAYER_NAME_BYTES (256)
    pub kind: LayerNodeKind,
    pub blend_mode: u8,           // BlendMode::to_u8 (>=22 → Normal via from_u8)
    pub opacity: f32,
    pub modifiers: u8,            // LAYER_FLAG_{VISIBLE,LOCKED,ALPHA_LOCKED,CLIPPING,IS_REFERENCE,ACTIVE}
    pub mask: Option<Box<LayerNode>>,
}

#[repr(u8)]
pub enum LayerNodeKind {
    Raster { width: u32, height: u32 } = 0,
    Mask { width: u32, height: u32, inverted: bool } = 1,
    Group { children: Vec<LayerNode>, collapsed: bool } = 2,
}
```

`LayerNode` é **metadata only** — os pixels reconstroem-se via replay de
`history` (ADR-0046 §2.7.1, "replay sobre layer_stack produz pixel-perfect
output"). Os 6 bools modifier do runtime `Layer` viram o bitfield `modifiers`;
o `mask: Option<LayerId>` (referência) vira ownership aninhado. `LayerNodeKind`
tem **3 variants** (mirror do runtime `LayerKind`); os outros "tipos" da spec
§2.1 (clipping / reference / alpha-lock) são modifiers, não kinds.

### 2.3 Id-width: runtime `u64` ↔ savefile `u32` (ponte, não bump)

`device::LayerId(u32)` e os stroke records (`layer_target`) ficam **u32** (sem
mexer em campo congelado). O runtime `LayerId(u64)` mapeia na fronteira: narrow
`u64→u32` no save (assert defensivo `id <= u32::MAX`; inalcançável — cap ≤999,
`next_id` from 1); widen `u32→u64` + `next_id = max+1` no load. Bumpar os stroke
records pra u64 seria mais invasivo (migração + re-lock dos records) — rejeitado.

### 2.4 Migração v1 → v2

`migrate_v1_to_v2`: files v1 (strokes miram `LayerId(0)` default) → reconstrói
**uma raster layer default** (id 0, ativa, dims do canvas) pros strokes
replayarem. Seta `version=2` + `recompute_checksum`.

### 2.5 Validação + cook-hash

- `validate_caps_post_deserialize` valida a árvore recursivamente: name length
  (`MAX_LAYER_NAME_BYTES=256`), profundidade de grupo (`MAX_GROUP_DEPTH=8`,
  mirror runtime), e **contagem total de nodes** (incluindo filhos + masks) ≤
  `MAX_LAYERS` (1000 = 999 + overflow; runtime cap 999 = spec §2.2, Procreate).
- Cook-hash (blake3 sobre os bytes serializados, ADR-0046 §2.7.1) re-locka
  automaticamente no save de qualquer v2. Files v1 verificam com o hash v1
  (pré-migração); a migração re-computa. **Sem fixtures on-disk** (procedurais)
  → nada a regenerar.

### 2.6 Caps preservados (sem amendment)

`PaintProject` ≤12 fields (segue 9 — `layer_stack` já existia) ✓;
`CanvasInfo` ≤8 ✓; `Reserved` discriminant 0 ✓ (gate
`layer_stack_entry_reserved_discriminant_is_zero`).

---

## 3. Consequências

### 3.1 Positivas
- Layer stack persistível losslessly preenchendo o stub como projetado.
- Forward-compat intacto (Reserved=0 frozen; Node=1; novos variants → 2,3,…).
- Block 2 GPU (key `u64`) e o modelo runtime ficam **intactos** (ponte só na
  fronteira de persistência).

### 3.2 Negativas / custos
- Schema bump → migração v1→v2 obrigatória (implementada + testada).
- A ponte runtime↔savefile (arena/tree, u64↔u32, mask nesting) é código do
  implementador em `ph2d-tool-painter` (esta ADR define só o formato congelado).

---

## 4. Implementação

- **Feito (Coord, este commit):** `device.rs` (`LayerStackEntry::Node` + `LayerNode`
  + `LayerNodeKind` + `LAYER_FLAG_*`), `SCHEMA_VERSION=2`, validação recursiva +
  `MAX_GROUP_DEPTH`/`MAX_LAYER_NAME_BYTES`, `migrate_v1_to_v2`. Gates:
  discriminant pins + field-count + round-trip (device.rs); migração + validação
  + round-trip de árvore (`tests/layer_stack_v2.rs`); version pins atualizados.
- **Implementador (`ph2d-tool-painter`):** a ponte `layers::LayerStack` ↔
  `device::LayerStack` (arena/root → tree top-first, u64↔u32, mask → nested,
  active flag, `next_id` derive). Não-bloqueante pro caminho in-memory.

---

## 5. Referências
- ADR-0046 §2.7.1 (formato v1 + stub Reserved).
- ADR-0045 (LayerStack home; adjustment OUT per 02_layers §12.2).
- 02_layers.md §2.1 (6 tipos = 3 kinds + 3 modifiers) + §2.2 (cap 999).
- HANDOFF_painter_w3_layerstack_divergence_RATIFIED.md (Opção A + ponte ii).
