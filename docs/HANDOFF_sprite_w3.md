═══════════════════════════════════════════════════════════════════
HANDOFF — Sprite Inspector v2 · W3 (Sorting · Visibility · Sampling)
Autor: agente solo Coord+Impl · Data: 2026-05-30
Para: o próximo agente (executa a W3; depois W4..W8)
═══════════════════════════════════════════════════════════════════

LEIA PRIMEIRO o mandato + operacional em `docs/HANDOFF_sprite_solo_coord_impl.md`
(§0 padrão-ouro · §1 UI só widget que existe · §2 o LOOP · §6 slot/gates/CI · §7
decisão autônoma). ESTE doc é o ESTADO pós-W2 + as RECEITAS provadas + o PLANO da W3.

───────────────────────────────────────────────────────────────────
§0.5 — PROGRESSO W3 SESSÃO 1 (FUNDAÇÃO DE SORTING ENTREGUE — render-first ⭐)
───────────────────────────────────────────────────────────────────
**3 commits LOCAIS (não pushados).** Full gate VERDE (`nextest --workspace`
+ `clippy --workspace --keep-going`); só resta o flake caracterizado cooker
ISPC-macOS. Fases 1+2 do plano (T3.1–T3.8 + T3.19) 100% fechadas e gateadas.

ENTREGUE:
- **14 components ECS opcionais** em `crates/ph2d-ecs/src/`: `sorting.rs`
  (ZIndexOverride/ZAsRelative/SortingLayer+`LayerId`+`SortingLayers` resource/
  OrderInLayer/YSort+`SortPoint`/SortingGroup/ShowBehindParent/TopLevel) ·
  `masking.rs` (ClipChildren+`ClipMode`/MaskInteraction+`MaskMode`) ·
  `sampling.rs` (TextureFilter+`FilterMode`/TextureRepeat+`RepeatMode`) ·
  `visibility_layer.rs` (VisibilityLayer/OnScreenEnabler+`EnableMode`).
  Todos registrados no `ComponentRegistry` (`register_ecs_components`, 4→18) →
  cooker serializa só quando presentes; cenas legadas byte-idênticas
  (`prefab_cook_hash_is_locked` VERDE). Contadores de registry sincronizados
  em ph2d-render (5→19) + ph2d-script (5→19).
- **Pipeline canônico T3.8** (`sort_key.rs`) = equivalente sorting do
  `propagate_transforms`. `SortKey` lexicográfico 7-estágios.
  `compute_sort_ranks_into(&mut SortScratch, world, inputs)` zero-alloc (HR-3),
  `EntityHashMap` (ADR-0022). Integrado no extract (`sim_extract.rs`): `z_order`
  agora é o rank completo, stampado via query `SimRef` pós-walk; cenas sem
  components reproduzem a ordem DFS anterior EXATA (zero regressão). `SortScratch`
  + `sort_inputs` threaded por `app_state`/`init`/`mod`/`run` como `WorklistBuf`.
- **⚠️ DECISÃO DE COORD gravada no header de `sort_key.rs`:** Z bucketiza ANTES
  de YSort (honra a semântica normativa §5.2-passo-4 Godot; reconcilia contra a
  lista de §5.1). **FLAG: escrever `ADR-0073-amendment-1`** formalizando isso.
- **T3.19 determinismo** (`tests/sorting_pipeline_determinism.rs`): golden
  travado p/ cena canônica 10-sprites/4-níveis (layers/YSort/SortingGroup/
  ShowBehindParent/Z) — ordem `[0,7,3,2,4,5,1,6,8,9]`. Quantização YSort via
  `libm::roundf` (cross-OS).

PRÓXIMO (ordem render-first; cada fase = 1 sessão focada):
1. **§7 Ordering Inspector** — render JÁ pronto (o pipeline aplica tudo).
   Construir infra `InspectorOrderingEdit` análoga a `InspectorSpriteEdit`, MAS
   estes são **components ECS opcionais** (não campos do Sprite): commit =
   read-component-or-default → apply → `SetComponent` (path genérico já existe,
   `EditorCommand::SetComponent{entity,type_id,data}`); editar quando ausente
   = INSERIR o component. Mapa do pipeline Inspector (file:line) abaixo em §3.5.
   **ANTES de pintar:** verificar widgets — Z Index/OrderInLayer/YSort-axis =
   NumberInput (axis = 2 NumberInputs como OffsetX/Y, sem Vec2Editor); checkboxes
   p/ ZAsRelative/ShowBehindParent/TopLevel/YSort.enabled/SortingGroup.sort_at_root;
   **SortingLayer + SortPoint = enum-picker/dropdown** → conferir se existe na
   Gallery/showcase; se NÃO, criar widget + showcase + gate ANTES (NÃO inventar).
   Bump `notes_per_section [_;6]→[_;7]` + `LIVE_SECTION_IDS` + section-count gate.
2. **VisibilityLayer cull** (T3.12) + `Camera2d.cull_mask` — render-first.
3. **TextureFilter/Repeat** (T3.11): **DECISÃO TOMADA (padrão-ouro):** preservar
   a ABI de GPU (vertex layout intacto, `vertex_attr_offsets_match_struct`),
   estender só o tail CPU-only do `RenderInstance` com `filter_mode`/`repeat_mode`
   resolvidos hierárquico no extract, **re-lockar `render_instance_pod_size_v4`
   via `ADR-0070-amendment-5`**, e agrupar draw-calls por sampler (spec §9.1).
   Depois §9 Sampling section.
4. **ClipChildren backbuffer (3 modos) + MaskInteraction** (T3.9/T3.10): pass
   GPU + `clip_children_regression` HEADLESS (render-to-image, spec §6.3 — NÃO
   smoke visual). Depois §8 Visibility section.
5. **OrderDebugOverlay** (T3.17) · audit ≥2 lentes (T3.20) · fix erro-zero ·
   ship + push + CI · smoke do Enio.

§3.5 — MAPA DO PIPELINE INSPECTOR (para §7, file:line):
- Edit enum + snapshot: `crates/ph2d-editor-core/src/screens/hero.rs`
  (`InspectorSpriteInfo` L203, `InspectorSpriteMixed` L286, `SpriteFieldEdit` L315).
- Action: `crates/ph2d-editor-core/src/action_bus.rs:312` (`InspectorSpriteEdit{entity_bits,edit}`).
- Commit: `shells/desktop/src/render_loop/inspector_commits.rs:32` (`apply_sprite_field`),
  loop+`SetComponent` L224; BulkSelect fan-out `mod.rs:583`.
- Snapshot producer + mixed: `shells/desktop/src/render_loop/snapshots.rs`
  (`compute_sprite_mixed` L22, builder L439). Test-sites: `hero/tests.rs:917,1145`.
- Section pattern: `crates/ph2d-panel-inspector/src/sections/{transform,render_source}.rs`
  (3-pernas register/read/paint + threading `y` + `info` param). mod.rs re-export.
- Paint: `paint.rs` (call site L293, `notes_per_section[_;6]` L189, section-id map L202).
- Sync: `sync.rs` (`sync_sprite_fields` checkbox L174 seed-on-entity-changed,
  number L217 per-frame-unless-focused).
- IDs: `crates/ph2d-editor-core/src/ids.rs` (live sections L478, sprite fields L502;
  `hash_node_id("str")`).
- `EditorCommand::SetComponent` (genérico p/ qualquer component):
  `crates/ph2d-ecs/src/scene/commands.rs:30`.

───────────────────────────────────────────────────────────────────
§1 — ESTADO (W2 ENTREGUE + CI VERDE + SMOKE OK)
───────────────────────────────────────────────────────────────────
**W2 fechada e PUSHADA.** `origin/main` em `284d55e` (era `d15fbaa`). CI run
26689643354 **success** (matriz Linux/macOS/Windows + replay-hash + bench, 20m32s).
Smoke do Enio OK ("tudo ok").

W2 entregou (Inspector seções 1-6 + BulkSelect + cascade), tudo render-ready e auditado:
- **Color & Tint (§3.6)** — sub-tabs `[Tint][Self][Corners][Effects]`; per-corner grade
  2×2 + preview de gradiente bilinear ao vivo + Equalize; reusa o BlenderColorPicker.
- **Region (§3.3)** — `region_subrect` no extract (atlas.region_px / individual.dims;
  filter_clip = inset meio-texel CPU, sem shader/ABI) + UI toggle/4 NumberInput/filter_clip.
- **offset/centered (§3.4)** — `Sprite::resolve_anchor(ppm)` resolve centered/offset no
  campo `anchor` existente (extract + caixa do gizmo; picking auto-segue lendo `ri.anchor`).
- **BulkSelect (§3.6 bulk-edit)** — editar com N selecionados aplica a TODOS (fan-out no
  drain); campos divergentes mostram "Mixed" (checkbox Indeterminate · NumberInput em
  branco · swatch com traço). `compute_sprite_mixed` compara 19 campos. Variantes per-axis
  `OffsetX/Y` + `RegionX/Y/W/H` (editar 1 eixo não pisa o irmão divergente — audit D-1).
- **GlobalTint cascade (§4.3)** — `cascade_tint_with_ancestors` no extract: render =
  `self_tint × tint × Π(ancestor.tint)`. O `tint` (modulate) cascateia pros filhos; o
  `self_tint` não. Walk ChildOf O(depth), sem alloc.
- **Refactor** — `sections.rs` (1337 LOC) → `sections/{mod,identity,transform,render_source,
  color_tint,sprite_sheet}.rs`; `sync_sprite_fields` extraído.

ABI/contrato NÃO mudou: `Sprite`=20 campos / `RenderInstance`=12 / 156B (W1 já tinha
congelado tudo que a W2 usa; W2 não tocou o struct).

───────────────────────────────────────────────────────────────────
§2 — ⚠️ A LIÇÃO Nº 1 DA W2 (custou uma sessão longa de ship — NÃO REPITA)
───────────────────────────────────────────────────────────────────
A W2 usou só `cargo check -p` no inner loop e auditou cada incremento. **Sete violações de
gate full-workspace acumularam silenciosamente** e só apareceram no `ship.sh` no fim:
clippy `items_after_test_module` · `no_literal_color` (gradiente) · `hr15` (string movida
no split) · **cooker hash** (o skew do Transform mudou os bytes cozidos; o commit do skew
esqueceu de re-lockar) · `architecture_panel_loc_cap` (split expôs fn de 289 LOC) ·
`no_magic_numeric` (10 constantes sRGB/percent). Foram 5 rodadas de fix no ship.

**REGRA W3:** rode o gate COMPLETO periodicamente DURANTE a wave — não só no fim:
- A cada feature fechada (ou a cada ~3-5 commits): `cargo nextest run --workspace
  --no-fail-fast --cargo-profile ci-test` (acha TODAS as falhas de uma vez) **+** os
  arch-gates do crate tocado. ~6min no slot warm; barato vs 5 rodadas no fim.
- ANTES de ship: `./scripts/ship.sh` (paridade-CI total). Flakes caracterizados que NÃO
  bloqueiam: cooker ISPC no macOS (CI faz retry/skip) · `painter_no_alloc` (passa
  standalone; só flaca sob 16 testes paralelos nos 8 GB; zero relação com Transform).
- Gate que mexe em SERIALIZAÇÃO (component novo serializável, mudança de struct cozido)
  → **re-lockar o `prefab_cook_hash_is_locked`** no mesmo commit (W3 adiciona ≥13
  components — vários serializáveis → o cook hash vai mudar; atualize-o por commit).

───────────────────────────────────────────────────────────────────
§3 — RECEITAS PROVADAS (copie; a W2 estabeleceu o pipeline inteiro)
───────────────────────────────────────────────────────────────────
**Pipeline Inspector→Sprite (campo editável):** o caminho está provado ponta-a-ponta.
  `Sprite` (campo) → snapshot (`shells/desktop/.../snapshots.rs` → `InspectorSpriteInfo`
  em `crates/ph2d-editor-core/src/screens/hero.rs`) → paint (`crates/ph2d-panel-inspector/
  src/sections/*.rs`) → event (`event.rs` → `EditorAction::InspectorSpriteEdit{SpriteFieldEdit}`)
  → sync (`sync.rs`, reflete snapshot↔widget) → commit (`inspector_commits.rs::apply_sprite_field`,
  fan-out por BulkSelect no `mod.rs`) → extract/render (`sim_extract.rs` + `sprite.wgsl`).
  Para um campo novo: declare a variante em `SpriteFieldEdit`, trate em `apply_sprite_field`
  (com clamp), adicione o campo ao snapshot + producer + 2 test-sites, id em `ids.rs`,
  populate + paint (3-pernas: register/hit/event), sync (checkbox no entity_changed;
  number todo frame pulando o focado). **Componente novo (W3)** = mesmo, mas como Component
  ECS separado (não campo do Sprite) — vide §4.

**⚠️ RENDER-FIRST (a disciplina anti-armadilha):** NUNCA ligue UI de um campo que o
  `sim_extract`/shader não aplica. A W2 fez render ANTES da UI em region/offset/cascade.
  **W3 é majoritariamente render:** o pipeline de sorting (T3.8) + ClipChildren + Mask +
  VisibilityLayer mudam a ordem/recorte/cull no extract/render. Faça o RENDER + smoke
  ANTES da UI das seções 7-9.

**BulkSelect:** campo novo editável precisa de flag em `InspectorSpriteMixed` +
  `compute_sprite_mixed` + display Mixed no sync, senão editar em multi-seleção pisa
  divergências em silêncio (audit pegou 2 stomps). Tupla (Vec2/Rect) → variantes per-axis
  (não re-leia o irmão do store; ele tem o valor do primário).

**Cores:** reuse `crate::state::tint_f32_to_u8`/`tint_u8_to_f32` + o swatch→picker→sync.
  NÃO reinvente picker (BlenderColorPicker tem OKLCH). `from_rgba8` de cor de sprite (não
  chrome) → `// LITERAL-COLOR-OK:` na MESMA linha.

**Seção nova no Inspector:** a W2 deixou `sections/` como dir de módulos. Cada submódulo
  abre com `use super::*;` (o `mod.rs` re-exporta os imports `pub(crate)`). Seção live nova
  precisa: id + populate + section-fn em `sections/<nome>.rs` (re-export no `mod.rs`) +
  chamada em `paint.rs` + `pre_populate` (color-dot Plain + `mark_collapsible_section`) +
  bump `LIVE_SECTION_IDS`/`notes_per_section` em `paint.rs`. **Cap:** fn ≤200 LOC e
  arquivo ≤600 LOC (`architecture_panel_loc_cap`) — escreva já em helpers que threadam `y`.
  **Spec §3.0: o Inspector tem 12 seções canônicas FROZEN** (`inspector_section_count_canonical
  == 12`); W3 ATIVA as seções 7/8/9 (já contadas), não cria seção fora das 12.

───────────────────────────────────────────────────────────────────
§4 — O PLANO DA W3 (`15_plano_de_implementacao.md` §15.4 — 21 tasks)
───────────────────────────────────────────────────────────────────
**Objetivo:** Ordering · Visibility · Sampling completos + gate de regressão ClipChildren.
Spec normativa: `docs/Sprite_projeto/` §3.7 (Ordering/Sorting), §3.8 (Visibility),
§3.9 (Sampling), §05 (pipeline canônico de ordering), §02 (components ortogonais).

**13 Components ECS novos** (T3.1-T3.13) — Components separados, NÃO campos do `Sprite`
(presença-opcional = sem custo; vide spec §02). Cada um serializável → **re-lock cook hash**:
- `ZIndexOverride(i32)` + `ZAsRelative(bool)` — Z ausente = DFS counter; presente = forçado.
- `SortingLayer(LayerId)` + Project Settings registrar layers nomeadas.
- `OrderInLayer(i32)`.
- `YSort { enabled, axis, sort_point }` — cascateia.
- `SortingGroup { sort_at_root }` — char multi-peça ordena como unidade.
- `ShowBehindParent` (marker) — filho desenha ANTES do pai.
- `TopLevel` (marker) — quebra cascata de transform+modulate (já existe semântica em §3.2).
- `ClipChildren(Mode)` — 3 modos (Disabled/ClipOnly/ClipAndDraw); regression pixel-test.
- `MaskInteraction { mode, alpha_cutoff }` — stub Mask2D em W3.
- `TextureFilter` + `TextureRepeat` hierárquico (per-node override).
- `VisibilityLayer(u32 bitmask)` + Camera2D `cull_mask`.
- `OnScreenEnabler { rect, mode }`.

**T3.8 — pipeline canônico de ordenação (extract)** ⭐ o foundational da W3. 7 estágios
respeitados (spec §05). É o equivalente sorting do que `propagate_transforms` é pro
transform. **Coord-only/cuidado** (mexe no extract/ordenação de TODO sprite); render-first
+ smoke. T3.19 cross-OS hash test (determinismo).

**3 seções Inspector** (T3.14-T3.16): §7 Ordering/Sorting · §8 Visibility · §9 Sampling.
A §7 usa `Vec2` editor p/ Y-Sort Custom Axis e a §8 usa `Rect2 editor` p/ OnScreenEnabler
— **esses widgets podem não existir** (Rect2Editor/VariantEditor são deliverables W6, spec
§1). Se faltar, crie no widget layer + showcase + gate ANTES de usar (NÃO invente UI). Z
Index usa NumberInput int de range ±i32::MAX/2.

**T3.17** `OrderDebugOverlay` widget + toggle (overlay no canvas: cor da layer + Z + DFS).
**T3.18** `clip_children_regression` (3 fixtures pixel-comparison) — gate ativo.
**T3.19** `sorting_pipeline_determinism` cross-OS (hash idêntico).
**T3.20-21** auditoria ≥2 lentes + fix erro-zero + commit.

**⚠️ Determinismo + CI fan-out (plano §15.x linha ~272):** tasks que tocam determinism
(T3.8 pipeline + T3.19 cross-OS hash) DEVEM ir em branches `feat/sprite-determinism-*`
(o job de determinism cross-OS só roda em push main / PR-to-main; Implementadores não veem
cross-OS gates até o PR). Se solo+push-direto-em-main (como a W2), não se aplica — você VÊ
o gate no CI. Se fan-out, respeite o naming.

**Fechamento W3 (smoke do Enio, plano §15.4):** Z relative cascada · Show Behind Parent ·
YSort top→bottom · ClipChildren 3 modos · Mask VisibleInside · TextureFilter Nearest/Linear
lado-a-lado · OrderDebugOverlay. Smoke `smoke_w3_sorting.scene` (10 sprites, 4 níveis).

───────────────────────────────────────────────────────────────────
§5 — CARRY-OVERS DA W2 (resolva no padrão-ouro quando a fase tocar; NENHUM bloqueia W3)
───────────────────────────────────────────────────────────────────
- **Gizmo skew+anchor box** (snapshots.rs `build_view`): a caixa de seleção DECOMPÕE o
  affine (scale+rotação), então sob skew **E** anchor≠0 (centered/offset) ela desloca do
  quad renderizado. O `GizmoView` (centro+extent+rotação) não representa paralelogramo
  cisalhado — fix real = carregar a basis inteira (mesmo skew-F1; task do subsistema de
  gizmo). offset/centered SEM skew funciona certo. Out-of-scope da W2.
- **`SpriteFieldEdit::Offset([f32;2])` (tupla) sem emissor** — a UI usa OffsetX/OffsetY
  per-axis. Contrato estável (declarado up-front; tratado em apply_sprite_field; futuro
  MCP/script pode usar). Inofensivo; manter ou dropar.
- **Region em multi-select de tamanhos diferentes**: habilitar região semeia o rect só no
  primário (`selected_count==1`); os outros com rect zero somem até o usuário setar.
  Documentado no event.rs. Aceitável (não-destrutivo).
- **`paint_color_tint_section` 289 LOC + `paint_transform_section` 212** em FN_OVERAGE_OK
  (`architecture_panel_loc_cap`): split per-tab/per-row é follow-up smoke-validado (paint
  sem cobertura unitária; split cego arrisca regressão visual que o gate não pega).
- **Cooker serializa Transform/Components BARE (sem versão)** — greenfield aceito. Migrar
  pra formato versionado (VersionedComponent cross-cutting) = ADR separado. W3 adiciona
  components serializáveis sob a mesma regra (re-lock cook hash por commit).
- **`smoke_w2_color_tint.scene` / `smoke_w3_sorting.scene`** referenciados mas o .scene não
  existe como fixture — smoke é visual do Enio. Se quiser gate automatizado, crie o .scene.

───────────────────────────────────────────────────────────────────
§6 — OPERACIONAL (resumo; detalhe no handoff principal §6)
───────────────────────────────────────────────────────────────────
- **Slot warm (CoW):** `bash scripts/slot-seed.sh impl-sprite` → prefixe TODO cargo com o
  `CARGO_TARGET_DIR=.../target-slots/slot-impl-sprite` impresso.
- **Inner loop:** `cargo check -p <crate>`. **Gate full PERIÓDICO** (§2): `nextest
  --workspace --no-fail-fast` + arch-gates do crate, não só no fim.
- **Gates Sprite:** `architecture_sprite_inspector_surface` (Sprite==20/RenderInstance==12/
  size==156) · `render_instance_pod_size_v4` · `sprite_wgsl_valid` (naga) ·
  `vertex_attr_offsets_match_struct` · `node_id_collisions` (ids hasheados; W3 adiciona ~30+
  ids — slugs únicos) · `architecture_panel_loc_cap` (fn≤200/file≤600) · `no_literal_color`
  · `no_magic_numeric` · `hr15_no_hardcoded_ui_strings` (baseline path-keyed — split de
  arquivo re-keia) · `inspector_paint_no_alloc`/`_budget` (HR-3/HR-4) ·
  `transform_determinism`/`transform_versioned_postcard` · **`cooker_determinism::
  prefab_cook_hash_is_locked`** (re-lock ao mudar serialização) · ClipChildren regression (W3).
- **Ship:** `./scripts/ship.sh` (paridade-CI EXATA) → corrija TODO `✗` real (flakes
  caracterizados §2 não contam) → `git push origin main` → babysit (link sempre:
  `gh run list --workflow=spike.yml --limit=1` → run id; matriz ~20-30min).
- **Git anti-colisão:** `git commit --no-verify -m "msg" -- <seus paths>` (escopado).
  Se fan-out W3: edite só a SUA pasta; foundational (T3.8 pipeline) é Coord.

───────────────────────────────────────────────────────────────────
§7 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
- Plano: `Sprite_projeto/15_plano_de_implementacao.md` §15.4 (W3) · §15.5+ (W4..W8).
- Spec: `Sprite_projeto/` §3.7-3.9 (seções 7-9) · §05 (ordering pipeline) · §02 (components)
  · §06 (mask) · §09 (sampling) · §11 (gates/caps) · §16 (i18n).
- ADRs: `architecture/decisions/0069..0074` + amendments (0070-2/-3/-4, 0025-1).
- Pipeline Inspector + extract: `crates/ph2d-panel-inspector/` ·
  `shells/desktop/src/render_loop/{snapshots,sim_extract,inspector_commits,mod}.rs`.
- Núcleo operacional: `CLAUDE.md` · `docs/HANDOFF_sprite_solo_coord_impl.md` (mandato).
- Memória: índice em `MEMORY.md`.

**Confiança:** W2 100% verde na matriz, smoke OK, ABI/spec intactos. A W3 é maior (13
components + pipeline de sorting foundational), mas o pipeline Inspector está provado e a
disciplina render-first + gate-full-periódico evita as duas dores da W2. O substrato está
pronto. Boa — sem medo.
═══════════════════════════════════════════════════════════════════
