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
