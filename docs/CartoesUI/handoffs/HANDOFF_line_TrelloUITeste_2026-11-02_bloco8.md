# Handoff — linha `TrelloUITeste`, bloco 8: T57–T59 (02/11/2026)

Linha de TESTE (não integrada); o que sobrevive é `docs/CartoesUI`. Worktree `Worktrees/line-TrelloUITeste`, branch `line/TrelloUITeste`. Nunca houve push. Anterior: [blocos 3–7](HANDOFF_line_TrelloUITeste_2026-11-02_blocos3a7.md) (os ABERTOS dele continuam, menos os resolvidos abaixo).

**Commits:** `8a9edc2620` (T58) · `e3aa2c466b` (T59) · `4acb2c077c` (T57 + cena 6 + testes + fotos) · `abfd99a2a4` (docs) — base `dc3214da6e`.

**Smoke do dono: OK em 02/11** (cena `PH2D_VEC_CARTOES_SMOKE=6`: arrastar o Player aberto, movê-lo solto, rolar, subsecções e pares).

## Feito (detalhe e contraprovas: bloco 8 de [04](../04_plano_de_construcao.md))

| item | mecanismo medido | cura | teste |
|---|---|---|---|
| T58 | `paint_frame::begin_section` registava o cabeçalho absorvido (id da secção) e a pega antiga de 30 px por cima do título do cartão | `widget::absorbing_next_header` ⇒ não regista | `an_open_component_card_drags_onto_the_canvas_and_moves_there` |
| T59 | moldura das subsecções do Player = 25 px/linha; a linha do cartão mede 51 (66 com rótulo em 2 linhas) | `rows_moldura::card_frame_with` (linhas numa cena à parte, moldura medida); pares `player.rs::PLAYER_PARES` (16) e roda Radius\|Out Radius | `the_player_card_rows_never_overlap`, `the_player_card_pairs_share_a_row` (rola até ao fim), `the_wheel_card_puts_its_two_radii_on_one_row` |
| T57 | — | `panel/cards_solto_rola.rs`: corpo do solto na `widget::scroll_area` com dono `loose_rect_of(k)`; pára acima do HUD de baixo | `a_loose_card_taller_than_the_canvas_scrolls_inside` |

Todos em `ph2d-panel-registry-init/tests/it/o_bloco8_player_solto.rs`. `cards.rs` 1443 → 1419. `fotografa_cena.sh` ganhou `FOTO_HOME_SEMENTE`.

## ABERTO (novo nesta rodada)

1. **Alfinete nos cartões de componente** (pergunta do dono no smoke: «porque o pin não aparece nos vários cards novos?»): só o Transform tem retrato próprio (`is_card_pinnable`, `card_subject_ops.rs`); cada família de componente precisa de um retrato do objeto preso (valores + escrita para o preso) antes de ganhar o alfinete — é o ABERTO 2 do handoff anterior.
2. **Reset nos cartões de componente** (dono no smoke: «nem o botão reset»): o cabeçalho do cartão de componente nasce sem ações (`cards_componentes::cartao`, `actions: Vec::new()`); só o Transform tem o ↺. Falta: a ação no cabeçalho de cada componente que repõe o `Default` do componente pela fila de undo da shell (e o Auto Key), uma lei para todos — medir antes quais componentes têm `Default` e se a secção clássica já tinha um reset (não confirmado).
3. **Vermelhos ANTIGOS** do `ph2d-panel-inspector --test it` (iguais na base `dc3214da6e`): `seam_wheel` 7, `seam_player` 18, `seam_joint` 26, `seam_anim_card` 7–8; o conjunto `seam_` varia 134–139 entre corridas (instável). Não tratados.
4. T29 2.ª metade continua por fazer (contagem no handoff anterior).

## Regras aprendidas

- Teste de clique sobre um título: pegue a pega MAIS LARGA com o id (`iter_registrations`), nunca `rect_for` — um registo menor com o mesmo id fez o teste do T58 passar com o defeito vivo.
- Moldura pintada antes do conteúdo: no cartão a altura não é a de hoje; meça (cena à parte + `append_transformed`).
- `zsh` não parte `$L` em palavras: listas de ficheiros em `bash -c`.
