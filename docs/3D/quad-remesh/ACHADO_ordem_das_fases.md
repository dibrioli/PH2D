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
