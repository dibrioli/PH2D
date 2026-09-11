# 03 — As dez curas que fecharam o problema, com o mecanismo e os números

> Uma secção por cura, na ordem em que elas caíram. Cada uma traz **o report que a abriu**, **o
> mecanismo** e **a medição**. ⚠️ Nenhuma delas é uma afinação: todas nasceram de uma régua
> nova ou de uma régua corrigida (ver [`02_as_reguas_e_as_barras.md`](02_as_reguas_e_as_barras.md)).

## §1 — A IDEMPOTÊNCIA do botão (28/08) — *«piora severa»*

**O report:** carregar o botão outra vez destruía a peça.

**O mecanismo:** o alvo saía do piso `0,75 × aresta média(malha da cena)` — e depois de uma
retopologia **a malha da cena É a saída**. Com o `Detail` parado em `0,50`:
`19 786 → 1 747 → 520 → 281` quads (`−98,6 %`). A `281` uma ponta tem **duas faces**, que é
literalmente *«pontas com baixa resolução»*.

**A cura:** a faixa passa a ser **CONTADA e ancorada na ÁREA** (`edge_for_detail_by_count`,
`MIN_QUADS`…`MAX_QUADS`), que é o que as três referências fazem (ZRemesher *Target Polygons
Count*, QuadriFlow *Number of Faces*, Instant Meshes). Depois da cura os mesmos três apertos dão
`1 377 → 1 413 → 1 494`, com a forma a **melhorar**.

## §2 — O FURO que contava metade (28/08)

`open_edges` via só o **bordo**. O ficheiro que o dono exportou tinha `19 786` quads impecáveis
com **`2` arestas não-manifold** num ponto só. Hoje `open_edges = bordo + não-manifold`, e ele
decide **e** arma a tentativa seguinte.

## §3 — A ALMOFADA e o DOUBLET (28–29/08)

**O report:** *«faces completamente soltas, buracos»*.

**A almofada** é o mesmo quadrado emitido **duas vezes**, um virado ao contrário
(`[68,69,70,71]` e `[71,70,69,68]`), a flutuar sobre uma ponta. ⛔ **Nenhuma régua a via:** `χ`
conta os dois lados e dá `2`, o bordo é zero, o não-manifold é zero, e a contagem de quads
*sobe*. Quem a apanha é **contar os componentes ligados** (`2`, de `23 628` e de `2`). A causa é
uma dobra do mapa, e a extracção passa a descartar **os dois** lados.

**O doublet** é um vértice preso entre duas faces que partilham três cantos. A saída trazia
**`19`** deles, todos em pontas finas; o artista carregava outra vez e a fase zero — que só sabe
remalhar superfície — **rasgava a topologia** (`χ = 6`, aresta não-manifold, e o `ph2d-gridmap`
a estourar). Três curas: a extracção **não emite** (`dissolve_doublets`), o botão **repara** o
que a peça já traz, e uma tentativa que estoura **perde** em vez de derrubar tudo.

## §4 — A fase zero passa a GRADUAR, com renormalização (31/08)

**O report:** *«o remesh amputou pontas»*, com fotos.

**O mecanismo, que a régua já imprimia:** a razão `ALVO / F1 = 0,34 ×` — a malha de trabalho era
**três vezes mais grossa** que o quad pedido, e cortava `3` das `4` pontas **antes de a cadeia
começar**. ⚠️ E as duas metades do botão estavam ancoradas em coisas diferentes (o F1 em
`ALPHA × diagonal`, o quad em `área / contagem`), o que é **auto-derrotante**: *um espinho longo
infla a diagonal, logo uma peça com espinhos recebe uma malha de trabalho mais grossa POR TER
espinhos.*

**A cura:** a `SizingGrid` deixa de **inflar** e passa a **redistribuir** — o campo é escalado
por `√(N_previsto / N_pedido)`, medido pela própria grelha. O orçamento passa de `8,3 ×` para
`+7 %`…`+15 %`, e a avaria de topologia que a mantinha desligada **desaparece**.

**O tecto da graduação era EMPRESTADO:** o `ADAPT_RATIO` valia `4` porque era a cerca da **grade
de quads** (*«duas células cujas escalas diferem por mais do que isto deixam de ter aresta
comum»*) — e o consumidor aqui é um **remalhador de triângulos**, que não a tem. Com `4`, uma
agulha mais fina **satura** e recebe a mesma grelha de uma mais grossa (`8,5 %`–`14,3 %` dos
vértices no piso). Com **`16`** a saturação some (`0,2 %`) e as pontas cortadas ficam melhores
ou iguais nas quatro peças.

## §5 — A régua que concorda com o OLHO (02/09)

Quatro jornadas seguidas tinham sido reprovadas (*«absolutamente nenhuma melhoria»*) sobre
réguas que **nunca tinham sido corridas na retopologia que ele aprovou**. Corridas, dois
defeitos de calibração: o **piso do ápice** escondia as pontas da foto, e a **barra da grade**
saía do vazio entre os *nossos* defeitos. Ver [`02_as_reguas_e_as_barras.md`](02_as_reguas_e_as_barras.md) §1 e §3.

⇒ **portão sobre as fixturas dos DOIS lados** (`ph2d-quadfill/tests/it/pontas_do_dono.rs`): GREEN
na aprovada, RED nas duas reprovadas, com as margens exigidas.

## §6 — A CALOTA da ponta (03/09)

**O mecanismo:** a grade termina onde as **singularidades** param. Na malha que o dono aprovou
**todo** espinho fecha com um pólo `+1` — quatro valência-`3` a `≤ 2 h` do bico; nas nossas as
singularidades estavam a `9`–`15 h`, que é literalmente *«a grade termina a meio caminho»*. E o
pólo precisa de **`≥ 2` células resolvidas**, enquanto a fase zero entregava o bico a `2,22 ×` o
passo da grade.

**A cura:** [`ph2d_remesh_iso::Cap`] — uma **calota resolvida por espinho afiado** (passo `= 1 h`
do alvo, alcance `8 h`), que entra no campo por-vértice **antes** da renormalização (o orçamento
é pago pelo resto) e é **reclamada** depois.

| peça / realização | antes | com a calota |
|---|---|---|
| ⭐ `_base_sculpt` (a realização que o dono vê) | `1/5` amputada · gap `3,00` · grade `3,51` | **`0/5`** · `0,47` · **`0,79`** |
| `_base_sculpt` a `s = 0,7` | `0/5` · `0,45` · `1,66` (`3` acima) | `0/5` · `0,38` · `1,07` (`2`) |
| `sculpt_antes` (a agulha) | `1/4` · `3,00` · `1,15` (`1` acima) | `1/4` · `2,57` · `0,98` (`0`) |

⛔ **Afinar mais é pior, e está medido:** a `0,75` e a `0,5` a fase zero fica verde no bico e a
jusante devolve candidatas com `7`–`48` arestas de bordo. *O que a jusante não digere é a
INFLAÇÃO.*

## §7 — As faces do AVESSO: gravata, dobra e aba (03/09)

São **três** defeitos com a mesma cara e curas diferentes:

| espécie | o que é | cura |
|---|---|---|
| **gravata** (`Bowtie`) | o quad auto-intersecta | relaxação Laplaciana por grupo (`untangle_bowties`) |
| **dobra** (fold) | a normal aponta contra a média dos vizinhos | ⚠️ **a relaxação TROCA a espécie**, não remove: o critério é que decide |
| **aba** (flap) | um grupo de dobras juntas — a **fenda** que ele fotografou | apagar o disco e fechar com um leque (`remove_flaps`) |

⭐ **A régua que separa uma fenda de um vinco é o TAMANHO DO GRUPO:** a retopologia aprovada tem
`3` dobras **isoladas** (vincos reais da escultura); a nossa tinha `5` **juntas**. ⛔ Nem a
contagem de faces minúsculas nem o salto de tamanho separam — a aprovada é **pior** nos dois.

⭐ **A folga do selector (`INSIDE_OUT_SLACK = 20`)** é calibrada no vazio entre os dois vereditos
do dono: `125` = *«destruiu a malha»*, `6`–`8` = *«melhor resultado até agora»*. Sem ela, meia
dúzia de faces do avesso pagava uma ponta amputada.

**O apagador de abas**: o grupo acusado cresce **um anel**, o disco sai, e o buraco fecha com um
leque de `L/2` quads — recusando se a fronteira não for **um** laço, se for ímpar, ou se o
resultado não melhorar. Medido: dobras `6` (grupo de `6`) → **`1` isolada**, `21 928 → 21 914`
faces, pontas intactas.

## §8 — O REMATE do bico (04/09)

**O report:** *«bons resultados em muitas pontas no mesmo mesh e apenas uma ruim — muito
estranho»*.

⛔ **Nenhuma das três réguas sabia dizer QUAL** — todas mediam ponta a ponta e deitavam fora o
índice. ⇒ a tabela [`tip_rows`].

**O mecanismo:** a MESMA extracção (`21 914` faces), acabada de duas maneiras, dá `gap 0,18` e
`0,51` no mesmo bico — **com a grade do bico dentro da barra nas duas**. *O acabamento pousa
cada vértice na escultura, e a projecção ao ponto mais próximo de uma superfície **nunca escolhe
o ápice***: o bico é um ponto de medida nula e o pé da perpendicular cai no flanco. Cada ronda
embota a ponta, e *quanto* depende da cerca de viagem daquela saída — **é por isso que uma ponta
sai boa e a vizinha não.**

**A cura** ([`snap_tips`]) e as três cercas que a tornam honesta:

1. `gap ≤ TIP_GAP_MAX` — ⛔ acima da barra o que falta é **célula**, e mexer no vértice
   esconderia do selector um defeito que ele tem de ver;
2. viagem `≤ 1` célula — o `gap` mede ponto→FACE e o vértice mais próximo pode estar mais longe;
3. o **censo global** (`>60°` e faces do avesso) não pode subir, **uma ponta de cada vez**.

| peça do dono, `Detail 1` | sem o remate | com o remate |
|---|---|---|
| pior `gap` das `5` pontas | `0,18` | **`0,00`** (as cinco) |
| aspecto p50 / p99 | `1,15` / `1,88` | **`1,09` / `1,57`** |
| enviesamento p50 / p99 | `4,3°` / `29,6°` | **`3,0°` / `20,0°`** |
| faces `> 60°` | `8` | **`4`** |
| tentativas do botão | `9` | **`4`** |
| relógio | `337 s` | **`123 s`** |

⭐⭐⭐ **O relógio e a forma são CONSEQUÊNCIA:** a cascata pára quando uma candidata satisfaz as
réguas, e com a fase fechada a **segunda** já satisfaz. Antes eram precisas nove para achar uma
que ganhava por `0,18` contra `0,51` no bico — e essa pagava a diferença em forma, porque *a
cerca que protege a ponta é a mesma que impede a relaxação de trabalhar*.

## §9 — O `Follow Curvature` a nascer no MÁXIMO (04/09)

**O report:** *«a ponta problemática ainda não tem a densidade de faces adequada como as
outras»*, com foto e seta.

⛔ A régua do §8 mede as **três células do bico**, e ali as cinco pontas são iguais. A foto é
sobre o **espinho inteiro** ⇒ a faixa `3`–`12 h` ([`TipRow::shaft`], [`tip_band`]) e a contagem
do **anel** — *quantas faces dão a volta*, que é o número que o olho lê: um espinho fino com
quads pequenos ainda pode ter **cinco** faces em volta.

| `Curv` (peça do dono, `Detail 1`) | quads | amputadas | grade pior | `>60°` | envies. p50/p99 | relógio |
|---|---|---|---|---|---|---|
| ⛔ `0` (o padrão de então) | `22 165` | **`2` de `5`** | **`2,37`** | `27` | `3,9` / `29,9` | `150 s` |
| `0,5` | `22 679` | `0` de `5` | `0,93` | `10` | `4,0` / `27,1` | `208 s` |
| ⭐ `1` | `21 914` | **`0` de `5`** | **`0,95`** | **`4`** | **`3,0` / `20,0`** | **`123 s`** |

O anel, que é a foto: com `0`, as pontas `9218` e `12279` dão a volta com **`3`–`5`** e
**`0`–`8`** faces; com `1`, com `23`–`29` e `7`–`21`.

⚠️ **A troca está dita na segunda peça** (`sculpt_antes`): o `1` paga enviesamento mediano
`2,9° → 4,2°` — dentro da banda do oráculo (`4,8`–`7,1°`) — e `+50 %` de relógio, e compra
`>60` de `5` para **`0`** e a grade do bico abaixo da barra em **todas** as pontas.

⛔ **A recusa de 28/08 já não respondia** (*«pede-se 400 % e a saída move-se 7 %»*): desde ela a
fase zero passou a graduar com renormalização, ganhou a calota e o acabamento ganhou o remate.
*Uma recusa medida responde UMA pergunta.*

## §10 — E o defeito que a própria cura revelou: o default escrito DUAS vezes

O `sculpt3d_birth.rs` guardava a sua cópia dos dois defaults do botão — **com o comentário do
vizinho a dizer que *«um default escrito duas vezes é o que diverge»***. Ele divergiu no dia em
que o painel mudou. Hoje o botão **deriva** do painel, com gate a proibir o literal
(`the_curvature_knob_opens_where_it_was_measured`).
