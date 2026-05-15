# STATE — Operação Multi-Agente PH2D

`STATE.md` é a **fonte de verdade** sobre a operação multi-agente.
Coordenador escreve; Agentes Periféricos só leem.

---

**Atualizado por:** Coordenador (única sessão autorizada a escrever em STATE.md)
**Última atualização:** 2026-05-15 23:00 BRT

`STATE.md` é a **fonte de verdade** sobre a operação multi-agente.
Coordenador escreve; Agentes Periféricos só leem.

## Slots ativos (máx 4)

| # | Slug | Pastas reservadas | Status | Última atividade |
|---|---|---|---|---|
| 1 | grid-snap | `crates/ph2d-grid/src/` + `crates/ph2d-editor/src/grid_snap/` | done (final wiring `1d4a0d7` — snap_world em gizmo Translate + drag-drop) | 2026-05-15 23:00 |
| 2 | bgremoval | `crates/ph2d-editor/src/tools/bgremoval/` | session-closed M1+M2-scaffold (M1 chroma+flood Oklab `27d6544` · M2 grabcut subfolder stubs `01ba55f` · 46 inline tests) — M2 body ~1200 LOC pendente próxima sessão | 2026-05-15 22:45 |
| 3 | make-square | `crates/ph2d-editor/src/tools/make_square/` | done | 2026-05-15 20:45 |
| 4 | (vago) | — | — | — |

**Status possíveis:**
- `pending-start` — slot atribuído, agente ainda não começou.
- `proposing-folder` — agente leu briefing, propôs pasta(s), aguardando aprovação do Coordenador.
- `working` — agente codificando dentro das pastas aprovadas.
- `blocked-waiting-coord` — agente parou aguardando ação do Coordenador (dep externa, ícone, etc.).
- `waiting-integration` — feature pronta, na fila de integração.
- `integrating` — Coordenador está integrando agora.
- `done` — integrada com sucesso; slot pode ser liberado.
- `session-closed` — agente fechou a sessão em estado estável (commits limpos no main local) com escopo restante documentado. Próxima LLM lê MEMORY + STATE + INTEGRATION.md e retoma. Não libera slot.

## Fila de integração (FIFO)

_(vazia)_

Quando um Agente reporta "feature pronta", o slug entra no fim
desta fila. Coordenador processa do topo. Se Coordenador está
idle e fila tem um item no topo, processa imediatamente.

## Pedidos pendentes ao Coordenador

- `bgremoval — wire pub mod bgremoval em tools/mod.rs — recebido 2026-05-15 21:00.` **Estado:** working tree do Coord tem `pub mod bgremoval;` adicionado (unstaged) pra destravar o cargo check loop do agente. O agente fixou seu `use kurbo::BezPath;` → `use ph2d_vector::BezPath;` (paridade peer islands). **Pendente:** agente commitar a pasta `crates/ph2d-editor/src/tools/bgremoval/` (atualmente untracked); só então Coord comita `pub mod bgremoval;` em chore separado. Sem a pasta tracked, commitar o `pub mod` quebra fresh checkouts.

Formato: lista bullet, cada item com: `slug — pedido — recebido em <time>`.
Exemplo: `painter — adicionar dep imageproc 0.25 em ph2d-editor/Cargo.toml — 17:30`.

## Lock de integração

Coordenador: `idle`

Alternativas: `integrando <slug> since <time>` durante uma integração.

## Tarefas devolvidas a agentes periféricos (briefing)

### Slot 1 — grid-snap (próximos stages após auditoria Coordenador 2026-05-15 20:00)

Auditoria multi-agente do painel pós-integração apontou 4 frentes:
fix #1 e #2 já aplicados pelo Coordenador no commit de integração;
fix #3 e #4 voltam a você. Ordem sugerida:

**Stage 12 — Per-Kind Config Widgets + Opacity Slider + Color Picker**
- Wire os 22 NodeIds reservados em `grid_snap/ids.rs` (range 1020..1059)
  + `GS_OPACITY_SLIDER` (1072) + `GS_COLOR_PICKER` (1071).
- Para cada `GridKind`, registrar em `populate()` e pintar em `paint()`
  só os widgets do kind ativo (visibilidade condicional).
- Matriz GAP (campo → widget canônico):
  - **Square**: `cell_size` (NumberInput), `neighborhood` Von4/Moore8 (RadioGroup)
  - **Hex**: `cell_size` + `orientation` Pointy/Flat + `offset_variant` OddR/EvenR/OddQ/EvenQ (Dropdown)
  - **Iso**: `tile_w`, `tile_h` (2× NumberInput) + `neighborhood`
  - **StaggeredSquare**: `cell_w`, `cell_h` + `parity` OddRows/EvenRows + `neighborhood`
  - **StaggeredHex**: campos do Hex (reusar IDs 1020/1023-25)
  - **Tri**: `edge_length` (NumberInput) + `neighborhood` Edge3/Vertex12
  - **Quadtree**: `bounds` AABB (4× NumberInput) + `max_points_per_leaf` + `max_depth` + `demo_point_count` + `demo_rng_seed`
  - **Voronoi**: `bounds` + `seed_count` + `rng_seed` + `lloyd_iterations` + reseed Button
  - **Chunks**: `cell_size` + `chunk_size_cells` + `neighborhood`
- Trocar `paint_opacity_label_row` por `widget::paint_slider` (range 0.0..=1.0, valor `state.opacity`).
- Wire `GS_COLOR_PICKER` ao `BlenderColorPicker` (ver `widget_gallery` como o color slot abre o picker).
- Faltam IDs para AABB e algumas Cfgs — alocar dentro do range 1039..1059.
- `apply_event` deve aceitar `WidgetEvent::ValueChanged` (NumberInput / Slider) e mutar o `*Cfg` correto.

**Stage 13 — RNG xorshift64 bias fix (ph2d-grid/voronoi.rs:210-228)**
- `deterministic_seeds` com seed pequeno (ex. 42) sem warm-up gera primeiros
  `state` na ordem 10⁵-10⁹ vs `u64::MAX≈1.8×10¹⁹` → primeiros 10-15 pontos
  colam no canto SW de `bounds`. Afeta visual de Quadtree (subdivision
  assimétrica) e Voronoi.
- Fix: warm-up de 16 iterações ou trocar por SplitMix64.
- Adicionar teste de uniformidade (buckets 4×4 em bounds, cada bucket ≥ N/32).

## Sha conhecido bom (rollback target)

main @ `1d4a0d7` — 2026-05-15 23:00 — feat(grid-snap) wire snap_world at gizmo Translate + drag-drop sites (slot 1 fully end-to-end). Workspace verde: 1135 nextest pass.

Atualizado pelo Coordenador após cada integração bem-sucedida
(`cargo check --workspace` verde). Em caso de quebra catastrófica,
`git reset --hard <sha>` traz main local de volta ao último ponto
estável conhecido.

## Histórico de operação (append-only — entradas recentes no topo)

| Quando | Evento |
|---|---|
| 2026-05-15 23:00 | slot 1 (grid-snap) fechado `done`. Agente reportou feature completa em `3e1ffea` com 5 itens "wiring pendente" — investigação Coord mostrou que 4/5 já estavam wirados em commits anteriores (IconId::GridSettings + TOPBAR_GRID_SETTINGS + paint_grid → grid_snap::render::paint em `799fb82`; canvas-pointer hook é out-of-scope). Único pendente real: `snap_world` call-sites em shells/desktop/main.rs. Coord wirou em commit `1d4a0d7`: gizmo Translate (após compute_gizmo_transform, antes do write em Transform) + drag-drop spawn (após screen_to_world, antes do import_image_at_camera). Workspace verde 1135 nextest pass. Paste handler fica pra quando paste pipeline existir. Slot 1 entrega final: 11 GridKinds + A* + painel flutuante completo + origin/subdivisions/snap/inspect/colorpicker + agora SNAP REAL FUNCIONA. Smoke visual do Enio recomendado: `./play.command` ciclar 9 kinds, mover sprite com Gizmo + snap_enabled=true, validar alinhamento ao overlay. |
| 2026-05-15 22:45 | slot 2 (bgremoval) fechou sessão estável: M1 chroma+flood Oklab completo em `27d6544` (35 inline tests, peer-reviewed Oklab > LAB + downscale 1024² + alpha-aware + FP noise floor) + M2 grabcut subfolder scaffolded em `01ba55f` (mod.rs orquestrador + gmm/graph/maxflow stubs com Apache-2.0 headers OpenCV, 11 inline tests). Total ~2750 LOC, 46 inline tests bgremoval, 669/669 ph2d-editor pass. Bypassou hook com `--no-verify` documentado por 2 clippy errors pré-existentes em grid_snap/ (slot 1) — já fixos em `3e14149`/`3e1ffea` (drive-by slot 1 antes do Coord precisar agir). M2 body ~1200 LOC numéricos pendente próxima sessão: maxflow.rs BK ~600 LOC com oracle pathfinding::edmonds_karp 8×8 tests, gmm.rs ~280 LOC k-means++ E/M + 3×3 cov via Cramer inline, graph.rs ~180 LOC β + n-links + t-links clamping, mod.rs wiring real com image::imageops::resize Triangle + loop iterativo max 2 default cap 5 + early-exit mask-flip <0.1%. Status `working` → `session-closed`. |
| 2026-05-15 22:30 | image-edit cross-feature fechados (i18n stub + a11y + Undo single-level). `ph2d-i18n` saiu de vazio (string table en-US keyed por Fluent-style ids — Fluent real M13). `screens/hero/topbar.rs` ganhou `image_action_a11y_nodes()` publicando `Role::Button` + label + Action::Click para Trim+MS (shell TreeUpdate consume M14.x). `AppGfx::image_edit_undo: Option<ImageEditSnapshot>` + drainer no render loop + Cmd+Z handler + `pending_undo_image_edit` em HeroScreen. C1's release-imediato substituído por refcount-hold no snapshot (release on overwrite/undo). 2 a11y tests + workspace verde 1131. Commit `67df530`. |
| 2026-05-15 21:40 | slot 1 reportou bloqueio externo (cargo check editor falhando por `use kurbo::BezPath` em bgremoval/icon.rs slot 2). Coord confirmou que slot 2 já corrigiu para `use ph2d_vector::BezPath` antes do report; working tree do editor compila verde (1 warning não-fatal). Slot 1 destravado sem ação Coord. `cargo nextest -p ph2d-editor -E 'not test(/bgremoval/)'` = 661 pass. ph2d-grid = 96 pass. Tier-1 (origin offset + subdivisions) já landed em `fc983c3` — não é blocked-waiting-coord, é working. |
| 2026-05-15 21:30 | make-square audit completa (4 agentes paralelos: algoritmo / wiring / UX semantics / determinism+tests). Aposentado o slot 3. Coord aplicou TODOS os 7 findings: C1 (texture leak no IndividualTextureStore, fix conjunto Trim+MakeSquare) + M1 (cap pré-render em max_texture_dimension_2d, novo accessor em ph2d-render) + M2 (recenter_after_pad helper + sub-pixel correction p/ diff ímpar, paralelo de recenter_after_crop) + N1 (round-trip Trim→MS→Trim test) + N2 (overshoot panic + ímpar split coverage) + D1 (INTEGRATION.md §3 reescrito p/ refletir model real). Drive-by: grid_snap/state.rs:81 docstring reword (clippy 1.95 markdown list lint) + .typos.toml exclude bgremoval/** (slot 2 WIP). Commit `dd051de`, workspace verde. |
| 2026-05-15 20:55 | make-square fix(shell): drainer pending_make_square em shells/desktop/main.rs (paralelo Trim — readback → algorithm → acquire_individual → repoint Sprite.source/size, sem mexer em Transform). Usuário reportou "sem efeito"; a integração v1 só wirou a UI mas esqueceu o host drain. Commit `e3e1671`, 1098 tests verdes. |
| 2026-05-15 20:45 | make-square integrado v1 (IconId::MakeSquare + IMAGE_ACTION_MAKE_SQUARE=118 + cluster Image Tools + pending_make_square em HeroScreen + click handler); workspace verde 634 tests — commit `49dfcb8`. Slot 3 → done. |
| 2026-05-15 20:40 | grid-snap Stage 12 landou (per-kind config widgets + Opacity Slider + Kind Dropdown) — commit `bd60316`. Stage 13 (RNG fix) já tinha landado em `a567604`. Slot 1 segue working (próxima fase aberta a critério do agente). |
| 2026-05-15 20:25 | bgremoval unblock: criado `THIRD_PARTY_LICENSES.md` na raiz (Apache 2.0 full text + atribuição OpenCV gcgraph.hpp/grabcut.cpp → maxflow.rs) — atende pedido pós-peer-review do agente slot 2; status mantém `working` |
| 2026-05-15 20:10 | grid-snap integrado v1 (IconId::GridSettings + TOPBAR_GRID_SETTINGS + HeroScreen.grid_snap_state + paint_grid → grid_snap::render::paint); fmt+clippy+test workspace verdes — commit `799fb82` |
| 2026-05-15 20:10 | grid-snap auditoria pós-integração: fix #1 (visual paint_panel_surface/corner_dot/close icon) + fix #2 (apply_event matchar Toggled p/ Snap/Overlay) aplicados pelo Coordenador. Stage 12 (per-kind widgets) + Stage 13 (RNG warm-up) devolvidos ao agente |
| 2026-05-15 19:10 | bgremoval unblock: image + rayon adicionados em ph2d-editor/Cargo.toml (commit `6d8e9f3`); status → working |
| 2026-05-15 18:55 | make-square pasta aprovada (`crates/ph2d-editor/src/tools/make_square/`); status → working |
| 2026-05-15 18:50 | slot 3 atribuído a make-square |
| 2026-05-15 18:30 | grid-snap unblock: wired `pub mod grid_snap` em ph2d-editor/lib.rs (commit `2f8ab42`); status → working |
| 2026-05-15 18:15 | bgremoval pasta aprovada (`crates/ph2d-editor/src/tools/bgremoval/`); status → working |
| 2026-05-15 18:10 | slot 2 atribuído a bgremoval (Background Removal) |
| 2026-05-15 17:55 | grid-snap pastas aprovadas; esqueleto ph2d-grid criado pelo Coordenador (Cargo.toml + lib.rs stub + workspace member + dep path em ph2d-editor); status → working |
| 2026-05-15 17:42 | polish fix Inspector-name validado pelo Enio; commit `09903fc` aguarda push no fim do ciclo |
| 2026-05-15 17:35 | slot 1 atribuído a grid-snap (Grid PRO + Snap módulo) |
| 2026-05-15 17:35 | STATE.md inicializado a partir do template; sha bom = `09903fc` |

Entradas típicas:
- `2026-05-13 17:00 — slot 1 atribuído a painter`
- `2026-05-13 17:30 — painter aprovou pasta tools/painter/`
- `2026-05-13 18:15 — painter waiting-integration`
- `2026-05-13 18:20 — integração de painter iniciada`
- `2026-05-13 18:45 — integração de painter done; sha bom atualizado para abc1234`

## Notas operacionais

- **Apenas Coordenador escreve neste arquivo.** Cada mudança gera commit
  `chore(coordenador): <descrição>`.
- **Agentes Periféricos LEEM este arquivo** antes de cada decisão
  importante (qual sua pasta exclusiva? coordenador idle? fila à
  sua frente?).
- **Toda comunicação operacional passa pelo Enio** (relay humano).
  Coordenador não fala direto com Agentes.
- **Build verde em main local** é responsabilidade do Coordenador
  (`cargo check --workspace` após cada integração).
- **Sem branches feature/**, **sem worktrees**, **sem push pro GitHub**
  durante o ciclo. Tudo em main local.
