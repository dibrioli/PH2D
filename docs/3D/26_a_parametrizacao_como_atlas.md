# W0 — A PARAMETRIZAÇÃO VISTA COMO ATLAS, medida

> **Ordem do dono (2026-09-20):** *«OK. Implemente»* / *«siga»*, sobre a recomendação da
> [avaliação](25_avaliacao_o_painter_na_malha.md) §8, corrigida pela §11.
>
> ⚠️ **Esta é a wave ZERO, e ela é a que a própria avaliação encomendou:** a §9 lista
> cinco perguntas *«que faltam MEDIR antes da primeira linha»*, e o §0.0 do `CLAUDE.md`
> proíbe escrever um limite antes da medição. Nenhuma linha de produto muda aqui.
>
> **Instrumento:** [`atlas_probe`](../../crates/ph2d-quadchain/examples/atlas_probe.rs),
> versionado.
> ```
> cargo run --release -p ph2d-quadchain --example atlas_probe -- <peca.obj|esfera:24>
> ```

---

## §1 — A tabela

Três peças do próprio dono (as fixturas de `ph2d-quadfill/tests/fixtures/pontas/`) e uma
esfera de controlo, cada uma medida **duas vezes**: na malha CRUA que o artista esculpiu
e na remalhada pelo F1, que é o que o botão de retopologia faz.

| peça | entrada | V | patches | **ILHAS** | dobras | corte sentido | aprov. na caixa | escala |
|---|---|---|---|---|---|---|---|---|
| `_base_sculpt` | **CRUA** | 16 898 | 99 | **13** | **0,51 %** | 14,33× | 26,2 % | 0,747 |
| `_base_sculpt` | F1 | 1 520 | 55 | 4 | 5,20 % | 6,52× | 40,9 % | 0,779 |
| `sculpt_antes` | **CRUA** | 13 682 | 88 | **4** | **0,16 %** | 11,72× | 37,9 % | 0,858 |
| `sculpt_antes` | F1 | 1 993 | 67 | 10 | 1,73 % | 9,00× | 50,8 % | 0,879 |
| `Sculpt_Blender` | **CRUA** | 8 293 | 116 | **11** | **0,77 %** | 15,65× | 30,9 % | 0,779 |
| `Sculpt_Blender` | F1 | 1 932 | 57 | 5 | 1,84 % | 8,78× | 42,1 % | 0,811 |
| `esfera:24` | CRUA | 830 | 11 | 2 | 0,12 % | — | 48,2 % | 0,842 |
| `esfera:24` | F1 | 2 544 | 16 | 2 | **17,94 %** | — | 39,1 % | 0,169 |

*corte sentido* = comprimento das costuras que **rodam** (as únicas que um atlas tem de
abrir), em múltiplos de `√área`. *escala* = área do mapa em unidades de mundo contra a
área da superfície: `1,000` seria isométrico.

---

## §2 — ⭐⭐⭐⭐ O achado que muda o plano: a malha do ARTISTA parametriza-se MELHOR

A §8 da avaliação encomendava a W1 como *«a malha ganha UV»* com o F1 implícito, porque é
assim que a cadeia do botão corre. **Medido, é ao contrário:**

| | dobras na CRUA | dobras depois do F1 |
|---|---|---|
| `_base_sculpt` | `0,51 %` | `5,20 %` (**10×**) |
| `sculpt_antes` | `0,16 %` | `1,73 %` (**11×**) |
| `Sculpt_Blender` | `0,77 %` | `1,84 %` (**2,4×**) |

⭐ **E isto resolve a pergunta que decidia a arquitectura inteira.** Uma textura tem de
viver na malha que o artista esculpiu; se a parametrização exigisse o F1, o caminho da
textura **substituiria a escultura dele** — e não exige.

⚠️ **A escala diz a mesma coisa por outro caminho:** na CRUA o mapa conserva `0,75`–`0,86`
da área, no F1 `0,17`–`0,88`. A célula de `0,169` é a esfera UV remalhada, onde o mapa
colapsou.

⛔ **O que isto NÃO diz:** que o F1 seja inútil. Ele é a fase zero da **extracção de
quads**, e ali a medição de 24/08 é clara (sem ele a mesma cadeia dá `10°`–`12°` de
enviesamento, o dobro). *Um passo certo para uma fase pode ser errado para outra que
partilha o motor.*

---

## §3 — ⭐⭐⭐ Um PATCH não é uma ILHA: são 4 a 13, não 88 a 116

A §4 da avaliação escreveu que empacotar as ilhas *«é um bin-packing de rectângulos»* e
deixou o **tamanho do trabalho** por medir. Ele está medido, e é pequeno.

O mapa soldado acopla os dois lados de cada costura, e onde o salto de período é
`0 (mod 4)` **os dois lados leem a mesma função a menos de uma translação**: ali não há
corte nenhum — há uma linha que só existe porque o traçado partiu a peça em quads. Uma
ilha de textura nasce onde a transição **RODA**.

| peça (CRUA) | patches | costuras coladas | costuras rodadas | **ilhas** |
|---|---|---|---|---|
| `_base_sculpt` | 99 | 139 | 144 | **13** |
| `sculpt_antes` | 88 | 133 | 141 | **4** |
| `Sculpt_Blender` | 116 | 165 | 162 | **11** |

⭐ **Cerca de metade das costuras são cortes de verdade** (`49 %`, `41 %`, `51 %` do
comprimento), e o resto é fronteira interna que o atlas não vê.

⚠️ **E a contagem de ilhas NÃO é monótona no refinamento:** `sculpt_antes` dá `4` na crua
e `10` depois do F1; `_base_sculpt` dá `13` e `4`. *Ela é uma propriedade do traçado, não
do tamanho da malha* — logo um plano que a preveja a partir da contagem de vértices está
a adivinhar.

---

## §4 — O relógio, e o que ele obriga o produto a ser

⚠️⚠️ **As colunas de relógio foram tiradas com a máquina a `load 30`** (outra linha a
correr binários de teste a 100 % em vários núcleos), logo **não são um veredito** — a lei
do `load ~5` deste repo vale aqui. O que é robusto é a **FORMA**, porque o solver é `O(V)`
e as duas entradas diferem por `~10×` de vértices.

| peça | entrada | campo | traçado | corte | mapa contínuo | TOTAL |
|---|---|---|---|---|---|---|
| `_base_sculpt` | CRUA | 10 053 | 133 | 57 | **29 688** | 39 930 ms |
| `_base_sculpt` | F1 | 362 | 10 | 5 | **2 872** | 3 249 ms |
| `sculpt_antes` | CRUA | 8 537 | 88 | 44 | 21 453 | 30 122 ms |
| `sculpt_antes` | F1 | 634 | 15 | 6 | 2 706 | 3 361 ms |
| `Sculpt_Blender` | CRUA | 3 641 | 42 | 22 | 18 635 | 22 340 ms |
| `Sculpt_Blender` | F1 | 493 | 17 | 6 | 3 077 | 3 593 ms |

⇒ **a parametrização da malha do artista custa dezenas de segundos, e isso é uma decisão
de PRODUTO, não um defeito:** ela não pode correr por quadro nem ao pegar num pincel. Ela
é um acto **sob comando**, com espera visível — exactamente como o botão de retopologia,
que custa `375 ms` no corte e minutos na cadeia inteira.

⭐ **E o número que a §9 dizia não existir já existia:** a `ChainTiming` parte a cadeia em
sete colunas desde que a `ph2d-quadchain` nasceu, e o [`chain_time`](../../crates/ph2d-quadchain/examples/chain_time.rs)
imprime-as **pela porta do produto**. Na peça do dono ela lê `G1/G2 = 6 ms` e
`G3/G5 = 4 486 ms` de um total de `7 884 ms`. ⛔ *O «123 s» que a avaliação citava é o
BOTÃO — quatro tentativas em cascata sobre dois campos —, e eu li-o como se fosse a
cadeia.*

---

## §5 — Resolução e desperdício

| peça | 1024² | **2048²** | 4096² |
|---|---|---|---|
| `_base_sculpt` (CRUA) | 1/13,2 do quad | **1/26,4** | 1/52,8 |
| `sculpt_antes` (CRUA) | 1/14,0 | **1/27,9** | 1/55,9 |
| `Sculpt_Blender` (CRUA) | 1/12,6 | **1/25,3** | 1/50,6 |

*Um texel de `2048²` mede `~0,004` de mundo numa peça de `~4` de lado* — vinte e cinco
vezes mais fino que o quad que a retopologia pede. ⇒ **`2048²` é folgado e `1024²` já é
utilizável**, o que põe a memória do §7 da avaliação em `16,8` a `67,1 MB` por camada.

⛔ **E o desperdício do empacotamento está medido, com a nota de que ele é OPTIMISTA:** o
aproveitamento **dentro da caixa de cada ilha** é `26 %` a `51 %`. Isso é o que se perde
*antes* de um empacotador arrumar as caixas umas contra as outras — a perda real é maior.
⇒ *a nota da §4 («um empacotamento ingénuo gasta metade dos texels») deixa de ser um
palpite: gasta mais de metade já dentro da caixa.*

---

## §6 — ⛔ Quatro coisas que a sonda apanhou EM MIM

1. **A 1.ª corrida imprimiu `dobras 0/0 (0,00 %)`** — que se lê como *«o mapa não dobra em
   lado nenhum»* e queria dizer *«nada foi medido»*: todos os triângulos tinham sido
   saltados por um `uv.get` que falhava. ⇒ coluna `SEM (u,v)` + piso de população no
   próprio instrumento. *Um zero de «não medido» e um de «perfeito» são o mesmo byte* — a
   lei que as duas réguas de valência do quad remesh já pagaram.
2. **Chamei `ph2d_gridmap::solve`, o motor que o produto deixou de correr.** Aquele é o G3
   com a costura **penalizada**; a obra A de 24/08 substituiu-o pelo **soldado**. Medido na
   esfera: `10,76 %` de dobras e `5 784 ms` contra `0,12 %` e `198 ms` — `29×` de relógio,
   pela razão que o doc do `welded_rounds` já escrevia (`160 000` rondas contra `8 000`).
   *Escolher a função pelo nome mais óbvio mede um programa que o produto abandonou.*
3. **Escrevi uma TERCEIRA cópia do resíduo de costura, e errei-a duas vezes** (rodei o lado
   errado; depois esqueci a translação), lendo `2,17e1` e `3,84e1` células sobre um solder
   que solda. A casa tem [`seam_residual`], cuja convenção é `zb − turn2(za, j) − shift`;
   com ela, `p50 = 0,00` e `max = 9,8e-7`. *Uma terceira cópia de uma lei é onde ela
   diverge.*
4. **A avaliação dizia que ninguém tinha medido a parametrização sozinha.** Tinha —
   `ChainTiming` + `chain_time`, §4 acima. *Audite a lista contra o código antes de pegar
   num item dela.*

---

## §7 — O que isto muda no plano da avaliação

| §8 da avaliação | depois desta medição |
|---|---|
| **W1** «a malha ganha UV», com o F1 implícito | ⭐ **sem F1**: a parametrização corre na malha do artista e sai **melhor** (§2). O que a W1 tem de fazer é o canal `uvs`, o **merge das ilhas** (union-find sobre as costuras coladas) e o empacotamento de `4`–`13` rectângulos |
| **W2** a textura + o shader | igual; a resolução de partida é **`2048²`** e está medida (§5) |
| **W3** o ponteiro chega ao Painter | igual; e a §11 mantém-se: **pintar no ECRÃ** e projectar |

⏳ **O que continua por medir, e é acto de uma janela `E`** (o Blender é GPL, parede
obrigatória): *o que o alvo faz na costura*. É a única linha da §9 que esta janela não
podia fechar.

⏳ **E uma decisão que é do dono, não minha** (a mesma da §9): com topologia dinâmica
ligada, a textura fica esticada onde a malha adensou. As três referências resolvem isso
**proibindo** — pinta-se em textura *depois* de retopologizar.
