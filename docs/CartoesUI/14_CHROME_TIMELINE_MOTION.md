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
| Time, Frame, Dur, com setas | campo Figma: pega de scrub + valor, Esc, duplo clique; o passo exacto das setas é ↑↓ no campo focado (Dur 0,2 s) | `*_NUM` |
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
| Migalhas `Bg2/Bg3` | as mesmas caixas e `crumb_rect`, tinta da lista, sem borda |
| Nó: corpo `Bg2`, raio 7, borda `Border` | corpo na tinta do cartão, raio 7 de sempre (a cápsula partilha-o); cabeçalho da categoria fica; caixas dos params na tinta do campo |

Não muda: geometria dos nós, portas, fios, cápsula, véu, hits, divisor. Sem título nem barra: o grafo não perde altura (0 px).
Legibilidade: o cabeçalho colorido por categoria e o contraste dos nomes de porta ficam; a régua é a foto ao zoom 1 e no fit.

## Bloco I2: o que faltou aos painéis de desenho próprio

| Painel | A área de trabalho (intocada) | O que mudou à volta | Altura (mock) |
|---|---|---|---|
| Timeline, sobras | folha, curvas | sem o «…»; régua, barra, zebra, costura e listas abertas na tinta do cartão (`folha_tinta.rs`) | 126 px, como na I1 |
| Audio Editor | a onda, régua, seleção, cabeça (shell) | 8 secções = 8 cartões com o mesmo id, corpo pelos mesmos pintores (`cards.rs`, `cards_fileira.rs`); moldura da janela na crate do painel (`cards_moldura.rs`) | lista: rola; janela da onda a mesma |
| Pixel Timeline | grelha camadas × frames | moldura sem cartão, barra em ícones, segmentos e chip (`barra_cartoes.rs`); propriedades na tinta do cartão | 6 linhas → 6; 1.ª linha 103 → 104 px |
| Pixel Preview | a imagem | pílula, título e verbos da lista | título 42 → 42 px |
| Editor de scripts | o texto, a calha | `chrome::Ui` e a barra de estado na tinta da lista: barra, busca, console, lista | 0 px |

**O «…» saiu, não ganhou função.** As linhas dele repõem cartões, e a moldura sem cartão não tem nenhum. Resets de layout
têm outra casa; inventar-lhe uma função só para o manter é o botão morto com outro nome. `pinta_moldura` é a única moldura
sem cartão: toda lista com cartões conserva o menu. A catraca `a_marca` reconhece a moldura pelo «×».

**A fileira de parâmetro do Áudio** não virou campo Figma: o valor é uma string do motor, com a curva de cada efeito
(`fileira_de_param.rs`). No cartão: rótulo ao lado, `CardBar` com o id do `Slider`, leitura à direita.

**Recusa medida:** `SEM_RECORTE` com `f32::MIN + f32::MAX` dá 0 e nenhum alvo cabe: a foto saía bonita e nada se clicava. Só
o teste por gesto apanhou. A bateria foi corrida antes das fotos finais.

**Fica:** os números das propriedades da Pixel Timeline continuam com as setas (`paint_field_row` não pinta no cartão).
