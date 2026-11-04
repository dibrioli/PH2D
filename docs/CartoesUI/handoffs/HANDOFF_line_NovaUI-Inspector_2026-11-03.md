# Handoff — linha `NovaUI-Inspector`: Inspector em cartões, estilos de cor do Vector, cola na shell (03/11/2026)

Worktree `Worktrees/line-NovaUI-Inspector`, branch `line/NovaUI-Inspector`, HEAD `6ae6677665`.
Base: `line/NovaUI` `11b6952897`; merge-base com `main` = `3a3896a7ab`. 12 commits à frente da base,
164 ficheiros, +8 831 −1 447 (`git diff --stat 11b6952897 HEAD`). **Não integrada, sem push.**
Leitor: a próxima LLM / o integrador. Plano-mãe: [06](../06_COMO_PORTAR_PARA_O_MAIN.md) §3 · galeria: [handoff GALERIA](HANDOFF_line_NovaUI_2026-11-03_GALERIA.md).

> ⚠️ **O smoke do dono está EM CURSO.** Nada aqui está aprovado por ele; integrar não é aprovar.
> ⚠️ `line/NovaUI` vai avançar (ph2d-91: galeria, scroll_area, date_picker, calendar, rich_editor, level_meter).
> Na integração a galeria de `line/NovaUI` MANDA; `CardCtx::paint_line` fica daqui.

## 1 — Superfície de colisão

**Commits** (`git log --oneline 11b6952897..HEAD`, do mais velho): `03bc6dc718` (família 7, estilos) ·
`fe14fb2343` (6, Inspector) · `9b05cb020d` (8, cola da shell) · `6736824980` (10, cliques reais) ·
`c47127e2da` (Fill/Stroke mostram a base) · `a6f43762d8` · `9095e924bd` · `915e42e3be` (clippy) ·
`45983d376c` (fotos) · `0548397bb8` (portões do fecho) · `d324393af2` (base ↔ estilo) · `6ae6677665`.

**Famílias de 06 §3:** 6 Inspector em cartões · 7 estilos de cor · 8 cola na shell · 9 cenas
`PH2D_VEC_CARTOES_SMOKE=1..7` (router `max_level` 7, `crates/ph2d-app-vec/src/lib.rs`) · 10 testes
(`cartoes_mao.rs`, `o_alfinete_do_inspector.rs`, `a_tinta_de_base_no_cartao.rs`, `o_bloco5/7/8/9/11`,
`o_cadeado_do_cartao_objeto`, `os_*`, `toda_dica_dos_cartoes`). `cards.rs` (791) partido em
`cards.rs`, `cards_corpos.rs`, `cards_pilha.rs`.

**Esquema (recontar na integração: `python3 scripts/schema-recount.py`):**
`VEC_SCENE_SCHEMA_VERSION` 28 → 29 (`crates/ph2d-vec-scene/src/schema.rs:113`) · `PROJECT_SCHEMA` 215 → 216
(`shells/desktop/src/project_schema.rs:515`) · `PROJECT_SCHEMA_SEM_ESTILOS` = 215 (`:518`; `fio_do_projeto`
`<=215` → 28) · tripla `(216, 13, 29)` (`project_schema_tests.rs:357`) · gate `vec_v28_fixture` sobre bytes
gravados por `11b6952897` (sha `919aa42d…`). Sem degrau extra pelo `BASE_FILL`/`BASE_STROKE`.

**Contadores (delta):** shell `.rs` medida agora **188 714** (cat dos `.rs` sob `shells/desktop`, sem `target`) contra
188 892 na base GALERIA = **−178**; tecto `TETO_LOC` 189 041 (inalterado). `ph2d-app-vec` 94 053 → 93 147
(tecto 93 637). Cenas `PH2D_VEC_CARTOES_SMOKE` 0 → 7.

**Crates novas / movidas:** `ph2d-cursor-preso` (lei do cursor preso, nova). Shape Builder (906 linhas) desceu
de `ph2d-app-vec` para `ph2d-vec-boolean` (tecto da crate). `BasePaint`, ids da base e `base_paints` descem para
`ph2d-tool-vector::appearance`. `project_schema_history_v83.rs` saiu da shell: o verbatim já vive em
`docs/archive/project-schema/escada_v83_a_v98.md` (só uma linha difere, reescrita depois).

## 2 — O que um merge pode partir (anote cada um)

- **D25b:** `WidgetStore::publish_add_at_top`/`add_at_top` publicados por `mode_drive`; o par antigo
  `publish_object_mode/object_mode` NÃO volta (fechava o ciclo interaction↔object_mode).
- **`set_cards_on` por thread, chamado em `main()`** (`PH2D_CARDS=0` não liga): sem ele o painel pinta a coluna
  clássica, que os ~130 `seam_*` medem — na linha de teste estavam vermelhos.
- **Linhas do Fill/Stroke:** a linha de teste só listava camadas. Aqui vêm de `Appearance::stack()`, topo em cima
  (lei do main), com a tinta de BASE (olho, «−», hex, opacidade; misto e degradê dizem-no).
- **Opacidade da camada** = campo da LINHA (`vector_paint_row_opacity_id`, F44), semeado em `semeia_as_linhas`; o
  `FieldFace` antigo apontava ao ⚙ do main.
- **`CardCtx::paint_line`:** a lei da coluna estreita (cada caixa e rótulo cabem, senão duas linhas); a galeria usa-a.
- **A base liga-se a estilos:** `ItemId::BASE_FILL` = `u32::MAX`, `BASE_STROKE` = `u32::MAX-1`, recusados por
  `aloca_item`; `solta_bases_mexidas` no passe `ids_da_pilha::passe`, ANTES da captura do undo. Decisão a
  confirmar pelo dono: o estilo manda só no RGB; a alfa é a Opacity da linha e mexer nela não solta.
- **Lista de cartões e estilos:** a linha do ⠿ é a camada aberta desse tipo, senão a de cima.
- **`smoke_bone.rs`/`bone_smoke_foco`** ficam os do main (`=8` = losango F40); a cena dos cartões do Bones é `=9`,
  na linha Bones.
- **Ids da galeria** `CARD_*` → `GAL_CARD_*` (colidiam). Elisão a 220 px: 18 → 16 (Noto). Censo de portas lê
  `property_row/marca.rs`.
- `tests/it/main.rs` e `Cargo.toml` de `ph2d-panel-registry-init`: conflito trivial esperado.

**Colisão com `line/NovaUI-Bones` (ph2d-a4):** o único ficheiro tocado pelas duas é
`shells/desktop/src/render_loop/fase_selection_mirror_bone_focus.rs`. Iguais nas duas: `let (win, cam)`,
`chave::resolve`, a linha de `ph2d-card-rope` no `Cargo.toml`. Daqui: `anda(INSP)`. De lá: `publica_presos`,
`anda(BONES)`, `publica_o_osso`. Ficam lá `fase_bus_drain` (`live_bone`), `fase_skeleton_verbs`, `vec_bone_smoke`, e
os testes `a_lista_de_cartoes_flutua_no_hero`, `o_alfinete_com_corda`, `o_cartao_solto_bloco4`,
`o_cartao_solto_no_canvas`. Saíram daqui `every_pinned_loose_card_moves_with_another_class_and_its_tab_behind`
(T55) e `two_ik_cards_pin_to_two_bones`, que pedem as duas linhas — confirme com o handoff do Bones.

## 3 — Prova de fecho

- **Verdes:** fmt e clippy `-D warnings` nas crates tocadas; `ph2d-editor-core` por nome (DAG, LOC, ids, wiring,
  cores, px, a11y, z-walk); `the_shell_only_shrinks` e `_and_every_app_crate_too`; `ph2d-panel-inspector` inteira
  (30 + 523); `ph2d-panel-registry-init` 193 ok, 2 vermelhos, 33 ignorados (+ os testes novos depois).
- **Vermelhos (ambos já em `line/NovaUI`, agora com ph2d-91):**
  `a_altura_de_abertura_de_um_painel_so_encolhe` (inspector 1507 vs 1488: é a letra Noto, Inter mede 1488;
  painter_layers 1618 vs 1593 «Sync with other tools» quebra; tokens 2377 vs 2302, três fileiras `card-*`) e
  `dentro_de_um_troco_o_valor_arranca_numa_coluna_so` (a galeria diz 3, mede 4).
- **Mutações:** 8 + 20 + 6 aplicadas e restauradas; 19 de 20 da última rodada a vermelho; a sobrevivente era
  código redundante e saiu.
- **Limites honestos:** `a_tinta_de_base` prova o olho pela estrutura dos caminhos pintados, e o hex com `Submit`
  por `apply_event` sem teclas reais; o T55 não tem cobertura nesta linha.
- **NÃO correram:** suíte completa e impactados (da integração).

## 4 — Smokes (binário `./target/smoke/ph2d-host-desktop`; compilar com `--profile smoke`)

Sempre: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI-Inspector`.

- Cenas 1..6: `PH2D_VEC_CARTOES_SMOKE=<n> ./target/smoke/ph2d-host-desktop`.
- Cena 7 (precisa de semente): `mkdir -p /tmp/h7/.ph2d && printf 'loose.1645348144910494528=6892305470752409708,700,140,300\n' > /tmp/h7/.ph2d/sections.txt && HOME=/tmp/h7 PH2D_VEC_CARTOES_SMOKE=7 ./target/smoke/ph2d-host-desktop`
- Coluna estreita: `mkdir -p /tmp/hn/.ph2d && printf '# PH2D layout\nactive=vector\n\n[vector]\ndock_w_left=210\ndock_w_right=220\n' > /tmp/hn/.ph2d/layout.txt && HOME=/tmp/hn PH2D_VEC_CARTOES_SMOKE=1 ./target/smoke/ph2d-host-desktop`
- «+» do topo: `PH2D_OBJECT_MODE_SMOKE=42 ./target/smoke/ph2d-host-desktop`.
- Fotos: `docs/CartoesUI/maquete/inspector_portado/` (`m_*` = aprovada × portada).
- Estado: o smoke do dono está EM CURSO; este doc não o dá por aprovado.

## 5 — ABERTO

- Título do cartão solto a 240 px corta («Platform Pla...»).
- «On Hit» no fim de Physics Body ainda à moda antiga (T29, 2.ª metade).
- Barra FILL de cima vazia numa forma só com camada (lê a base; comportamento do main).
- `layers_differ`: o cartão só mostra as bases, sem «Mixed layers» (o painel Vector diz).
- Camada traço: Position/Cap/Join vivem no ⚙ do painel Vector; o cartão só tem Weight.
- «Used by N layers» do «Edit style» conta também as bases.
- Cura do cartão Camera (05 §8–§9, «se sobrar tempo»): não feita.
- Docs 06 e 07 citam `FOTO_HOME_SEMENTE`, que não existe neste ramo: semeia-se com `HOME=...`.
- Os dois vermelhos do §3 (herdados de `line/NovaUI`) e a decisão da alfa do estilo (§2) esperam o dono/integrador.
