# A PARAMETRIZAÇÃO COMO ATLAS — a medição (W0), o atlas (W1) e o corte (W2)

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
| **W1** «a malha ganha UV», com o F1 implícito | ✅ **FEITA na §8, e sem F1**: a parametrização corre na malha do artista e sai **melhor** (§2). ⛔ E **sem** o canal `uvs` na `Mesh`: o atlas é por CANTO, que é a menor unidade em que ele é exprimível. ⏳ Fica o vermelho da sobreposição |
| **W2** a textura + o shader | igual; a resolução de partida é **`2048²`** e está medida (§5) |
| **W3** o ponteiro chega ao Painter | igual; e a §11 mantém-se: **pintar no ECRÃ** e projectar |

⏳ **O que continua por medir, e é acto de uma janela `E`** (o Blender é GPL, parede
obrigatória): *o que o alvo faz na costura*. É a única linha da §9 que esta janela não
podia fechar.

⏳ **E uma decisão que é do dono, não minha** (a mesma da §9): com topologia dinâmica
ligada, a textura fica esticada onde a malha adensou. As três referências resolvem isso
**proibindo** — pinta-se em textura *depois* de retopologizar.

---

## §8 — ⭐⭐⭐ W1: O ATLAS EXISTE — e a medição dele tem um vermelho

> **Crate nova:** [`ph2d-uv-atlas`](../../crates/ph2d-uv-atlas/) — zero dependências
> externas, como a `ph2d-cloth`, a `ph2d-pose` e a `ph2d-boundary`. Ela recebe o que a
> cadeia já produz (malha triangulada · corte · mapa contínuo · saltos) e devolve **um
> `(u, v)` por CANTO em `[0,1]²`**, com as ilhas juntas, assentes e arrumadas.
>
> ⚠️ **Por CANTO e não por vértice:** um vértice sobre um corte tem `(u, v)` diferente de
> cada lado, e um plano por-vértice não o sabe dizer. *O canto é a menor unidade em que um
> atlas é exprimível* — e é por isso que a `Mesh` **não** ganhou um quinto plano nesta
> wave, ao contrário do que a §4 da avaliação supunha.

### O que ela faz, em três passos

| passo | o que decide |
|---|---|
| **juntar** | costura com salto `0 (mod 4)` não é corte ⇒ união dos patches (§3) |
| **assentar** | um deslocamento por patch, acumulado ao longo de uma **ÁRVORE** |
| **arrumar** | prateleiras, mais altas primeiro, com o **vão do mip** (`8` texels a `2048²`) |

### A tabela, nas peças do dono

| peça | entrada | ilhas | cantos | órfãos | cortes que o atlas obrigou (rasgo) | aprov. | **texels pintados 2×** | relógio |
|---|---|---|---|---|---|---|---|---|
| `_base_sculpt` | CRUA | 13 | 101 376 | **0** | 53 (`12,9`) | 74,6 % | **10,06 %** | 0,3 ms |
| `_base_sculpt` | F1 | 4 | 9 108 | 0 | 27 (`23,4`) | 50,5 % | 6,58 % | 0,1 ms |
| `sculpt_antes` | CRUA | 4 | 82 080 | 0 | 49 (`33,6`) | 67,7 % | **34,07 %** | 0,3 ms |
| `sculpt_antes` | F1 | 10 | 11 946 | 0 | 31 (`7,1`) | 82,3 % | 8,80 % | 0,1 ms |
| `Sculpt_Blender` | CRUA | 11 | 49 746 | 0 | 60 (`49,3`) | 55,7 % | 16,73 % | 0,2 ms |
| `Sculpt_Blender` | F1 | 5 | 11 580 | 0 | 30 (`6,3`) | 61,4 % | 12,60 % | 0,1 ms |

⭐ **O atlas custa `0,1`–`0,3 ms`** — nada ao lado dos `22`–`52 s` da parametrização que o
alimenta. *A peça cara é a de cima, e ela já estava construída.*

### ⛔⛔⛔ O VERMELHO, e ele é de correcção: `6,6 %` a `34 %` dos texels são pintados DUAS VEZES

Uma ilha assentada ao longo de uma árvore **não tem holonomia e pode dobrar-se sobre si
mesma** — nada no assentamento o impede. Um texel coberto por dois sítios da superfície é
tinta que aparece onde ninguém a pôs, e é o defeito que um atlas não pode ter.

⚠️ **Nenhuma régua desta wave o via**, e a razão é a de sempre nesta casa: as doze do
`ph2d-uv-atlas` olham **caixas** (cabe no quadrado · não sobrepõe a vizinha · a costura não
rasga) e **uma dobra acontece DENTRO de uma caixa**. A medição existe hoje na sonda
(`sobreposicao`, rasterizando a `1024²`) e ⏳ **a cura é a wave seguinte**: é o passo de
CORTE que todo desenrolador tem, e que esta versão não tem.

⇒ ⛔ **O atlas ainda NÃO serve para pintar.** Ele serve para exportar, para medir e para
ver; e o número que falta descer é este.

> ✅ **CURADO na W2 — e a atribuição mostrou que a causa não era a que esta secção supõe.**
> A frase *«uma ilha assentada … pode dobrar-se sobre si mesma»* está certa; o que faltava
> era saber **quanto** de cada mecanismo, e a régua nova diz que `97 %`–`99,5 %` do
> vermelho é o ASSENTAMENTO a pôr duas cartas da mesma ilha uma em cima da outra, contra
> `0,07 %`–`0,10 %` de dobra do solver. ⇒ [`§9`](#9--w2-o-corte--o-vermelho-desce-a-zero-e-o-preço-tem-número)

### O que a construção ensinou, e não estava previsto

1. ⭐⭐⭐ **Os «cortes que o atlas obrigou» não são um defeito — são o género da peça.** O
   assentamento percorre uma árvore, e toda costura colada que sobra depois dela é uma
   aresta que não cabe no plano. *Uma esfera não se desenrola sem um corte.* ⛔ O que
   **seria** defeito é existirem e ninguém as contar: aí o rasgo lê-se como *«o solver
   falhou»* em vez de *«a peça tem género»* — por isso o relatório tem a coluna.
2. ⛔⛔ **A régua da holonomia era um ESPELHO.** A 1.ª redacção comparava `oa − t` com
   `ob`, que é literalmente a expressão que assentou as cartas, e a mutação que trocava o
   SINAL do assentamento **sobreviveu**. Hoje ela pergunta a coisa que interessa: *com as
   cartas postas, os dois lados caem no mesmo ponto?*
3. ⛔⛔ **O empacotador só verificava a ALTURA**, e uma ilha mais larga que o quadrado era
   «colocada» na primeira prateleira com o laço a declarar que coube (`u = 1,42`). Quem o
   apanhou foi o gate do quadrado unitário, na primeira corrida.
4. ⛔⛔⛔ **A fixtura de DUAS cartas deixava dois braços do código fora do alcance da
   prova** — o `(Some, None)` do assentamento só corre quando a fita FECHA, e o
   `position(…)` da rotulagem só corre com três patches. Duas mutações sobreviveram por
   não serem alcançadas, o que num relatório se lê exactamente como *«o gate não vê o
   defeito»*. ⇒ a fixtura passou a ser **três cartas com um interruptor de ANEL**.
5. ⛔⛔ **E as cartas nasciam CONTÍGUAS**, o que dá translação de costura **zero** — e `+0`
   e `−0` são a mesma coisa, logo o sinal do assentamento continuava inatacável. *Um
   corpus no ponto NEUTRO de um parâmetro não testa esse parâmetro.*

**Gates:** 12, com **10 mutações e 10 a sangrar** (as três últimas nasceram das
sobreviventes). O controlo de produto vive no `atlas_probe`, que corre sobre as peças do
dono e desenha o atlas (`PH2D_ATLAS_DUMP=<dir>`, um `.ppm` com uma cor por ilha).

---

## §9 — ⭐⭐⭐ W2: O CORTE — o vermelho desce a ZERO, e o preço tem número

> **Ordem do dono (2026-09-20):** *«smoke parece OK. Siga»* — sobre o vermelho que a §8
> deixou nomeado.

### §9.1 — ⛔⛔ Passo zero: a régua velha dizia QUANTO e não dizia DE QUEM

A W1 mediu a sobreposição por TEXEL e leu `0,04 %` numa peça e `38 %` noutra. *Uma régua
que conta QUANTOS nunca vê QUAIS* — a lei que esta casa já pagou no `edge_max` cego ao
quad fino, no `χ` cego à almofada e nas três réguas da ponta que deitavam fora o índice
antes de devolver.

A régua nova ([`ph2d_uv_atlas::sobreposicao`](../../crates/ph2d-uv-atlas/src/sobreposicao.rs))
mede a **área exacta** em que dois triângulos se cruzam (recorte de polígono convexo em
`f64`, com o piso em `1e-6` da área do menor — ⛔ *relativo e não absoluto: uma malha fina
tem triângulos de `1e-6` do atlas, e um epsilon absoluto acusaria a vizinhança inteira*) e
**atribui cada cruzamento a um mecanismo**:

| classe | o que aconteceu | onde está a cura |
|---|---|---|
| `dobra` | os dois triângulos são **vizinhos na peça** e mesmo assim se cruzam ⇒ o mapa inverteu-se | a montante, no solver contínuo (G3) |
| `mesma-carta` | uma carta não é injectiva **sozinha** | a montante |
| `mesma-ilha` | duas cartas da mesma ilha foram **assentadas uma em cima da outra** | o CORTE |
| `ilhas-diferentes` | ⛔ **CONTROLO — tem de ser ZERO** | o empacotador |

**E a tabela decidiu a wave inteira**, sobre a malha CRUA, que é o caminho do produto:

| peça | área cruzada | `dobra` | `mesma-carta` | **`mesma-ilha`** | `ilhas-diferentes` |
|---|---|---|---|---|---|
| `_base_sculpt` | `9,56 %` | `0,10 %` | `0,15 %` | **`9,31 %`** | `0,00 %` |
| `sculpt_antes` | `31,57 %` | `0,08 %` | `0,07 %` | **`31,42 %`** | `0,00 %` |
| `sculpt_Depois` | `26,41 %` | `0,07 %` | `0,42 %` | **`25,93 %`** | `0,00 %` |
| `esfera:24` | `0,05 %` | `0,04 %` | `0,01 %` | `0,00 %` | `0,00 %` |

⇒ **`97 %` a `99,5 %` do vermelho é o assentamento**, e não o solver a dobrar. *Sem a
atribuição eu teria começado por afinar o G3, que responde por um décimo de ponto.*

⛔⛔ **E a última linha é a mais importante para quem escrever gates: a ESFERA NÃO CONTÉM
O FENÓMENO QUE ESTA WAVE ATACA.** Ali `mesma-ilha` lê `0,00 %` — *um gate escrito sobre a
peça de demonstração ficaria verde a afirmar nada*. As fixturas dos gates desta wave são
construídas para se dobrarem, e cada uma traz o controlo plano ao lado.

⚠️ **Isso NÃO quer dizer que o corte não faça nada na esfera, e a 1.ª redacção desta linha
dizia-o:** medido, ela vai de `2` para `14` peças, porque o corte separa também os pares da
classe `dobra` **entre faces vizinhas** (`62` na esfera crua). *Uma classe que responde por
um décimo de ponto de ÁREA pode responder por seis vezes a contagem de peças* — e na
esfera F1, com `17,94 %` dos triângulos invertidos, ela leva `2` peças a **`153`**.

### §9.2 — A lei do corte, e porque ela TERMINA

Uma peça cresce por vizinhança a partir de uma face semente e **só aceita uma face que não
cruze nenhuma das que já lá estão**, com o teste exacto acima. Daí saem as duas
propriedades que interessam: cada peça é injectiva **por construção e não por promessa**, e
o laço acaba porque toda passagem coloca pelo menos a semente.

⛔⛔ **A unidade é a FACE e nunca o triângulo.** O atlas guarda um `(u, v)` **por canto**, e
os dois triângulos de um quad partilham dois cantos: pô-los em peças diferentes pediria
dois `(u, v)` no mesmo canto, que é inexprimível. ⇒ *o que este corte não separa é uma face
que se dobra sobre si mesma* — e essa é exactamente a coluna `dobra` da tabela, cuja cura é
a montante.

### §9.3 — ⛔⛔ A 1.ª redacção ficou CERTA e ILEGÍVEL, e a medição disse porquê

Ela crescia **uma peça até ao fim** e só depois semeava a seguinte:

| redacção | peças | de uma face só | maior peça | aproveitamento | texels 2× |
|---|---|---|---|---|---|
| **sem corte** (W1) | `13` | — | — | `74,6 %` | `10,06 %` |
| crescer até ao fim | `485` | **`361`** | `21 801` | `61,4 %` | `0,00 %` |
| ronda-a-ronda | `382` | `55` | `1 457` | `67,7 %` | `0,00 %` |
| **+ fusão** (o que shipa) | **`226`** | `53` | `4 047` | `67,7 %` | `0,00 %` |

*(`_base_sculpt`, malha CRUA, `33 792` faces.)* ⚠️ A linha do meio foi medida **com** a
oferta a uma peça vizinha que a [§9.5](#95--⛔-o-que-a-construção-ensinou-e-não-estava-previsto)
mostrou valer `±5 %` e que foi apagada; as outras três são o caminho que shipa.

⚠️ **As `361` peças de uma face não eram geometria, eram a ORDEM:** uma face recusada
ficava para trás enquanto a peça que a recusou **lhe comia todos os vizinhos**, e quando
ela enfim era semeada já não tinha para onde crescer. *Uma peça que cresce até ao fim antes
de a seguinte nascer não está a repartir a ilha — está a ficar com ela.* ⇒ cada peça activa
avança **uma face por ronda**, e uma face recusada vira semente **na mesma corrida**, logo
compete pelos próprios vizinhos em vez de os perder.

⭐⭐ **E a ronda-a-ronda traz o defeito oposto, que a FUSÃO desfaz:** duas frentes que se
encontram sem se cruzarem ficaram separadas por nada — a maior peça caiu de `21 801` para
`1 457` faces, que é um corte que a geometria não pediu. Duas peças que se encostam e cujo
conjunto continua injectivo passam a ser **uma**, por conjunto disjunto sobre as peças
(⭐ *a grelha de busca nunca é reescrita, e é isso que torna a fusão barata o bastante para
correr em rondas*).

### §9.4 — A tabela final, nas peças do dono

| peça | entrada | ilhas → peças | de uma face | fusões | aprov. | **texels 2×** | atlas |
|---|---|---|---|---|---|---|---|
| `_base_sculpt` | CRUA | `13` → **`226`** | `53` | `260` | `74,6 → 67,7 %` | **`10,06 % → 0,00 %`** | `83 ms` |
| `_base_sculpt` | F1 | `4` → `93` | `15` | `72` | `50,5 → 67,7 %` | `6,58 % → 0,00 %` | `5,3 ms` |
| `sculpt_antes` | CRUA | `4` → **`129`** | `30` | `244` | `67,7 → 67,7 %` | **`34,07 % → 0,00 %`** | `94 ms` |
| `sculpt_antes` | F1 | `10` → `88` | `17` | `70` | `82,3 → 67,7 %` | `8,80 % → 0,00 %` | `7,6 ms` |
| `sculpt_Depois` | CRUA | `37` → **`424`** | `86` | `428` | `50,5 → 61,4 %` | **`21,94 % → 0,00 %`** | `139 ms` |
| `sculpt_Depois` | F1 | `5` → `86` | `18` | `84` | `67,7 → 67,7 %` | `4,99 % → 0,00 %` | `4,9 ms` |
| `esfera:24` | CRUA | `2` → `14` | `2` | `24` | `67,7 → 61,4 %` | `0,04 % → 0,00 %` | `5,0 ms` |
| `esfera:24` | F1 | `2` → `153` | `26` | `106` | `50,5 → 67,7 %` | `38,12 % → 0,00 %` | `14,4 ms` |

⭐⭐ **O que sobra está ABAIXO DE UM TEXEL, e isso é uma MEDIÇÃO e não um arredondamento
da tabela.** A régua imprime o pior par de cada corrida: no `sculpt_Depois` ele mede
**`1,57e-10`** do quadrado unitário, e um texel a `1024²` mede `9,5e-7` ⇒ o pior
cruzamento que fica é **`6 000×` mais pequeno que um texel**. ⚠️ *Sem essa linha, um `1`
na coluna de uma classe lê-se igual a um `1000`* — e a contagem de `mesma-ilha` chega a
`1` numa das seis corridas, que é o piso de `f32` entre o plano da ilha e o `[0,1]²`, não
uma ilha que se dobrou.

⭐ As `18`–`144` ocorrências que a coluna conta são, todas, dessa ordem; a classe delas é
`dobra` — as faces que se dobram sobre si mesmas, que o corte por face não separa **por
desenho**.

⚠️ **O aproveitamento vai nos dois sentidos** (`−15` a `+11` pontos): muitas peças pequenas
arrumam-se melhor que poucas esparramadas numa peça e pior noutra. **O preço com nome é a
CONTAGEM DE ILHAS** — é ela que conta as costuras que o artista pode ver.

⭐ **O relógio do atlas (a coluna da direita) inclui o corte** e vai de `5` a `139 ms`, ao
lado dos `22`–`52 s` da parametrização que o alimenta. *A peça cara continua a ser a de
cima.*

### §9.5 — ⛔ O que a construção ensinou, e não estava previsto

1. ⛔⛔ **Uma linha que a prova de mutação não consegue matar é um comentário com sintaxe
   de código — e a MEDIÇÃO que a julga tem de ser a do produto, não a de uma peça.** Eu
   escrevi uma oferta *«antes de abrir peça nova, dá-a a uma peça vizinha»*; a mutação que
   a apagava deixou os `25` gates verdes, logo ela não é lei nenhuma. Medida nas quatro
   corridas:

   | corrida | com a oferta | sem ela |
   |---|---|---|
   | `_base_sculpt` CRUA | `227` peças | **`226`** |
   | `_base_sculpt` F1 | `92` | `93` |
   | `sculpt_antes` CRUA | `122` | `129` |
   | `sculpt_antes` F1 | `89` | `88` |

   ⇒ ela **troca de sinal entre peças** e vale `±5 %`. *Uma heurística que ganha numa peça
   e perde noutra não é uma alavanca; é ruído com dez linhas de código* — foi **apagada**,
   com a tabela ao lado para quem a quiser reconstruir saber o que compra.
2. ⛔⛔ **O primeiro gate do rasgo media a coisa errada, e ele nasceu VERMELHO.** Eu
   desloquei um VÉRTICE do plano para fabricar um corte, e isso move os cantos dos dois
   lados juntos — é uma aresta **esticada**, não um rasgo, e as duas faces continuam a
   encostar. *Um corte só existe onde o mesmo vértice tem `(u, v)` DIFERENTE de cada lado*,
   e a fixtura passou a deslocar por CANTO.
3. ⛔⛔ **Duas mutações sobreviveram e cada uma nomeou uma régua em falta:** o elo verificar
   **as duas pontas** de uma aresta (as fixturas deslocavam a costura inteira, e nenhuma a
   deslocava numa ponta só — que é a forma de uma costura em LEQUE) e a face **meio posta**
   (`all` trocado por `any`: nenhuma fixtura tinha uma face com dois cantos de três, e meia
   face entra com o canto que falta em `(0, 0)`). Com as duas leis escritas: **14 mutações,
   14 sangram.**
4. ⚠️ **HR-5 mordeu onde importa.** O clippy recusou `HashMap`/`HashSet`, e aqui não é
   decoração: a ordem em que os elos saem alimenta a partição, e uma tabela de dispersão
   daria outro atlas a cada arranque. Tudo em `BTreeMap`/`BTreeSet`.
5. ⭐ **A tolerância do elo virou PORTA** (`topo::TOLERANCIA_DO_ELO`): ela tinha três
   chamadores a escrever `1e-2` cada um, e *três respostas à mesma pergunta divergem no dia
   em que alguém afina uma*.
6. ⛔⛔⛔ **A IMAGEM DO SMOKE NÃO CONTINHA O FENÓMENO, e eu só o vi ao olhar para ela.**
   A sonda desenhava **uma cor por ilha e mais nada** — e uma ilha que se pinta duas vezes
   desenha a MESMA cor por cima de si mesma: a peça do dono saía um bloco vermelho
   impecável com `34 %` da área em duplicado. *Comparar essa imagem com a do depois ensina
   que a segunda tem mais cores, não que a primeira está partida.* ⇒ o que é pintado mais
   de uma vez sai a **BRANCO**, pelo MESMO percurso que a linha `dobra` conta — logo a
   imagem e o número são o mesmo facto, e não duas medições que podem discordar.
7. ⚠️ **O `Opcoes { cortar: false }` é uma PORTA e não uma variável de ambiente** — uma env
   lida dentro da crate alcançaria todo chamador e faria um gate medir a máquina em vez da
   lei. A sonda é o único sítio que a passa, e é ela que desenha o lado sem corte, que é o
   CONTROLO da wave.

### §9.6 — ⏳ O que fica ABERTO, com o mecanismo

- ⏳⏳ **O corte parte por um cruzamento de QUALQUER tamanho, e num mapa limpo isso é caro:**
  na esfera crua ele paga **`12` peças a mais para tirar `92` texels** de `1024²`. ⭐ O
  recurso da cura tem nome — *um cruzamento menor que um TEXEL não é um defeito que o
  artista veja* —, e é por isso que ela não entra nesta wave: o texel só existe depois de
  se saber o lado do quadrado, e o lado do quadrado depende das peças que o corte deu.
  *Quebrar essa circularidade (estimar o lado antes de cortar, e gatear que a estimativa
  nunca cresce o erro) é uma wave com espec própria.*
- ⛔ **O `dobra` que sobra é a montante.** `18`–`52` triângulos por peça com a área UV
  invertida; a cura é do G3 e não do atlas, e o corte por face não a pode alcançar.
- ⏳ **A contagem de peças é ALTA porque o mapa é amarrotado.** `226`–`413` peças para
  `13`–`37` ilhas de superfície. O corte é fiel: ele parte onde a superfície se dobra. ⇒ a
  alavanca seguinte **não é o corte**, é a parametrização produzir ilhas mais chatas —
  cortar a ilha **antes** de assentar, nas arestas de maior distorção, que é o passo
  *seamster* que esta cadeia nunca teve.
- ⏳ **O empacotador continua a ser de prateleiras** sobre caixas, e com centenas de peças
  pequenas ele tem mais a ganhar do que tinha com treze. Não medido.
- ⏳ **A dilatação da costura não existe** (o vão do mip está reservado e ninguém o pinta) —
  é o que impede um fio de fundo de aparecer na borda de uma ilha ao afastar a câmara.
