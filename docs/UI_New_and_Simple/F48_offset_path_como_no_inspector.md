# F48 — o Offset Path no padrão do Inspector (dono 02/11)

> Pedido do dono, com a foto da secção antiga: *«expand não é intuitivo, auto explicativo e provavelmente
> não funciona perfeitamente. Veja o layout da ferramenta em outros apps»* → aprovado: *«faça tudo como no
> inspector»*.

## O que estava errado (medido no código, gate vermelho antes da cura)

| # | Defeito | Prova |
|---|---|---|
| 1 | `Side · Inner` movia só os FUROS — numa forma sem furo não fazia nada; o artista lia «encolher» | `expand_tests::inner_side_moves_only_the_holes`; F48 `o_inner_que_nao_fazia_nada_saiu` |
| 2 | `Offset 9` era 9 % da meia-dimensão, sem unidade: o mesmo número dava bordas diferentes por forma | `params::slider_to_offset_frac`; F48 `a_distancia_e_um_campo_numerico` |
| 3 | O Offset trocava a forma; o uso típico (contorno de adesivo, sombra de texto) quer a cópia atrás | `offset_live_behind_tests` |
| 4 | Três ferramentas sem relação numa secção (offset, traço→forma, perfil de largura) | F48 `o_perfil_de_largura_mora_no_stroke_settings`, `o_outline_stroke_mora_no_convert` |
| 5 | `Apply Offset` aparecia sempre, sem dizer que o que se via era prévia | F48 `apply_e_remove_so_com_offset_vivo` |

## Os outros apps (layout, não código)

- **Illustrator** — *Object › Path › Offset Path…*: Offset em unidades (negativo encolhe), Joins
  Miter/Round/Bevel, Miter limit, Preview; cria uma CÓPIA. *Outline Stroke* é comando de menu, sem opções.
- **Inkscape** — Outset/Inset e o *Dynamic Offset*: uma alça na forma que se arrasta.
- **Affinity Designer** — ferramenta *Contour*: arrastar na tela; largura e quina na barra de contexto.
- **Figma** — sem offset; *Outline stroke* no menu de contexto, `Ctrl+Alt+O`.

## O que ficou

Secção **Offset Path** (o id continua `VECTOR_SECTION_EXPAND`): `Distance (px)` (campo do Inspector, px do
documento → mundo pela `DocPx`), `Corners` em glifos (os do Join do traço), `Miter limit` só no Miter
(1–500, a faixa do Illustrator; `VecOffset::miter`, `offset_path_with`), `Holes · Offset too | Keep`
(códigos 2/0 do motor; o 1 fica para documentos antigos), `Result · New shape behind | Replace`
(`VecOffset::behind`, nasce atrás), `Apply`/`Remove` só com offset vivo. Alça na tela no modo Select
(`offset_handle`), a um quarto da altura da borda direita. Secção **Convert** com `Outline Stroke` e
`Ctrl+Alt+O`. Os quatro W e o `Apply Power Stroke` foram para o «Stroke settings», com nomes inteiros.
Canais novos `OFFSET_MITER` (213) e `OFFSET_ATRAS` (214).

## ⛔ Recusas

| Recusado | Porquê |
|---|---|
| Manter a % do tamanho | O número não diz uma distância; nenhum dos quatro apps a usa |
| Alça a meia altura da borda | Cai em cima da pega de escala do gizmo e roubava-a |
| Menu de contexto do vetor | Não existe no app; o atalho cobre o pedido sem inventar um menu só para um item |
