# ADR-0070-amendment-5 — `RenderInstance.sampling: u32` (per-node TextureFilter/Repeat, CPU-only)

**Status:** Accepted (W3.T3.11, 2026-05-30) — render shipado + smoke do Enio (§9 Sampling demonstrável).
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.7 ABI.
**Slot rationale:** `-1` reservado (dual-buffer perf), `-2` back-compat, `-3` flip_uv flags, `-4` basis. Próximo livre.
**Spec sections clarified:** `docs/Sprite_projeto/09_sampling.md` (§9.1/§9.3), `docs/Sprite_projeto/01_anatomia_canonica.md` §1.7.
**Reference:** [`crates/ph2d-render/src/sprite.rs`](../../../crates/ph2d-render/src/sprite.rs) (`sampling` + `pack_sampling`/`unpack_sampling`), [`crates/ph2d-render/src/renderer.rs`](../../../crates/ph2d-render/src/renderer.rs) (run grouping + `ensure_atlas_sampler_bg`), [`crates/ph2d-ecs/src/sorting.rs`/`texture_sampling.rs`] (`TextureFilter`/`TextureRepeat` + `resolve_texture_filter/repeat`).

---

## 1. Context

A seção §9 Sampling expõe **TextureFilter** (Inherit/Nearest/Linear) e **TextureRepeat** (Inherit/Disabled/Enabled/Mirror) **por-node**, resolvidos hierarquicamente (sobe a cadeia `ChildOf` até achar um override). O renderer precisa, por instância, saber qual **sampler** usar — mas o sampler é estado de bind group (`@group(1)`), não um vertex attribute. Trocar sampler por-sprite exige agrupar as instâncias por sampler e bindar o certo por run.

## 2. Decision

`RenderInstance` ganha um campo **CPU-only no tail**, `sampling: u32`, empacotando o par resolvido:

```rust
pub sampling: u32,   // filter (low byte) | repeat << 8
// helpers: pack_sampling(filter, repeat) / unpack_sampling(s) -> (filter, repeat)
```

- **ABI:** 156 → **160 bytes** (13 campos). É CPU-only (depois de `texture_id`/`z_order`) → **a layout GPU (148 B / 11 attrs) é inalterada**; nenhum vertex attr move. O `vertex_attr_offsets_match_struct` não é afetado.
- **Render:** o renderer ordena por `(z_order, texture_id, sampling)` e `compute_runs` quebra runs por essa chave. Pra runs de atlas, um bind group por `sampling` é construído lazy (`ensure_atlas_sampler_bg`) com o sampler do filter/repeat resolvido. `0` = `Inherit/Inherit` → sampler default do projeto.
- **Resolução hierárquica** (`resolve_texture_filter`/`resolve_texture_repeat`) acontece no extract; o resultado vira o `sampling` packed. `TextureFilter`/`TextureRepeat` são Components OPCIONAIS (ausência = herda).
- O **repeat mode resolvido** também vai pros bits 3-4 de `flip_uv` (o fragment faz o wrap in-rect) — ver [ADR-0070-amendment-6](0070-amendment-6.md) §UV.

## 3. Why CPU-tail and not a GPU attr

O sampler é selecionado em CPU (escolhe bind group) — não há nada pro vertex/fragment shader ler de `sampling`. Mantê-lo fora da layout GPU preserva os 11 attrs frozen e o gate de offsets, a custo de 4 B/instância no upload (CPU→GPU buffer carrega o struct inteiro, mas o attr layout ignora o tail). Mesmo padrão de `texture_id`/`z_order`.

## 4. Consequences

- Gates re-lockados na época: `render_instance_pod_size_v4` (160), `architecture_sprite_inspector_surface` (13 campos). (Posteriormente amendment-6 → 14, amendment-7 → 16.)
- Per-node sampling em **texturas individuais** (não-atlas) usa o sampler global do store (follow-up documentado); o agrupamento por `sampling` cobre o caminho de atlas.
- Forward: novos eixos de sampling (aniso, mip bias) estendem o packing de `sampling` (ainda há 16 bits livres) sem mexer na layout GPU.

## 5. Provenance

W3.T3.11 (per-node TextureFilter/Repeat), implementação 2026-05-30; smoke do Enio confirmou §9 Sampling demonstrável. Doc retroativa (Phase 8 ADR debt closure, 2026-05-31).
