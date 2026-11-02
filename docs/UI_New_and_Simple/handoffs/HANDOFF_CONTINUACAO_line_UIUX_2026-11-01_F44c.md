# HANDOFF de continuação — line/UIUX, F44b + F44c (01/11) → F45

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `af40c0ab8f` (F44c), **46 commits** sobre
o main. **Nada integrado, nada enviado.** Antes de ler: `cd` + `pwd` + `git branch --show-current`.
Handoff anterior: [F44](HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F44.md) (a F44 reprovou no smoke; §3 dele
é o pedido que a F44b atendeu).

## 1 — F44b (`61d4cd9bf0`) — smoke do dono APROVADO («o layout ficou muito bom»)

- Toda linha de Fill/Stroke é igual: `[cor]◇ [%]◇ [⚙] [👁]◇ [−]` (◇ = losango Auto Key). Base e extras.
- **Olho da base** = `VecPath::fill_hidden` / `stroke_hidden` (a base esconde a tinta sem a perder).
  **VEC_SCENE schema 25 → 26**, **PROJECT_SCHEMA 211 → 212** (`shells/desktop/src/project_schema.rs`;
  211 = `PROJECT_SCHEMA_SEM_TINTA_ESCONDIDA`). Fixture de migração:
  `crates/ph2d-vec-scene/tests/it/fixtures/vec_scene_v25_tinta.bin` + `vec_v25_fixture.rs`.
- **`PaintSlot { Base, Layer(i) }`** (`crates/ph2d-vec-scene/src/paint_slot.rs`): roteia a edição do corpo
  (cor, tipo, degradê, padrão, tracejado…) para a linha cujo ⚙ está aberto. Verbos por slot em
  `ph2d-app-vec` (`paint_stack.rs`, `vector_bridge_*`, `stroke_paint.rs`, `texture_pattern_edit.rs`).
- Canal Auto Key `OFF_EXISTE` da base = o olho da base. O «−» não tem losango. Glifos de Position em
  círculos. O ⚙ do Fill é igual ao do Stroke.
- Fotos: `img_F44b/cena45.png`, `cena46.png`.

## 2 — F44c (`db17fcfbea` + `af40c0ab8f`) — pedidos do smoke do dono 01/11

1. **Position em duas tintas**: `ph2d_editor_core::paint::paint_icon_toned` (`paint_icons.rs`): anel em
   Accent, aresta sólida em Text1; usado por `paint_segmented_icon_group`. Os SVGs
   `stroke-inside/center/outside.svg` viraram só o fallback monocromático.
2. **O gizmo do degradê edita a camada**: `crates/ph2d-vec-render/src/gradient.rs` (desenho, hit, arrasto,
   paradas da rampa) recebe `PaintSlot`; os invólucros da app `paint_stack::grad_hit` / `grad_drag` /
   `open_centre` usam `open_slot()`; o offset da camada vale via `stack_draw::camada_xf`. Teste
   `the_gizmo_edits_the_open_layer_gradient`.
3. **Caixa de cor só cor, sem hex**, **quadrada em TODA linha** (2.ª volta: «caixas quadradas como em
   stroke»); **Weight na mesma linha**, ao lado da cor, em todo traço (base `VECTOR_WIDTH_NUM`; camada:
   id novo `vector_paint_row_width_id(i)` — o evento abre a camada `i` e reenvia `SetValue(VECTOR_PAINT_WIDTH)`;
   canal Auto Key `TRACO_LARGURA` via `marcas.rs`). Linha de preenchimento deixa a coluna do Weight vazia.
4. **Position foi para o corpo do ⚙** de todo traço: `base_position_row`
   (`paint_stroke_settings.rs`, só quando nenhuma camada está aberta) e `layer_position_row`
   (`paint_stack_rows.rs`). Removidas: a linha fixa «Position | Weight» e a linha solta de Width da camada
   (com semente, registo, `ValueChanged` e entrada em `marcas` — eram código morto).
5. **Defeito corrigido**: a largura do traço de camada era mostrada/gravada em METROS do mundo; agora
   `published(…, DocPx)` / `set_width(…, px, DocPx)` — px.
6. Cena 45 termina em modo Node (seta branca, via bus `Click VECTOR_MODE_NODE`) para as alças radiais do
   brilho aparecerem. Fotos: `img_F44c/cena45.png`, `cena46.png`.

## 3 — Prova

- clippy `-D warnings` limpo em 5 crates. Testes por nome, verdes: `f44` (11), `seam_paint_stack`,
  `seam_key_mark`, `architecture_every_editable_field_is_a_channel` (o gate ganhou `paint_line` em `LINHAS`
  e `layer_line` atualizado em `POR_FUNCAO`), `enum_order_matches_svgs`, `canonical_icon_button`,
  `no_magic`, `loc_cap`, `the_shell_only_shrinks`, `marcas`, `paint_stack`, `appearance`, `stroke_present`,
  `vector_mode`, `gradient`, `input_text`. Shell com LOC líquido zero.
- **Sem suíte completa** (é do integrador). **Sem prova de mutação** (ver §5).

## 4 — Smoke da F44c (para o dono; já aprovado nas duas voltas)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=45 ./target/smoke/ph2d-host-desktop` (e `=46`).
2. Inspector do Vector: seções Fill e Stroke; clique no ⚙ de uma linha de traço.
3. Deve acontecer: Position (círculos de duas tintas) está dentro do ⚙; Weight ao lado da cor; caixas de
   cor quadradas; o degradê de uma camada aberta se edita pelo gizmo.
4. Deu errado se: Position reaparece fora do ⚙, o gizmo mexe na camada de baixo, ou a largura de uma
   camada sai 100× fora de escala.

## 5 — ABERTO (carregue)

- Perfil de largura não é por camada de traço.
- Os ids novos de Position não estão no censo de costuras (`seam`).
- **Provas de mutação de F43/F44/F44b/F44c: não corridas** (fecho da linha).
- `migra_pilha_v23` sem chamadores.
- «Apaguei um Fill e ele continuou visível» (dono, antes): nunca reproduzido — duas sondas mostraram o
  Fill removido.
- Integração só por ordem do dono (46 commits; `collision-surface.sh` pelo caminho absoluto).
- Não rastreados antigos (`.log`/`.png` em `img_F30/F33/F34/F40/F41`) não são desta onda.

## 6 — PRÓXIMO ITEM: F45 (dono, 01/11, verbatim)

> «Em vez de duas seções separadas, melhor uma única seção para fill e stroke e que a fila de sobreposição
> funcione na ordem em que são colocadas. Assim fill pode sobrepor stroke e vice-versa. Outra coisa:
> clicar e arrastar o ícone de config deve ser usado para mudar a ordem das camadas fill/stroke»

**Hoje** (`crates/ph2d-vec-render/src/lib.rs`, `draw_path` ~:452–534): base fill → base stroke
(`draw_stroke_with`) → `stack_draw::draw_extra_paints` (camadas em ordem). A base tem posição FIXA
embaixo da pilha. A F45 precisa que fill e stroke da base ocupem uma posição livre numa ÚNICA pilha
ordenada.

**A decidir pelo padrão-ouro (decisão técnica delegada) — MEÇA os leitores primeiro** (`git grep` em
`VecPath::fill`, `.stroke`, `fill_hidden`, `stroke_hidden`, `paints`):
- (A) a base entra na pilha (`PaintEntry`) e `fill`/`stroke` deixam de existir como campos; ou
- (B) a base fica e ganha índices de posição (`fill_z`, `stroke_z`) intercalados com a pilha.
Superfície a tocar, qualquer que seja: modelo + degrau do VEC_SCENE (hoje 26) + `PROJECT_SCHEMA` (hoje
212; reconte com `python3 scripts/schema-recount.py`) + fixture de migração; renderer (`lib.rs`,
`stack_draw.rs`); leitores de hit-test, bounds e export; painel numa única seção «Fill & Stroke» (hoje
`paint_fill_stroke.rs`, duas seções); canais Auto Key (`marcas.rs`); desfazer; `PaintSlot` (já roteia
Base/Layer); verbos `StackVerb` Up/Down (`paint_stack.rs`) existem.

**Arrastar pelo ⚙**: reordena as camadas. Segue a lei do dono
`project-memory/feedback_every_drag_reorder_flies_like_the_kanban_card.md`: a linha arrastada VOA sob o
cursor (`ph2d_editor_core::paint_flight`), as outras abrem vão, e uma linha de inserção em Accent.

**Fecho da F45**: cena de smoke refeita e fotografada (`docs/Components/ferramentas/fotografa_cena.sh`;
próxima cena = **47**), smoke do dono em passos numerados (comando com `cd` · onde clicar · o que
acontece · como saber que falhou), com um caso de fill por cima de stroke e o inverso.

## 7 — Números

Commits sobre o main: **46**. `PROJECT_SCHEMA` = **212** (a F44c não o mexeu). VEC_SCENE = **26**.
Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..46`. Gate `f44`: 11 testes.

## 8 — Prompt de continuação

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: F45 (fill e stroke numa só seção, ordem de sobreposição livre, arrastar pelo ⚙).
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F44c.md (§6 = pedido + plano)
Meça os leitores de fill/stroke ANTES de escolher a representação. Só tokens e i18n, app só em inglês,
Auto Key em todo campo. Cena fotografada antes do smoke; PARE no meu smoke. Sem integrar nem push.
```
