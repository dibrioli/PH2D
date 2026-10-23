# CONTINUAÇÃO — `line/MiroClone` W24: a barra de estilo foge do chrome, riscas de longe, doc 17 §6 — 2026-10-23

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO6_SKETCH_BARRA](HANDOFF_CONTINUACAO_W23_BLOCO6_SKETCH_BARRA_2026-10-22.md)
> · lista-mãe: [HANDOFF_INTEGRACAO_line_MiroClone_W23_2026-10-22](HANDOFF_INTEGRACAO_line_MiroClone_W23_2026-10-22.md) §8 (ABERTO 1–5)
> · plano/medidas: [16_bloco4_velocidade.md](../16_bloco4_velocidade.md) §5 · [17_bloco6_plano.md](../17_bloco6_plano.md) §6.
> Base = main `972f411a6f` (linha reaberta por fast-forward em 23/10). Commits (`git log main..HEAD`): `6ad11719f8`,
> `a9cfdddbc0`, `fbceb7352e`, `bd06efd3e7`, `17fccb4c9c`, `9952261792` (fmt). Sem integração nem push: só por ordem do Enio.

## 1. Feito

**(a) ABERTO 1+2 da lista-mãe, UM bloco — a barra de estilo foge de todo o chrome.** `float_bar::strip_rects_clear_of(area, sel,
groups, s, gap, keep)` substitui `strip_rects_fitting` (só o quadro a usava).
- Regra: as formas da barra (`strip_layouts`: 1 linha; k linhas equilibradas; com a sala mais estreita que um grupo, grupos
  partidos em pedaços de c botões, um por linha) são tentadas por ordem; cada uma por cima da selecção, senão por baixo (clamp
  como antes), deslizada para o vão livre mais perto do centro (`free_x`; folga = `Spacing::Xs` + `gap` — a foto encostava os fundos).
  Sem vão, para TODAS as formas, a altura livre mais perto (rente a um `keep` ou bordo). Último recurso: a forma mais baixa que cabe
  na largura, y preso à área, x nunca à esquerda da sala.
- `board_bar_layout::style_rects_of` ganhou `keep: &[Rect]` (`style_rects` passa `&[]`).
- `keep` = `board_nav::fit::painted(nav)` = `Nav::covers` (painéis fixos, já existentes para o «enquadrar»: barra curta, canto,
  frames, busca, equipa, facilitação, Live, minimapa, HUD) ∪ `Nav::floats` (NOVO em `documents_nav.rs`, limpo no `fit::settle`;
  registo por `fit::float(nav, r)`): gavetas abertas (formas, notas, caneta, modelos — registadas em `board_bar::paint` pelas mesmas
  fns de rect), aviso do Organizar (`paint_notice` agora devolve o rect), alerta (`board_alert`), cartão de um fio e mensagem da
  equipa (`board_team_paint`), lista de menções (`board_team_mention`), barra do vídeo e faixa «instale» (`board_video_bar`, só fora
  do ecrã inteiro).
- Ordem de pintura em `paint.rs`: nav → alerta → vídeo → `board_bar` (a de estilo por ÚLTIMO); no ecrã inteiro só o vídeo.
- **Decisões:** bolinhas dos fios FORA (são marcas no desenho); modais (menu, Send to board, apresentação) fora.
- Achado pelo gate: o 1.º último-recurso pegava na forma mais estreita (coluna mais alta que a janela, y = −470 a 260×420) —
  curado em `a9cfdddbc0`.

**(b) Cena 29** (`PH2D_BOARD_SMOKE=29`, `max_level` 28→29; `ph2d-app-board/src/smoke_w24_bars.rs`; textos
`ph2d-i18n/src/board_w24.rs`, chaves `board.smoke.w24.*`): quadro «Bars», frame 1800×1400, caixa da esquerda (x 220) em baixo à
altura da gaveta das formas, seleccionada, gaveta das formas e painel dos Frames abertos à abertura; caixa da direita; três lições.
Fotos: `docs/MiroClone/ferramentas/w24_fotos/cena29_1366.png` (a barra desliza para a direita da gaveta, com folga) e
`cena29_1930.png` (há lugar: uma linha por baixo da gaveta).

**(c) `fotografa_cena.sh`:** a guarda recusava a tela virtual — comparava com `wayland-0` fixo; hoje a tela do dono é `wayland-1`
e a virtual nasce `wayland-0`. Agora compara com `DISPLAY`/`WAYLAND_DISPLAY` de quem chama (mantém `:0`).

**(d) ABERTO 4 — riscas de longe.** Régua nova `ph2d-board-render/tests/it/measure_hatch_far.rs` (custo + visibilidade no mesmo
processo, tom varrido 0,1–0,7).
- Custo, 10 mil formas de 5–10 px (ms, CPU / placa): riscas à mão 25–29 / 15–33; de longe 9–11 / 4–9; sólido do produto 9–11 / 4–7.
- Visibilidade (alvo ≤ 1) até 10 px: riscas tom 0,25 → 0,46 / 0,65 / 0,74 a 5/8/10 px, 0,90 a 11; cruzadas tom 0,5 → 0,46 / 0,68 /
  0,84, 1,01 a 11; importado (final, riscas) tom 0,25 → 0,54 / 0,70 / 0,80, 1,00 a 11. Zigzag desenha-se como riscas
  (`sketch::options`) e mede igual.
- Lei: `Detail::hatch_final_below_px` = `HATCH_FINAL_BELOW_PX` = 11; `Detail::hatch_tone` = `HATCH_TONE` = `[0.25, 0.5, 0.25]`
  (`frames.rs`). Em `paint_shape`, `tone` ⇒ final com contorno ×2,5 e preenchimento na cor × tom. O export fica à mão
  (`ph2d-board-export` põe 0).
- Gate `sketch::tests::a_far_hatched_shape_is_drawn_final_in_its_measured_tone` (`sketch_tests.rs`). Doc 16 §5 novo + linha da
  recusa trocada.
- ⚠️ A mutação «limiar 0» é INVÁLIDA (desenha a −1 px e pendura) — não a repita.

**(e) ABERTO 5 — doc 17 §6** trocado por tabela propriedade → gate REAL (conferida por `grep`). Seis ficaram SEM gate («nenhum»):
B13 «toda forma reconhecida pousa num elemento ou fica tinta», «sem segurar fica tinta», «o quadro não tem limiares próprios
(censo)», «oráculo Draw to shape»; B14 «a menção chega ao vivo e no ficheiro (duas réplicas, reabrir)»; e parciais (copiar guarda
setas/frames/mapa; menu pelo escolhedor; linha a tocar forma liga-se; Ctrl+Z só caixa; menção = pessoa).

## 2. O que um merge pode partir

- `strip_rects_fitting` deixou de existir (substituída por `strip_rects_clear_of`); assinatura de `style_rects_of` (+`keep`).
- `Nav::floats` novo (`documents_nav.rs`); ordem de pintura em `paint.rs` (alerta e vídeo ANTES da barra); `paint_notice` devolve `Rect`.
- `Detail` ganhou 2 campos (`hatch_final_below_px`, `hatch_tone`): literais completos em `measure_detail_cost.rs` e `ph2d-board-export`.
- `ROUTERS` `max_level` 29; i18n `board.smoke.w24.*`; `measure_bloco4` expõe `at_px`/`cache_for`/`loadavg`/`diff`/`SIDE`/`LOOK` como `pub(super)`.
- **Posições da barra de estilo mudam onde havia chrome** — gates de posição de outras ondas reconferem.
- Formato 17, protocolo 11, `PROJECT_SCHEMA`, §6: inalterados (DELTA zero).

## 3. Prova da onda

Gates em `ph2d-editor-core`, `board_bar::tests::place` (`board_bar_place_tests.rs`); a verdade é o `hit_index` do ecrã, não a lista.
Instrumento de teste: `board_bar::STYLE_BAR` (`thread_local`, `cfg(test)`).
- `the_style_bar_never_covers_an_open_drawer` (4 gavetas × 1366/1024/800 × 5 posições; 15 casos com barra por gaveta)
- `the_style_bar_never_covers_the_right_hand_panels` (frames + comentários + minimapa abertos)
- `in_a_tiny_window_the_style_bar_stays_right_of_the_toolbar` (420×560, 320×420, 260×420, 220×420, 200×400)
- `smoke::w24_bars::tests::the_w24_scene_opens_with_the_left_box_and_its_bar` (`ph2d-app-board`)
- `sketch::tests::a_far_hatched_shape_is_drawn_final_in_its_measured_tone` (`ph2d-board-render`)

**Mutações:** `keep` vazio ⇒ gavetas e painéis vermelhos · gavetas sem registo ⇒ gavetas vermelho · sem partir grupos + x à esquerda
⇒ janela minúscula vermelho (a 1.ª versão do gate SOBREVIVIA: só 420/360/320, a sala ainda levava o maior grupo; ganhou
260/220/200) · tom ignorado ⇒ vermelho · tons `[0.3;3]` ⇒ vermelho.

**Fecho da onda:** `cargo fmt --all`; clippy `-D warnings --all-targets` limpo em `ph2d-editor-core`, `ph2d-app-board`,
`ph2d-board-render`, `ph2d-board-export`, `ph2d-i18n`. Por nome verdes: `place::` 3, `a_far_` 5 (render), w24 1. Filtros `board_` +
`float_bar` de `ph2d-editor-core`: 260/260; `ph2d-app-board` smoke 38/38; render/export filtrados verdes. Binário de smoke compilado.
⚠️ Suíte NÃO corrida (cadência: 6 commits < 10); clippy do workspace e bateria só no `/pd-linha-fechar`.

## 4. ABERTO (nesta ordem)

1. **Smoke do dono da cena 29** (§5) — se chegar, trate-o primeiro (mecanismo, não sintoma).
2. **ABERTO 3 da lista-mãe (elementos antigos de nascença 0) — MEDIDO, à espera do dono.** As fixtures v13–v16 são quadros
   sintéticos; nada muda à vista ao abrir (nascença 0 = como tudo se desenhava antes da W23). A diferença só existe para formas
   desenhadas LONGE antes da W23. Sonda (elipse 100×60 em rascunho, traço 2, fixture v16): a zoom 0,293 a ondulação no ecrã é 1,32 px
   (velha) contra 1,70 px (se nascesse ali) e o traço 0,59 px contra 1,97 px; a zoom 0,15: 0,67 vs 1,63 px de ondulação, traço 0,30
   vs 2,02. O ficheiro não guarda o zoom em que cada forma foi desenhada ⇒ nenhuma migração sabe a escala certa. Opções ao dono:
   deixar como está, ou um comando «redesenhar à escala desta vista» para a selecção. Só construir com a decisão dele; se mudar o
   formato: sobe para 18 (contar no código) + `legacy_v17`.
3. Lacunas de gate do doc 17 §6 («nenhum», ver §1e) — decidir se entram no fecho.
4. Fecho da linha (`/pd-linha-fechar`): bateria 1× sobre o diff, handoff de integração, §5 do CLAUDE.md (cenas 1..29), binário 2×.
5. Herdados, ainda abertos: letra que não nasce à escala do ecrã; README do oráculo (W14 nomeia um teste apagado); pendentes do §4.2
   da lista-mãe.

## 5. Smoke ao dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=29 ./target/smoke/ph2d-host-desktop`
2. Abre o quadro «Bars». À esquerda, a gaveta das formas (a grelha de formas ao lado da barra de ferramentas) está aberta; a caixa de
   baixo à esquerda está selecionada e a barra dela (cores, traço, letra) aparece por cima, À DIREITA da gaveta, sem a tapar. Em cima
   à direita está aberto o painel «Frames».
3. Na barra da esquerda abra, uma de cada vez, Nota, Caneta e Modelos: a barra da caixa afasta-se da gaveta aberta. (Atenção: com
   Nota ou Caneta na mão um clique no quadro desenha — volte à seta antes de selecionar outra coisa.)
4. Clique na seta da barra da esquerda, selecione a caixa da direita, e abra em baixo à direita os Comentários e o Minimapa: a barra
   da caixa não fica por cima de nenhum painel.
5. Estreite muito a janela: a barra parte-se em mais linhas e nunca passa por cima da barra de ferramentas.
6. Bónus (riscas de longe): numa forma, escolha um preenchimento às riscas e afaste-se muito (roda do rato): de longe a forma
   continua a ler-se riscada/tom igual, sem piscar ao cruzar o ponto em que muda.
7. Deu errado se: a barra de uma caixa tapa uma gaveta, um painel ou a barra de ferramentas; sai da janela; ou de longe as formas às
   riscas mudam de cor de repente ao aproximar/afastar.
