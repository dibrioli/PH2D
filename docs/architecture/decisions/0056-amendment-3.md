# ADR-0056-amendment-3 — Region→procedural-fill reference (W6 shader graph / W7 diffusion)

**Status:** Accepted (2026-06-05)
**Amends:** [ADR-0056 §2.3](0056-vector-network-data-model.md) (Vector Network data model — `StyleTable` / `FillRef`) + §2.6 (`Ph2dVectorAsset` bounded decode).
**Decisor(es):** Enio + Claude (Coord, sessão Vector W6/W7).
**Trigger:** W6 (`ph2d-vector-fill` procedural shader graph, ADR-0060) e W7 (diffusion-curve mesh gradient) precisam que uma `Region` aponte para um fill PROCEDURAL, não só uma cor sólida. O contrato congelado `StyleTable.fills: BTreeMap<FillRef, FillSolid>` (W1) só guarda cores sólidas — faltava o caminho `Region → FillGraph` / `gradient_id → DiffusionCurveSet` (flagado no `HANDOFF_vector_w6_fill_closed_coord.md §4.1` + `HANDOFF_vector_w7_poisson_cpu_impl.md §4`).

---

## 1. Contexto

`Region.fill: Option<FillRef>` (ADR-0056 §2.3) referencia um fill por índice `u32` na `StyleTable`. Em W1 a `StyleTable` só mapeava `FillRef → FillSolid` (uma cor OKLCH). Os fills procedurais — o `FillGraph` SSA-WGSL do W6 e o `DiffusionCurveSet`/`ColorField` do W7 — vivem **render-side** (`ph2d-vector-fill`), e a `Region` não tinha como apontar pra eles.

Restrições do contrato congelado:
- **`Region` está no cap de 5 campos** (gate `architecture_vector_contract_surface`) — não pode ganhar um 6º campo. A referência tem que reusar o `fill: Option<FillRef>` existente.
- **`ph2d-vector-doc` é desacoplado de `ph2d-vector-fill`** (o data model não conhece o `FillGraph`/`DiffusionCurveSet`) — o doc só pode guardar um id opaco.
- **Wire additivo** (convenção do módulo: campos `#[serde(default)]` appendados, ex. `StrokeStyle.width_profile` / `Ph2dVectorAsset.dormant_fractures`) — sem bump de schema version.
- **Sem vetor de alocação ilimitada** no surface do asset (ADR-0056-amendment-2 R1 lens-D adicionou caps pra `StyleTable.strokes`/`.fills`) — um novo `BTreeMap` exige um cap em `AssetBounds`.

## 2. Decisão

Estender a `StyleTable` aditivamente para que um `FillRef` possa resolver para um fill procedural, mantendo o doc desacoplado e o wire additivo.

### 2.1 Tipos novos (`ph2d-vector-doc::style`)
- **`ProceduralFillKind`** — enum de DISPATCH (qual pipeline render-side resolve o id): `ShaderGraph` (W6 `FillGraph`) | `Diffusion` (W7 `DiffusionCurveSet`). **Cap ≤ 4 variantes** (gate; room pra pattern/image resource-bound sem surface ilimitado). Hoje 2.
- **`ProceduralFill`** — `{ kind: ProceduralFillKind, id: u32, fallback: OklchColor }`. O doc guarda SÓ o kind + um **id opaco** na registry render-side + um **fallback sólido** (preview / tier sem o pipeline procedural / id não-resolvido → graceful degrade per ADR-0053/0068). `#[non_exhaustive]`. Helpers `shader_graph(id, fallback)` / `diffusion(id, fallback)`.

### 2.2 `StyleTable` (additivo)
- Campo novo **`#[serde(default)] procedural: BTreeMap<FillRef, ProceduralFill>`**, na **MESMA namespace de `FillRef`** que `fills`.
- **Resolução:** `resolve_fill(FillRef) -> Option<ResolvedFill>` — `Procedural` tem precedência sobre `Solid`; `None` = ref dangling (region stroke-only). O renderer despacha sobre isso.
- **Id allocation compartilhada:** `insert_fill`/`insert_procedural` usam `next_fill_id()` = max(keys de AMBOS os maps) + 1 → solid e procedural nunca colidem.

### 2.3 Segurança (`Ph2dVectorAsset` bounded decode, §2.6)
- **`AssetBounds.max_style_procedural`** (default `4096`, igual `max_style_fills`) + check em `bounded_decode` (`styles.procedural.len() ≤ cap`). O novo `BTreeMap` é attacker-controllable no asset → mesmo per-field cap que `fills` (não repete o gap que o amendment-2 fechou).

### 2.4 Schema version
**Permanece `PH2D_VECTOR_ASSET_SCHEMA_VERSION = 1`** — additivo per a convenção do módulo (`width_profile`/`dormant_fractures` appendados `#[serde(default)]` sem bump; `.ph2d-vector` é pré-ship, sem arquivos v1 no mundo). O caminho de bump+migrator (postcard é posicional) fica documentado em `postcard_schema.rs` para quando o formato shipar.

## 3. Consequências

- ✅ `Region → fill procedural` viabilizado **sem** tocar `Region` (reusa `fill: Option<FillRef>`), **sem** acoplar `ph2d-vector-doc` a `ph2d-vector-fill` (só id opaco), **sem** bump de schema.
- ✅ Fallback sólido dá graceful-degrade nativo (tier sem GPU / preview).
- ✅ Cap de segurança fecha o vetor de amplificação do novo map.
- ✅ `ProceduralFillKind` pinado no gate (contrato, não verbal).
- ⏭️ **Próximo (Coord):** o renderer resolve `ProceduralFill.id` contra a registry render-side e injeta o `fill_main` (W6) / amostra o `ColorField` (W7) no fragment — embed em `ph2d-vector` / `vector_graph_bridge` (o "smoke live" do Enio). A registry `id → FillGraph`/`DiffusionCurveSet` vive render-side.

## 4. Alternativas rejeitadas

- **`fills: BTreeMap<FillRef, Fill>` (enum Solid|Procedural):** muda o wire dos VALUES (`{color}` → `{"Solid":...}`) → quebra v1 → exige bump+migrator. Rejeitado (não-additivo). (Untagged enum preservaria o wire mas adiciona ambiguidade/erro-ruim.)
- **Embed do `FillGraph` no doc:** acopla `ph2d-vector-doc` → `ph2d-vector-fill` (layering errado: doc é abaixo do render). Rejeitado.
- **Campo novo em `Region`:** `Region` está no cap de 5 (frozen). Rejeitado.

## 5. Gates / testes
- `architecture_vector_contract_surface::procedural_fill_kind_is_capped_at_4_variants` (novo).
- `style::tests::{fill_refs_share_one_namespace…, resolve_fill_prefers_procedural…, style_table_with_procedural_round_trips_postcard}` (novos).
- `bounded_decode` cobre `styles.procedural` (cap enforced).
- 75 lib tests + 13 gate tests verdes; clippy `--all-targets` limpo.
