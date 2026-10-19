# CONTINUAÇÃO — `line/MiroClone` W23: B5(2) textos ao longo da linha feito; o próximo é o B5(4) — 2026-10-19

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_SMOKE19_B5_1](HANDOFF_CONTINUACAO_W23_SMOKE19_B5_1_2026-10-19.md) ·
> lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md). HEAD `527761e134` (+ o commit de docs que
> leva este ficheiro), base = main `100d34c8e0`. Sem integração nem push: só por ordem do Enio.

## 1. B5(2) — FEITO

Commits: `95993242be` oráculo (tldraw 5.5.2 e Excalidraw 0.18.1/next corridos, 15 leis) · `a5c3a8025c` a feature · `527761e134`
oráculos W8/W9/W10 regravados · + o commit de docs (doc 14, README, doc 07, este handoff).

Uma seta tem VÁRIOS textos (`Connector::captions`), cada um numa fracção do comprimento de arco, deitados ao longo da linha (Miro) ou
na horizontal (`turn`); arrastar desliza-o, duplo clique na linha cria um, esvaziar larga-o. Formato 14 → 15, `PROTOCOL` 8 → 9.
Mecanismo, escolhas, recusas medidas e gates: **[14_texto_ao_longo_da_linha.md](../14_texto_ao_longo_da_linha.md)**. Cena 26
(`PH2D_BOARD_SMOKE=26`, `smoke_w23_text.rs`, gate `the_w23_text_scene_opens_with_the_words_in_reach`).


**Smoke do dono 19/10 (2.º) — curado:** (1) «as palavras estão retas mas deveriam seguir as curvas»: o texto
ao longo de uma CURVA dobra letra a letra — `CaptionFrame` com `Bend` sobre uma `Ruler` (a linha aplanada a
0,02, para lá das pontas segue recta); `to_world`/`to_local` medem o espaço do texto pela linha;
`ph2d_board_layout::paint_along` (uma letra por transformação); recorte, selecção e grega dobrados
(`warped`); cursor pela linha. Numa linha de rectas (as cúbicas de pegas na corda contam como rectas) o
texto continua rígido. Gate `along_a_curve_the_text_bends_letter_by_letter`. (2) «o menu do botão direito
não some ao clicar fora»: o Down no quadro é tomado por `board_view::canvas_down` ANTES do fecho do
despacho (`pointer_down.rs:210`); agora, com menu aberto, o clique não-direito só o fecha (o padrão de
`pointer_down.rs:100`). Gate `right_click_on_a_shape_opens_its_menu` (mutação «sem o fecho»: vermelho). O
texto da cena 26 passou a «along the curve» (o longo quebrava em duas linhas na largura 160). `Esc` fecha os
menus do quadro (pedido do dono 19/10; a selecção fica) — `board_keys::key`, o mesmo gate (mutação: vermelho).

## 2. O que um merge pode partir (para o integrador)

- `FORMAT_VERSION` 15 e `PROTOCOL` 9 são números que SOMAM entre linhas — só esta linha os toca; reconte, não escolha.
- `Field::Captions` mudou de codificação (era `Field::Label`, mesma posição) e `Field::TextTurn` entrou no FIM: `Field::ALL` = 40.
  Outra linha que mexa no fio reabre os gates do `ph2d-board-sync`.
- `CLAUDE.md` §5 (Quadro) diz `PH2D_BOARD_SMOKE=1..22` → conta-se **1..26** (23 = lado de quem entra, 24–26 desta linha).
- Novo `ContextMenuKind::BoardArrow{along}` (`board_menu.rs`, `menu_tables.rs`).
- i18n: chaves `board.el.text_along` / `board.el.text_level` e `board.smoke.text.*`.

## 3. Prova de fecho de onda

- `cargo fmt --check` (11 crates) e `PH2D_SUITE=fecho cargo clippy … --all-targets -- -D warnings` (10 crates): limpos.
- Suítes inteiras (motivo: formato gravado) das 9 crates do quadro: 0 falhas (edit 230 · model 130 · sync 60 ·
  render 58+9 · excalidraw 45 · route 31 · export 19 · layout 9 · app-board 88); `ph2d-editor-core` por filtro
  `board`/`menu`/`palette`/`i18n` verdes; censos da árvore combinada 121/121; `file_loc_caps`/`fn_loc_caps` verdes.
- ⚠️ `ph2d-i18n` `nenhuma_frase_que_o_artista_le_esta_em_portugues` estava VERMELHO desde o B5(1): «undoes» lê-se
  como português (cenas 25 e 26, lição 4) — frases trocadas para «takes back»; verde.
- Provas de mutação dos gates do B5(2): NÃO corridas.

## 4. ABERTO (nesta ordem)

1. **B5(4)** desfazer as operações de aba (hoje apagar um quadro pergunta antes; desfazer não existe).
2. Bloco 4 (B6, B7, B10, B11 — MEDIR antes) · bloco 5 (B9, A11) · bloco 6 (B12, B13, B14 com `/pd-feature`).
3. `textAlignVertical` (top/bottom do Miro): por medir, só existe na API — sem UI nem oráculo.
4. **Provas de mutação dos gates do B5(2): NÃO corridas.**
5. Perguntas ao dono sem resposta: YouTube Premium; «Rascunho/Final» e a letra importada.
6. Smoke do dono por fazer: cena 26 (§5).

## 5. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=26 ./target/smoke/ph2d-host-desktop`
2. Arraste as palavras «along the curve», na seta curva, ao longo dela: deslizam pela linha e DOBRAM com ela, letra a letra.
3. Dê duplo clique noutro ponto da seta curva e escreva: aparece um segundo texto ali.
4. Clique com o botão direito na seta reta (Plan → Done): «Text along the line» deita o «then» ao longo dela; «Horizontal text» volta a pô-lo em pé.
5. Apague todas as letras de um texto: ele some. Ctrl+Z desfaz cada passo.

Deu errado se: o texto sai da linha, aparece de cabeça para baixo, a linha passa por cima das letras, ou o Ctrl+Z precisa de duas
vezes. Fotos: `../ferramentas/w23_fotos/cena26_texto_na_seta_{1366x768,1930x1040}.png`.
