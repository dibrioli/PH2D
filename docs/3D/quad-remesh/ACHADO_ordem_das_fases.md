# ACHADO — a ORDEM das fases: o vinco vem PRIMEIRO, e o nosso passo zero apaga-o

> **Fonte:** o *paper* do alvo, `ph2d-quadbench/docs/papers/quadwild-2021.pdf` — Pietroni,
> Nuvoli, Alderighi, Cignoni, Tarini, *Reliable Feature-Line Driven Quad-Remeshing*,
> SIGGRAPH 2021. ⭐ **Papers são fonte LÍCITA do Implementador** (`SKILL_Cleanroom` §3.I:
> *«suas fontes de código são espec+papers+PH2D»*). ⛔ Nada aqui vem do fonte do alvo.
>
> ⚠️ **Ele vive AQUI e não em `docs/3D/cleanroom/`, de propósito:** a regra daquela pasta é
> que o Implementador lê **só** os `SPEC_*` — tudo o resto lá é do E e do R, porque pode
> nomear internos do alvo. ⭐ *Este doc é derivado do **paper** e não nomeia nenhum*, e quem
> precisa de o ler é exactamente quem implementa. **Uma nota fora do alcance de quem executa
> não existe** (`CLAUDE.md` §2).
>
> **Data:** 2026-08-25, depois de o artista reportar três vezes o mesmo grupo de defeitos
> (pontas com furo · relevo não obedecido · superfície irregular na malha densa).

---

## §1 — O achado, numa frase

⭐⭐⭐ **O título do método é a especificação dele:** *Feature-Line **Driven*** — o vinco é
decidido **antes de tudo** e **nenhuma fase pode destruí-lo**. A nossa cadeia decide o vinco
**depois** do passo que o apaga.

| fase | ⭐ o alvo | ⛔ nós |
|---|---|---|
| 1 | marca as **arestas de vinco** (limiar de ângulo diedro; num bordo aberto o bordo também é vinco) | — |
| 2 | remalha **proibindo** qualquer operação que parta um vinco · **e afina onde eles se juntam** | remalha isotropicamente, **sem saber que vinco existe** |
| 3 | campo cruzado **colado ao vinco por construção**, e só depois difundido | campo da curvatura, com o vinco a entrar (ou não) como termo suave |
| 4 | os vincos **já são** fronteira de patch — os caminhos extra só completam o layout | o traçado sai só do campo |

⇒ **Tudo o que a linha construiu em 2026-08-25 para DETECTAR vincos está a tentar recuperar
uma informação que o nosso próprio passo zero apagou três fases antes.** É a explicação
mecânica de por que a detecção brigava com a malha fina: quanto mais fina a remalha, mais
lisa fica a quina que ela devia preservar.

---

## §2 — Os quatro detalhes que mudam desenho, não só ordem

1. ⭐⭐ **A remalha é ADAPTATIVA perto dos vincos.** Duas passagens: a primeira com alvo
   uniforme a **metade** da aresta final; a segunda com alvo **local**, interpolado entre
   `0,3×` e `3×` o pedido, em função da razão de aspecto dos triângulos da primeira (com o
   percentil 10 grampeado). *A justificação publicada é que perto de um aglomerado de
   vincos o campo tem feições de alta frequência e precisa de resolução.*
   ⇒ **Isto é a PONTA.** Uma ponta é exactamente onde o detalhe se concentra, e a nossa
   remalha dá-lhe o mesmo triângulo grosso da barriga.

2. ⭐ **Dois vincos no mesmo triângulo não existem — o triângulo é PARTIDO** até cada face
   tocar no máximo um. ⇒ ⛔ a nossa cerca de conflito (*«duas leituras a mais de 5° uma da
   outra largam a face»*, `ph2d_crossfield::CONSTRAINT_AGREEMENT`) resolve por **desistência**
   um caso que o alvo faz **não acontecer**.

3. ⭐⭐⭐ **Um patch mau não é reparado — não é EMITIDO.** Três condições de validade, e o
   traçado insere caminhos até **todas** valerem em **todos** os patches:
   - **topológica:** homeomorfo a um disco (um único laço de bordo);
   - **valência:** entre `3` e `6` lados;
   - **convexidade:** só vértices *straight* e *right-turn* na fronteira.
   ⇒ ⛔ Os **2 patches partidos** que o `chain_info` passou a imprimir em 2026-08-25
   (`CutReport::unopened`) são exactamente o que estas condições existem para impedir. Nós
   emitimos e **descobrimos** depois.

4. ⚠️ **A quantização dos lados é um ILP global**, cuja função objectivo favorece
   explicitamente patches que admitam a quadrangulação **regular** do passo final. *As
   condições de patch descem da estratégia de preenchimento, e não o contrário.*

---

## §3 — ⚠️ O aviso que o próprio paper dá, e que é sobre NÓS

> *«With CAD models, automatic labeling based on dihedral angles works well, but with other
> categories of models, such as scanned meshes, this requires more care and it is not
> trivial.»* (§8.1, Limitations)

⭐ **Uma escultura é essa segunda categoria.** ⇒ a parte que o alvo declara difícil é
precisamente a nossa — e é a justificação, medida e não por gosto, para a lei de três degraus
com janela de estabilidade que a `ph2d_mesh::feature_dirs` implementa. *Não é
sobre-engenharia: é o caso que o autor do método nomeia como o duro.*

⚠️ E o preço do lado deles também está declarado: *«The strict preservation of feature-lines
makes the system susceptible to miscategorized feature-edges»* — marcar mal é pior que não
marcar, que é a mesma cerca que a `SPEC_restricoes_por_eliminacao.md` §3.1 já escreve.

---

## §4 — ⛔ O que este achado NÃO diz

- ⛔ **Não diz que a nossa segunda metade está errada.** A cadeia deles é *layout + preencher
  cada patch*; a nossa é *mapa de grade inteira + extrair isolinhas* (a família do MIQ 2009).
  São famílias diferentes a jusante do layout, e o achado é **a montante** dele.
- ⛔ **Não promete paridade.** O que ele localiza é **onde a informação se perde**, e é uma
  perda que nenhuma fase seguinte pode desfazer.

---

## §5 — ⛔⛔ A OBRA A CORREU E MATOU AS OBRAS B–D (2026-08-25, no mesmo dia)

A hipótese era: *«o nosso passo zero apaga os vincos, e toda a detecção que a linha
construiu está a recuperar informação que nós próprios deitámos fora»*. ⭐ Ela é **falsa**, e
o instrumento que a matou é o [`crease_census`](../../../crates/ph2d-quadextract/examples/crease_census.rs).

### ⛔ Metade 1 — o vinco NÃO é apagado

Censo do ângulo diedro na peça do artista, antes e depois do passo zero:

| | arestas | p50 | p90 | p99 | **MAX** | `> 30°` | `> 45°` | `> 60°` |
|---|---|---|---|---|---|---|---|---|
| antes | 59 668 | `1,4°` | `5,3°` | `38,6°` | **`180°`** | `1,50 %` | `0,78 %` | `0,45 %` |
| ⭐ depois | 6 977 | `2,9°` | `11,4°` | `44,8°` | **`180°`** | **`2,59 %`** | **`0,99 %`** | **`0,64 %`** |

⭐ **A cauda aguda não encolhe — ela CRESCE em fracção**, e o máximo fica onde estava. Na
peça com cristas o máximo vai de `48,0°` a `50,0°`. ⇒ *a remalha isotrópica preserva as
quinas de facto*, apesar de não saber que elas existem.

⚠️ **E a leitura tem uma armadilha que é preciso nomear:** a malha fica **8,5× mais
grossa**, e numa malha grossa **toda** a distribuição sobe (`p50` de `1,4°` para `2,9°`).
*Comparar fracções entre duas resoluções diferentes mede em parte a resolução* — por isso o
que decide aqui é o **MAX**, que é invariante a isso, e ele não se move.

### ⛔ Metade 2 — a ponta NÃO fica sub-amostrada

| | aresta média no corpo | ⭐ na PONTA (raio `> 1,15×`) | razão |
|---|---|---|---|
| antes | `0,0301` | `0,0412` | ⛔ **`1,37×`** (a ponta era a parte GROSSA) |
| ⭐ depois | `0,0824` | `0,0736` | ⭐ **`0,89×`** (passa a ser a parte FINA) |

⇒ O passo zero **melhora** a resolução relativa da ponta. A obra D (afinar perto dos vincos)
atacaria um défice que não existe nesta peça.

### ⇒ O que sobra do §1 e do §2, corrigido

⛔ **A tabela do §1 descreve uma diferença REAL de ordem entre as duas cadeias — e não é ela
que produz os defeitos do artista.** *Uma diferença de desenho verdadeira não é, por isso,
uma causa.*

| # | obra | veredito |
|---|---|---|
| **A** | medir quanto o passo zero destrói | ✅ **feita — e refutou B, C e D** |
| ~~B~~ | marcar vincos antes do passo zero | ⛔ **morta**: nada há a salvar |
| ~~C~~ | o passo zero preservar o que está marcado | ⛔ **morta**: ele já preserva |
| ~~D~~ | o passo zero afinar perto dos vincos | ⛔ **morta nesta peça**: a ponta já sai mais fina que o corpo |
| ⭐⭐⭐ **E** | **as CONDIÇÕES DE VALIDADE do patch, impostas por construção** (§2.3) | **é a que fica** — ver abaixo |
| F | partir o triângulo com dois vincos (§2.2) | ⏳ regra melhor que a nossa cerca de desistência, mas é afinação ao lado da E |

### ⭐⭐⭐ Por que a E é a que fica

Ela é a única que **casa com tudo o que foi medido em 2026-08-25**:

- os **2 patches partidos** (`χ ≥ 2`) na peça do artista, e `0` em todas as outras;
- o contador `CutReport::unopened`, cujo próprio doc diz *«resultado vermelho, e a fase
  seguinte não os parametriza»* — e que **nenhum instrumento imprimia**;
- as **8 transições inexactas** e a região do mapa genuinamente degenerada, que só existem
  nessa peça;
- os furos, que só existem nessa peça.

⇒ **Nós emitimos patches e DESCOBRIMOS os maus; o alvo não emite um mau.** As três condições
— disco · valência `3`–`6` · convexidade — são verificáveis **antes** de o patch sair, e o
traçado insere caminhos até todas valerem. *É a diferença entre uma cadeia que repara e uma
que não produz o defeito.*


---

## §6 — ⭐⭐⭐ A obra E, medida: NÓS FUNDIMOS, ELES CORTAM

O censo das três condições, no corpus (o `chain_info` passa a imprimi-lo):

| peça | valência | ⛔ fora de `3..6` | ⛔ não-disco | ⛔ degenerados que SOBREVIVEM |
|---|---|---|---|---|
| ⛔ **do artista** | `{0:1, 2:1, 3:13, 4:18, 5:2, 8:1}` | **3** | **5** | **5** |
| com gancho | `{3:12, 4:18, 5:4}` | `0` | `0` | `0` |
| com orelha | `{3:8, 4:8, 5:1, 6:1}` | `0` | `0` | `0` |
| com cristas | `{3:13, 4:5, 5:3}` | `0` | `0` | `0` |
| enrugada | `{3:8, 4:10}` | `0` | `0` | `0` |

⭐ **Separação total, pela terceira vez no dia:** a peça do artista é a única que viola as
condições e a única com furos. Um patch com **zero** lados, um com `2`, um com `8`.

### ⛔ E a limpeza que existe PARA nisto, na primeira ronda

O contador novo `TraceReport::cleanup_stop` diz porquê: **`2` — «a ronda PIORAVA a
topologia»**. A dissolução de facto remove paredes; o resultado é topologicamente pior, e a
guarda estrita (que existe por uma medição de 22/08 no toro) recusa.

⭐⭐⭐ **É aqui que o desenho do alvo diverge, e a direcção é oposta:**

| | reparação |
|---|---|
| ⛔ **nós** | **FUNDIMOS** — dissolvemos a parede entre o patch mau e o vizinho |
| ⭐ **o alvo** | **CORTA** — insere mais caminhos até as três condições valerem |

⚠️ **Fundir só pode tornar os patches maiores e mais complexos** — é exactamente por isso que
a guarda vê a topologia piorar e desiste. *A nossa cura empurra na direcção do defeito, pela
segunda vez neste módulo* (a primeira foi o endurecimento local, `weld_solve::STIFFEN_PASSES`).

⇒ **A obra E é: reparar por CORTE, não por fusão.** O `dissolve` fica como está — ele não
está errado para uma lasca, que de facto é *uma parede a mais*; o que falta é o outro lado,
para o patch que é *uma parede a MENOS*.

⚠️ **E um defeito verdadeiro que NÃO é este** ficou nomeado no sítio: o ramo `None` da
`patches::dissolve` (o patch de zero lados) é **morto** — ele filtra os arcos perguntando ao
`side_arcs[p]` que já está vazio, e o comentário prometia *«a fronteira inteira sai»*. ⛔ Curá-lo
não cura a peça (a guarda é que trava), e por isso ele está **nomeado e não remendado**.


---

## §7 — O retrato dos 5, e as DUAS negativas do caminho

⛔ **«5 degenerados» junta pelo menos duas avarias com curas opostas.** O retrato
(`chain_info`, coluna `DEGENERADOS (patch, lados, laços, χ)`) na peça do artista:

| patch | lados | ⛔ laços de fronteira | `χ` |
|---|---|---|---|
| 10 | 4 | **2** | 1 |
| 15 | ⛔ **0** | ⛔ **0** | 1 |
| 21 | 3 | **3** | 1 |
| 24 | ⛔ **8** | **3** | 1 |
| 33 | **2** | **2** | 1 |

⭐⭐⭐ **Quatro dos cinco têm 2–3 laços de fronteira** — são anéis, e a cura publicada de um
anel é **CORTAR entre os laços**, não fundir. ⇒ a obra E fica confirmada pelo retrato, e não
só pelo *paper*. ⚠️ O `patch 15` é outra espécie (`0` lados e `0` laços: uma componente sem
parede à volta) e o `patch 24` é o único que a régua de valência do alvo (`3..6`) rejeitaria.

⚠️ **O `χ` sai `1` nos cinco, e isso não bate com «2 laços»** — para uma superfície ligada,
`b = 2` dá `χ = 0`. ⇒ *ou o `χ` desta tabela não é o `χ` do patch, ou o `loops_per_patch`
conta outra coisa.* **Não o resolvi**, e fica nomeado: a coluna que decide aqui é a dos
laços, que é a que a máquina de reparação consome.

### ⛔ Negativa 1 — forçar a cura que existe

`PH2D_CLEANUP_FORCE=1` deixa a limpeza passar por cima da guarda de topologia:

| | normal | forçado |
|---|---|---|
| ⭐ transições inexactas | `8` | ⭐ **`4`** |
| ⛔ arestas de bordo | **`8`** | ⛔ `10` |
| enviesamento p50 | `7,3°` | `7,6°` · `>60` de `5` para `7` |

⭐⭐ **Fundir METADE o defeito de montante e paga na geometria** ⇒ os patches maus **são**
causa das transições inexactas (a ligação estava por provar), e **fundir é a direcção
errada** (estava por medir). *As duas metades do §6 ficam medidas com um interruptor.*

⚠️ E o forçado pára na 2.ª ronda por `cleanup_stop == 1` — a `dissolve` devolve `false`, que
é o ramo morto do `patch 15` a fazer-se sentir. ⭐ *Ele deixou de ser um defeito lateral no
dia em que a guarda foi levantada.*

### ⛔ Negativa 2 — a poda dos tocos não é isto

`prune::PRUNE_STEMS` está desligada com uma recusa medida — e **a premissa dela dissolveu**:
ela foi medida contra o preenchimento por patch, que já não é o caminho que shipa. ⇒ liguei-a
(`PH2D_PRUNE_STEMS=1`) e ela remove **`0` tocos** nesta peça: a saída é **byte-idêntica**.
⭐ *A recusa pode estar obsoleta e ainda assim não ser a cura* — reabrir uma recusa é barato,
e concluir dela sem medir é que não.


---

## §8 — ⛔⛔ A reparação por CORTE foi construída e REJEITADA — e corrige o §7

O §7 leu *«quatro dos cinco têm 2–3 fronteiras ⇒ são anéis, e a cura de um anel é cortar»*.
O corte foi construído ([`ph2d_trace::patches::open_rings`]) e a medição diz outra coisa.

| | valor |
|---|---|
| anéis encontrados | `4` |
| ⛔ **paredes acrescentadas** | **`4`** — ou seja **UMA aresta por anel** |
| saúde `(distância, degenerados)` | `(1, 5)` ⇒ ⛔ **`(2, 6)`** |

### ⭐⭐⭐ O mecanismo

⚠️ **Um caminho de UMA aresta entre as duas fronteiras significa que elas se TOCAM.** Estes
patches não são anéis gordos com um buraco no meio: são **ESTRANGULADOS**, com as duas
fronteiras a passar a um triângulo uma da outra. ⇒ *cortar ali não abre nada — acrescenta um
toco*, e o toco produz mais um degenerado.

⇒ **É uma TERCEIRA espécie**, e nenhuma das duas curas serve:

| espécie | o que é | cura |
|---|---|---|
| lasca | uma parede **a mais** | fundir (`dissolve`) — ✅ existe |
| anel gordo | uma parede **a menos** | cortar (`open_rings`) — ✅ existe, desligado |
| ⛔ **estrangulado** | uma parede **no sítio errado** | ⛔ **não existe** |

⛔⛔ **A contagem de fronteiras não distingue o anel gordo do estrangulado** — só a
**distância entre elas** distingue, e nenhuma régua desta linha a media. *É a mesma forma de
erro do §6: um contador que junta duas avarias com curas opostas.*

⇒ **A obra seguinte é a RÉGUA**, não mais uma cura: medir a distância entre as fronteiras de
cada patch multi-loop, e só então escolher entre cortar (longe) e outra coisa (perto). Quando
ela existir, o `open_rings` é o consumidor dela — por isso ele fica construído e desligado.

⚠️ **E a leitura do §7 que este bloco corrige não era descuido de medição: era uma inferência
do NOME.** *«Duas fronteiras» chama-se anel na topologia, e o nome trouxe consigo a cura do
anel.*


---

## §9 — ⭐⭐⭐ A RÉGUA DO VÃO, e o que ela mata

A régua ([`ph2d_trace::patches::ring_gaps`]) mede o **vão** entre as duas fronteiras de cada
patch multi-fronteira, em arestas. Ela é o que o §8 pedia — e o que ela devolve fecha a
espécie inteira.

| peça | patch | lados | fronteiras | vão | faces |
|---|---|---|---|---|---|
| do artista | 10 | 4 | 2 | `2` | 16 |
| do artista | 21 | 3 | 3 | `2` | 8 |
| do artista | 24 | 8 | 3 | ⛔ `1` | 13 |
| do artista | 33 | 2 | 2 | ⛔ `1` | 7 |
| ⭐ **furada** | **2** | **16** | **6** | `4` | ⭐ **1 011** |
| furada | 14 · 19 · 21 | 10 · 2 · 2 | 4 · 2 · 2 | ⛔ `1` | 38 · 6 · 2 |

### ⛔⛔⛔ NÃO EXISTE UM ANEL GORDO NO CORPUS INTEIRO

O maior patch multi-fronteira tem **1 011 faces** e as duas fronteiras dele passam a
**4 arestas** uma da outra. ⇒ **o vão nunca cresce com o patch**, e a espécie «anel gordo»
— aquela para a qual o corte é a cura publicada — **não tem um único exemplar aqui**.

⚠️ **E a porta do vão não resgata o corte:** com `MIN_RING_GAP = 2` a barrar o
estrangulamento, a peça do artista continua a piorar (`(1,5)` ⇒ `(2,6)`) e a furada
**empata** (`(5,6)` ⇒ `(5,6)`). *Uma cura que empata no melhor caso e piora no resto não é
uma cura.*

### ⇒ A espécie certa, e a obra que ela pede

⭐⭐ **O defeito é o ESTRANGULAMENTO, e ele não é «uma parede a mais» nem «uma parede a
menos» — é uma parede NO SÍTIO ERRADO.** Não se cura acrescentando nem tirando: cura-se
**movendo-a**, que é re-traçar aquela região do layout.

⇒ Isso é uma obra maior que todas as deste doc, e é a primeira que toca o traçado em si em
vez das reparações à volta dele. ⚠️ **E é onde o desenho do alvo de facto difere**: ele nunca
chega a emitir a parede naquele sítio, porque as três condições de validade governam a
**inserção** dos caminhos, e não uma reparação posterior.

### ⭐ O que fica vivo desta jornada

| peça | estado |
|---|---|
| `patches::ring_gaps` | ⭐ **viva no instrumento** — é ela que nomeia a espécie |
| `PatchLayout::loops` | ⭐ viva — o layout calculava e deitava fora |
| `patches::open_rings` + `MIN_RING_GAP` | construídas, **desligadas**, com a tabela |
| `TraceReport::cleanup_stop` · `opened_rings` · `pruned` | ⭐ vivas — três «porquês» que não existiam |
| `PH2D_CLEANUP_FORCE` · `PH2D_PRUNE_STEMS` · `PH2D_OPEN_RINGS` · `PH2D_BRIDGE_LOG` | sondas, desligadas |


---

## §10 — ⭐⭐⭐ A RAIZ: a travessia de fronteira PERDE A FRONTEIRA

O §9 pediu que se medisse o vão. Medir os **tamanhos** das fronteiras, na mesma corrida, deu
a resposta que fecha a sequência inteira.

| peça | patch | faces | ⛔ **tamanhos das «fronteiras»** |
|---|---|---|---|
| do artista | 10 | 16 | `[1, 9]` |
| do artista | 21 | **8** | ⛔ **`[1, 1, 1]`** |
| do artista | 24 | 13 | `[2, 4, 4]` |
| do artista | 33 | 7 | ⛔ **`[1, 1]`** |
| furada | 2 | 1 011 | `[1, 4, 9, 10, 35, 100]` |

⛔⛔⛔ **Uma «fronteira» de UM vértice não é um laço** — um laço precisa de três. E um patch
de **8 faces** não tem três fronteiras de um vértice: ele tem uma fronteira de muitas
arestas. ⇒ **a travessia está a perder a fronteira nesses patches**, e o que sai não é uma
descrição do patch — é destroço.

### ⭐ Isto resolve a contradição que o §7 deixou aberta

O §7 registou: *«o `χ` sai `1` nos cinco, e isso não bate com duas fronteiras»*, e deixou-a
por resolver. ⭐⭐ **O `χ` estava certo o tempo todo** — eles **são** discos. Quem estava a
mentir era a contagem de fronteiras, e a linha construiu **duas curas** (o corte, a fusão
forçada) contra um número inventado.

### ⇒ A cascata inteira, relida

`boundary_loops` falha nestes patches ⇒ o `side_arcs` sai errado (o `patch 15` tem **`0`
lados**) ⇒ o `degenerate()` acusa discos ⇒ a limpeza tenta curar não-problemas e a guarda
recusa (correctamente) ⇒ o corte e a fusão forçada atacam a espécie errada ⇒ a jusante o
mapa recebe uma descrição do layout que não corresponde ao layout ⇒ região degenerada ⇒
transições inexactas ⇒ **furo**.

⚠️ **E não é uma diferença de desenho com o alvo.** É um defeito nosso, e passou porque
**nenhum instrumento imprimia nenhum destes números**.

### O que entra já

⭐ [`PatchLayout::real_loops`] — uma peça de menos de três vértices não conta como fronteira.
Ela tira **um** falso positivo na peça do artista (`5` ⇒ `4` degenerados) e deixa o produto
**byte-idêntico** (`8` bordo · `χ = 1` · `7,3°`), porque o defeito de fundo continua lá.
⛔ *Ela corrige a CLASSIFICAÇÃO, não o defeito* — e é por isso que o número de furos não se
mexe.

### ⇒ A obra seguinte

**Achar por que a travessia perde a fronteira num patch pequeno.** É a primeira desta
sequência que é um **bug nosso com endereço**, e não uma escolha de arquitectura: tudo o que
está a jusante consome o que ela devolve.


---

## §11 — ⭐⭐⭐ A RAIZ DA RAIZ: a escultura entra NÃO-MANIFOLD, e é na PONTA

O §10 disse que a travessia de fronteira morre. Ela morre **por uma razão**, e a razão está
na entrada.

| peça | `χ` da entrada | ⛔ **arestas não-manifold** | onde elas moram |
|---|---|---|---|
| ⛔ **do artista** | ⛔ **4** | ⛔ **2** | ⭐ **raio `1,30×` — a PONTA** |
| com gancho | `2` | `0` | — |
| enrugada | `2` | `0` | — |
| furada | `1` | `0` | (tem bordo por construção) |

E ao longo do passo zero:

| | `χ` | ⛔ não-manifold |
|---|---|---|
| entrada triangulada | `6` | ⛔ **`4`** |
| ⭐ depois do F1 | ⭐ **`2`** | ⛔ **`2`** |

⇒ **O F1 cura o `χ` e deixa DUAS arestas não-manifold vivas.**

### ⭐⭐⭐ O mecanismo, e ele fecha a sequência inteira

O layout percorre a fronteira dos patches pivotando num mapa de meias-arestas —
`(a, b) → face`, **uma face por aresta dirigida**. ⛔ **Numa aresta não-manifold há três ou
mais faces a reclamar a mesma aresta dirigida, e o mapa guarda uma.** ⇒ o pivô entra na face
errada ou não acha nenhuma, a travessia **morre**, e o pedaço parcial sai como se fosse um
laço — os `[1, 1, 1]` do §10.

⇒ **A cascata, do princípio ao fim:**

> 2 arestas não-manifold **na ponta** ⇒ o mapa de meias-arestas é inconsistente ali ⇒ a
> travessia de fronteira morre ⇒ laços de um vértice ⇒ patches classificados como
> degenerados sem o serem ⇒ a limpeza persegue fantasmas e a guarda recusa (correctamente)
> ⇒ o mapa recebe uma descrição do layout que não é o layout ⇒ região degenerada ⇒ transições
> inexactas ⇒ **furo na ponta**.

⭐⭐ **E isto fecha o círculo com o primeiro report do artista**, de 2026-08-24: *«furos nas
pontas»*. As arestas não-manifold estão a raio `1,30×`; os furos estão a raio `1,29×`. **É o
mesmo sítio.**

### ⇒ As duas obras que isto abre

| # | obra | nota |
|---|---|---|
| **1** | ⭐ **reparar o não-manifold na porta** (partir a aresta, duplicando o vértice) | é o que a cadeia precisa, e ⚠️ **não existe nada em `ph2d-mesh` que o faça** — há `fill_holes` e `merge`, e mais nada |
| **2** | ⚠️ **por que a escultura sai não-manifold do nosso próprio módulo de escultura** | é um nível acima, e é decisão do dono do produto se vale a pena — a cadeia tem de ser robusta a malha importada de qualquer forma |

⚠️ **A obra 1 não depende da 2**, e é a que desbloqueia tudo o que este documento descreve.


---

## §12 — ⭐⭐⭐ A RAIZ CONFIRMADA — e as QUATRO reparações que a curam e pioram a peça

O §11 nomeou a raiz. Esta secção **prova a ligação causal** e mede quatro curas.

### ⭐⭐ A prova: partir os vértices leva as transições inexactas de `8` a `ZERO`

Não é correlação — é o interruptor. `PH2D_MANIFOLD_REPAIR=1` e o número que perseguimos o dia
inteiro desaparece.

### ⛔ E as quatro variantes, todas piores que não reparar

| variante | bordo da saída | `χ` | transições inexactas | enviesamento | `>60°` |
|---|---|---|---|---|---|
| ⭐ **não reparar** | **`8`** | **`1`** | ⛔ `8` | **`7,3°`** | **`5`** |
| partir ANTES do remalhe | ⛔ `148` | ⛔ `−16` | ⭐ `0` | ⛔ `13,7°` | ⛔ `63` |
| partir + fechar buracos | ⛔ **saída VAZIA** | — | `0` | — | — |
| partir DEPOIS do remalhe | `8` | `0` | ⛔ `12` | ⛔ `9,1°` | ⛔ `11` |
| deitar a aleta fora | `8` | `1` | ⛔ `10` | ⛔ `8,3°` | ⛔ `11` |

### ⭐⭐⭐ O mecanismo comum, e ele fecha a família

⚠️ **Todas as quatro ABREM a superfície.** Partir uma aresta ambígua numa peça fechada
separa-a **por construção**; deitar a aleta fora deixa o buraco onde ela estava. E a medição
diz o que ninguém tinha perguntado:

> ⭐⭐ **Esta cadeia tolera pior um BURACO do que uma aresta ambígua.**
> Com o defeito: `8` arestas de bordo na saída. Sem o defeito mas com um rasgo: `148`.

⚠️ **E o remalhe CRIA não-manifold sozinho:** `4 ⇒ 0` na porta e **`2` outra vez** depois do
laço. *Reparar a malha que entra não é reparar a malha que sai* — e quem a cadeia consome é a
que sai.

### ⇒ A obra seguinte, agora com forma

**Uma cura que mantenha a peça FECHADA**: **soldar** as duas folhas na aresta ambígua
(colapsá-la) em vez de as separar. É a única direcção que a tabela acima não fechou, e é
outra operação — não uma afinação destas.

⚠️ **A partição fica construída, gateada e desligada** (`ph2d_remesh_iso::MANIFOLD_REPAIR`,
`PH2D_MANIFOLD_REPAIR` reabre), porque quando a solda existir ela é o controlo dela.

---

## §13 — ⭐⭐⭐ A CURA: não era aleta nenhuma — a sonda mediu, e era uma FOLHA DE ESPESSURA ZERO

**2026-08-26.** O §12 fechou com quatro reparações construídas e as quatro **piores que o
defeito**. ⚠️ **As quatro foram desenhadas a partir do NOME** — *«uma aleta»*, *«duas
folhas»*, *«um beliscão»* — e **nenhuma** a partir da estrutura medida. A quinta inverteu a
ordem: primeiro a sonda, depois a cura.

### §13.1 — O instrumento

[`manifold_census`](../../../crates/ph2d-quadextract/examples/manifold_census.rs), e a tabela
que ele existe para responder — *cada linha escolhe uma cura diferente*:

| o que a aresta é | como se lê na sonda | a cura que isso escolhe |
|---|---|---|
| uma **aleta** | 3 faces, uma com diedro ~0 | deitar a face fora |
| duas **folhas coladas** | 4 faces, dois pares | soldar |
| uma face **repetida** | mesmo conjunto de vértices | deduplicar |
| um **beliscão** | vértices coincidentes à volta | soldar por posição |

### §13.2 — A resposta, e a coluna que decidiu tudo

```
sculpt_t001: bordo 0 · NAO-MANIFOLD 4
  vertices COINCIDENTES: 0 grupos
  faces REPETIDAS: 4 conjuntos (4 copias a mais)
     · destas, 0 com a MESMA orientacao, 4 com orientacao OPOSTA
```

⭐⭐⭐ **A coluna da orientação é a que impede a cura errada.** Cópias com a **mesma**
orientação são lixo — a segunda não acrescenta superfície, e deduplicar é correcto. Com
orientação **oposta** são um par `(triângulo, espelho)`: uma **bolsa de volume zero**.
Apagar *uma* das duas tira metade de uma superfície fechada — ⇒ **é por isso que as quatro
tentativas do §12 abriram a peça.** Apagar **as duas** não tira superfície nenhuma.

⚠️ *Sem esta coluna, «4 faces repetidas» leva-se para as duas curas opostas com a mesma
confiança.*

### §13.3 — A cura guarda-se a si própria

[`ph2d_mesh::drop_doubled_faces`] mede o **bordo** antes e depois e **desfaz-se** se ele
subir. *A régua da recusa é a mesma que reprovou as outras quatro, e agora corre **dentro**
da cura em vez de depois dela.* Há gate a provar que a recusa dispara
(`the_cure_refuses_itself_when_it_would_tear_the_surface`).

### §13.4 — O que ela fez à cadeia inteira (`sculpt_t001`)

| régua | sem | **com** |
|---|---|---|
| não-manifold na porta | `2 ⇒ 2` | ⭐ **`0 ⇒ 0`** |
| patches fora de valência 3..6 | ⛔ `3` | ⭐ **`0`** |
| não-discos · degenerados sobreviventes | ⛔ `5` · `4` | ⭐ **`0` · `0`** |
| dobras no contínuo | `14` | ⭐ **`0`** |
| **transições inexactas** | ⛔ `8` | ⭐⭐⭐ **`0`** |
| resíduo de translação máx | `4,8e-1` | ⭐ **`9,5e-7`** |
| órfãs | `10` | **`2`** |
| **bordo da saída (os furos)** | ⛔ `8` | ⭐⭐ **`4`** |
| enviesamento p50 · >60° | `7,3°` · `5` | **`7,1°` · `3`** |
| aspecto p99 · >4× | `2,04` · `1` | **`1,91` · `0`** |

⭐ **Inércia provada no corpus REAL:** 11 das 12 peças saem **byte-idênticas**; só a `t001`
muda, e só para melhor. Por isso ela **shipa ligada**
(`ph2d_remesh_iso::DOUBLED_REPAIR = true`, `PH2D_DOUBLED_REPAIR=0` bissecta).

## §14 — ⛔⛔ UMA AFIRMAÇÃO MINHA REFUTADA: o remalhe NÃO cria não-manifold

O doc do `MANIFOLD_REPAIR` dizia *«o remalhe cria não-manifold sozinho — `4 ⇒ 0` na porta e
`2` outra vez depois do laço»*, e foi **essa frase** que pôs a reparação no fim do passe.

⭐ **O controlo nunca tinha sido corrido.** Medido em **onze** peças limpas do corpus, o
remalhe cria **zero** (`0 ⇒ 0` em todas). O `4 ⇒ 2` era da `t001`, que **entra** com `4`:
ele **propaga**, não cria.

⚠️ *Eu tinha comparado dois números da MESMA peça partida sem nunca olhar uma peça limpa.*
⇒ a chamada gémea depois do laço foi **retirada**; o único motivo dela era esta frase.

## §15 — ⭐⭐ A RÉGUA DO BORDO É O PERÍMETRO, nunca a contagem de arestas

A mesma varredura acusou o F1 de **alargar buracos** (`sculpt_punctured`: `38 → 107` arestas
de bordo). ⛔ **A contagem de arestas é função do PASSO** e não mede buraco nenhum: com a
régua certa — **laços + perímetro** — a mesma peça anda `5,6463 → 5,7390`, **+1,6 %**.

⭐ E com a régua certa o alargamento **real** aparece noutro sítio: a `t002` do Enio vai de
`0,6046` a `0,7841`, **+30 %**, porque o buraco dela é pequeno demais para o passo do
remalhe. *Um laço continua um laço nas duas.*

## §16 — ⭐⭐⭐⭐ O BORDO É UMA LINHA DE FEIÇÃO — e era a maior alavanca da cadeia

### §16.1 — O retrato das três peças do artista

| peça | entrada: bordo · não-manifold | pós-F1 | veredito do Enio |
|---|---|---|---|
| `t001` | `0` · ⛔ **`4`** | `0` · `2` | furos |
| `t002` | ⛔ **`8`** · `0` | `10` · `0` | *«furos nas pontas»* |
| `t003` | **`0` · `0`** | **`0` · `0`** | ⭐ *«melhor resultado até agora»* |

⭐⭐ **A peça que chega limpa é a que ele gostou** — a correlação é perfeita nas três.

### §16.2 — A pergunta que ninguém tinha feito

A `t001` curada tem **`0`** dobras no mapa contínuo; a `t002` tem **`38`**. A diferença entre
as duas é **o buraco** — e o *paper* é explícito desde o §2 deste doc: uma linha de feição
restringe o campo, e **um bordo é feição por definição**. A maquinaria de restrição existia
desde a Obra B. ⚠️ *Nunca lhe tinha sido dado o bordo.*

⭐ [`ph2d_mesh::boundary_feature_edges`] é a única feição **exacta** da cadeia: sem limiar,
sem curvatura, sem janela — cinco coeficientes a menos que a irmã dela.
⚠️ **A direcção é a tangente do LAÇO, não a da aresta** (a poligonal serrilha em torno da
curva verdadeira), pela **mesma** lei de média-de-eixo que a `feature_edges` já usava.

### §16.3 — O resultado

| peça | régua | sem | **com** |
|---|---|---|---|
| `sculpt_punctured` | enviesamento p50 · p99 · >60° | ⛔ `23,1°` · `84,8°` · `85` | ⭐⭐⭐ **`5,7°`** · `29,6°` · `1` |
| | aspecto p50 · p99 · >4× | ⛔ `1,68` · `11,60` · `93` | ⭐ **`1,11`** · `1,67` · `0` |
| | área spread · quads | ⛔ `11,40` · `468` | ⭐ **`1,24`** · `1890` |
| `sculpt_t002` | dobras contínuo · domínio | ⛔ `38` · `35` | ⭐ **`0` · `0`** |
| | anel das células | `(2,4)(4,1413)(5,5)(6,3)(8,2)` | ⭐⭐ **`(4, 1839)`** |
| | órfãs · células colapsadas | `8` · `6` | ⭐⭐⭐ **`0` · `0`** |
| | `χ` · bordo · não-manifold | ⛔ `−2` · `31` · `1` | ⭐ **`1`** · `14` · **`0`** |
| | enviesamento p50 · >60° | ⛔ `12,9°` · `22` | ⭐⭐⭐ **`7,1°`** · `1` |

*(a barra do oráculo: enviesamento p50 `4,8–7,1` · aspecto p50 `1,08–1,22` · >60°: `0–4`)*

⭐⭐⭐ **A `sculpt_punctured` passa a ficar ABAIXO da barra do oráculo**, e a `t002` — a peça
da queixa — **dentro** dela, com `χ = 1`, que é o valor **certo** para uma casca com um
buraco.

### §16.4 — Por que pode ficar ligado

⚠️ **Inerte em peça fechada por construção** (sem bordo, a lista sai vazia), e **medido**:
as outras **12** peças do corpus saem **byte-idênticas**.

⛔⛔ **E o que isto diz do processo é maior que o ganho:** as **duas** peças com bordo eram as
**duas piores do corpus inteiro**, há meses, e a causa era uma pergunta que nenhuma régua
fazia. *Um defeito que nenhuma sonda pergunta não aparece em nenhum relatório verde.*

## §17 — ⛔⛔⛔ A LEI DO REBORDO: construída, MEDIDA e REJEITADA — e ela reprecifica um «defeito»

O §15 mediu que o buraco da `t002` crescia **+30 %** em perímetro ao atravessar o F1. O
mecanismo estava à vista assim que se olhou para `relax_and_project`: **ele não tinha
tratamento de bordo nenhum.** Um vértice do rebordo era

1. alisado na direcção da média dos vizinhos — que são quase todos **interiores**, logo
   puxam-no para dentro da peça; e
2. projectado na **superfície** de referência, que perto de um rebordo aberto **continua
   para lá dele** — então ele desliza para onde a peça é mais larga.

### §17.1 — A lei, e ela CUMPRE o que promete

Um vértice de bordo é alisado **ao longo do rebordo** (só os vizinhos de bordo contam) e
projectado na **poligonal** da referência, nunca na superfície. E o peso do alisamento é
**zero** — ver a tabela em [`ph2d_remesh_iso::BORDER_LAMBDA`]:

| `λ` | `sculpt_punctured` (entrada `5,6463`) | `sculpt_t002` (entrada `0,6046`) |
|---|---|---|
| **sem lei nenhuma** | `5,7390` (⛔ +1,6 %) | `0,7841` (⛔ **+30 %**) |
| ⭐ **`0,0`** | ⭐ **`5,6463`** — exacto | ⭐ **`0,6046`** — exacto |
| `0,1` | `5,4124` (⛔ −4,1 %) | `0,5387` (⛔ −10,9 %) |
| `0,5` | `5,2566` (⛔ −6,9 %) | `0,5199` (⛔ −14,0 %) |

⭐ **Alisar uma poligonal encurta-a por construção** — é fluxo de encurtamento de curva. O
sinal do erro muda com a lei; a magnitude não desaparece sozinha.
⭐⭐ **E o rebordo continua a ser REAMOSTRADO** (`38 → 104` arestas na `punctured`) com o
perímetro exacto, porque as divisões caem **sobre** a poligonal.

### §17.1-bis — ⛔⛔⛔ E o PRODUTO fica pior, em todas as colunas

| `sculpt_t002` | perímetro | `χ` · não-manif. | enviesamento p50 · >60° | aspecto p99 · >4× |
|---|---|---|---|---|
| ⭐ **sem a lei** | `0,7841` (+30 %) | **`1` · `0`** | ⭐ **`7,1°` · `1`** | ⭐ **`1,72` · `0`** |
| ⛔ `λ = 0` (rebordo exacto) | ⭐ `0,6046` exacto | `1` · ⛔ `1` | ⛔ `9,9°` · `22` | ⛔ `3,25` · `13` |
| `λ = 0,1` | `0,5387` | ⛔ `0` · `0` | `6,2°` · `2` | `1,83` · `0` |
| `λ = 0,5` | `0,5199` | ⛔ `−2` · `0` | `8,5°` · `6` | `2,01` · `1` |

| `sculpt_punctured` | enviesamento p50 · >60° | aspecto p99 · >4× |
|---|---|---|
| ⭐ **sem a lei** | ⭐ **`5,7°` · `1`** | ⭐ **`1,67` · `0`** |
| ⛔ `λ = 0` | ⛔ **`24,3°` · `72`** | ⛔ **`14,49` · `87`** |

⭐⭐⭐ **O mecanismo, e ele reprecifica o «defeito» do §15.** O rebordo de um buraco esculpido
**SERRILHA** — viragem média `43,7°` na `punctured` e `53,6°` na `t002`, contra os `10°` de um
círculo de 36 lados. Preservá-lo **exactamente** preserva o serrilhado, e desde o §16 o bordo
é uma **linha de feição**: o campo cruzado passa a ser forçado a segui-lo, e os patches saem
do que essa zig-zag pede.

⇒ *A Laplaciana interior a arrastar o rebordo — os `+30 %` que o §15 chamou de defeito —
estava a **pagar** por uma coisa que ninguém tinha precificado: **um rebordo LISO**. Tirar o
defeito tirou o pagamento.*

⚠️ **A cura seguinte não é esta afinada:** é alisar o rebordo **e repor o comprimento dele**,
que é outra operação. ⚠️ E note-se que o §15 e o §16 são **do mesmo dia**: foi o §16 —
tornar o bordo uma feição — que **criou** o preço que o §17 mede. *Quem move o número que
tornava uma nota verdadeira tem de reconferir a nota* (CLAUDE.md §0.0).

### §17.2 — ⛔⛔ A primeira fixtura do gate era um TUBO, e as três mutações sobreviveram

O primeiro gate usava um cilindro sem tampas. O rebordo dele é um **círculo plano** sobre
uma superfície que passa exactamente por ele: alisar um polígono regular de 48 lados
encolhe-o `0,2 %` por passo, e a projecção de superfície devolve-o ao mesmo sítio. ⇒ **as
três leis eram indistinguíveis ali**, e o gate passava com qualquer uma.

A fixtura que contém o fenómeno é **um buraco de rebordo SERRILHADO numa superfície curva**
— e ela **prova que o contém**, com a régua da *viragem média do rebordo* (`148,7°` contra
os `10°` de um círculo de 36 lados). ⚠️ *É esta grandeza que decide se uma fixtura de bordo
tem o que medir.*

| mutação | resultado |
|---|---|
| `λ = 0,5` (o rebordo volta a ser alisado) | ⭐ **morre** (`2,3145 ⇒ 0,5831`, −75 %) |
| a lei do rebordo não existe | ⭐ **morre** (`2,3145 ⇒ 1,2531`, +53 %) |
| sem a projecção na poligonal | ⚠️ **sobrevive** na fixtura — vale `+0,36 %` na `t002` e `+0,09 %` na `punctured`, medido no corpus |
| sem o salto da projecção de superfície | ⛔ **inerte por PROVA** — o rebordo é subconjunto da superfície, logo o pé dele nela é ele próprio; a linha foi **removida** |

⚠️ **A barra do gate saiu de `1 %` para `0,1 %`**, porque a primeira era **cem vezes mais
frouxa que o que o código entrega** (`0,000 %` na fixtura, exacto nas duas peças reais).
*Uma barra escolhida à mão mede a folga de quem a escolheu.*

## §18 — ⭐⭐⭐ A 3.ª QUEIXA MEDIDA: a aspereza é DELE — e a extracção não tinha ACABAMENTO

A terceira queixa do artista (2026-08-25) era *«quanto mais densa a malha gerada, maiores as
irregularidades da superfície que deveria ser lisa»*. ⛔⛔ **Nenhuma régua desta cadeia a
media** — todas falam da FORMA dos quads (aspecto, enviesamento, área), e nenhuma da
**distância entre a peça que sai e a peça que ele fez**.

### §18.1 — As duas réguas novas

`chain_info` passa a imprimir:

- **FIDELIDADE** — a distância de cada vértice da saída à **escultura crua** e ao **F1**, em
  % da diagonal. ⚠️ *Contra a crua, nunca só contra a remalhada: medir contra o F1 responde
  «o extractor seguiu o F1?», que é outra pergunta.*
- **RUGOSIDADE** — a **dobra entre faces vizinhas**, nas três malhas (crua, F1, saída) com a
  contagem de faces ao lado. ⚠️ *É a normal que o sombreado mostra: uma peça pode estar a
  `0,1 %` de distância e parecer um diamante.* ⛔ E ela **depende da densidade** — sem a
  coluna das faces, comparar 44 000 triângulos com 2 000 quads é comparar duas réguas.

### §18.2 — ⭐⭐ A queixa é REAL, e o controlo diz de quem é

| p95 da dobra | escultura crua | depois do F1 | saída |
|---|---|---|---|
| `sculpt_t003` (dele) | **`10,9°`** · `411` arestas >30° | `18,6°` · `146` | `24,4°` · `118` |
| `sculpt_eared` (sintética) | **`1,9°`** · `145` | `3,7°` · `52` | `6,4°` · `15` |

⭐ A escultura dele chega **5,7× mais rugosa**, e a contagem de arestas ásperas **CAI** ao
longo da cadeia nas duas (`411 → 118`, `145 → 15`). ⇒ **a cadeia alisa; ela não enruga.**

E a varredura de densidade na peça dele confirma a segunda metade:

| quads | fidelidade p95 | rugosidade p50 | **arestas >30°** |
|---|---|---|---|
| `475` | `0,101` | `10,6°` | `93` |
| `1 942` | `0,106` | `5,4°` | `118` |
| `7 750` | `0,106` | `2,9°` | ⛔ **`143`** |

⭐⭐⭐ **A fidelidade não melhora com a densidade** — fica cravada em `0,10 %` num intervalo
de **16×** — e o número de **vincos visíveis SOBE**. ⚠️ E a hipótese óbvia (*«é o F1 a
facetar»*) foi **REFUTADA por interruptor**: pousar a saída exactamente sobre a escultura
leva a fidelidade a `0,000` e **não** baixa a rugosidade (`118 → 134` vincos). ⇒ *a aspereza
que ele vê é a da escultura dele; a grade fina RESOLVE-A.*

### §18.3 — ⭐⭐⭐ E a medição achou um buraco de PROCESSO: só um dos dois caminhos tinha acabamento

O caminho do `ph2d_quadfill::fill` corre **`SMOOTHING_ROUNDS = 6`** passos de Laplaciano
tangencial com reprojeção **desde sempre**. O caminho da **extracção** — o que o Enio smokou
a 24/08 e chamou *«o melhor resultado conseguido até agora»* — chamava
`ph2d_quadextract::extract(&cm, None)` e entregava a malha **crua**.
*Dois caminhos para o mesmo botão, e só um com acabamento.*

⚠️ **A 1.ª versão desta experiência escreveu a lei de novo em vez de a chamar** — Laplaciano
INTEIRO e reprojecção COM direcção — e a segunda metade é uma **recusa medida** do
`finish.rs`: com direcção, as dobras foram de `1` para `10` e a aresta máxima de `2,58×` para
`5,85×`. *Uma experiência que reescreve a lei mede outra coisa.*

**Medido nas 14 peças** (`chain_info`, `PH2D_OUT_RELAX=0` contra `6`):

| régua | resultado |
|---|---|
| distância à ESCULTURA p95 | ⭐ **`0,000 %` nas 14** (era `0,033`–`1,500`) |
| aspecto `>4×` | ⭐ **melhora ou empata em 13/14** — zero em 12 delas |
| enviesamento p99 | ⭐ **melhora em 12/14** |
| enviesamento `>60°` | ⭐ **melhora ou empata em 13/14** |

⛔ **As três regressões, nomeadas:** `sculpt_t003` sobe de `1` para `4` faces com `>4×` (o
p99 do aspecto **desce**, `1,97 → 1,65` — são três faces isoladas) · `sculpt_punctured` sobe
o enviesamento p99 de `29,6°` para `31,8°` (e o `>60°` desce de `1` para `0`) ·
`torus_64x32`, que já é patológico (10 arestas não-manifold, 133 faces `>4×`), sobe `79,0°`
para `80,6°` — e o `>4×` dele **desce** de `133` para `100`.

⚠️ **O acabamento NÃO alisa a superfície, e isso é o achado:** a rugosidade fica onde estava
(`14,2° ⇒ 14,3°`), porque a reprojecção repõe os vértices na peça. *Ele endireita a GRADE, não
a FORMA.*

⭐ **O preço, medido:** `425 ms` sobre `7 750` quads numa cadeia de `7,0 s` — **6 %**, na
densidade mais fina medida (melhor de 3: `6 979` contra `7 404 ms`).

### §18.4 — ⛔⛔ E o mesmo ficheiro tinha um SEGUNDO buraco, achado por acidente

O caminho da extracção **constrói o próprio campo cruzado** (`Dual::build(&work)`). Quando o
§16 ligou a restrição de bordo, ela foi escrita em `retopo_global.rs` — e este ficheiro ficou
**sem ela por meia hora**, com o smoke a dizer que estava tudo bem.

⚠️ *Dois caminhos que constroem o mesmo objecto precisam da mesma lei escrita duas vezes, ou
de uma porta só — e a porta ainda não existe.* Fica **nomeado** como dívida: hoje há dois
sítios a montar um `Dual` para o mesmo botão, e nada os obriga a concordar.

## §19 — ⭐⭐⭐ A 2.ª QUEIXA MEDIDA COM CONTROLO: a grade «não olhou» para o relevo dele

### §19.1 — ⛔⛔ E a minha régua de fidelidade era TAUTOLÓGICA

O §18 celebrou *«a saída assenta na escultura: `0,000 %` nas 14 peças»*. ⛔ **Isso é a
definição da operação:** desde o acabamento (§18.3) cada vértice é **pousado** na referência,
logo `saída → referência` dá zero por construção.

⚠️ **O aviso já estava escrito, com o número**, no doc do [`ph2d_quadfill::detail_lost`]: em
2026-08-21 uma régua desta família mediu `0,0000` na malha **destruída** contra `0,0015` na
boa — *a destruída pontuava melhor*. A coluna fica (ela ainda separa *«a saída vive sobre o
F1»* de *«vive sobre a escultura»*), mas passa a chamar-se **SOBRE-O-QUE** e a dizer-se
tautológica.

⭐ As réguas a sério **já existiam nesta árvore** e nenhum instrumento as chamava:
[`detail_lost`] (`referência → saída`) e [`follows_relief`].

### §19.2 — ⭐⭐ A régua do relevo, com o CONTROLO que a valida

`follows_relief` devolve o desvio 4-RoSy entre cada aresta da saída e a direcção principal de
curvatura, **ponderado pela anisotropia**, com **`22,5°` = «não olhou»**.

| peça | confiança | sem feição | com feição |
|---|---|---|---|
| `sphere_uv_96x144` (**controlo**) | **`0,00`** | `23,4°` | `23,4°` |
| `sculpt_wrinkled` | `0,07` | `12,2°` | `12,2°` (não dispara) |
| `sculpt_ridged` | `0,10` | `17,2°` | ⭐ **`13,7°`** |
| `sculpt_hooked` | `0,13` | `19,0°` | ⭐ **`14,5°`** |
| ⛔ **`sculpt_t003`** (dele) | **`0,56`** | **`21,7°`** | `20,7°` |
| ⛔ **`sculpt_t002`** (dele) | **`0,54`** | **`21,9°`** | `20,4°` |

⭐⭐⭐ **O controlo valida o nulo:** uma esfera lisa não tem direcção preferida (confiança
`0,00`) e lê `23,4°`, o valor de «não olhou». ⇒ E as peças do artista lêem **`21,7°`/`21,9°`
com confiança `0,54`–`0,56`**: *há muita direcção na peça dele, e a grade não segue quase
nenhuma.* **É a 2.ª queixa dele, com controlo.**

⚠️ **Os regimes de confiança são diferentes** (`0,56` contra `0,07`–`0,13`) e os números não
se comparam de frente. O que se compara é cada peça **contra o seu próprio nulo**.

⚠️ E o detector de feição **quase não dispara** nas esculturas dele: ganha `3,5°`–`4,5°` nas
peças do corpus e só `1,0°`–`1,5°` nas dele.

### §19.3 — ⛔⛔⛔ O PESO DO ALINHAMENTO NÃO É A ALAVANCA — varrido e recusado

O `ALIGN_WEIGHT` shipa a `0,03` desde 2026-08-22, e o número foi escolhido pelo **campo do
oráculo** — ⚠️ *não* por esta régua, que só chegou hoje. Varrido na `sculpt_t003`:

| peso | relevo | ⛔ **bordo (furos)** | enviesamento p50 · `>60°` |
|---|---|---|---|
| `0,0` | `22,1°` | `10` | `7,9°` · `3` |
| ⭐ **`0,03`** (shipa) | `21,7°` | ⭐ **`6`** | ⭐ **`6,6°` · `1`** |
| `0,10` | `20,8°` | ⛔ `24` | `8,3°` · `2` |
| `0,30` | `20,4°` | ⛔ `18` | `7,2°` · `3` |
| `1,00` | `20,4°` | ⛔ **`64`** | `7,8°` · `3` |

⭐⭐⭐ **A SATURAÇÃO é o achado:** de `0,30` para `1,00` o relevo **não se move** (`20,4°` nos
dois) e os furos **triplicam**. ⇒ *o campo já está tão alinhado quanto este peso o consegue
pôr, e o relevo continua em «não olhou».* O `0,03` que shipa é o melhor ponto **das duas**
outras colunas ao mesmo tempo.

⇒ **A perda do relevo não está no peso do campo.**

### §19.4 — ⛔⛔ E a FASE ZERO também não é a alavanca

O `ALPHA` é uma constante (`0,02`) e o produto **nunca o move**, qualquer que seja a densidade
pedida. Se o relevo morresse ali, nada a jusante o recuperaria. Varrido na `sculpt_t003`:

| `ALPHA` | faces do F1 | relevo | ⛔ bordo · não-manif. | enviesamento p50 · `>60°` |
|---|---|---|---|---|
| ⭐ **`0,020`** (shipa) | `4 850` | `21,7°` | ⭐ **`6` · `0`** | ⭐ **`6,6°` · `1`** |
| `0,014` | `9 988` | `21,0°` | ⛔ `10` · `2` | ⛔ `9,0°` · `9` |
| `0,010` | `19 504` | `20,1°` | ⛔ **`58`** · `0` | `7,7°` · `2` |

⇒ **`4×` a densidade da fase zero compra `1,6°` de relevo** — que continua em «não olhou» —
**e paga com `10×` os furos** (`6 → 58`). O caminho é monótono nas duas colunas: cada grau de
relevo custa mais buracos que o anterior.

⭐⭐⭐ **Duas hipóteses varridas e as duas recusadas ⇒ a família está fechada:** nem o peso do
campo nem a densidade do substrato recuperam o relevo (CLAUDE.md — *duas boas hipóteses a
falhar refutam a FAMÍLIA, não as duas*). O suspeito que sobra é a **quantização/layout**, que
é quem decide onde as linhas de grade de facto caem — e é a única fase entre um campo
comprovadamente alinhado e uma saída que não segue o relevo.

## §20 — ⭐⭐⭐ O RESGATE PELA FACE GÉMEA: o comentário nomeava a avaria e ninguém a curava

O ramo «sem parceira» da travessia tinha, desde 2026-08-25, um comentário que **nomeia
exactamente** a avaria:

> *"um nó de aresta nasce **uma vez por aresta**, no lado canónico, e fica registado com a
> FACE desse lado. Um traço que chegue ao mesmo ponto pela face **do outro lado** procura
> `(face, ponto, direcção)` com a *sua* face — e não acha nada. **O nó existe; a chave é que
> é de outra pessoa.**"*

⇒ A cura é perguntar **à outra pessoa**: transportar o ponto pela transição daquele lado
(`topo.xf[face][k]`) e procurar a chave na face gémea (`topo.twin[face][k]`), com a **mesma**
troca de direcção que o laço faz quando o sinal da área inverte.

### §20.1 — O resultado

| peça | antes | **depois** |
|---|---|---|
| `sculpt_t001` — bordo · `χ` | `4` · `1` | ⭐⭐ **`0` · `2`** (casca FECHADA) |
| órfãs | `2` | `1` |
| as outras **13** peças | — | ⭐ **byte-idênticas** |

⭐⭐⭐ **Uma única invocação fecha a peça.** O resgate corre em **1 das 14** peças, uma vez.

### §20.2 — ⛔ O que ele NÃO cura, medido

Na `sculpt_t003` as **4** órfãs também estão sobre uma aresta e o resgate salva **`0`**.
A coluna irmã explica: só **`1`** delas tem *nó lá*. ⇒ há **duas** avarias com o mesmo
sintoma, e esta cura só serve a primeira:

| avaria | sintoma | cura |
|---|---|---|
| o nó existe, a chave é da face gémea | `on_edge` **e** `node_exists` | ⭐ **esta** |
| ⛔ o nó é de VÉRTICE, e o dono é o LEQUE | `on_edge` e **`on_corner`** | por construir |

⭐⭐⭐ **E a sonda diz qual é, sem ambiguidade: `num CANTO: 4` de `4`.** As quatro órfãs que
sobram na `sculpt_t003` caem num **canto** do triângulo, não no meio de uma aresta.

⇒ Um ponto de grade sobre um vértice é um nó `Site::Vertex`, e as saídas dele estão
espalhadas pelo **leque** — cada uma emitida com a **sua** face
(`by_key: (face, u, v, dir) → saída`). ⚠️ *É a mesma classe de avaria com um **terceiro**
dono possível, e um resgate por um lado só nunca lá chega.* **A obra seguinte é percorrer o
leque**, com a transição acumulada de face em face.

### §20.3 — ⛔⛔ E o contador MENTIU antes de eu o ler direito

A primeira leitura dizia **`RESGATADAS: 0`** enquanto a peça fechava — uma contradição que
só se resolveu com uma sonda. ⚠️ **O argumento novo tinha entrado uma posição cedo demais na
lista do `println!`**, e o valor real (`1`) foi impresso na coluna do lado — que dizia
*«num triângulo de 1»* onde antes dizia *«de 1,112»*.

⭐ **Compilou**, porque o número de argumentos batia e o `Display` dos inteiros **ignora a
precisão `{:.3}`** em silêncio. *Uma coluna lida no slot errado lê-se ao contrário, e o
compilador não a vê.*

### §20.4 — Os gates

`on_edge_side` passou a receber **os três cantos** em vez do `Topo` inteiro, para ser
gateável. Quatro gates sobre a **convenção do lado** (`k` = a aresta do canto `k` para o
`k+1`, que é a que indexa `twin` e `xf`), com **duas provas de mutação**: rodar os índices
mata três deles; fazer o interior contar como lado `0` mata o quarto.

⚠️ **Nenhuma fixtura deste repositório alcança o resgate** — as duas de referência não têm
órfã nenhuma, e há asserção a pinar essa **inércia**. *O caso real vive fora da árvore, e
dizê-lo é mais barato que descobri-lo depois.*

## §21 — ⭐⭐⭐ O RESGATE PELO LEQUE, e a GUARDA que o `cube` escreveu

O §20.2 nomeou a segunda avaria: as órfãs que sobram caem num **canto**, e um ponto de grade
sobre um vértice é um nó `Site::Vertex` cujas saídas estão espalhadas pelo **leque** — cada
uma emitida com a **sua** face, e a chave é `(face, u, v, dir)`.

⇒ A cura percorre o leque (`fan_of`), usando `to_here` para a transição de carta a carta.

### §21.1 — ⛔⛔ E ela precisou de uma GUARDA, que a medição escreveu

Num leque **fechado**, ir de um canto a outro pela ordem do leque ou pelo outro lado dá
transições que diferem pela **holonomia**. Se ela não é a identidade — que é precisamente o
que uma **singularidade** é — as duas rotas apontam para saídas **diferentes** do mesmo
vértice, e escolher uma é um palpite.

⚠️ **Foi o `cube` que o disse.** Sem a guarda o resgate corria `2` vezes ali e as arestas de
bordo iam de `4` para **`6`**: *ligar ao par errado abre mais buracos do que deixar a órfã em
paz.* Com a guarda (`holonomia == identidade`, ou leque **aberto**, que não tem ambiguidade),
o `cube` volta a `4` e os resgates bons ficam.

### §21.2 — O corpus inteiro, com o binário CONGELADO

| peça | resgates | bordo antes | **bordo agora** |
|---|---|---|---|
| ⭐⭐ `sculpt_t001` | `1` | `4` | ⭐⭐ **`0`** (`χ = 2`, casca FECHADA, **`0` órfãs**) |
| ⭐ `sculpt_t003` | `1` | `6` | ⭐ **`4`** (órfãs `4 → 2`) |
| `sculpt_hooked` | `1` | `0` | `0` (neutro) |
| as outras **11** | `0` | — | ⭐ **byte-idênticas** |

### §21.3 — ⛔⛔ E uma «não-determinismo» que era MÉTODO

Duas corridas da mesma peça deram números diferentes e eu escrevi *«pode ser
não-determinismo»* — que num módulo cujo contrato é o determinismo (HR-5) é uma acusação
séria. ⭐ **Três corridas do binário parado saíram idênticas.**

⚠️ A causa: eu **reconstruí o binário enquanto a varredura de 14 peças ainda o invocava**.
Um `cargo build` substitui o ficheiro **no sítio**, e o laço de shell resolve o caminho a
cada iteração ⇒ as quatro primeiras peças mediram uma lei e as restantes outra. *Uma tabela
assim não é de um programa: é de todos os que existiram durante ela.*
⇒ A varredura acima corre contra uma **cópia congelada**.

## §22 — ⭐⭐⭐⭐ A `sculpt_004` do artista: o ALINHAMENTO AO RELEVO é que parte a orelha

**Smoke de 2026-08-26, e o veredito dele foi *«melhor resultado até agora e com grande salto
de qualidade»*** — com **uma** ponta má: a única cuja malha de entrada era complicada.

### §22.1 — A entrada está LIMPA, logo o defeito é NOSSO

`0` bordo, `0` não-manifold, `0` faces repetidas, `0` vértices coincidentes. ⇒ nada das curas
de entrada de hoje se aplica.

### §22.2 — ⛔⛔ E MALHA MAIS FINA NÃO CURA — medido em 6× de densidade

| escala | quads | dobras no contínuo | enviesamento p50 |
|---|---|---|---|
| `1,0` | `666` | **`142`** | `23,5°` |
| `0,6` | `1 827` | **`142`** | `24,7°` |
| `0,4` | `4 243` | **`142`** | `23,7°` |

⭐ As dobras **não se movem uma unidade** num intervalo de `6×`. *Arrastar o `Detail` não é a
resposta, e o artista precisava de o saber.* O layout dá só **16 patches** (a `t003` dá `31`),
um deles de **valência 12, `χ = −1` e não-disco** — a área complicada virou **um patch
monstro**, e a limpeza parou porque **piorava a topologia**.

### §22.3 — ⭐⭐⭐ A causa: o termo que segue o RELEVO

| peça | alinhado (`0,03`) | liso (`0,0`) |
|---|---|---|
| ⛔ **`sculpt_004`** | `23,5°` · `43` faces `>60°` · `14` bordo | ⭐⭐ **`7,8°` · `3` · `4`** |
| `sculpt_eared` | `7,8°` | ⭐ `5,1°` |
| `sculpt_hooked` | `6,6°` · `1` não-manifold | ⭐ `6,4°` · `0` |
| `sculpt_ridged` | p99 `31,4°` | ⭐ p99 `22,0°` |
| `sculpt_t002` | `6,7°` | ⭐ `5,5°` |
| ⭐ `sculpt_t003` | **`6,6°` · `4` bordo** | `7,9°` · `6` bordo |

⭐⭐ **O liso ganha em 5 de 6 e o alinhado em 1** — e **nenhum ganha sempre**.

⚠️ **E o termo não entrega o que foi acrescentado para entregar:** medido no mesmo dia com a
régua `follows_relief` (§19), ele compra **`0,4°`** (`22,1° → 21,7°`, ambos ao lado dos
`22,5°` que significam «não olhou»). *O `0,03` foi escolhido em Agosto pelo campo do oráculo,
quando esta régua não existia.*

### §22.4 — A cura: **as duas correm, e a medição escolhe**

⛔⛔ O irmão desta cadeia já tinha duas tentativas — mas **cai para a lisa só quando a
alinhada RECUSA**. ⚠️ *Uma rede que dispara na recusa não apanha o layout que **fecha e sai
péssimo***, e foi exactamente isso que a orelha mostrou.

**A ordem do critério: furos → faces `>60°` → enviesamento mediano.** Os furos vêm primeiro
porque são o que o artista **vê** — foi a queixa dele três vezes seguidas. *Uma ordem que
pusesse o enviesamento à frente escolheria a peça mais bonita com um buraco na ponta.*
Gate + **3 provas de mutação** (trocar a ordem, tirar o `>60`, afrouxar a comparação).

⚠️ **E o `aligned` do relatório tinha DOIS SENTIDOS**, o que fazia o log **mentir**: na
extracção ele carregava a *exactidão do arredondamento*, e o texto imprimia *«o alinhado nao
fechou»* sempre que uma translação saísse fraccionária. Hoje `aligned` diz **qual campo
produziu a malha**, e o novo `measured` distingue *«o liso saiu MELHOR»* (produto) de *«o
alinhado não fechou»* (defeito). ⛔ *Os dois liam-se igual.*

⭐ **O preço, medido:** uma passagem custa `4 475 ms` na `sculpt_004`, logo o botão passa de
~4,5 s a **~9 s**. ⛔ A saída barata (sair cedo quando a 1.ª tentativa já tem `0` furos e `0`
faces `>60°`) foi considerada e **não tomada**: ela perderia a melhoria da mediana onde ela
existe (`sculpt_eared`, `7,8° → 5,1°`). *É uma troca de qualidade por espera, e a escolha é do
dono do produto.*

### §22.5 — ⭐⭐ A TERCEIRA tentativa, e ela corre SÓ SE AINDA HÁ FURO

As linhas de feição por curvatura **custam bordo** na maioria das peças (`sculpt_t001`
`4 → 14`, `sculpt_t002` `14 → 18`, `sculpt_hooked` `0 → 4`) — é por isso que não são um
default. ⚠️ **Mas na `sculpt_004` do artista elas levam o bordo a `0`** (`4 → 0`, com o
enviesamento em `9,6°`).

| `sculpt_004` | bordo | faces `>60°` | enviesamento p50 |
|---|---|---|---|
| alinhado `0,03` | ⛔ `14` | ⛔ `43` | ⛔ `23,5°` |
| liso `0,0` | `4` | `3` | `7,8°` |
| ⭐ **feição ligada** | ⭐⭐ **`0`** | `5` | `9,6°` |

⇒ A condição **não é um limiar escolhido à mão**: é *«a chave da frente do critério ainda não
está satisfeita»*. Peça que já fecha não paga nada; peça que ainda tem furo paga mais uma
passagem — **exactamente onde a queixa do artista vive**.
⚠️ **E é segura por construção:** entra pelo mesmo `worse`, logo *só vence onde é melhor*.

### §22.6 — A verificação PONTA-A-PONTA, na cadeia do produto

`the_button_delivers_the_global_chain` (`#[ignore]` + GPU) corre o botão de verdade:

| | duas tentativas | **com a terceira** |
|---|---|---|
| saída | `1459` quads · `0` não-quads · `8` irregulares · **bordo `0`** | ⭐ **idêntica** |
| tempo | `8 609 ms` | `8 688 / 8 450 / 8 411 ms` |

⭐⭐ **A malha sai a mesma e o tempo não muda** — a terceira **não corre** numa peça que já
fecha, como o desenho promete.

⚠️ **E uma leitura de `11 319 ms` foi descartada como CARGA**, não como custo: a máquina
estava em `load 12–14`, e a regra da casa diz que nenhum relógio vale acima de `~5`
(CLAUDE.md §5). *Uma corrida só não separa ruído de custo — foram precisas três.*

## §23 — ⭐⭐⭐⭐ O NÍVEL SEGUINTE tem nome e número: **LOOPS FECHADOS**

**2026-08-26, depois do 3.º smoke.** O artista: *«resultado de alta qualidade. sem investir
nos edge loops, eu diria que chegamos ao pro. para onde vamos agora para alcançar o nível
deus?»* ⇒ por palavras dele, os **edge loops** são o que separa este módulo do nível seguinte
— e eles eram a **única** das quatro queixas **sem régua nenhuma**.

### §23.1 — A régua, e o que ela mede

[`loop_census`](../../../crates/ph2d-quadextract/examples/loop_census.rs): um loop atravessa
um vértice tomando a aresta **oposta**, o que só está definido num vértice de **valência 4**.
⇒ **um loop morre numa singularidade**, que é exactamente o que *«área de transição de
topologia»* quer dizer.

### §23.2 — ⛔ E a hipótese óbvia foi REFUTADA na primeira medição

**Os nossos loops não são curtos.** Normalizados pelo tamanho da malha eles são iguais ou
**mais longos** que os do oráculo (`sculpt_eared`: mediana **`344`** arestas em `2 013` quads
contra **`114`** em `4 658`). *A queixa não é sobre comprimento.*

### §23.3 — ⭐⭐⭐ O que salta é outra coluna, e o corpus inteiro concorda

| peça | oráculo: quads · loops · **fechados** | nosso: quads · loops · **fechados** |
|---|---|---|
| `cube` | `4816` · `94` · **`82`** | `3915` · `25` · ⛔ **`0`** |
| `sculpt_eared` | `4658` · `64` · **`44`** | `2013` · `12` · ⛔ **`0`** |
| `sculpt_hooked` | `4262` · `70` · **`46`** | `1420` · `35` · `8` |
| `sculpt_punctured` | `4266` · `143` · `17` | `1890` · `101` · ⛔ **`0`** |
| `sculpt_ridged` | `4109` · `38` · `10` | `2115` · `33` · ⭐ **`13`** |
| `sculpt_wrinkled` | `4696` · `74` · **`62`** | `2021` · `19` · `7` |
| `sphere_noisy` | `29865` · `2698` · `0` | `2104` · `16` · ⭐ `4` |
| `sphere_shuffled` | `4428` · `67` · **`55`** | `2078` · `12` · ⛔ **`0`** |
| `sphere_uv_96x144` | `3352` · `38` · **`26`** | `2152` · `12` · ⛔ **`0`** |
| `torus_64x32` | `5538` · `181` · **`181`** | `807` · `102` · `24` |

⭐⭐⭐ **Em 6 de 10 peças temos ZERO loops fechados** onde o oráculo tem `26`–`82`.
⚠️ **Duas excepções, e elas são a metade honesta:** no `sculpt_ridged` fazemos **mais** (`13`
contra `10`), e no `sphere_noisy` a corrida do oráculo é degenerada (`29 865` quads, mediana
`7`, `2 698` loops, **`0`** fechados) — *a barra não é o oráculo em toda peça; é o oráculo
quando ele funciona.*

⇒ **Nós fazemos poucos loops LONGOS e ABERTOS; ele faz muitos MÉDIOS e FECHADOS.** Um loop
fechado é um **anel que dá a volta na forma** — é o que um modelador quer dizer com «edge
loop» à volta de um braço, de um chifre, de um olho. Os nossos correm e **morrem numa
singularidade** em vez de voltarem.

### §23.4 — ⭐⭐ E a direcção CONVERGE com o §19, por dois caminhos independentes

Um anel fecha quando as singularidades estão arranjadas para que um circuito de vértices de
valência 4 volte a si próprio. **Isso é decidido pelo LAYOUT/QUANTIZAÇÃO** — a mesma fase que
o §19.4 nomeou como o único suspeito que sobra depois de o campo e o substrato terem sido
ilibados por varredura.

⇒ *Duas perguntas diferentes — «por que a grade não segue o relevo?» e «por que os anéis não
fecham?» — apontam para a mesma fase.* É por aí que se vai.

### §23.5 — ⛔⛔ DUAS hipóteses sobre as singularidades, as duas REFUTADAS

| hipótese | previsão | ⛔ medido |
|---|---|---|
| «os anéis dele fecham porque as singularidades estão **AGRUPADAS**» | oráculo agrupado, nós espalhados | **oráculo `0%` agrupadas em TODAS as peças**; nós `0–33%` |
| «…porque o **ARRANJO** delas é o dos cantos de um cubo» | oráculo ≈ `70°`, nós irregular | **o NOSSO é o quase-cúbico** (`61 64 65 66 69 69 69 77…`); o dele espalha de `44°` a `94°` |

⚠️ *Duas boas hipóteses a falhar refutam a FAMÍLIA:* **a colocação das singularidades não
explica os anéis fechados.** Na `sphere_uv_96x144` as duas malhas têm **8** irregulares numa
esfera lisa, com distâncias parecidas, e ele fecha `26` anéis e nós `0`.

### §23.6 — ⭐⭐⭐⭐ A causa: as nossas linhas de grade **ESPIRALAM**

⭐ **A contagem bate exactamente e é ela que aponta.** Num quad-mesh de esfera cada vértice de
valência 3 termina **três** pontas de loop; com `8` deles são `24` pontas ⇒ **`12` loops
abertos, obrigatórios pela topologia**. E é o que as duas têm: nós `12` **no total**, o
oráculo `12` abertos **mais `26` fechados**.

⇒ A pergunta certa não é *«por que os nossos não fecham?»* — os `12` **não podem** fechar. É
**«por que esses `12` cobrem a peça INTEIRA?»**

**A régua: quantas voltas um loop dá** (comprimento ÷ circunferência da grade, `≈ 2·√quads`):

| peça | oráculo p50 · p90 | nosso p50 · p90 |
|---|---|---|
| `sphere_uv_96x144` | ⭐ **`1,0×` · `1,0×`** | ⛔ **`2,8×` · `10,1×`** |
| `sculpt_eared` | ⭐ **`0,8×` · `0,9×`** | ⛔ **`3,8×` · `9,0×`** |
| `sculpt_wrinkled` | ⭐ **`0,8×` · `0,8×`** | `0,9×` · ⛔ **`8,2×`** |

⭐⭐⭐ **O anel dele dá EXACTAMENTE uma volta; o nosso dá três a quatro na mediana e oito a
dez no p90.** Uma linha que dá quatro voltas sem fechar **volta ao pé de onde partiu,
deslocada de uma linha, e recomeça** — é um **espiral**.

### §23.7 — O que isso nomeia, e é a obra

Um anel fecha quando a **translação inteira acumulada ao dar a volta** é zero na direcção
transversa. Se não é, a linha reentra deslocada e espirala. ⇒ **é holonomia do mapa**, e quem
a decide é a **quantização** (`ph2d-quantize` + as costuras do `ph2d-gridmap`).

⇒ **O alvo é um número:** `VOLTAS p50 → 1,0×`. ⚠️ E ele é *independente* das réguas de forma
que já batem a barra do oráculo (aspecto e enviesamento) — *uma malha pode ter quads
perfeitos e espiralar*, que é exactamente o que o artista viu quando chamou o resultado de
«pro» **excluindo** os loops.

⭐⭐ **E a direcção CONVERGE pela terceira vez** com o §19.4 e o §23.4: campo e substrato
ilibados por varredura, e agora a holonomia do mapa. *Três perguntas independentes, a mesma
fase.*

### §23.8 — ⛔⛔⛔ E NÃO é grade grossa: a DENSIDADE foi varrida e o espiral PIORA

A hipótese barata — *«o oráculo tem mais quads, por isso sobram bandas que não topam
singularidade»* — foi testada na `sphere_uv_96x144`:

| escala | quads | loops | **fechados** | voltas p50 |
|---|---|---|---|---|
| `1,0` | `2 152` | `12` | ⛔ **`0`** | `2,8×` |
| `0,7` | `4 431` | `12` | ⛔ **`0`** | ⛔ **`4,2×`** |

⭐⭐⭐ **Com `4 431` quads — mais que os `3 352` do oráculo — continuam a ser exactamente os
mesmos `12` loops, zero fechados, e o espiral fica PIOR.** ⇒ *a estrutura é que está errada; a
densidade só a amplia.*

### §23.9 — A família a que este defeito pertence já tem nome nesta linha

O `CLAUDE.md` §5 regista, de 24/08: ⭐⭐ *«Duas grandezas estavam a ser lidas como uma: o G5
torna a costura **INTEIRA** (`shift_frac_max = 0`, e há gate) e **não** a torna **FECHADA**»*.

⇒ **Uma costura pode ser perfeitamente inteira e ainda assim desalinhar as linhas por uma
célula cheia** — e um desalinhamento de uma célula por travessia é **exactamente** um espiral.
⚠️ *Um teste de integralidade é cego a isso por construção:* `1,0` é tão inteiro como `0,0`.

⇒ A obra é uma **régua de ALINHAMENTO da grade através da costura** (quantas células a linha
salta ao atravessar), que hoje não existe — e depois a restrição que a leva a zero na
quantização. *A régua primeiro: sem ela, qualquer cura é um palpite, e esta linha já pagou
quatro vezes por isso hoje.*

### §23.10 — ⭐⭐ O experimento que parte a cadeia ao meio: o espiral vem do MAPA

[`fixture_extract`](../../../crates/ph2d-quadextract/examples/fixture_extract.rs) extrai um
**mapa de referência** (`docs/3D/cleanroom/fixtures/`, verificado a `3,55e-15`) sobre a
**nossa** malha, e compara-se com a nossa cadeia inteira na mesma peça:

| `sculpt_hooked` | loops | **fechados** | voltas p50 · p90 |
|---|---|---|---|
| mapa de **referência** | `32` | **`0`** | `0,1×` · `4,0×` |
| **nosso** mapa | `35` | **`8`** | `0,8×` · `2,5×` |

⛔ **Ele NÃO isola o mapa como se esperava, e dizer o contrário seria ler o resultado que se
queria:** a fixtura é um mapa **verificado**, não um **bom layout** — ela existe para provar a
correcção da extracção e não tem razão nenhuma para ter bons anéis. *(De facto o nosso mapa
sai melhor nela.)*

⭐ **Mas ela prova uma coisa:** trocar **apenas** o mapa muda a estrutura dos anéis de `0` para
`8` fechados na mesma peça. ⇒ **a extracção não impõe o espiral; o mapa é que o traz.** A
morada estreita-se para o **G3/G5**, que era a previsão — agora com prova.

⚠️ **A régua da §23.9 foi construída — e refutou-se a si própria: ver a §23.11.** Ela pedia quantas células a linha salta ao
atravessar cada costura. *Este experimento diz em que crate ela tem de viver; não a substitui.*

### §23.11 — ⭐ A régua da §23.9 foi CONSTRUÍDA — e ela refuta as duas leituras que a motivavam

[`ph2d_gridmap::measure_alignment`](../../../crates/ph2d-gridmap/src/align.rs), 13 gates.

#### ⛔ A régua que a §23.9 pedia não existe, e a razão é dupla

A §23.9 pediu *«quantas células a linha salta ao atravessar cada costura»*. Essa
grandeza **não é medível**, e não por falta de instrumento:

1. **É de calibre.** Somar uma constante ao `(u, v)` de um patch muda a translação de
   *todas* as costuras que lhe tocam sem mudar coisa nenhuma na peça
   ([`gauge`](../../../crates/ph2d-gridmap/src/gauge.rs) já o provava). *Uma coluna por
   costura mediria a escolha de origem de cada carta.*
2. **Depois da soldadura ela é zero por construção.** As duas metades satisfazem
   `z_b = R^k·z_a + t` **exactamente** — é uma substituição, não um acordo — e o G5 põe
   `t` em inteiros. ⇒ as isolinhas inteiras de um lado encontram sempre as do outro.

⇒ *A régua que aquela secção pedia só podia imprimir `0`.* O que sobrevive ao calibre é
a **holonomia de um ciclo**: percorrer uma volta do grafo de patches e compor as
transições devolve `(R^K, T)`, e com `K = 0` o `T` é invariante.

#### ⛔⛔ Primeira leitura REFUTADA pelo padrão: `T ≠ 0` não é o espiral

A 1.ª redacção contou ciclos com `T ≠ 0`. O corpus saiu **bimodal** — `0` ou enorme
(`51`, `75`, `103`, `130`) — e um parafuso de geometria daria um contínuo.

⭐ `T` tem duas metades e só uma é defeito. Num tubo a transição é `z ↦ z + (N, 0)`:
o `N` é o **comprimento** da volta e os anéis fecham na mesma. Num parafuso é `(N, 1)`,
e o anel que sai de `v = j` volta a `v = j+1`. *O `N` é grande e inofensivo; o `1` é
pequeno e é o defeito* — uma norma `∞` lê o primeiro e enterra o segundo. A leitura
certa é **por família**: `a = 0` ⇒ as linhas de `u` fecham; `b = 0` ⇒ as de `v` fecham.

#### ⛔⛔ Segunda leitura REFUTADA pelo toro: contar ciclos depende da ÁRVORE

Com a base `{(N,0), (0,M)}` um toro limpo não tem ciclo a espiralar; com
`{(N,0), (N,M)}` — a mesma peça, outra árvore de expansão — tem um. *A base é minha; a
peça não.* E um ciclo diagonal não é defeito: ele só diz que **nenhuma linha de grade dá
aquela volta**. Gate: `a_diagonal_cycle_does_not_move_the_lattice`.

⇒ A grandeza da peça é o **subgrupo `L ⊂ ℤ²`** gerado por todas as holonomias planas, e
a pergunta lê-se nele: `L ∩ ({0}×ℤ) ≠ 0` ⇒ as linhas de `u` **podem** fechar. O
parafuso gera `L = ℤ·(N,1)`, cujas duas intersecções são triviais — é a assinatura
exacta do espiral, e é o que o gate `the_screw_closes_no_family` fixa.

#### ⛔⛔⛔ E o RETICULADO também não explica o espiral — medido em 8 peças

| peça | ordem de `L` | período (u · v) | loops (fechados) | voltas p50 |
|---|---|---|---|---|
| `sculpt_eared` | 1 | **`0` · `0`** | `12` (**`0`**) | `3,8×` |
| `sculpt_wrinkled` | 1 | **`0` · `0`** | `19` (**`7`**) | `0,9×` |
| `sphere_uv_96x144` | 1 | **`0` · `0`** | `12` (`0`) | `2,8×` |
| `sculpt_ridged` | 2 | `928` | `33` (`13`) | `0,8×` |
| `sculpt_hooked` | 2 | `1 700` | `35` (`8`) | `0,8×` |
| `sphere_shuffled` | 2 | `2 522` | `12` (`0`) | `1,8×` |
| `torus_64x32` | 2 | **`2`** | `102` (`24`) | **`0,1×`** |
| `cube` | 2 | **`2`** | `25` (**`0`**) | `2,5×` |

⛔ **Dois contra-exemplos limpos matam a correlação:** o `cube` tem o **melhor** período
possível (`2`) e **zero** anéis fechados; a `wrinkled` tem o **pior** (`0`, o parafuso
puro) e tem **sete**. *O período do reticulado não prevê o censo de anéis.*

⚠️ **Por que ele falha, e é a lição:** `L` agrega **todas** as voltas, incluindo as que
linha de grade nenhuma percorre. Uma única volta esquisita — uma ponte que abre um anel,
uma diagonal — envenena o subgrupo inteiro e manda o período a `2 522`. *Um invariante
que soma sobre voltas que ninguém percorre responde a uma pergunta que ninguém fez.*

#### ⭐ O que FICA, e é pouco mas é sólido

- **A régua existe e é exacta**, com o calibre e a árvore provados por gate. A
  distinção **comprimento vs deriva** é nova e é o que matou a 1.ª leitura.
- ⭐ **Só o toro tem ciclos em que UMA família fecha** (`4` deles). Todas as sete peças
  de topologia de esfera têm **zero** — e o toro é, de longe, a melhor no censo
  (`0,1×` de voltas, `24` anéis de `102`). ⚠️ **É `n = 1`**, logo é uma pista e não uma
  prova; mas é a única coluna desta jornada em que a peça boa se separa das outras.
- ⛔⛔ **A morada estreita-se outra vez, e agora CONTRA os ciclos planos:** por peça há
  `12` a `31` ciclos planos e **`12` a `32` que RODAM** — e estes últimos ficaram todos
  de fora desta régua, por o `T` deles ser de calibre. ⚠️ *Numa esfera as voltas que
  interessam contêm cones quase sempre*, e uma linha que dá a volta a um cone de `90°`
  volta trocada de família — ela pode fechar ao fim de **quatro** voltas, e nenhuma
  régua desta linha sabe ler isso. **É aí que a próxima janela tem de olhar**, e o
  invariante de um ciclo que roda é o **ponto fixo**, não a translação.

⚠️ **Nada disto tocou no produto:** a régua é um instrumento, o `chain_info` imprime-a, e
a saída do botão `Quad Retopology` é byte-idêntica.

### §23.12 — ⛔⛔⛔ Os CONES a meia célula: a 3.ª coluna, o 3.º contra-exemplo, e o que ela achou

Os `12`–`32` ciclos que **rodam** por peça ficaram fora da §23.11 por o `T` deles ser de
calibre. O invariante deles é o **ponto fixo** de `w ↦ R^K·w + T`, que é o sítio onde o
cone está na carta — e a distância dele à grade é invariante (mudar a origem da carta
desloca-o de um inteiro).

⭐⭐⭐ **E o denominador é `2`, sempre:** `det(I − R^K)` vale `2` num quarto de volta e `4`
em meia. ⇒ *um ponto fixo genérico cai em MEIO-INTEIRO* — a mesma meia célula que o
[`weld_flat`](../../../crates/ph2d-gridmap/src/weld_flat.rs) já nomeava ao contar
`det = 2` nos pivôs. ⚠️ Um vértice de valência `3` **é** um vértice de grade: se o mapa o
quer a meia célula, a extracção encaixa — e meia célula de encaixe é um rasgo.

| peça | cones a **meia célula** | de | loops (fechados) | voltas p50 |
|---|---|---|---|---|
| `sculpt_eared` | **`4`** | `17` | `12` (`0`) | `3,8×` |
| `sphere_shuffled` | **`6`** | `16` | `12` (`0`) | `1,8×` |
| `cube` | **`4`** | `32` | `25` (`0`) | `2,5×` |
| `sculpt_ridged` | **`3`** | `23` | `33` (**`13`**) | `0,8×` |
| `sculpt_wrinkled` | **`2`** | `15` | `19` (**`7`**) | `0,9×` |
| `sphere_uv_96x144` | **`0`** | `17` | `12` (**`0`**) | `2,8×` |
| `sculpt_hooked` | `0` | `32` | `35` (`8`) | `0,8×` |
| `torus_64x32` | `0` | `12` | `102` (`24`) | `0,1×` |

⛔ **Contra-exemplo nos dois sentidos:** a `sphere_uv` tem **zero** meias células e **zero**
anéis fechados; a `ridged` tem **três** e tem **treze**. *A meia célula não prevê o espiral.*

#### ⭐ Mas a coluna não sai vazia — ela achou um defeito que nunca foi medido

**`19` de `152`** ciclos que rodam, no corpus inteiro, põem um cone a **meia célula** — e
a extracção **tem** de os encaixar num inteiro. ⚠️ *É um rasgo por construção, e nenhum
gate desta cadeia o via*: os gates de integralidade olham as **translações**
(`shift_frac_max`, que é `0`), e um ponto fixo a `½` passa em todos eles. **O sintoma
dele é o buraco, não o espiral** — e é a lista onde a próxima caça aos furos começa.

#### ⛔⛔⛔ E o veredito das três colunas juntas corrige a §23.10

Três grandezas invariantes de calibre, todas medidas em 8 peças:

| coluna | o que diz do nosso mapa | prevê o espiral? |
|---|---|---|
| **deriva** por ciclo plano | `0` na mediana em todas as peças | ⛔ não |
| **reticulado** `L` | `0` holonomias fraccionárias em **todas** | ⛔ não (o `cube` refuta) |
| **ponto fixo** dos cones | inteiro em `133` de `152` | ⛔ não (a `sphere_uv` refuta) |

⇒ ⭐⭐⭐ **A estrutura INTEIRA do mapa está essencialmente correcta**, e isso corrige o que
a §23.10 deixou implícito. Ela concluiu, com prova, que *«o espiral vem do mapa»* — e
estava certa; mas a morada que se leu a seguir (**G3/G5**, o andar das transições) é a
**errada**. ⚠️ *O mapa traz o espiral pela GEOMETRIA — por onde as isolinhas correm —, não
pela topologia das suas transições.* As transições estão inteiras, os cones estão na
grade, e as voltas não derivam.

⇒ A próxima janela **não** deve mexer no G3/G5. A morada que sobra é onde as
singularidades são **colocadas** — o F2/F3 — e essa é a única fase da cadeia que nenhuma
régua desta jornada tocou.

### §23.13 — ⭐⭐⭐ A FASE QUE FALTA TEM NOME: o F4 não está na rota que shipa

Depois de sete hipóteses refutadas, a primeira que **sobrevive** — e ela não é uma
hipótese sobre o algoritmo, é sobre a **lista de fases**.

#### O que a rota do produto atravessa

[`sculpt3d_history_retopo_extract.rs`](../../../shells/desktop/src/sculpt3d_history_retopo_extract.rs)
(o default desde 25/08) vai:

```text
F1 remesh-iso → F2 crossfield → F3 trace → G1 corte → G2 pente → G3/G5 mapa → extracção
```

⛔⛔⛔ **`ph2d-quantize` (o F4) não aparece.** A rota do *fill*
([`…_retopo_global.rs`](../../../shells/desktop/src/sculpt3d_history_retopo_global.rs))
chama-o — `quantize_within(&l, Budget::new(…))` — e é a **única** diferença de fase entre
as duas antes do preenchimento.

⚠️ **E o `lib.rs` da própria `ph2d-gridmap` descreve a intenção contrária:** *«os inteiros
já existem: o **F4** decide quantos segmentos leva cada arco ⇒ o que falta desta fase é
LINEAR»*. *Uma intenção escrita no doc de um módulo não é um passo do caminho* — nada
alimenta o mapa com aquelas contagens, e ele resolve livre.

#### ⭐ O A/B, com UMA variável

[`fill_chain`](../../../crates/ph2d-quadfill/examples/fill_chain.rs) repete a montante do
`chain_info` passo a passo — mesmo F1 (`ALPHA`), mesmo `h` (aresta mediana pós-F1), mesmo
campo (`solve_miq`, **não** o alinhado). ⇒ só o F4 difere.

| peça | rota | quads | loops (**fechados**) | ⭐ **voltas p50** · p90 |
|---|---|---|---|---|
| `sculpt_eared` | **sem F4** | `2 013` | `12` (`0`) | **`3,8×`** · `9,0×` |
| `sculpt_eared` | **com F4** | `4 251` | `36` (**`2`**) | **`1,0×`** · `5,3×` |
| `sculpt_hooked` | sem F4 | `1 420` | `35` (`8`) | **`0,8×`** · `2,5×` |
| `sculpt_hooked` | com F4 | `1 592` | `66` (`4`) | **`0,3×`** · **`1,2×`** |
| `sphere_uv_96x144` | sem F4 | `2 152` | `12` (`0`) | **`2,8×`** · `10,1×` |
| `sphere_uv_96x144` | com F4 | `2 229` | `26` (`0`) | **`0,9×`** · `6,2×` |

⭐⭐⭐ **As voltas melhoram nas TRÊS peças**, na mediana e no `p90`. E a contagem de anéis
**sobe** nas três (`12→36`, `35→66`, `12→26`): a malha ganha estrutura para além das
separatrizes obrigatórias.

⚠️ **A densidade é um confundidor, e a medição fecha-o de dois lados:** as duas peças de
densidade quase igual (`hooked` a `1,12×`, `sphere_uv` a `1,04×`) são as que mais
melhoram (`0,8→0,3` e `2,8→0,9`); e a **§23.8** já mediu que **mais** quads faz o espiral
**piorar** nesta cadeia. *Um confundidor que empurra ao contrário do efeito não o explica.*

⛔ **E o que NÃO melhorou, dito por inteiro:** na `hooked` os anéis fechados **caem** de
`8` para `4` (de `23 %` para `6 %` do total). *A régua das voltas e a contagem de anéis
discordam nessa peça, e nenhuma das duas é a errada* — a `hooked` com F4 tem muitos anéis
curtos, poucos deles fechados.

#### ⛔ E a resposta NÃO é trocar de rota

A rota do *fill* é geometricamente **muito pior** — enviesamento `27°` contra os `6,8°` da
extracção (§4 do `PLAN.md`), que é a razão de a extracção ter passado a default. ⇒ *não é
«voltar atrás»; é trazer o F4 para dentro da cadeia da extracção.*

⭐ **A forma da obra já existe nesta crate:** o F4 diz quantos segmentos leva cada arco, e
[`arc_marks`](../../../crates/ph2d-gridmap/src/marks.rs) já sabe ler onde as isolinhas
inteiras cruzam um arco. A restrição *«este arco leva `n` isolinhas»* é **linear** nas
mesmas incógnitas que a costura já elimina — é a família do
[`SPEC_restricoes_por_eliminacao`](../../cleanroom/SPEC_restricoes_por_eliminacao.md), com
a costura como precedente medido.

⚠️ **A ordem importa:** a costura entrou por eliminação e fechou a peça; a contagem de arco
é a mesma maquinaria com outro sujeito. *Construí-la como termo de energia repetiria,
um nível acima, o defeito que a Obra A curou.*

### §23.14 — ⭐⭐⭐ O MECANISMO: as separatrizes do F3 **não são linhas de grade** do nosso mapa

A §23.13 disse *que fase falta*. Esta diz **o que exactamente ela imporia**, e o número é
total.

[`ph2d_gridmap::measure_arc_quantization`](../../../crates/ph2d-gridmap/src/align.rs)
lê, para cada arco do layout, o deslocamento entre os dois cantos dele no plano — e esse
deslocamento diz **duas** coisas de uma vez:

| metade | o que é | o que devia ser |
|---|---|---|
| **ao longo** | o maior componente | ⭐ a contagem que o F4 pede |
| **atravessado** | o menor | ⛔ **zero** — senão o arco **não é uma isolinha** |

| peça | arcos | ⭐ concordam | discordância p50 · max · **soma** | ⛔⛔ **não são isolinhas** | atravessam p50 · max |
|---|---|---|---|---|---|
| `sculpt_eared` | `47` | **`0`** (0 %) | `1,51` · `29,0` · **`192`** | **`47`** | **`0,96`** · `3,27` |
| `sculpt_hooked` | `85` | `1` (1 %) | `1,13` · `19,6` · **`203`** | **`85`** | `0,61` · `3,46` |
| `sculpt_ridged` | `56` | `3` (5 %) | `1,05` · `10,0` · `90` | **`54`** | `0,69` · `4,00` |
| `sculpt_wrinkled` | `46` | **`0`** (0 %) | `0,86` · `11,6` · `77` | **`46`** | `0,89` · `7,24` |
| `sphere_uv_96x144` | `44` | **`0`** (0 %) | `0,85` · `11,9` · `92` | **`44`** | `0,75` · `3,63` |
| `sphere_shuffled` | `48` | **`0`** (0 %) | `0,98` · `13,4` · `109` | **`48`** | `0,70` · `3,36` |
| `cube` | `91` | **`0`** (0 %) | `1,00` · `17,2` · `199` | **`90`** | `0,70` · `8,81` |
| `torus_64x32` | — | ⛔ o F4 **recusa** este layout (`Infeasible`) | | | |

⭐⭐⭐ **A leitura, e ela é o mecanismo do espiral:** *praticamente NENHUM arco do layout é
uma linha de grade do mapa.* Eles atravessam **~1 célula inteira** na mediana. E uma
separatriz é, por definição, onde a grade **termina** num cone — se ela não é uma
isolinha, as linhas de grade **não terminam ali**: passam ao lado e continuam. *É a
descrição exacta de um anel que não fecha.*

#### ⚠️ E isto NÃO é um defeito de código — é a arquitectura, dita por inteiro

O nosso G3 resolve um mapa **global alinhado ao campo**, e o layout do F3 é usado **só
para CORTAR**. ⇒ *nada nunca pediu que os arcos fossem isolinhas*, e eles não são. A
`ph2d-gridmap` e a `ph2d-trace` são **duas respostas independentes à mesma pergunta**, e
elas discordam em **todos** os arcos das sete peças.

⛔ **O que a tabela não prova:** que fechar o desvio cure o espiral. A coluna
«atravessam» **não** ordena as peças como as voltas (`wrinkled` atravessa `0,89` e faz
`0,9×`; a `sphere_uv` atravessa `0,75` e faz `2,8×`). *A evidência de que ajuda é o A/B da
§23.13, não esta correlação* — e dizer o contrário seria ler o número que se queria.

#### ⭐ A obra, com as duas metades separadas

1. ⭐⭐⭐ **«este arco é uma isolinha»** (atravessado `= 0`) — **não depende do F4 nenhum** e
   é a metade fundamental. Uma restrição linear por arco nas mesmas incógnitas que a
   costura já elimina.
2. ⭐ **«este arco leva `n` arestas»** (ao longo `= n`) — essa sim precisa do F4.

⚠️ **A ordem é essa**, e pela mesma razão que a costura veio antes da feição: a segunda é
a mesma maquinaria com um sujeito a mais, e herdaria qualquer defeito da primeira.
⛔ Nenhuma das duas entra como **termo de energia** — o precedente medido é a Obra A, onde
penalizar em vez de eliminar deu `NaN` e `6,4e17`.

### §23.15 — ⛔⛔ Duas curas BARATAS medidas e mortas, e o endereço fica em G3

A §23.14 nomeou a obra. Antes de a construir, duas versões baratas foram tentadas — e
**as duas caíram**, cada uma com o mecanismo à vista. *É por isso que se mede primeiro.*

#### ⛔ Cura barata nº 1 — «pregar também os cantos do layout»

Hoje só as singularidades do **campo** vão ao G5 com pedido de ponto inteiro. Pregar
**também** os cantos do layout (`PH2D_PIN_CORNERS=1` no `chain_info`) parecia a mudança
mais pequena com efeito possível.

⛔ **Saída BYTE-IDÊNTICA nas três peças** — mesmos quads, mesmas órfãs, mesmo `X`, mesma
tabela de arcos. E o contador já dizia porquê: *«`8` singularidades (de `32`)»*.

⚠️ **O mecanismo está no código:** as variáveis inteiras do caminho soldado são **as
LIVRES do sistema reduzido** ([`weld_round`](../../../crates/ph2d-gridmap/src/weld_round.rs)),
e uma classe livre é a que carrega um **fecho**. Um canto **regular** não tem fecho: ele é
escrito por **substituição** a partir dos vizinhos. ⇒ *ele não tem valor próprio para
pregar.* A lista de pedidos é filtrada pela estrutura da soldadura, não honrada como
vem — e nada nela é um defeito.

⭐ *A régua da §23.14 pagou-se aqui:* sem ela, «pregar os cantos» teria parecido uma wave
razoável.

#### ⛔⛔ Cura barata nº 2 — «é o arredondamento que desalinha»

A escada gulosa move cada cone no máximo **meia célula**; um arco tem dois extremos ⇒ o
G5 sozinho não pode desalinhar mais de **uma**. E o desvio medido é `0,61`–`0,96`. *Os
dois números são compatíveis* — o que torna a dedução tentadora e a medição obrigatória.

| peça | ⭐ **antes** (G3 contínuo) | depois (G5) | o que o G5 acrescenta |
|---|---|---|---|
| `sculpt_eared` | `47/47` · **`0,90`** | `47/47` · `0,96` | `+0,06` |
| `sphere_uv_96x144` | `44/44` · **`0,77`** | `44/44` · `0,75` | **`−0,02`** |
| `sculpt_hooked` | `84/85` · **`0,62`** | `85/85` · `0,61` | **`−0,01`** |
| `sculpt_ridged` | `56/56` · **`0,65`** | `54/56` · `0,69` | `+0,04` |

⛔⛔⛔ **O desvio já está TODO no contínuo**, e em duas peças o arredondamento até o
**melhora**. ⇒ *a cura não é uma regra de escolha na escada gulosa* — e construí-la teria
sido uma wave inteira sobre a fase errada.

#### ⇒ O endereço final desta jornada

**G3, o sistema contínuo.** Ele minimiza o desalinhamento do **gradiente** contra o campo,
e mais nada. O layout do F3 entra **só** para cortar. A restrição *«este arco é uma
isolinha»* tem de entrar **ali**, como equação — e por **eliminação**, nunca por peso
([`SPEC_restricoes_por_eliminacao`](../../cleanroom/SPEC_restricoes_por_eliminacao.md), a
Obra A como precedente medido).

⭐ E a forma dela é conhecida: com as classes livres a serem os cones, um arco entre dois
cones dá **uma equação escalar** — a componente atravessada da diferença deles é `0` —
nas mesmas incógnitas que a costura já elimina. ⚠️ *A diferença contra a Obra A é que ali
se eliminava uma variável 2-vector inteira, e aqui elimina-se **um escalar**: o sistema
reduzido tem de passar a ver as duas componentes em separado.* É essa a wave.

### §23.16 — ⭐⭐⭐ O PORTÃO DA WAVE: as equações dos arcos são consistentes, e quase todas eliminam

Antes de refazer o relaxador do G3, a premissa da wave foi **medida**.
[`ph2d_gridmap::arcline`](../../../crates/ph2d-gridmap/src/arcline.rs), 5 gates.

#### A equação, escrita

Um arco vai do canto `A` ao canto `B` na carta de um patch. Com `e` o eixo **atravessado**,
a exigência é `e·z_B − e·z_A = 0`, e cada cópia é `z = R^rot·y_classe + off`.

⭐⭐ Como `e·(R^rot·y) = turn2(e, −rot)·y` e **um quarto de volta leva um eixo a outro eixo
com sinal**, a equação colapsa em

```text
    s_B · y_B[j_B]  −  s_A · y_A[j_A]  =  c        com s ∈ {+1, −1}
```

⭐⭐⭐ **Dois ESCALARES, coeficientes `±1`** — a mesma forma que a costura já elimina, com
metade da variável. ⚠️ *A identidade que tudo isto assenta é verificada por gate que
avalia os dois lados* (`the_axis_identity_holds_for_every_turn`), não por outra dedução à
mão: é ali que um sinal troca sem nada deixar de compilar.

#### ⭐ O portão, medido em 7 peças

Um conjunto de diferenças com sinal é consistente **se e só se toda volta fechar**.

| peça | equações | ⭐ **eliminam** | fecham ciclo | ⛔⛔⛔ **conflitos de sinal** | desacordo | ⚠️ ambíguas |
|---|---|---|---|---|---|---|
| `sculpt_eared` | `47` | **`47`** | `0` | **`0`** | — | `1` |
| `sculpt_hooked` | `85` | `83` | `2` | **`0`** | `2,000` | `2` |
| `sculpt_ridged` | `56` | `55` | `1` | **`0`** | `3,000` | `1` |
| `sculpt_wrinkled` | `46` | **`46`** | `0` | **`0`** | — | `1` |
| `sphere_uv_96x144` | `44` | **`44`** | `0` | **`0`** | — | `0` |
| `sphere_shuffled` | `48` | **`48`** | `0` | **`0`** | — | `1` |
| `cube` | `91` | `88` | `3` | **`0`** | `0,000` | `0` |

⭐⭐⭐ **ZERO conflitos de sinal em todas.** A eliminação é possível como está escrita.

⭐⭐ **E o grafo das restrições é quase uma FLORESTA:** `0` a `3` ciclos por peça, contra
`44`–`91` equações. ⇒ **96 %–100 % delas eliminam um escalar directamente**, sem sistema
nenhum a resolver. *A wave é muito mais barata do que a da costura.*

⚠️ **Por que uma floresta, e não um emaranhado:** num canto onde quatro arcos se
encontram, os dois horizontais restringem o escalar `v` e os dois verticais o `u` — o
grafo **parte-se por eixo**, e é isso que mata os ciclos.

#### ⚠️ O que fica nomeado, e não escondido

- **Os poucos ciclos discordam por INTEIROS exactos** (`2` e `3` células). *Não é erro
  contínuo, é escolha de quantização* — e o `c` deles depende das **translações**, que são
  variáveis livres. ⇒ eles entram no **mesmo sistema de fechos** que a costura já usa
  ([`weld_flat`](../../../crates/ph2d-gridmap/src/weld_flat.rs)), sem maquinaria nova.
- ⚠️ **`0`–`2` arcos por peça são AMBÍGUOS** (perto de `45°`, onde o eixo atravessado é uma
  **leitura** e não um facto — [`AMBIGUOUS_RATIO`]). *Um conflito nascido daí seria meu, não
  da peça*, e por isso a coluna existe antes de alguém a poder confundir com um.

⛔ **E o que este portão NÃO é:** ele não impõe coisa nenhuma. *A saída do botão continua
byte-idêntica* — o que ele compra é a certeza de que a wave seguinte assenta numa premissa
lida, e não numa que se supôs.

### §23.17 — ⛔⛔⛔ A wave foi CONSTRUÍDA, e a medição diz que ela está na CAMADA errada

A restrição dos arcos está implementada de ponta a ponta: as amarras
([`arcline::build_arc_ties`](../../../crates/ph2d-gridmap/src/arcline.rs)), a relaxação de
um grupo amarrado ([`weld_solve::relax_tie`](../../../crates/ph2d-gridmap/src/weld_solve.rs))
e o 2.º passe atrás de `PH2D_GRIDMAP_ARCLINE=1`. Dois gates provam que ela **entra** e que
**move o mapa** numa esfera analítica, e um terceiro prova que **desligada é inerte, bit a
bit**.

⛔ **Nas peças reais ela não faz nada — e o contador diz exactamente porquê:**

| peça | grupos entraram | ⛔ recusados | dependente | ⭐ **é incógnita LIVRE** | já pregada |
|---|---|---|---|---|---|
| `sculpt_eared` | **`0`** | `11` | `0` | **`11`** | `0` |
| `sculpt_hooked` | **`0`** | `17` | `0` | **`17`** | `0` |
| `sphere_uv_96x144` | **`0`** | `12` | `0` | **`12`** | `0` |

⭐⭐⭐ **Uma só razão, em `100 %` dos casos: os cantos dos arcos SÃO as incógnitas livres do
sistema dos fechos.** Os extremos de uma separatriz são cones, e um cone é precisamente a
variável que a soldadura da costura já possui.

⇒ **A restrição dos arcos não pode ser uma segunda camada de eliminação por cima da
primeira.** Ela tem de entrar **dentro** do [`ClosureSystem`], como uma terceira espécie
de fecho, ao lado do plano e do que roda.

#### ⚠️ E a lei que previa isto já estava escrita — na wave anterior

O [`weld_flat`](../../../crates/ph2d-gridmap/src/weld_flat.rs) diz, desde a Obra A:

> ⭐⭐⭐ **E as duas espécies de fecho entram no MESMO sistema.** Tratá-las em dois
> subsistemas cria uma realimentação cujo ganho é maior que `1` — a esfera vai a **NaN** e
> o toro a `6,4e17`, e amortecer não cura. *Duas eliminações que leem o que a outra
> escreve não são duas eliminações.*

⛔ *Eu construí o segundo subsistema à mesma.* A diferença é que desta vez a guarda
apanhou-o **antes** de divergir: o grupo é recusado inteiro e **contado**, em vez de
metade dele passar. ⚠️ **Uma lei escrita no doc do módulo vizinho não está no caminho de
quem implementa** — e é a segunda vez nesta linha que isso custa uma wave
([`feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it`]).

#### ⭐ O que fica, e é mais do que uma refutação

- **A álgebra está feita e gateada:** a equação de um arco é
  `s_B·y_B[j_B] − s_A·y_A[j_A] = c`, com a identidade dos eixos provada por gate que
  avalia os dois lados, e o portão da §23.16 a dizer que ela é consistente
  (`0` conflitos, grafo quase floresta).
- **A relaxação de um grupo está escrita e provada** (soma dos resíduos dos membros com
  sinal, sobre a soma dos denominadores) — ela migra para dentro do `ClosureSystem` sem se
  reescrever.
- ⭐ **E uma armadilha ficou medida:** a 1.ª versão resolvia o 2.º passe e depois construía
  o relaxador da **escada gulosa sem as amarras** — a escada desfazia tudo, e a saída era
  byte-idêntica ao controlo **com os grupos todos a entrar**. *Uma restrição imposta numa
  fase e não na seguinte não é uma restrição; é um ponto de partida.*

⛔ **Shipa DESLIGADO** (`PH2D_GRIDMAP_ARCLINE` ausente ⇒ byte-idêntico), com a tabela desta
secção ao lado.
