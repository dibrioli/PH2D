# ADR-0056-amendment-2 — Bounded-decode caps extension (7 new `AssetBounds` defaults)

**Status:** Accepted (2026-05-28)
**Amends:** [ADR-0056 §2.6](0056-vector-network-data-model.md) (Vector Network data model + `Ph2dVectorAsset` postcard schema).
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W1, multi-lens audit cycle).
**Trigger:** R1 audit Lens D (security + test reality) + R2 audit Lens F (cap defaults sanity) caught 7 unbounded fields that allow attacker-controlled `.ph2d-vector` payloads to amplify memory allocation past the original 6 caps' coverage. Original §2.6 list pre-dated the inner-structure analysis.

---

## 1. Contexto

[ADR-0056 §2.6](0056-vector-network-data-model.md) ratified six `AssetBounds` caps for safe deserialization:

| Cap | Default | Guards |
|---|---|---|
| `max_vertices` | 100 000 | `VectorNetwork.vertices` |
| `max_segments` | 200 000 | `VectorNetwork.segments` |
| `max_regions` | 10 000 | `VectorNetwork.regions` |
| `max_edit_log_ops` | 1 000 000 | `EditLog.ops` |
| `max_embedded_assets` | 64 | `Ph2dVectorAsset.embedded_assets` |
| `max_embedded_asset_size` | 16 MB | per `EmbeddedAsset.bytes` |

Round 1 + Round 2 adversarial audits (4 lenses paralelas — Lens D security + Lens F cap-sanity) found seven additional fields where postcard deserialization can amplify memory beyond the §2.6 cap surface area:

- **R1 Lens D HIGH-D4:** `AuthoringMetadata.author` and `app_version` Strings — no per-field cap. 99 MB single-author string fits within `MAX_ASSET_SIZE = 100 MB` legitimately.
- **R1 Lens D HIGH-D5:** `CrdtReplay.peer_clocks: BTreeMap<u64, u64>` — no entry count cap. Activates in W3+ multi-agent CRDT.
- **R1 Lens D (additional):** `StyleTable.strokes` and `.fills` BTreeMaps — no entry caps. Pattern: attacker ships 1M `StrokeStyle` entries, fits in MAX_ASSET_SIZE.
- **R2 Lens F MED-F1:** `Region.segments: SmallVec<[SegmentRef; 16]>` — no per-region cap. Inline 16 spills unbounded to heap. Attacker ships 1 region with 50M segrefs, smuggling past the global `max_regions = 10 000` count.
- **R2 Lens F MED-F2:** `EditLog.snapshots: Vec<(usize, NetworkSnapshot)>` — no cap. Each snapshot clones a full `VectorNetwork`; 100k snapshots × 100k vertices each multiplies memory amplification beyond global caps.

The original §2.6 list mitigated only top-level collection lengths. These seven fields are *interior* — they describe sub-elements of caps that §2.6 already guards (or, in the case of `CrdtReplay`, a top-level Option not enumerated when §2.6 was drafted).

---

## 2. Decisão

Extend [`AssetBounds`](../../../crates/ph2d-vector-doc/src/postcard_schema.rs) with **seven** new fields, with defaults calibrated to legitimate worst-case scenarios uncovered in the Lens-F downstream-consumer simulation:

| Cap | Default | Lens | Guards |
|---|---|---|---|
| `max_author_len` | 256 bytes | R1 D HIGH-D4 | `AuthoringMetadata.author.len()` |
| `max_app_version_len` | 64 bytes | R1 D HIGH-D4 | `AuthoringMetadata.app_version.len()` |
| `max_peer_clocks` | 1 024 | R1 D HIGH-D5 | `CrdtReplay.peer_clocks.len()` |
| `max_style_strokes` | 4 096 | R1 D extension | `StyleTable.strokes.len()` |
| `max_style_fills` | 4 096 | R1 D extension | `StyleTable.fills.len()` |
| `max_region_segments` | 10 000 | R2 F MED-F1 | per `Region.segments.len()` |
| `max_snapshots` | 1 000 | R2 F MED-F2 | `EditLog.snapshots.len()` |

`bounded_decode` enforces each via `check_bound` post-decode (mirrors the §2.6 pattern). Adversarial fixtures exercise each cap in `tests/triangle_round_trip.rs`:

- `bounded_decode_rejects_oversized_author_string`
- `bounded_decode_rejects_oversized_app_version_string`
- `bounded_decode_rejects_oversized_peer_clocks_map`
- `bounded_decode_rejects_oversized_style_strokes_table`
- `bounded_decode_rejects_oversized_style_fills_table`
- `bounded_decode_rejects_oversized_region_segments`
- `bounded_decode_rejects_oversized_snapshots`

Plus `asset_bounds_defaults_match_adr_0056_section_2_6` pins all 13 numerical defaults executavelmente (6 originals + 7 extensions); future drift fails CI.

### 2.1 Justificativa dos defaults (Lens F worst-case scenarios)

- `max_author_len = 256`: cobre CJK 50 chars (×3B UTF-8) + co-authors + suffixes acadêmicos. Generous 2-3× typical.
- `max_app_version_len = 64`: cobre "PH2D 1.0.0-rc.1 (Mac arm64 2026-05-28)" = 40B com folga. **Borderline** para toolchain strings compostas (Lens F LOW-F5); revisar W2+ se community feedback indicar pressure.
- `max_peer_clocks = 1 024`: cobre LLM4SVG community 100 active + 200 historical peers. Folga 3×.
- `max_style_strokes / max_style_fills = 4 096`: cobre brand kit master + procedural illustration. Folga 4×.
- `max_region_segments = 10 000`: cobre 1 region grande tipo "africa coastline" GeoJSON. Limite hard contra OOM via single-region amplification.
- `max_snapshots = 1 000`: consistente com `max_edit_log_ops / 100` (política "snapshot every 100 ops" do `EditLog` struct doc).

### 2.2 MAX_ASSET_SIZE consistency check

Cada cap fica abaixo do que cabe legitimamente em `MAX_ASSET_SIZE = 100 MB`:

- `max_vertices × sizeof(Vertex)` ≈ 100 000 × 24B = 2.4 MB ✓
- `max_peer_clocks × sizeof((u64, u64))` ≈ 1 024 × 16B = 16 KB ✓
- `max_style_*` × sizeof(`StrokeStyle`) ≈ 4 096 × 40B = 160 KB ✓
- `max_region_segments × sizeof(SegmentRef)` ≈ 10 000 × 8B = 80 KB ✓
- `max_snapshots × empty NetworkSnapshot` ≈ 1 000 × ~64B = 64 KB ✓

Sem inconsistência aritmética; MAX_ASSET_SIZE atua como umbrella cap consistente.

---

## 3. Consequências

### 3.1 Positivas

- **7 memory-amplification vetores fechados** que sobreviveriam ao §2.6 original.
- **Test coverage** sobe de 1 cap testado (apenas `max_vertices`) para 8 cobertos (todos os 7 novos + 1 original); 5 caps §2.6 originais ainda têm cobertura indireta apenas (`max_segments`, `max_regions`, `max_edit_log_ops`, `max_embedded_assets`, `max_embedded_asset_size`) — débito documentado para próximo audit.
- **AssetBounds defaults arch-gate** (`asset_bounds_defaults_match_adr_0056_section_2_6`) executa-os como contrato — refactor futuro que silenciosamente baixe um cap falha CI.

### 3.2 Negativas

- **`AssetBounds` field count cresce de 6 → 13.** Caller que constrói custom bounds precisa preencher mais fields (ou usar `..AssetBounds::default()` spread). Trivial.
- **5 caps R1 originalmente sem teste dedicado** ainda na débito (cobertos indiretamente pelos defaults arch-gate, mas sem fixture adversarial individual). Tracker em W1.T1.8 audit lens.

### 3.3 Neutras

- ADR-0056 §2.6 original NÃO é deprecado — os 6 caps continuam válidos. Esta amendment é puramente aditiva.
- `MAX_ASSET_SIZE = 100 MB` permanece umbrella cap; nenhum dos 7 caps novos requer bump.

---

## 4. Implementação

- **Crate:** `crates/ph2d-vector-doc/src/postcard_schema.rs`
  - `AssetBounds`: 7 fields novos + defaults.
  - `bounded_decode`: 7 `check_bound` calls novos.
- **Tests:** `crates/ph2d-vector-doc/tests/triangle_round_trip.rs`
  - 7 fixtures adversariais novas.
  - `asset_bounds_defaults_match_adr_0056_section_2_6` expandido para 13 assertions.

Total LOC delta: ~80 lines (production + tests).

---

## 5. Referências

- [ADR-0056 — Vector Network data model](0056-vector-network-data-model.md) (parent).
- Round 1 audit (4 lenses paralelas) — captured pré-fix em commit `38e6868`..`ee001e7` review.
- Round 2 audit (Lens E + Lens F) — captured pós-R1 em commit `8e723b5` review.
- Memory: [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md), [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md), [`feedback-audit-internal-state-grep`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_internal_state_grep.md).
