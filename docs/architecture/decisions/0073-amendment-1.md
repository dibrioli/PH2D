# ADR-0073-amendment-1 — Z bucketiza ANTES do YSort (reconcilia spec §5.1 lista vs §5.2 passo-4)

**Status:** Accepted (W3.T3.8/T3.20, 2026-05-30) — pipeline canônico shipado + golden-hash determinístico verde.
**Amends:** [ADR-0073 — Sorting canonical order](0073-sorting-canonical-order.md) §2.1 (pipeline de 7 estágios) + §2.2 (`SortKey`).
**Spec sections reconciled:** `docs/Sprite_projeto/05_ordering_sorting.md` §5.1 (enumeração de estágios) vs §5.2 passo-4 (semântica Godot).
**Reference:** [`crates/ph2d-ecs/src/sort_key.rs`](../../../crates/ph2d-ecs/src/sort_key.rs) (módulo doc "Z-before-YSort reconciliation" + `struct SortKey` + `impl Ord`), [`crates/ph2d-ecs/tests/`] (golden-hash determinism).

---

## 1. Context — a contradição §5.1 vs §5.2

A spec se contradiz sobre a prioridade relativa de **Z** (`ZIndexOverride` + `ZAsRelative`) e **YSort**:

- **§5.1** enumera os estágios com **YSort (estágio 3) ANTES de Z (estágio 4)** — uma lista conceitual.
- **§5.2 passo-4** descreve a semântica operacional: *"Z bucketiza; dentro do bucket o YSort ordena"* (semântica do Godot, que a spec adota por inteiro).

As duas leituras produzem ordens DIFERENTES quando um sprite tem `ZIndexOverride` divergente dentro de uma cascata YSort: se YSort outranquear Z (lista §5.1), um Z divergente NÃO quebra a cascata — errado; se Z outranquear YSort (§5.2), um Z divergente **quebra** a cascata e re-bucketiza — correto (Godot).

## 2. Decision

**Z outranqueia YSort na `SortKey`** (Z bucketiza, YSort ordena dentro do bucket). A ordem lexicográfica canônica de 7 estágios é:

```
1. Viewport (contexto pré-Sprite, Camera2D — não editável no Inspector)
2. SortingLayer (named) + OrderInLayer
3. Z (ZIndexOverride + ZAsRelative cascade)   ← bucketiza
4. YSort (projeção Y cascateada, quantizada)   ← ordena dentro do bucket Z
5. SortingGroup (campos 2–4 computados no group root; subtree contíguo)
6. ShowBehindParent (dobrado no draw_order)
7. DFS counter (fallback)
```

`§5.2 passo-4 é a verdade operacional`; `§5.1 é a enumeração conceitual` e a prioridade relativa Z/YSort é governada por §5.2. A `SortKey` reflete isso: o campo Z tem rank lexicográfico MAIOR que o YSort no `impl Ord`.

## 3. Determinismo cross-OS (HR-5)

A projeção YSort é quantizada via `libm::roundf` (fixed-point, escala ~1/1024 m) — `libm` garante o mesmo resultado bit-a-bit em macOS/Linux/Windows, ao contrário do `f32::round` da std (que pode divergir por backend). O golden-hash da pipeline tranca a ordem; uma mudança de prioridade re-quebraria o hash (gate).

## 4. Consequences

- A doc de §5.1 deve trazer uma nota: *"a lista enumera os estágios; a prioridade Z/YSort segue §5.2 (Z bucketiza, YSort ordena dentro)"* (Coord aplica na spec).
- Zero ambiguidade futura: qualquer ADR/feature de sorting cita esta reconciliação.
- A `SortKey` é a fonte de verdade executável; o gate de determinismo a protege.

## 5. Provenance

Decisão do Coord durante W3.T3.8 (pipeline canônico) + T3.20 (audit), ratificada pelo Enio. A reconciliação já vivia no módulo doc de `sort_key.rs` desde a implementação; este amendment a formaliza como ADR (Phase 8 ADR debt closure, 2026-05-31).
