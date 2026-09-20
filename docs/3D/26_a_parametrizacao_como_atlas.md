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

---

## §10 — ⭐⭐⭐ W3: O ESPAÇO — e a régua que eu publicava media o INVÓLUCRO

> **Ordem do dono (2026-09-20), depois de aprovar o smoke da W2:** *«smoke ok mas com
> espaço pouco otimizado»*.

### §10.1 — ⛔⛔⛔ Passo zero: o `aproveitamento` era `67,7 %` e a tinta era `18,7 %`

A W1 e a W2 publicaram uma coluna chamada `aproveitamento`, e ela conta **a fracção do
quadrado que as CAIXAS das ilhas ocupam**. O que o artista vê é outra coisa — **a tinta**
—, e as duas estão a um factor de `3,6×` uma da outra:

| `sculpt_antes` | tinta / quadrado | caixas / quadrado | tinta DENTRO da caixa |
|---|---|---|---|
| CRUA | **`18,7 %`** | `67,7 %` | `27,6 %` |
| F1 | **`26,0 %`** | `67,7 %` | `38,5 %` |

⇒ *uma régua que mede o invólucro não mede o que está lá dentro*, e era o invólucro que o
relatório publicava. O campo [`Relatorio::aproveitamento`] passa a ser **a tinta**, e o
invólucro fica com o nome que diz o que ele é (`caixas_no_quadrado`).

⭐⭐ **E a decomposição decidiu o que construir:** `56`–`60 %` do desperdício está **DENTRO
das caixas**. *Nenhum empacotador de rectângulos lhe toca* — uma ilha esguia e curva num
rectângulo é um rectângulo quase vazio, por melhor que os rectângulos se arrumem entre si.
⇒ a wave não é «um empacotador melhor», é **arrumar a FORMA**.

### §10.2 — As três peças, e o que cada uma comprou

| | `sculpt_antes` CRUA | `sculpt_antes` F1 |
|---|---|---|
| W2 (prateleiras sobre caixas) | `18,7 %` | `26,0 %` |
| **+ orientar** pela caixa mínima | `22,2 %` | `34,1 %` |
| **+ arrumar pela máscara** | `29,2 %` | `34,0 %` |
| **+ bissectar o lado** | `29,6 %` | `38,7 %` |
| **+ pagar a folga UMA vez** (o que shipa) | **`31,8 %`** | **`41,0 %`** |

**`1,70×`** e **`1,58×`** a tinta da W2, com `0,00 %` de texel pintado duas vezes em todas
as corridas.

1. **ORIENTAR** ([`orienta`](../../crates/ph2d-uv-atlas/src/orienta.rs)) — a caixa de área
   mínima de um convexo tem sempre um lado **colinear com uma aresta do casco**, logo o
   mínimo acha-se **exactamente** e não por varredura de ângulos. ⭐ É um movimento
   **rígido**: tudo o que o corte provou sobre a peça continua verdade por construção, e
   há gate a medir que rodar não muda uma distância.
2. **ARRUMAR PELA MÁSCARA** ([`empacota`](../../crates/ph2d-uv-atlas/src/empacota.rs)) — a
   peça vira uma silhueta de células e entra onde ela não bate na ocupação, o mais em
   baixo e à esquerda. ⛔⛔ **A rasterização é CONSERVADORA — uma célula que o triângulo
   TOCA fica marcada** —, e é só isso que faz a garantia valer em `[0,1]²` e não apenas na
   grelha: a cobertura real é um subconjunto das células marcadas.
3. **BISSECTAR** — a fase que cresce o quadrado anda `8 %` de cada vez, logo pára até
   `8 %` acima do necessário, **e o lado entra na conta ao quadrado: são `16 %` de área**.
   Cinco passos de bissecção entre o último que não coube e o primeiro que coube fecham a
   folga a menos de `0,3 %`.
4. ⭐⭐ **PAGAR A FOLGA UMA VEZ.** A 1.ª redacção engordava **as duas** máscaras de `g`
   células, logo entre duas peças ficavam `2g` — e a cadeia de mips pede `g`. *A folga
   estava a ser cobrada a dobrar, e a constante `VAO_EM_TEXELS` dizia uma coisa enquanto o
   atlas entregava o dobro.* A cura é assimétrica e é exacta: **quem PERGUNTA é a máscara
   com auréola, quem MARCA é o corpo** — a auréola de A evita o corpo de B e o corpo de A
   evita a auréola de B, ⇒ a separação é exactamente `g` nos dois sentidos, por
   construção.

### §10.3 — ⛔ Duas premissas que morreram, e as duas ficam à vista

1. **`Relatorio::aproveitamento` contava CAIXAS.** Ver a §10.1 — o campo mudou de
   significado e o antigo ficou, com o nome certo.
2. ⛔⛔ **«As caixas das ilhas são disjuntas» deixou de ser verdade, DE PROPÓSITO.** O gate
   `as_ilhas_nao_se_sobrepoem_no_atlas` media exactamente isso, e com o empacotador por
   máscara **duas caixas cruzam-se** — é daí que vem a tinta que a wave ganhou. *Manter a
   régua das caixas seria proibir a cura.* A lei que fica é mais forte e é a que interessa:
   **nenhum texel do atlas é escrito por duas peças diferentes**, que é a classe
   `ilhas-diferentes` da régua da W2, e ela lê `0` em todas as corridas.

### §10.4 — ⭐ O CONTROLO que torna a coluna nova confiável

A tinta é medida por **dois caminhos que não se conhecem**: a soma das **ÁREAS** dos
triângulos (dentro da crate) e a contagem de **TEXELS** pintados (na sonda, a `1024²`).
Elas leem `31,8 %` e `31,8 %`. *Se discordassem, uma das duas estaria a medir outro atlas.*

### §10.5 — A resolução da grelha é MEDIDA, e o joelho é nítido

| células por lado | tinta/quadrado (CRUA) | relógio | tinta (F1) | relógio |
|---|---|---|---|---|
| `128` | `19,9 %` | `121 ms` | `31,8 %` | `22 ms` |
| **`256`** | **`29,6 %`** | `151 ms` | **`38,7 %`** | `46 ms` |
| `512` | `30,2 %` | `287 ms` | `39,7 %` | `125 ms` |

⭐ `128 → 256` compra **`9,7` pontos**; `256 → 512` compra **`0,6`** por **`1,9×`** o
relógio. ⛔ E a grelha grossa perde por dois caminhos, não um: a forma fica pixelizada
**e** a folga obrigatória de uma célula à volta de cada peça passa a valer mais do que a
peça. *Com `129` peças, uma célula de folga em cada uma é o preço de existirem tantas.*

### §10.6 — ⛔⛔⛔ A prova de mutação apagou DUAS coisas que eu tinha escrito

**(a) O teste de colisão do empacotador era inalcançável, e há PROVA e não só medição.**
A 1.ª redacção tinha um `bate()` que verificava a máscara contra a ocupação, e a mutação
que o apagava **sobreviveu**. A causa não é uma fixtura fraca: `topo[c]` é a **marca de
água** da coluna `c`, logo *toda célula acima dela está vazia por construção*; e a fórmula
do sítio garante `y ≥ topo[x+c] − piso[c]` em toda coluna, ou seja a célula mais baixa da
peça cai **em cima ou acima** da marca. ⇒ *o teste nunca podia disparar*.

⭐ **E eu tentei torná-lo necessário ANTES de o apagar**, que é o que separa apagar de
desistir: deixei as peças pequenas procurarem `24` células **abaixo** do céu, para entrarem
debaixo de uma saliência. Medido: **`0,0` pontos** (`29,6 %` e `38,7 %`, iguais ao dígito)
por `+4 %` de relógio. ⇒ as duas coisas saíram, com o número ao lado.

**(b) Duas réguas minhas não continham o fenómeno.** O controlo da rasterização olhava uma
célula **fora da caixa** do triângulo, que a varredura nunca visita — um `toca` que
devolvesse `true` a tudo passava; hoje a fixtura é uma hipotenusa cuja **caixa cobre a
grelha inteira** e cujo corpo não. E o valor de fábrica das duas curas não era gateado, o
que deixava a mutação que as desliga passar em silêncio: *todos os outros gates passam as
opções à mão*.

**(c) E uma peça que colapsa num PONTO não tinha fixtura.** A rasterização visita
`floor(min)..ceil(max)`, que numa peça de extensão zero é **vazio** ⇒ máscara sem uma
célula ⇒ o empacotador recusa-a e o atlas INTEIRO cai para a rede das prateleiras. A cerca
existia e nenhuma fixtura a tocava; hoje há uma carta colapsada de propósito.

### §10.7 — ⏳ O que fica ABERTO, com o mecanismo

- ⏳ **`tinta DENTRO da caixa` não se mexeu** (`32,8 %` na CRUA): ela é uma propriedade das
  FORMAS e não da arrumação. Quem a quiser mover tem de entregar ilhas menos esguias — e
  isso é a montante, no desdobramento.
- ⏳ **A folga custa uma célula POR PEÇA, e agora UMA e não duas.** Com `129` peças a
  `256` células ela continua a ser a segunda maior parcela do desperdício depois da forma.
  *Menos peças compram-na de volta duas vezes: menos folgas e ilhas mais gordas* ⇒ é o
  mesmo item que a §9.6 já nomeia.
- ⏳ **O empacotador não experimenta rodar a peça `90°`** na hora de a colocar. A
  orientação escolhe a caixa mínima, que fixa qual lado é o comprido; um segundo candidato
  por peça é barato e não foi medido.
- ⏳ **A dilatação da costura continua por pintar** — o vão está reservado e ninguém o usa.

---

## §11 — ⭐⭐⭐ A FORMA das ilhas: a atribuição diz que o corte está ILIBADO

> **Ordem do dono (2026-09-20):** *«siga»*, depois do smoke do espaço.

A §10 deixou a coluna que não se mexeu: **`tinta DENTRO da caixa`**, `32,8 %` na malha crua
— ou seja, uma ilha ocupa um terço do rectângulo que a envolve. Antes de construir
qualquer cura a montante, a pergunta é **quem** a deixa assim: a parametrização, ou o meu
corte?

A régua é a mesma coluna, lida **dos dois lados**:

| peça | ANTES do corte | DEPOIS |
|---|---|---|
| `sculpt_antes` CRUA | `36,6 %` em `4` ilhas | `32,8 %` em `129` peças |
| `sculpt_antes` F1 | `45,1 %` em `10` ilhas | `45,7 %` em `88` peças |

⇒ **o corte está ilibado.** Ele custa `3,8` pontos numa peça e **ganha** `0,6` na outra:
as ilhas **já saem esguias da parametrização**, e retalhá-las não as torna pior de forma
apreciável. *Sem esta leitura eu teria gasto a wave seguinte a tornar o corte mais
esperto, que responde por um erro de arredondamento.*

⛔ **E isto muda o endereço da cura:** ela é a montante, no que produz as ilhas.

### §11.1 — ⭐⭐⭐⭐ A COLAGEM perde nos DOIS eixos, e o valor de fábrica muda

A montante das ilhas está uma decisão que a W1 tomou sem a medir: **colar as cartas** onde
a costura não roda. Ela junta `88` cartas em `4` ilhas — o menor número de costuras
possível — e era a base da §3 deste doc.

⚠️ **Medida com a porta [`Opcoes::colar`], ela perde nos dois eixos que interessam:**

| `sculpt_antes` | a colar (W1–W3) | **sem colar** |
|---|---|---|
| CRUA · tinta / quadrado | `31,8 %` | **`42,6 %`** |
| CRUA · costura que o pintor sente | `156,5` (`37,8×`) | **`129,3`** (`31,2×`) |
| CRUA · peças | `129` | `134` |
| CRUA · recusas do corte | `369` | `65` |
| CRUA · resíduo de cruzamento | `40` pares | **`0`** |
| F1 · tinta / quadrado | `41,0 %` | **`47,9 %`** |
| F1 · costura | `85,8` (`20,9×`) | `87,4` (`21,3×`) |

⭐⭐⭐ **O mecanismo: a continuidade que a colagem compra nas fronteiras que junta, o corte
paga-a de volta com JUROS noutro sítio.** Uma ilha colada enrola-se pela peça e dobra-se
sobre si mesma; o corte tem de a retalhar em `129` peças **por linhas que ele escolhe**, e
essas linhas somam mais comprimento do que as fronteiras de carta que a colagem tinha
poupado. Sem colar são `134` peças — praticamente as mesmas — cortadas pelas fronteiras
que a parametrização já desenhou.

⇒ ⛔ **`colar` shipa DESLIGADA**, e o gate `as_curas_do_espaco_shipam_ligadas` afirma-o com
o sinal ao contrário das outras três.

### §11.2 — ⛔⛔ A §3 deste doc fica CORRIGIDA, e a metade dela que se mantém

A §3 diz *«um patch não é uma ilha — são `4` a `13`, não `88` a `116`»*, e usou isso para
encolher o empacotador planeado. **A metade sobre a SUPERFÍCIE continua verdade**: uma
costura com salto `0 (mod 4)` não é um corte, e as componentes ligadas são mesmo `4` a
`13`. ⛔ **A metade sobre o ATLAS está errada:** a unidade certa para arrumar é a **CARTA**,
e transformar aquele facto topológico em ilhas de atlas custa um terço da textura.

*Um facto sobre a superfície não é, por si, uma decisão sobre o atlas* — e entre os dois
havia uma inferência que ninguém tinha medido.

### §11.3 — A tabela final, nas três peças do dono

Com `colar` desligada, `cortar`, `orientar` e `empacotar_por_mascara` ligadas:

| peça | entrada | cartas → peças | **tinta / quadrado** | costura | cruzamento |
|---|---|---|---|---|---|
| `sculpt_antes` | CRUA | `88` → `134` | **`42,6 %`** | `31,2×` | `0` triângulos |
| `sculpt_antes` | F1 | `67` → `106` | **`47,9 %`** | `21,3×` | `0` |
| `_base_sculpt` | CRUA | `99` → `242` | **`35,0 %`** | `36,3×` | `0` |
| `_base_sculpt` | F1 | `55` → `115` | **`46,1 %`** | `21,0×` | `0` |
| `sculpt_Depois` | CRUA | `285` → `468` | **`40,3 %`** | `47,7×` | `0` |
| `sculpt_Depois` | F1 | `58` → `98` | **`40,8 %`** | `21,4×` | `0` |
| `esfera:24` | CRUA | `11` → `14` | **`47,0 %`** | `8,1×` | `0` |
| `esfera:24` | F1 | `16` → `172` | **`38,5 %`** | `22,8×` | `0` |

⭐⭐ **E a coluna `cruza` lê `0` triângulos TOCADOS em todas as oito corridas** — não é
só a área que desce a zero: *não sobra um único par de triângulos que se cruze*, nem o
resíduo sub-texel de `dobra` que a W2 deixava. Sem colagem, as cartas que a
parametrização entrega já são injectivas, e o corte quase não tem o que fazer.

### §11.4 — ⛔⛔ E uma régua ficou a afirmar o que não mediu

Com a colagem desligada, a coluna do **rasgo** passou a imprimir `4,03e1` numa esfera —
ela percorre as costuras de salto `0` e mede a distância entre os dois lados **usando
deslocamentos que ninguém calculou**. Isso lê-se como *«o assentamento falhou»* quando a
verdade é *«não houve assentamento»*.

⇒ ela devolve `0` quando não se colou, e **o que a separa de um `0` de «assentou
perfeito» é a contagem de coladas ao lado**, que lê `0` também. *Um zero de «não medido»
e um de «perfeito» são o mesmo byte, e o que os separa é o piso de população.*

⚠️ **E a sonda tinha o mesmo defeito, um nível acima:** a porta de bissecção estava
escrita `env(...) != Ok("0")`, que com a variável por definir devolve **`true`** — logo
ela armava a colagem por conta própria e **ignorava o valor de fábrica**. No dia em que
ele mudou, a sonda continuou a medir o programa antigo e imprimiu a tabela de antes.
*Uma porta de bissecção que não cai no default mede outro programa que o produto.*

### §11.5 — ⏳ O meio-termo, que fica ABERTO com o mecanismo

Nem colar tudo nem nada: **colar duas cartas só quando a união delas continua injectiva**.
⭐ A maquinaria existe — é a fusão do [`corte`](../../crates/ph2d-uv-atlas/src/corte.rs),
aplicada às CARTAS antes do assentamento em vez de às peças depois dele. Ela compraria a
continuidade onde ela é de graça e nunca criaria o que o corte teria de partir.

⚠️ **Não é uma dominação garantida, e é por isso que não entra sem medição:** juntar duas
cartas dá uma ilha maior, que arruma pior — o ganho em costura pode não pagar a perda em
tinta. *As duas colunas têm de ser lidas juntas, como nesta secção.*
