# HANDOFF de continuação — line/UIUX, F48 (02/11): Expand e tools do painel sobre a pilha de tintas

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `3a16487f7c` (F48). Linha reaberta em 02/11 sobre o
main `3df5894489`; os commits F47 antes dele ficam intactos. **Nada integrado, nada enviado.** Antes de ler: `cd` + `pwd` +
`git branch --show-current`. Formato anterior: [F47](HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F47.md).

## 1 — Pedido curado (era o ABERTO 1 do F47)

Auditoria do dono: Expand (Offset, Side, Corner, Apply Offset, Outline Stroke, Profile) e as demais tools do painel Vector
contra a pilha F45/F47. Um agente explorador classificou os leitores (quem lê só `fill`/`stroke` em vez de
`paint_stack()`/`draw_order`); **todo defeito foi medido VERMELHO antes da cura**, família num bloco.

## 2 — Mecanismo

Novo `crates/ph2d-vec-scene/src/paint_split.rs`: `Split {Keep, Into(Vec<VecPath>), Fail}`, `VecPath::carry_look_from`
(fill, stroke, fill_hidden, stroke_hidden, paints, base_order, opacity, blend), `VecPath::stroke_alone`,
`VecPath::split_stack(convert)`. Percorre `draw_order`: cada traço visível pergunta a `convert`; o resto vai para peças
vivas por corrida (`run_piece` mantém a ordem interna via `base_order` recomputado; itens ocultos/desligados viajam; corrida
sem nada visível cai). `inked` veste a faixa: cor do traço, opacidade/blend do objecto e os da camada como entrada Fill (vivo
e assado pintam igual), mais a translação X/Y da camada. `Into(vazio)` = traço sumiu (dilate aniquilou).

- **app-vec `expand.rs`:** `expand_layers` OutlineStroke e `power_stroke_layers` passam por `split_stack` (Power Stroke:
  perfil só no traço de BASE, camadas contornadas lisas; sem traço de base visível ⇒ nada). `stroke_view` (geometria dilatada
  via novo `paint_dilate::dilated_geometry`, partilhado com o desenho vivo). `one_object` funde as peças de UM traço num só
  caminho composto (`paint_dilate::merge` agora `pub(crate)`) — a foto da cena 50 mediu 536 objectos de um contorno tracejado
  (comprimentos de traço múltiplos da largura).
- **`align_live.rs`:** `align_items` usa `split_stack` ⇒ Inner/Outer numa camada de traço agora DESENHA e a faixa mantém a
  posição na pilha (antes ia por cima); guarda barata lê `paint_stack`.
- **vec-boolean:** `compound_from` (offset + booleanas), `wear` do pathfinder, `cut.rs` (aberto e fechado) levam o look
  completo; o operando de clip de `aligned_stroke` nasce com look limpo.
- `shape_build.rs`: `carry_look_from` nos restos e no fundido. `fx_silhouette.rs`: gate `lays_a_band` sobre `paint_stack`;
  `regions_of` com paints extra = união das peças de `expand_layers(OutlineStroke)`; a normalização limpa o look todo.
- **vec-render:** `silhouette.rs` `push_path` recusa o exacto-da-fonte quando uma camada move a borda (faixa de traço, ou
  Fill com offset/dilate); `placa.rs` recusa formas com paints extra.
- `svg_export.rs`: moldura = união de `ph2d_vec_render::path_visual_bounds`, margem = 1 unidade de mundo na escala do ficheiro
  (`px_por_unidade`); comentário do teste `one_object_goes_out_alone...` actualizado (24/2400 intacto).
- `pick.rs` `hover_outline` e `blend_live_edit` `pick_preview`: o realce é só o acento (`carry_look_from`).

**Gates (todos vermelhos antes, verdes depois):** app-vec `expand_stack_tests.rs` (11, incl.
`a_dashed_stroke_outlines_into_one_compound_object`, `the_core_keeps_the_order_of_its_own_fills`;
`a_hidden_base_fill_leaves_no_visible_core` é CONTROLO, verde antes) · `align_live_tests` +2 · `shape_build_tests` +1 (só
vermelho com `compound_from` também revertido: o shape_build herda a cura da booleana) · `svg_export_tests` +1 ·
`fx_silhouette_tests` +1 · `pick_look_tests.rs` +1 · vec-boolean `look_tests.rs` (4: offset, booleana, pathfinder, faca) ·
vec-render `silhouette_tests` +1, `placa_tests` +1.

## 3 — Catraca do `ph2d-app-vec` (cura por CORTE)

Medido 94 182 contra tecto 93 637. Mais 24 cabeçalhos `//!` sem leitores de fonte movidos verbatim para
[`docs/Vector Module/50_cabecalhos_do_app_vec.md`](../../Vector%20Module/50_cabecalhos_do_app_vec.md) (2.º lote anotado no
cabeçalho do doc). Verde.

## 4 — Cena nova `PH2D_OBJECT_MODE_SMOKE=50`

`vector_mode_fill_stroke.rs` `etiqueta` (`qual == 5`): rectângulo com degradê de base, camada Stroke branca 0.4 no FUNDO,
camada Fill de sombra com offset, traço de base escuro Inner 0.08, camada Stroke amarela tracejada (3.0/1.5) Outer no TOPO;
`base_order {fill:2, stroke:3}`; abre o grupo «Advanced» e rola até Expand (`rola_ate`, factorado de `abre`). Env
`PH2D_OBJECT_MODE_GESTO=outline` clica Outline Stroke pelo despacho do produto (para fotos). Tabela `vector_mode_scenes.rs`
9..=50 / 43..=50 / handles 31..=36. **Próxima cena: 51.** Fotos em `img_F48/`: `cena50.png` (antes) · `cena50_outline.png`
(depois: 4 objectos, mesmo look).

## 5 — Formato e contadores (DELTA)

- **Sem mudança de formato:** `PROJECT_SCHEMA` e `VEC_SCENE` intocados (214 / 28 do F47).
- Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..50`.
- **Para o integrador:** CLAUDE.md §5.1 UI/UX passa os smokes a `6..50` e troca o link do handoff; a catraca do app-vec soma
  entre linhas.
- Superfície de colisão: vec-scene (`paint_stack.rs`, `paint_split.rs`, re-export `Split` em `lib.rs`), vec-boolean (`lib.rs`
  `compound_from`, pathfinder, cut, align), vec-render (silhouette, placa), app-vec (expand, align_live, paint_dilate,
  shape_build, fx_silhouette, svg_export, pick, blend_live_edit, `vector_mode_*`), 24 cabeçalhos do app-vec,
  `docs/Vector Module/50_cabecalhos_do_app_vec.md`, `img_F48/`.

## 5b — Prova de fecho (02/11)

- fmt e `clippy -D warnings` limpos nas 4 crates (vec-scene, vec-boolean, vec-render, app-vec).
- Testes nomeados verdes: vec-boolean (tests, look_tests, expand, align, cut, pathfinder, offset, silhouette, overlap,
  arrangement; `tests/it`: silhouette_of_a_stroked_shape, offset, boolean) · vec-scene (paint, effect, corner) · vec-render
  (silhouette, placa, visual_bounds, stack_draw, dispatch, standalone, above_effects) · app-vec (expand, align_live,
  shape_build, svg_export, fx_silhouette, fx_live, paint_dilate, pick, blend_live, offset_live, profile_live, contour_live,
  bool_live, live_memo, symmetry_live, paint_stack, canais_campos, marcas, weld, bucket, trim, cut_line) · host-desktop
  (`the_boolean_cooks_before_the_alignment`, `the_width_sliders_author_the_live_profile`, `the_tree_settles_before_the_capture`,
  `outline`).
- Portões verdes: claude_md, loc_cap, hr15, no_literal_color, english, `architecture_vector_contract_surface`,
  `toda_semeadura_de_dobra_passa_pela_porta`, `the_shell_only_shrinks(_and_every_app_crate_too)`.
- **Mutação 13/13 sangrou** (2.ª rodada; a mutação 2 — default do `base_order` em `run_piece` — sobreviveu à 1.ª, fechada por
  `the_core_keeps_the_order_of_its_own_fills`).
- Nota: o censo `the_artless_draw_routes_are_declared`, citado em comentários do vec-render, NÃO existe como teste
  (pré-existente, não tocado).
- **Suíte completa NÃO corrida** (é da integração). Binário `smoke` de 02/11: 2.ª compilação «Finished `smoke` profile
  [optimized] target(s) in 0.39s», zero Compiling.

## 6 — Smoke do dono (cena 50)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=50 ./target/smoke/ph2d-host-desktop`
2. À direita, em «Expand», clique em «Outline Stroke».
3. Deve acontecer: a imagem não muda; a Hierarchy mostra 4 objectos (branco, miolo com sombra, escuro, amarelo tracejado);
   Ctrl+Z devolve UMA forma. Depois, com a forma seleccionada, arraste «Offset» e clique «Apply Offset»: a forma cresce e
   mantém as camadas branca, de sombra e amarela. Em «Fill & Stroke», ⚙ do amarelo → Position Inner: os traços passam para
   dentro.
4. Deu errado se: uma cor sumir ou trocar de lugar, aparecerem centenas de objectos, o offset perder camadas, ou o Ctrl+Z
   precisar de mais de um toque.

**Estado:** o smoke da cena 49, 2.ª rodada, segue NÃO confirmado pelo dono.

## 7 — ABERTO (por ordem)

1. Smoke do dono da cena 50 (e confirmar a 2.ª rodada da 49).
2. **NOVO, medido por leitura (sem gate ainda):** Inner/Outer + «Above Effects» — a faixa alinhada é tinta-como-Fill, logo
   `under_effects`/`filtra` não a retiram e `draw_above_effects` não a redesenha: sai borrada sob os filtros. Cura candidata:
   `align_live` mantém vivos os traços above-effects e o renderer desenha o traço alinhado por clip (`push_clip_with_rule`
   existe em ph2d-vector). Decidir a rota (duas rotas de render alinhado) antes de construir; gate vermelho primeiro.
3. Itens de geometria viva desenham as camadas da pilha com `dilated: None` (dispatch): camada dilatada numa forma que também
   tem geometria viva desenha sem dilatar (pré-existente).
4. Handles de largura/perfil só para o traço de base (= antigo item 4).
5. Hit-test `path_at` ignora por completo a largura dos traços (não é específico da pilha; `widest_stroke` sem chamador).
6. Restantes do F47 §7 (itens 3, 5, 6, 7, 8): Esc no arrasto de linha de painel (`panel_row_drag`); ids de Position fora do
   censo `seam`; `migra_pilha_v23` sem chamadores; ⚙ do traço com nada seleccionado (F44, confirmar com o dono); ⛔ régua
   de ferramentas flutuante com botões fora da janela baixa (430 px; medir no ECRÃ; detalhe no §8 do handoff F40–F45).
7. Fronteira nomeada: resultado multi-peça herda o blend do objecto por peça (exacto para opacidade, não para blend não-Normal
   sobre as próprias peças inferiores; o Illustrator agruparia).

## 8 — Observação de outra linha (não mexida)

`PH2D_CONSTRAINT_SMOKE=7` («Caixa com tampa») imprime «Edit + Regras», mas na foto fica em Object, sem a tool Regras (linha
Constraints; ver F46 §7).

## 9 — Prompt de continuação

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: ABERTO 2 — Inner/Outer + «Above Effects» (a faixa alinhada sai borrada sob os filtros), depois do meu smoke da cena 50 (e da 49, 2.ª rodada).
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F48.md
Meça antes de curar (gate vermelho primeiro), decida a rota de render alinhado antes de construir, família num bloco. Só tokens e i18n, app só em inglês, Auto Key em todo campo. Cena fotografada antes do smoke (próxima = 51). PARE no meu smoke. Sem integrar nem push.
```
