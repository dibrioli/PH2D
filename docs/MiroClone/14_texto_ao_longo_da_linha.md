# 14 — Os textos ao longo da linha de uma seta (W23 B5(2))

> Leitor: a LLM que implementa ou audita. Oráculos corridos em 19/10 (fixtures com cabeçalho em
> [`ferramentas/label_oracle/`](ferramentas/label_oracle/README.md), 15 leis numeradas). Antes: a seta tinha UM
> `label`, a meio da rota, horizontal. Agora: vários textos, cada um numa fracção do comprimento, deitados ao longo da linha.

## 1. Estado da arte — MEDIDO

| | quantos | posição | orientação | arrastar |
|---|---|---|---|---|
| **Miro** (só documentação) | REST `captions[]` até 20 | `position` 0–100 % (omissão 50 %); `textAlignVertical` top/middle/bottom | `style.textOrientation`: `aligned` (omissão) \| `horizontal`; opção no menu de contexto | clicar no texto da seta seleccionada e arrastar ao longo; duplo clique na linha acrescenta texto |
| **tldraw 5.5.2** (corrido) | 1 | `labelPosition` | horizontal | sim, com corte nas pontas (só no desenho) |
| **Excalidraw 0.18.1 / next** (corridos) | 1 | 0.18.1 ignora o campo; next usa-o | horizontal | só o next |

Fontes do Miro: <https://developers.miro.com/reference/create-connector-1> · <https://developers.miro.com/docs/websdk-reference-connector> ·
<https://community.miro.com/ask-the-community-45/how-do-i-move-text-along-a-connecting-arrow-line-17595>.

Leis do [`label_oracle`](ferramentas/label_oracle/README.md) que decidiram:
1. a fracção é de COMPRIMENTO DE ARCO (nos três) · 2. tldraw corta o centro a ≥ `e+p` das pontas (só desenho); next não corta ·
3. arrastar além da ponta satura em 0/1 · 4. arrastar na perpendicular projecta no ponto mais perto da linha ·
6. a pega do meio ganha ao rótulo · 7. texto sempre horizontal nos dois oráculos (o Miro `aligned` é o único que deita) ·
8. a linha é cortada atrás da caixa do rótulo (tldraw) · 10. um rótulo só; duplo clique no corpo edita-o ·
12. `labelPosition` ausente no next = meio dos PONTOS (vértice do meio / meio do trecho do meio), não 0,5 de arco ·
13. 0.18.1: o «meio» é o do índice e ignora o campo, mas guarda-o.

## 2. Escolhas

- **Fracção do comprimento de arco** (lei 1, os três concordam).
- **Vários textos por seta, ALONGO por omissão**: o Miro é o produto de referência do dono; tldraw/Excalidraw têm um rótulo horizontal.
- **Sem corte junto às pontas**: Excalidraw next (leis 2/3) e Miro 0–100 % honram o valor gravado; o corte do tldraw é só de desenho e
  exigiria a largura do texto na rota — recusado, o valor guardado é respeitado.
- **As pegas ganham ao texto** (lei 6, como em todos).
- **Vão da linha** = papel atrás das letras (lei 8); a área de clique É esse papel (`ph2d_board_layout::knock_box`, `CAPTION_PAD` 0,25 da letra).
- **Regra do texto em pé**: ângulo em [−π/2, π/2) — uma vertical lê-se de baixo para cima, qualquer que seja o sentido da seta.
- **`textAlignVertical` (top/bottom) NÃO feito**: só existe na API do Miro, sem evidência de UI nem oráculo — ABERTO, por medir.

## 3. Desenho

- **Modelo** `ph2d-board-model/src/connector.rs`: `Connector::captions: Vec<Caption{text, at}>` + `turn: TextTurn {Along, Level}`
  (`serde(default)` para JSON antigo); `captions_of`, `drop_empty_captions`. Formato 14 → 15: `legacy_v13.rs` genérico sobre a seta
  (`read_v13`/`read_v14`); o `label` antigo vira UM texto `Level` a 0,5; fixture `fixtures/format_v14.bin` do build `0353869e6d`.
  `Field::Label` renomeado `Field::Captions` (mesma posição) + `Field::TextTurn` novo (dois registos; gate
  `the_captions_and_their_turn_are_separate_fields`); `PROTOCOL` 8 → 9.
- **Rota** `ph2d-board-route/src/caption.rs`: `point_at`, `frac_of`, `caption_frame`, `upright`,
  `CaptionFrame::{to_local, contains, hits, block_origin}`.
- **Desenho** `ph2d-board-render/src/caption.rs`: `block_to_screen`; sal da chave da cache de texto por texto; alcance de culling =
  `LABEL_WRAP/2·√2`.
- **Edição** `ph2d-board-edit/src/caption.rs`: `CaptionBoxes` medidas em `overlay()` e no pointer-down; `caption_down/move/up` →
  `Gesture::Caption`, UM passo de desfazer; `text_turn`/`set_text_turn`, `Command::TextTurn`. `hit.rs` (saiu de `gesture.rs`): o acerto
  inclui os textos. `text.rs`: `CaptionPick::{Existing,New}` — duplo clique na linha = texto novo em `frac_of`, num texto = edita-o; o
  vazio larga-se ao confirmar; a confirmação compara por `changed_fields`.
- **Menu**: `ContextMenuKind::BoardArrow{along}` com «Horizontal text» / «Text along the line» (`board_menu.rs`, `menu_tables.rs`).
- **Excalidraw** `ph2d-board-excalidraw/src/caption.rs`: o 1.º texto vai preso com `labelPosition` SEMPRE; os outros soltos com
  `customData.ph2d.caption_of=[dono, i]` e `angle`; seta alheia: `labelPosition` ou `middle_of_points` (leis 12–13), turn Level. O
  `label` do JSON antigo migra por `old_label` com `get_mut` — ⛔ `Value[..]` CRIA a chave em falta e parte todo elemento que não é seta
  (medido: 12 testes vermelhos).

## 4. Gates (todos no código)

- rota: `a_caption_sits_on_the_line_at_its_fraction_of_the_length` · `along_text_never_reads_upside_down` ·
  `a_dropped_caption_lands_on_the_nearest_point_of_the_line`
- edição: `dragging_a_caption_slides_it_along_the_line_in_one_step` · `a_double_click_on_the_line_starts_a_new_caption_there` ·
  `an_emptied_caption_goes_away` · `a_click_on_the_far_end_of_a_caption_selects_the_arrow`
- render: `an_along_caption_block_is_centred_on_the_line_and_turned_with_it`
- modelo: `a_format_14_file_opens_with_its_label_as_one_level_caption` · `the_captions_and_their_turn_are_separate_fields`
- Excalidraw: `the_captions_go_to_excalidraw_and_come_back` · `a_foreign_label_comes_in_level_at_its_label_position` ·
  `without_a_label_position_the_text_sits_in_the_middle_of_the_points` (lê a fixture do oráculo)
- app: `the_w23_text_scene_opens_with_the_words_in_reach` (cena 26, pelo rato real da shell)

Oráculos W8/W9/W10 regravados em `527761e134` (diff só `label` → `captions`/`turn`, `labelPosition` no texto preso).
⚠️ **Provas de mutação: NÃO corridas.**

## 5. ⛔ Recusas MEDIDAS

| recusa | porquê |
|---|---|
| corte do tldraw junto às pontas | só desenho e precisa da largura do texto na rota; Miro e Excalidraw next não o têm |
| um rótulo por seta (tldraw/Excalidraw) | o produto de referência do dono é o Miro (até 20) |
| gravar `labelPosition` só quando ≠ 0,5 | o next, ausente, usa o meio dos PONTOS (lei 12) — errado numa seta de vários pontos; grava-se sempre |
| área de clique = bloco da largura de quebra (160) | um duplo clique na linha a 60 px de uma palavra curta caía dentro dele (medido no gate); vale o papel (`knock_box`) |

## 6. Smoke do dono 19/10 — o texto DOBRA com a curva

«As palavras estão retas mas deveriam seguir as curvas» (o `aligned` do Miro: «aligned with the curvature»).
Numa linha com curvas o espaço do texto mede-se PELA linha: `x` = distância ao longo dela (a `Ruler`, a
linha aplanada a 0,02, recta para lá das pontas), `y` = a normal; cada letra desenha-se no seu ponto e
com a direcção dali (`paint_along`); o recorte e a selecção dobram-se a cada 2 unidades (`warped`). O
sentido da leitura escolhe-se no centro (`upright`) e vale para o texto todo. Numa linha só de rectas o
texto fica rígido (rodado no centro). Gate `along_a_curve_the_text_bends_letter_by_letter`.
