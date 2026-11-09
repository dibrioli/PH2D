# Continuação — line/GameEngineFuncional, 09/11 → Align, onda A fechada; próxima: onda B

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` · branch `line/GameEngineFuncional`
· base `main` `f254e42bf1` · HEAD da linha antes desta onda `6209ff5046`. ⛔ Não tocar na `line-GameEngineFuncional02`.
Nada integrado, nenhum push. Antecessor: [`HANDOFF_CONTINUACAO_2026-11-09_ALIGN.md`](HANDOFF_CONTINUACAO_2026-11-09_ALIGN.md) ·
plano: [`28_alinhar_folhas_estudo.md`](../28_alinhar_folhas_estudo.md).

## Decisões do dono que valem (09/11)

- **SEM giro.** Folha = deslocamento + escala + o Pivot de sempre; quadro = só deslocamento; **nada «por animação»**.
- Onion skin = quadro anterior (ténue) + quadro de REFERÊNCIA fixado (outra tinta; omissão = 1.º quadro da animação de entrada;
  clique direito num quadro ▸ *Use as reference*). Linha de chão = base da caixa opaca da referência.
- Ondas: **A** dados + render (FEITA) · **B** prévia «Align» no editor de animações (onion, referência, cruz do Pivot, linha de
  chão; arrastar/escalar a folha) · **C** por quadro (setas 1 px, Shift 10 px; marca na faixa; «Alinhar pelos pés/centro», um
  Ctrl+Z) · **D** cena de smoke com folhas desalinhadas de propósito + fotos (a cena 5 já cobre parte, ver abaixo).

## Commits desta onda (sobre `6209ff5046`)

| Commit | O quê |
|---|---|
| `2396ffa5ab` | `CellOffsets` + escalar/mover a folha; render, caixa do gizmo e encaixe desenham o quadro com o acerto |
| `605f6c4261` | fix(hierarquia, smoke 09/11): um clique numa linha seleciona sempre |
| `3243e8a035` | chore: `clippy -D warnings` limpo nas crates da linha |

## Superfície de colisão / contadores (DELTA)

- `PROJECT_SCHEMA` 225→226, tripla `(226,13,29)`. Registo de componentes 119→120 (`ph2d::ecs::CellOffsets`; catálogo
  «Frame Offsets», categoria Animation, antes de ImageRegion; porta de macro Gravado). **Reconte no integrador**
  (`python3 scripts/schema-recount.py`): outra linha também sobe o schema/registo.
- `ExtraSheet` deixou de ser `Copy` (é `Clone`) e ganhou o último campo `offsets` e `cut()`; `SpriteGrid::cut()` novo. Qualquer
  merge que construa `ExtraSheet { .. }` por literal ou o copie por valor parte.
- `LONG_PRESS` removido (ver Hierarquia): `WidgetEvent::LongPress`, `HierarchyDragState.down_timestamp_ns` e os parâmetros de
  timestamp de `begin_hierarchy_drag` / `begin_panel_row_drag` saíram. Outra linha que os use parte.
- Tocados: `crates/ph2d-ecs/src/sprite_sheets.rs`, `crates/ph2d-anim-sheets/src/align.rs` (+`align_tests.rs`), `apply.rs`,
  `ph2d_render::nudged`, `shells/desktop/src/vec_snap_sprites.rs`, `crates/ph2d-sprite-anim-scene`,
  `crates/ph2d-editor-core/src/interaction/dispatch/pointer_up.rs`, `crates/ph2d-panel-hierarchy/tests/it/`.

## Modelo de dados

- `ph2d_ecs::CellOffsets { cut: [u32;2], cells: BTreeMap<u32,[f32;2]> }`: deslocamento por quadro em metros locais, **chaveado
  pelo corte** (hframes×vframes). Recorte novo invalida por construção: `of` devolve zero se o corte difere; `set` recomeça o mapa
  em corte novo; `[0,0]` remove a entrada.
  - Folha 0: componente `CellOffsets`. Folha k≥1: campo `ExtraSheet.offsets`. Leitor único: `ph2d_ecs::cell_offset(world, e, sheet, cell)`.
- **Mover a folha é o Pivot** (um controlo, uma grandeza): sem campo novo; `align::move_sheet` faz `anchor += d`.
- `align::scale_sheet(world,e,k,f)` assa `f` em tamanho/âncora/px do offset e nos offsets, em torno do pivô (recusa não finito,
  fora de `[1e-3,1e3]`, `f==1`). `align::nudge_cell(world,e,k,cell,by)` soma ao offset atual.
- Verbos expostos: `ImageUseEdit::{ScaleSheet(f32), MoveSheet([f32;2]), NudgeCell{cell,by}}`, agem na folha À VISTA (`apply.rs`).
  `replace_shown_sheet` mantém os offsets antigos; `promote_first_extra` leva-os de/para o componente.
- Deliberado: o **colisor não segue** o acerto por quadro.

## Render

- `ph2d_render::nudged`: `drawn_geometry` inclui o offset; `sheet_geometry` fica CRU (é o que Pivot/Trim editam).
  `SheetView::OwnNudged(Sprite)` para a folha 0 com offset (mantém grade/região/corte).
- `open_sheet_view` (Edit/Painter) **não** aplica offset. Pontos de encaixe leem `drawn_geometry`.
- `open_preview_cell` não aplica offsets: decidir na B se a prévia Align precisa.

## Cena de smoke 5 (`PH2D_GAME_SMOKE=5`)

Novo objecto «Aligned Sheets» em `[1.5,4.2]`: as mesmas duas folhas de «Two Sheets», com a `run` escalada por
`ESCALA_DA_RUN = 2/3` no quadro de script 10 (célula 0,48 → 0,32 m). Fotos novas: `FolhasTroca`, `AlinhadasIdle`, `AlinhadasRun`
(`Foto::vista()` dá câmera mais próxima). A escala é aplicada pelo SCRIPT da cena, não pelo dono.

## Hierarquia: um clique seleciona sempre (`605f6c4261`)

Defeito: o dono precisava de 2 cliques para selecionar «Two Sheets». Causas (duas convergiram em 3 auditorias paralelas):
1. `WidgetEvent::LongPress` (≥ 400 ms) suprimia o Click e já não tinha consumidor (renomear passou a duplo clique). A shell carimba
   eventos com `SystemTime::now()` ao processá-los (`app_state_devices.rs`, `timestamp_ns`), então quadros lentos (animação a tocar)
   esticavam cliques rápidos. Removido o variante e a supressão. `LONG_PRESS_THRESHOLD_NS` fica (usado por `HOLD_NS` do vídeo do Quadro).
2. Tremor > 5 px armava o arraste de reparentar; solto sobre a própria linha (`over_self`) fazia `return` antes de `apply_click` e
   deixava `active` armado. Agora cai no clique, como o arraste de linha de painel já fazia.
Gate: `crates/ph2d-panel-hierarchy/tests/it/one_click_on_an_animated_sprite_row.rs` (3 testes; os 2 do defeito nasceram VERMELHOS).
⛔ Hipóteses mortas, não reinvestigar: registo de acerto na lista rolada, a shell a reverter a seleção, instabilidade do id da linha.

## Prova de fecho (por nome, esta onda)

Verdes: align 7 · cell_edit 6 · use_frame 3 · recut 4 · shown_edit 11 · apply 10 · render sheets 8+1
(`o_render_desenha_o_quadro_com_o_acerto_dele`) · `interaction::dispatch` 191 · `interaction::touch` 15 · hierarquia `os_cartoes` 27 +
`one_click` 3 · `the_shell_only_shrinks` · registry · catálogo ordenado · schema · undo de folhas
(`os_acertos_do_align_desfazem_se_num_passo`, `shells/desktop/src/undo_folhas_tests.rs`) · bgremoval. `clippy -D warnings` e
`cargo fmt --all` limpos nas crates tocadas.

Smoke binário: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build -p ph2d-host-desktop --profile smoke`
(depois `PH2D_GAME_SMOKE=5` sobre o binário `smoke`).

## Smoke do dono (09/11): OK

«Two Sheets» cresce/afunda nos quadros da run; «Aligned Sheets» mantém tamanho e linha dos pés; caixa de seleção encaixa · etiqueta só
em Edit (pendente do round anterior): OK · aviso ao trocar a imagem da folha à vista («New image on this sheet, cut 4 × 1 (was 6 × 2)…»,
run marcada fora da grade): OK · clique único na Hierarquia: OK após a correção.
Passo errado do roteiro: «Ctrl+Z desfaz a escala» — a escala vem do script da cena, não do dono. **Reteste do undo na onda B**, quando
ele escalar com o rato.

## ABERTO

- ⚠️ Teste `ph2d-app-macros` `tutorial_painter::outra_tela_diz_o_tamanho_gravado` aborta por *stack overflow* na thread
  «macro-referencias». Alheio a este diff, **não conferido no main**: o integrador confere.
- Provas de mutação: devidas ao fecho da linha (não feitas).
- Colisor sem offsets por quadro (deliberado); `open_preview_cell` sem offsets (decidir na B).
- Ainda abertos do handoff anterior: «Delete empty animations» não apaga animações de 0 quadros (deliberado); o «+» do cartão
  Animation do Inspector não seleciona a animação nova.

## Próximo: onda B

A prévia «Align» no editor de animações (`ph2d-anim-editor`): quadro anterior + quadro de referência fixado, translúcidos por baixo;
cruz do Pivot; linha de chão; mover/escalar a folha com o rato e deslocar um quadro (setas/arrasto) — tudo pelos verbos
`ImageUseEdit::{MoveSheet, ScaleSheet, NudgeCell}` acima, num único passo de undo por gesto. Antes de construir, MEÇA o que a
composição atual já exprime (CLAUDE.md §5.0). Cada controlo novo: texto de UI em inglês, tokens, nada de hex. Depois C e D.
