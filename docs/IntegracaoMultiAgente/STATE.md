# STATE — Operação Multi-Agente PH2D

`STATE.md` é a **fonte de verdade** sobre a operação multi-agente.
Coordenador escreve; Agentes Periféricos só leem.

---

**Atualizado por:** Coordenador (única sessão autorizada a escrever em STATE.md)
**Última atualização:** 2026-05-15 20:10 BRT

`STATE.md` é a **fonte de verdade** sobre a operação multi-agente.
Coordenador escreve; Agentes Periféricos só leem.

## Slots ativos (máx 4)

| # | Slug | Pastas reservadas | Status | Última atividade |
|---|---|---|---|---|
| 1 | grid-snap | `crates/ph2d-grid/src/` + `crates/ph2d-editor/src/grid_snap/` | working (Stage 12 + RNG fix) | 2026-05-15 20:10 |
| 2 | bgremoval | `crates/ph2d-editor/src/tools/bgremoval/` | working | 2026-05-15 19:10 |
| 3 | make-square | `crates/ph2d-editor/src/tools/make_square/` | working | 2026-05-15 18:55 |
| 4 | (vago) | — | — | — |

**Status possíveis:**
- `pending-start` — slot atribuído, agente ainda não começou.
- `proposing-folder` — agente leu briefing, propôs pasta(s), aguardando aprovação do Coordenador.
- `working` — agente codificando dentro das pastas aprovadas.
- `blocked-waiting-coord` — agente parou aguardando ação do Coordenador (dep externa, ícone, etc.).
- `waiting-integration` — feature pronta, na fila de integração.
- `integrating` — Coordenador está integrando agora.
- `done` — integrada com sucesso; slot pode ser liberado.

## Fila de integração (FIFO)

_(vazia)_

Quando um Agente reporta "feature pronta", o slug entra no fim
desta fila. Coordenador processa do topo. Se Coordenador está
idle e fila tem um item no topo, processa imediatamente.

## Pedidos pendentes ao Coordenador

_(nenhum)_

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

main @ `799fb82` — 2026-05-15 20:15 — feat(integration) grid-snap wired into editor v1 + post-audit polish (workspace verde: fmt+clippy+nextest)

Atualizado pelo Coordenador após cada integração bem-sucedida
(`cargo check --workspace` verde). Em caso de quebra catastrófica,
`git reset --hard <sha>` traz main local de volta ao último ponto
estável conhecido.

## Histórico de operação (append-only — entradas recentes no topo)

| Quando | Evento |
|---|---|
| 2026-05-15 20:10 | grid-snap integrado v1 (IconId::GridSettings + TOPBAR_GRID_SETTINGS + HeroScreen.grid_snap_state + paint_grid → grid_snap::render::paint); fmt+clippy+test workspace verdes |
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
