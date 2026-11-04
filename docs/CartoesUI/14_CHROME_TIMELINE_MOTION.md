# 14 — O cromo da Timeline e do Motion Graph em cartões (bloco I)

Só o que rodeia a área de trabalho. Dope sheet, curvas, grafo, porta e fio não mudam de comportamento. Mesmos ids, mesmos
eventos, `PH2D_CARDS=0` dá o de antes. Medidas: `ph2d-panel-timeline/tests/it/os_cartoes.rs`, mock e janela 1930 × 1040.

## Decisão: moldura sem cartão por dentro

O Frames usa `paint_card_list` com um cartão único: `CROMO_H` 80 px. Na Timeline a altura é o recurso, e a banda é baixa.
Daí `panel/cards_banda.rs`: a moldura da lista (cantos, sombra, título, «…», «×», `metrics`) sem cartão, com os controlos
sobre a tinta da lista, como a busca da Hierarchy aprovada (`f_hierarchy_ao_lado_do_inspector.png`). O campo leva o rótulo
AO LADO, não em cima: em cima custa 23 px, uma linha de faixa. Recusa medida, não preferência.

## Timeline: inventário

| Antes | Depois | Id |
|---|---|---|
| Título `Lg` na fileira | título da lista, 13/700 | — |
| Abas `Keys · Contain… · Arrange` a 84 px | segmentado pelo conteúdo («Containers» inteiro) | `TIMELINE_TAB_*`, `TIMELINE_CRUMB[n]` |
| Chips Animation e clip | cara `paint_card_chip`; a lista aberta continua a de sempre | `TIMELINE_ANIM_DD`, `TIMELINE_CLIP_DD` |
| `+ ⧉ T 🗑`, transporte, `I`, engrenagem | fila de ícones na caixa do cartão, com dica | os mesmos |
| `+M`, `+ Track`, `+ Lane`, `+ Container` | botão do cartão | os mesmos |
| Time, Frame, Dur, com setas | campo Figma: pega de scrub + valor, Esc, duplo clique | `*_NUM` |
| 11 interruptores `[nome│switch]` à largura da coluna mais larga | caixa T64 à largura do nome | os mesmos, `Toggle` |

Não muda: régua, faixas, curvas, costura de nomes, barra de rolagem, popovers, renomear, pega do topo da banda.

## Timeline: altura

| Janela | Faixas antes (px) | Faixas depois (px) | Linhas a `ROW_H_PX` |
|---|---:|---:|---:|
| 1930 (banda 1253) | 116 | 126 | 5,3 → 5,7 |
| 1600 (banda 923) | 91 | 92 | 4,1 → 4,2 |
| 1100 (banda 423) | 0 | 0 | a barra enche a banda nos dois; arrasta-se a costura |

## Motion Graph: inventário

| Antes | Depois |
|---|---|
| 9 chips soltos no canto, moldura `Bg2/Border` | uma pílula flutuante (tinta e sombra da lista), a fila de ícones do cartão; aceso = segmento aceso em vez de anel |
| Migalhas `Bg2/Bg3` | a mesma pílula, segmentos de texto, tinta da lista |
| Nó: corpo `Bg2`, raio 7, borda `Border` | corpo na tinta do cartão, raio `CARD_RADIUS`; cabeçalho da categoria fica; caixas dos params na tinta do campo |

Não muda: geometria dos nós, portas, fios, cápsula, véu, hits, divisor. Sem título nem barra: o grafo não perde altura (0 px).
Legibilidade: o cabeçalho colorido por categoria e o contraste dos nomes de porta ficam; a régua é a foto ao zoom 1 e no fit.

## Fica para a fatia seguinte, se não couber

Barra de rolagem da folha e régua na tinta do cartão · lista suspensa aberta no tom do cartão (ABERTO de 05 §9) · o que o
«…» oferece: hoje só os três resets, sem efeito num painel sem cartões.
