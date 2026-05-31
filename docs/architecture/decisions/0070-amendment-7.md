# ADR-0070-amendment-7 — `RenderInstance` CPU-tail `clip_group` + `clip_meta` (ClipChildren + Mask2D stencil grouping)

**Status:** Accepted (W3 Phase 6, 2026-05-30) — ratificado pelo Enio (smoke ClipChildren + Mask Interaction OK).
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.7 ABI.
**Slot rationale:** `-1` segue pré-reservado (dual-buffer perf, ADR-0070 §2.5); `-2` back-compat empírico; `-3` flip_uv flags; `-4` basis; `-5` sampling CPU-tail; `-6` uv_xform GPU @location(15). Este é o próximo slot livre.
**Spec sections clarified:** `docs/Sprite_projeto/01_anatomia_canonica.md` §1.7 (ABI tail), `docs/Sprite_projeto/06_mask_clip.md` §6.2/§6.4.
**Reference:** [`crates/ph2d-render/src/sprite.rs`](../../../crates/ph2d-render/src/sprite.rs) (campos + helpers), [`crates/ph2d-render/tests/render_instance_pod_size_v4.rs`](../../../crates/ph2d-render/tests/render_instance_pod_size_v4.rs), [`crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs`](../../../crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs).

---

## 1. Context

ClipChildren (spec §6.2) recorta os descendentes pela silhueta do node-pai; Mask2D/MaskInteraction (§6.4) mascara responders pela silhueta de uma fonte externa. Ambos são features **de stencil por silhueta** (ver [ADR-0074-amendment-1](0074-amendment-1.md) pela escolha de técnica). Pra renderizá-los, o renderer precisa, por instância, saber:

1. A que **grupo de clip** a instância pertence (clip-parent + seus descendentes formam um grupo; identidade `0` = sem clip);
2. O **papel** dentro do grupo (mask source vs member) + o **alpha cutoff** que binariza a silhueta;
3. O **papel de máscara** (Mask2D source / responder VisibleInside / VisibleOutside).

Esses dados são **CPU-only** — agrupam draw calls e selecionam pipeline/stencil-ref; NÃO viram vertex attribute na maioria das passes (exceção em §3). O agrupamento de clip é por subtree (contíguo em z); o de máscara é global.

## 2. Decision

`RenderInstance` cresce **2 campos CPU-only no tail** (depois de `sampling`), espelhando o padrão de `z_order`/`sampling`:

```rust
pub clip_group: u32,  // 0 = sem clip; senão (clip_parent.z_order_rank + 1) — único, nunca 0
pub clip_meta: u32,   // bitfield de stencil:
                      //   bits 0-1  = clip role: 0 member · 1 mask ClipOnly · 2 mask ClipAndDraw
                      //   bits 8-15 = alpha_cutoff quantizado u8 (round(cutoff*255))
                      //   bits 16-17 = mask role: 0 none · 1 Mask2D source · 2 responder Inside · 3 Outside
```

- **ABI:** 176 → **184 bytes** (164 B GPU / 12 attrs INTACTOS + 12→20 B CPU tail). Os gates passam de `RenderInstance fields == 14` → **16** e `size_of == 176` → **184**. A layout GPU (164 B / 12 attrs) é inalterada — `clip_group`/`clip_meta` são tail CPU, depois de `texture_id`/`z_order`/`sampling`.
- **Identidade / zero-regressão:** `clip_group == 0 && mask_role == 0` → caminho normal byte-idêntico ao legado. Sprites sem clip/mask nunca alocam stencil.
- **`clip_group`** = rank de z do clip-parent + 1: único por clip-parent, garante contiguidade do span (o subtree é contíguo em z — DFS, handoff §1.3) e nunca conflita com o sentinela `0`.
- **`clip_meta`** é empacotado SÓ pelos helpers `pack_clip_meta` / `clip_role` / `clip_cutoff` / `with_mask_role` / `mask_role` — nunca à mão.

## 3. Cutoff no shader sem 17ª vertex attribute

A pipeline **stencil-mark** (ClipChildren mask source + Mask2D source) precisa do `alpha_cutoff` NO fragment pra binarizar a silhueta (`discard` se `texel.a <= cutoff`). O plano original (handoff §1.4) mandava expor `clip_meta` como `@location(16)`. **Isso é impossível:** o limite de device é **16 vertex attributes (locations 0..15)** e já estão TODOS ocupados (quad 0..1 + instance 2..15).

**Decisão:** a pipeline de mark usa uma **vertex layout mínima própria** sobre o MESMO instance buffer, que **reusa `@location(5)`** (normalmente `tint`, não lido pelo mark shader) apontando-o, por offset explícito, ao campo `clip_meta`. O mark shader lê o cutoff de `clip_meta` bits 8-15 via `@location(5)`. Custo zero, sem 17º attr, sem mudar a layout das outras pipelines (cada pipeline interpreta o buffer pela sua própria layout declarada). Como bônus, carregar o cutoff por-instância (em vez de um uniform por-grupo) evita o pitfall de write-ordering do wgpu quando vários grupos compartilham um encoder.

## 4. Por que tail CPU e não bits de `flip_uv`

`flip_uv` (amendment-3) tem bits livres, mas o agrupamento de clip precisa de um `u32` inteiro (`clip_group` = rank, até ~milhões de sprites) — não cabe em bits residuais. `clip_meta`, ao contrário, é um bitfield novo e folgado: **o mask role (bits 16-17) cabe nele a custo ZERO de ABI** — por isso Mask2D NÃO adicionou campo (a feature é global, só precisa de um papel de 2 bits, ver [ADR-0074-amendment-1](0074-amendment-1.md)). `clip_group` é o único campo que justifica os 4 bytes; `clip_meta` carrega clip role + cutoff + mask role juntos.

## 5. Consequences

- **Gates re-lockados:** `render_instance_pod_size_v4` (184), `architecture_sprite_inspector_surface` (field count 16, size 184). O `vertex_attr_offsets_match_struct` é **inalterado** (clip é tail CPU; nenhum attr GPU moveu).
- **Forward-compat:** `clip_meta` bits 2-7 e 18-31 são reservados-zero pra futuros papéis de stencil. `clip_group == u32::MAX` permanece livre como sentinela se um dia precisar.
- **Precedência clip vs mask:** um sprite é clip-participant OU mask-participant, nunca ambos no W3 (o stencil é reusado por pass). O extract dá precedência ao clip (`resolve_mask_meta` não toca instâncias com `clip_group != 0`).
- **Bench:** ABI cresceu 176→184 (+4.5%); re-bench de `sprites_upload_144b_vs_72b` recomendado se um bottleneck de bandwidth aparecer (HR-4). O bench foi atualizado pra construir os campos novos.

## 6. Provenance

- W3 Phase 6 (handoff `docs/HANDOFF_sprite_w3_phase6_clipchildren.md`): implementação solo Coord+Impl, 2026-05-30.
- Desvio do plano §1.4 (`@location(16)`) forçado pelo limite de 16 attrs — descoberto ao validar a pipeline via naga; resolvido por repurpose de `@location(5)`.
- Verificação: `clip_children_regression` (4 px × 3 modos + cutoff) + `mask_interaction_regression` (Inside/Outside/source-invisível × 3 px), ambos passando em GPU; smoke do Enio OK.
