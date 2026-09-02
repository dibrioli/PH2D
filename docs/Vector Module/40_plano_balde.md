# Plano 40 — **O BALDE** (a região que o clique aponta vira forma)

> Enio, 2026-08-31: *"Agora O Balde: preenche áreas por linhas fechadas ou linhas sobrepostas."*
> E, 2026-09-01, depois do Weld funcionar: *"Vamos à implementação do Balde?"*

## §1 — A pesquisa (o que a indústria faz, e o que ela ABANDONOU)

| app | modelo | o que sai do clique |
|---|---|---|
| **Illustrator — Live Paint** | um GRUPO especial; as faces são estado VIVO do grupo | a face fica pintada **dentro do grupo**; editar uma linha repinta |
| **CorelDRAW — Smart Fill** | um **objecto novo** por clique | uma forma fechada, colocada por cima/atrás; independente daí em diante |
| **Inkscape — Bucket** | rasteriza a tela e traça o balde | um caminho novo, **traçado de pixels** (grosseiro, e depende do zoom) |
| **Krita / Flip** | raster + fecho de vãos + bola presa | pixels |

⛔ **O modelo do Illustrator foi MEDIDO e recusado para a v1**: um Live Paint Group é um tipo de
objecto novo, com estado próprio (as faces), sincronização por edição e um modo de selecção
próprio (o *Live Paint Selection Tool*). Ele é a feature certa **depois** de haver uma que
preencha; construí-lo primeiro é o mesmo erro que a `line/Vector` já pagou no morph.

⇒ **Adoptamos o CorelDRAW: um clique = um OBJECTO novo.** Ele compõe com tudo o que a casa já tem
(estilo, pose, undo, hierarquia, booleana) e não pede tipo nenhum.

⛔ **E não é o do Inkscape.** Traçar pixels dá uma forma que depende do zoom e que não coincide com
as linhas que a geraram. Aqui a fronteira é feita dos **próprios arcos**.

## §2 — A lei

> **O balde preenche a FACE que o clique aponta, e a fronteira dela é uma sequência de ARCOS
> INTEIROS da rede.**

⭐⭐⭐ **É por isto que o Weld veio primeiro.** Soldar parte todo contorno nos cruzamentos, então
cada arco vai de nó a nó e **nenhum ponto interior de um arco é fronteira de face**. A face é um
ciclo de arcos, e reconstruí-la em bézier é **concatenar arcos** — sem aproximação, sem faceta, sem
depender do zoom.

⚠️ **E o balde não exige que o artista tenha soldado.** Ele faz o mesmo corte **numa cópia**, pela
mesma porta (`trim_tool::crossings_against` + `weld::split_at` + `weld::cluster_endpoints`). Soldar
continua a ser o verbo que torna a rede **autorada**; o balde só precisa dela para o instante do
clique.

## §3 — ⛔ Por que isto NÃO é o Shape Builder

O `ph2d-vec-boolean::arrangement` já responde *"que face é esta?"* — e o doc dele explica que
**deliberadamente não tem DCEL**, porque uma face tem definição conjuntista:

```text
região(M) = (∩ das formas em M) − (∪ das formas fora de M)
```

⛔ **Essa definição não existe para um traço ABERTO** — uma linha não tem dentro, então nenhuma
pertinência a descreve. É exactamente o caso do pedido (*"linhas sobrepostas"*).

⇒ O balde é **o DCEL que o arranjo evitou**, e existe pela razão que o próprio doc do arranjo
nomeia como fronteira. Para formas fechadas os dois sabem responder: o Shape Builder fica com o
**arrasto** sobre faces, o balde com o **clique**. ⚠️ *É uma divergência declarada, não um
descuido*: duas leis diferentes sobre o mesmo caso, e cada uma com o seu gesto.

## §4 — O algoritmo

1. **Os contornos visíveis**, cozidos e no MUNDO (a convenção do `apply_vec_boolean`).
2. **Cortar nos cruzamentos** → arcos, cada um com a geometria bézier e as duas pontas.
3. **Fundir as pontas coincidentes** (`weld::cluster_endpoints`, folga = 2× flecha) → os NÓS.
4. **Meias-arestas**: cada arco dá duas (ida e volta). Em cada nó, ordenadas pelo ângulo da
   tangente de SAÍDA.
5. **`next(h)`** = a meia-aresta imediatamente anterior ao gémeo de `h`, em sentido anti-horário
   à volta do nó de chegada — o passeio clássico que produz as faces com orientação consistente.
6. **A face do clique**: entre os ciclos de área positiva que contêm o ponto, o de **menor área**
   (é o que resolve o aninhamento). A face externa tem área negativa e é descartada por
   construção.
7. **A forma**: os arcos do ciclo concatenados, fechados, com o preenchimento corrente.

⚠️ **A polilinha do passeio é a MESMA que a detecção de cruzamentos usa** (`arc_cut`): duas
amostragens diferentes discordariam sobre a existência de um cruzamento, e a face desapareceria
num sítio e não noutro.

## §5 — O CUSTO, medido (e o que ele refutou)

| contornos | arcos | montar a rede | achar a face |
|---|---|---|---|
| 4 | 8 | `0,06 ms` | `0,01 ms` |
| 10 | 136 | `0,72 ms` | `0,05 ms` |
| 20 | 280 | **`3,80 ms`** | `0,08 ms` |
| 40 | 628 | **`26,3 ms`** | `0,18 ms` |
| 80 | 1293 | **`188 ms`** | `0,35 ms` |

⛔ **Montar a rede por QUADRO está refutado** — ela estoura o orçamento de `16,7 ms` aos ~20 traços.
⭐ **Achar a face por quadro é de graça.** ⇒ a rede é **guardada**, e a chave do cache é o
**CONTEÚDO** (uma soma sobre âncoras e alças), não a contagem de caminhos: mover uma forma não muda
quantas há.

⚠️ **Com o balde na mão o documento é quase estático** — não há gizmo neste modo —, então a
reconstrução acontece uma vez por preenchimento (e num undo). O pico de `188 ms` num desenho de 80
traços é um **soluço no primeiro hover**, não um congelamento por quadro; fica **medido e aceite**,
com a broadphase por grelha nomeada como a saída se alguém a pedir.

## §6 — O que a wave achou fora do balde

⛔⛔ **Um cruzamento na EMENDA de um anel era descartado** (defeito no `weld::split_at`, plano 39):
a fracção `0` de um contorno FECHADO é um ponto interior, e o filtro que serve a um contorno ABERTO
(onde `0` e `1` são as PONTAS) apagava-o. Um círculo cortado exactamente sobre a âncora de partida
saía com **um** arco em vez de dois. ⚠️ A `ellipse` começa em `(cx + r, cy)` — que é justamente onde
uma recta horizontal pelo centro a corta —, então o caso não é exótico: é o primeiro que se desenha.
Vale **também para o Soldar**, que tinha o mesmo buraco.

⚠️ **E a folga dos nós precisou de um segundo piso.** A régua da flecha, agora correcta (ela mede a
distância à CORDA, não ao ponto médio dela), é **zero numa recta** — e aí os dois lados de um
cruzamento, que calculam o mesmo ponto com resíduo de `~1e-15`, **não se juntam**: a rede fica
desligada e não há face nenhuma. O piso é uma **fracção da diagonal** da arte (`1e-5`), que é a lei
que o `ph2d-flip-fill` já usa para a mesma pergunta.

## §7 — ⭐⭐⭐ O REPORT DE 2026-09-01 (com fotos) — três defeitos, e o 2.º muda o modelo

> Enio: *"Ficou interessante mas muito limitado. 1) ao usar o balde nas áreas coloridas, ele para de
> funcionar nas áreas não coloridas. 2) se movo os nós da linha, o preenchimento não acompanha. A
> área deveria permanecer perfeitamente preenchida mesmo modificando o path. 3) o preenchimento está
> acima do stroke, mas deveria estar abaixo."*

### 1. ⛔⛔ Um preenchimento não é uma PAREDE

A forma depositada tem por fronteira **os mesmos arcos** que as linhas. De volta à rede, ela punha
lá arestas **coincidentes**, com direcção de saída idêntica — e o passeio de faces passava a
escolher entre duas meias-arestas indistinguíveis. As regiões vizinhas deixavam de fechar.

⇒ Quem tem `VecBucketFill` **não entra na rede**. *Uma parede é o que o artista desenhou; um
preenchimento é o que ele pediu.* ⚠️ A exclusão vive numa **porta com nome** (`fora_da_rede`), e não
num fecho no sítio da chamada: a 1.ª redacção tinha-a inline, e a mutação que apagava o termo do
preenchimento **sobreviveu** — o gate media o fecho que o **teste** construía.

### 2. ⭐⭐⭐ O preenchimento é VIVO, e a receita é o PONTO

⚠️ **Guardar a lista de ARCOS não resolveria**: um arco nasce de um corte em fracções, e mover um nó
**muda os cruzamentos**, logo muda a própria lista. *Qualquer receita feita de pedaços da rede é uma
receita sobre uma rede que já não existe.*

⇒ A receita é o que o artista de facto fez: **apontou ali**. `VecBucketFill { seed }` guarda o ponto,
e a área é a resposta de hoje — re-cozida sempre que a rede muda, **em qualquer ferramenta** (ele
arrasta um nó com a seta branca). ⚠️ Uma semente que deixou de cair em face nenhuma **congela** a
forma onde ela está, em vez de a fazer sumir — a escolha do conector e do morph.

⭐ **Isto é o Live Paint do Illustrator com outro substrato** — e sem o tipo de objecto novo que o §1
recusou: lá a face é estado vivo de um grupo especial; aqui é uma pergunta que se refaz.

### 3. ⛔ O `insert_path(0, …)` NÃO é o fundo

Quem manda no desenho é o **`RootOrder` da ENTIDADE**, e o `vec_entities::sync` dá a toda entidade
nova **o maior** — a frente. *O índice na cena e a ordem de desenho são duas listas, e a que o olho
vê é a segunda.* ⇒ a forma é mandada para o fundo (`ZOrder::ToBack`) na mesma passagem em que a
receita lhe é presa, logo depois do `sync` (no clique a entidade ainda não existe).

### ⚠️ E o INSTRUMENTO mentiu, no meio disto

O arnês de mutação restaurava o ficheiro com `mv`, e o mtime voltava **para trás**: o cargo ficava
com o **mutante compilado** e a fonte curada no disco, e as corridas seguintes mediam o mutante.
Sintoma: uma função com gate **verde na sua própria crate** devolvia o comportamento **antigo** a
quem dependia dela. ⇒ o arnês faz `touch` depois de restaurar.

### 4. ⛔ E a área re-cozida saía DESLOCADA — pelo próprio centro

> Enio, 2026-09-01 (com foto): *"o preenchimento está nascendo deslocado para fora do stroke."*

⚠️⚠️ **A rede fala MUNDO e o documento guarda LOCAL** — a regra-mãe do módulo, e o re-cozimento
esquecia-a. A forma **nasce certa** (uma entidade nova está na identidade); no quadro seguinte o
`settle_origins` muda a **origem** dela para o centro da própria caixa, e a partir daí escrever
mundo naquele `VecPath` desloca-o **pelo centro dele**.

⭐ **A foto confirma o mecanismo antes de uma linha de código:** cada área estava desviada por um
vector DIFERENTE, e cada vector era o centro da sua própria região — a de cima-esquerda para cima e
para a esquerda, a da direita para a direita, a de baixo para baixo. *Um desvio constante seria uma
câmara; um desvio por-forma é a pose de cada uma.*

⇒ `para_local` desce a área ao espaço do caminho antes de a escrever. ⛔ O `apply_bucket` **não**
estava errado: ali a entidade ainda nem existe — e é por isso que o defeito aparecia *"ao nascer"*,
que é o primeiro re-cozimento.

### 5. ⭐⭐⭐ A LEI, escrita pelo Enio (2026-09-01)

> *"O nó de uma solda é um só para todas as linhas. As alças daquele nó devem servir
> simultaneamente para o stroke e para os preenchimentos, senão é impossível que sejam
> transformados juntos."*

Ela tinha **dois** buracos, e nenhum era o desenho — eram omissões:

⛔ **A alça de ENTRADA não entrava na chave do cache.** Arrastar a alça de saída refazia a rede e a
área acompanhava; arrastar a de entrada mudava o traço e o preenchimento **ficava com a curva de
antes**. ⚠️ E o gate que devia apanhá-lo media **uma das duas** (`the_key_sees_a_handle_move`) —
*um gate que mede metade da população aprova a metade que não mediu.* As duas alças de um vértice
são dois graus de liberdade.

⛔⛔ **E o preenchimento mostrava alças PRÓPRIAS.** Ele tem a mesma fronteira que os traços, então
cada nó tinha **dois** conjuntos empilhados: o dedo agarrava o mais próximo, o traço não se mexia, e
o re-cozimento seguinte apagava a edição. *Duas alças no mesmo sítio não são um nó partilhado: são
duas coisas que discordam.*

⇒ `VecViewState::derived` — **uma lista, dois leitores**: o desenho dos gizmos (`draw_overlays`
salta-a) e o `is_pickable` (não se agarra). ⭐ **É a lei que a gaiola do Envelope já escrevia** (a
forma sob ela é a saída do warp, e os nós dela não se desenham); aqui ela deixa de ser um caso
especial da shell e passa a ser uma propriedade da vista.

### 6. ⛔⛔ *"A depender da posição dos pontos o preenchimento SOME"* (2026-09-01)

**Duas perdas, medidas antes de qualquer cura.**

#### (a) Uma parede a cair em cima de outra levava a REDE INTEIRA

⚠️ Medido: com um traço arrastado até coincidir com a aresta vizinha, a rede passava de **3 faces a
1** — e a região do outro lado do desenho perdia o preenchimento junto, sem ter nada a ver com
aquilo. *Duas meias-arestas com a mesma direcção de saída são indistinguíveis para o passeio, e ele
fecha um ciclo só, gigante.*

⇒ `descartar_duplicados`: dois arcos que ligam **o mesmo par de nós** e passam **pelo mesmo sítio**
são o mesmo arco. ⛔ **O par de nós sozinho não chega** — duas curvas diferentes entre os mesmos dois
nós são uma **lente**, que é uma região legítima; por isso a comparação inclui o ponto do meio.

⭐⭐ **E isto reabriu uma recusa:** o gate que dizia *"uma cópia coincidente envenena as vizinhas"*
existia para justificar manter o preenchimento fora da rede. Ele **caducou** — hoje mede a cura. *A
política fica e o motivo dela mudou*: um preenchimento continua fora por ser **derivado**, não por
envenenar (o descarte guarda **um** dos dois arcos, e se fosse o da cópia a rede passaria a depender
de geometria que outro motor reescreve).

#### (b) A semente colada à borda perdia-se à primeira parede que passasse

⚠️ Medido: com a semente a `0,5` de uma parede, arrastá-la por cima do ponto fazia `face_em`
devolver `None` — e o preenchimento **congelava**, que na tela se lê como *"deixou de acompanhar"*.

⇒ Depois de cada re-cozimento a semente **re-semeia-se no ponto mais FUNDO da face**
(`Rede::interior_point`: o centroide quando cai dentro; numa face côncava, a amostra da grelha mais
afastada da borda). Assim a parede tem de varrer o **miolo** para a perder — que é quando a região
de facto deixou de existir. O gate mede os dois lados: com re-semeadura a varredura inteira
sobrevive; sem ela, não.

### 7. ⛔⛔⛔ A FOLGA DE VÃO foi construída, shipada e **REVERTIDA** (2026-09-02)

> Enio, 2026-09-02, sobre o report *"a depender da posição dos pontos o preenchimento ainda some"*:
> depois da cura — **"o comportamento piorou"**. ⇒ revertida em `e5b2fb0a1` (revert), com a árvore de
> volta ao estado de `5f80b6fef`.

**O que foi medido e continua verdade** (a fixtura é a foto dele: rectângulo arredondado + curva que
sai do lado direito e volta a ele, fechando uma bolsa; varre-se a ponta à volta da parede `x = 100`):

| ponta em `x` | a bolsa é região? |
|---|---|
| `100,00` | sim |
| `100,05` · `100,5` · `101` · `102` | **não** |
| `99,9` | não — e o rectângulo **funde-se** com a bolsa |

⚠️ A causa daquele quadro é real: uma ponta que POUSA numa parede (a junção em «T») só conta como
toque a menos da **flecha** da parede, e a flecha — corrigida em 01/09 para medir só o desvio
perpendicular — é **zero numa recta**. *A flecha antiga, errada, dava `0,55` de folga por acidente, e
era esse acidente que segurava as junções em T.*

**A cura tentada:** `aproximar_pontas` — toda ponta solta a menos de `folga` de uma parede (ou de
outra ponta) era **movida** até lá antes de a rede ser cortada, com `folga` = a **largura do traço
mais grosso do documento**.

#### ⛔ Por que ela piorou, e o que a minha bancada não podia ver

⚠️⚠️ **A cena de smoke NUNCA dispara a regra.** Medido nas 10 pontas soltas dela: a parede mais
próxima de qualquer uma está a **30 unidades**, e a folga do documento é `3`. Com `folga` até `14`,
**zero** pontas se moveriam. *Uma fixtura em que a feature nunca dispara não pode mostrar o que ela
parte* — o gate ficou verde sobre um produto que muda o desenho do artista noutro sítio.

**Os quatro perigos, nomeados:**

1. ⛔ **A folga é o MÁXIMO do documento inteiro.** Um traço grosso num canto define a tolerância de
   toda ponta do desenho. Num desenho denso (o dele), isso é uma regra global com raio grande.
2. ⛔ **Ela MOVE geometria.** A fronteira do preenchimento deixa de coincidir com o traço que o
   artista vê — até uma largura de traço fora.
3. ⛔ **A parede mais próxima ganha**, e à escala da largura do traço é fácil a mais próxima ser a
   errada: a ponta cola-se a outra linha que passava por perto.
4. ⛔ **Duas pontas soltas dentro da folga vão para o meio**, fechando um vão que o artista podia
   querer aberto — e juntando duas regiões numa.

#### ⏳ O que fica desenhado, e não construído

A saída estreita **não move geometria nenhuma**: usar a folga só para (a) reconhecer o toque em «T»
(`trim_tool::touches`, cuja tolerância é hoje a flecha) e (b) fundir a IDENTIDADE do nó
(`cluster_endpoints`) — os arcos ficam exactamente onde estão e só o **grafo** muda. A fronteira
emitida teria um salto invisível (`0,05`) em cada junção, e nada no traço se mexe.
⚠️ **E a folga precisa de uma régua que a bancada consiga disparar**: uma fixtura com pontas a
`0,05`–`2` de uma parede tem de entrar no corpus **antes** de a regra voltar.

### 8. ⭐⭐⭐ A 2.ª tentativa: **UMA PASSAGEM, uma folga MINÚSCULA, e uma RECUSA** (2026-09-02)

Depois do revert, a varredura foi ampla em vez de partir de um caso sintético — e achou **três**
defeitos, dois deles muito maiores que o do «T».

#### (a) ⛔⛔ O tecto de amostragem era um PENHASCO MUDO

O `crossings_against` soma as arestas do alvo **mais as de todos os outros**, e o balde perguntava
**por contorno** — ou seja, o mesmo total era construído e comparado com o mesmo tecto `n` vezes:
`O(n³)`. Medido em círculos que se cruzam:

| círculos | arestas | montar a rede | a lente entre dois |
|---|---|---|---|
| 64 | 4 096 | **764 ms** | `2 235` ✓ |
| **65** | 4 160 | 9,9 ms | **`7 844`** ⛔ |

⚠️ **Acima do tecto TODOS os cruzamentos desapareciam em silêncio**, cada forma voltava a ser um anel
inteiro, e o preenchimento saltava para a forma toda. *Uma resposta errada em silêncio é pior que
nenhuma resposta.*

⇒ `trim_tool::crossings_all`: as arestas são construídas **uma vez**, o motor corre **uma vez**, e o
tecto é comparado **uma vez**. **64× mais rápido** (764 → 11,9 ms) e, acima do tecto, uma **recusa**
(`Rede::recusada`, rede vazia, sem faces) que a shell **diz** em vez de agir.

⭐ **E o tecto novo é MEDIDO**: `MAX_SAMPLES_BATCH = 12 288` arestas (768 segmentos no documento),
onde o relógio marca **102 ms** — o critério de morte de um soluço numa mudança de desenho. ⚠️ **São
dois orçamentos e por isso dois números**: o `MAX_SAMPLES = 4096` serve o Trim, que pergunta **por
quadro**.

#### (b) ⛔⛔ A fusão de travessias era GLOBAL — três linhas concorrentes perdiam duas

Ela existe para colapsar *a mesma travessia vista por duas arestas vizinhas*, **dentro do par que a
produziu**. Comparada contra tudo o que já saiu, ela apagava o cruzamento de um **segundo par no
mesmo ponto**. ⚠️ **Ficou escondida enquanto cada contorno perguntava numa chamada própria** (a lista
nascia vazia de cada vez) — *a passagem única expô-la, e ela vale para todo desenho com três linhas
pelo mesmo ponto*.

#### (c) A junção em «T», agora com a folga estreita e SEM mover geometria

`TOUCH_FRACTION = 1e-3` da diagonal (`0,4` num desenho de 400), usada nos **dois** sítios que fazem a
mesma pergunta: o reconhecimento do toque (`trim_tool::touches`) e o agrupamento de nós. ⛔ **Nenhuma
geometria se move** — os arcos ficam onde estão e só o **grafo** muda; a fronteira emitida tem um
salto invisível no nó. Medido na fixtura da foto: fecha de `0` a `~0,35` de vão, e **recusa** um vão
de `2` unidades, que se vê.

⚠️ **Duas fixturas minhas mediam outra coisa**, e as duas foram apanhadas por mutação sobrevivente:
uma tinha curvas, cuja flecha já cobria a folga por acidente; a outra fechava por um **cruzamento na
quina** em vez do toque. *Uma fixtura que fecha por outra razão aprova a lei que ela não mede.*

## §9 — ⏳ Nomeado e fora da v1

- **Vazamento**: se o clique cai na face externa (a região não fecha), o balde **recusa e diz
  porquê**. ⛔ Fechar vãos automaticamente é a lei do `ph2d-flip-fill` (bola presa + extensão de
  pontas) e pertence a uma wave própria — soldar já é o gesto que fecha o que o artista quis.
- **Ilhas**: uma forma solta dentro da face fica por cima; ela ainda não vira buraco (subpath).
- ✅ **Uma região que se PARTE em duas dá a tinta a UMA das metades** — **FECHADO em 2026-09-02**
  (§10). ⚠️ E a saída desenhada aqui — *"uma **lista** de sementes na receita"* — foi **recusada por
  medição**: uma lista de pontos diz *onde* a tinta estava e continua sem dizer **quanto** da face
  era de quem, que é exactamente o que o caso da **FUSÃO** (o outro metade do mesmo report) exige.
- **Live Paint** (a face como estado vivo do grupo) — §1.

## §10 — ⭐⭐⭐ O REPORT DE 2026-09-02: **atravessar uma linha com um nó quebrava a tinta**

> *"quando levamos um nó para fora da área onde está, quando atravessamos uma linha com um nó, os
> preenchimentos se quebram. Então veja qual a melhor solução: não permitir que os preenchimentos
> sejam destruídos preenchendo corretamente as áreas novas que vão surgindo ou limitar a
> movimentação dos nós de modo que se movam apenas dentro das linhas da área em que estão"* — Enio

### §10.1 — ⛔ A segunda opção foi RECUSADA, e não por gosto

Prender um nó à região em que ele está não existe em editor vectorial nenhum, e o motivo é
estrutural: **o desenho manda na tinta, nunca o contrário.** O artista deixaria de poder redesenhar
a forma que pintou; e a restrição é impossível de exprimir no instante em que a área de destino
ainda não existe — é o nó que a cria.

### §10.2 — O defeito, medido

Sonda sobre um quadrado de área `400` com uma linha a atravessá-lo:

| o gesto | a rede passa a ter | com UMA semente por preenchimento |
|---|---|---|
| a linha entra e **PARTE** a região | 2 faces de `200` | a tinta fica com **uma**: metade da cor some |
| um nó atravessa o topo e **FUNDE** | `4,17` + `395,83` | **as duas sementes caem na MESMA face** ⇒ duas tintas empilhadas, e a lasca sem dono |

⇒ *uma semente diz **onde** a tinta estava; ela não diz **quanto** da face era dela.*

### §10.3 — A lei: **uma face herda a tinta da região que mais a cobria**

É o *Live Paint* do Illustrator, e as duas metades da regra deles saem de **uma** lei só aqui:
partir pinta **as duas** metades; fundir dá a face à **maior**. A receita deixa de ser um ponto e
passa a ser a **região que o preenchimento pintou da última vez** (que é a geometria dele, já no
documento — ⛔ nenhum campo novo, nenhum bump de schema).

A implementação é uma **votação** ([`vec_bucket_claim`](../../shells/desktop/src/vec_bucket_claim.rs)):
as amostras do miolo de cada face votam na região que as contém, e ganha a mais votada. Como as
amostras são uniformes sobre a caixa da face, a contagem é proporcional à **área** coberta.

- **Face sem voto nenhum** ⇒ fica por pintar. Uma região que ninguém tinha pintado é nova, e
  pintá-la sozinha inventaria uma decisão do artista.
- **Empate** ⇒ ganha quem tem a **semente** dentro desta face; persistindo, o índice mais baixo
  (a ordem do documento). ⛔ Ao acaso, a cor piscaria entre duas enquanto a mão treme.
- **Preenchimento sem face** ⇒ **CONGELA** (a lei que já existia). ⭐⭐ E aqui ela vale mais do que
  preservar trabalho: a forma congelada **é** a receita, então arrastar o nó de volta faz a tinta
  voltar sozinha. *No Live Paint do Illustrator ela não volta.*
- **Uma região que partiu continua a ser UM objecto** — os pedaços são contornos do mesmo caminho,
  o substrato que a rede soldada estreou no mesmo dia.

⚠️ **A resolução da votação é ~`1/15` da face**: uma fila de amostras em cima da parede antiga vale
~7% dos votos, e uma margem menor do que isso é decidida **pela grelha**, não pela área. É
determinístico, e o perdedor congela (e volta). ⛔ *A 1.ª redacção do gate da fusão afirmava o
vencedor com margem de 1% — estava a medir o alinhamento da grelha e chamou-lhe lei.*

### §10.4 — ⛔ O CUSTO, e a afirmação que a medição DERRUBOU

A 1.ª redacção do doc-comment dizia *"e isto é **mais barato** do que era: o `face_em` reconstruía
as faces uma vez por preenchimento"*. A construção de facto deixou de se repetir — e a votação paga
muito mais do que isso poupa. Grelha de linhas, 8 preenchimentos, `--release`:

| faces | montar a rede | 1 `face_em` por fill (antes) | a votação (agora) |
|---|---|---|---|
| 9 | `0,15 ms` | `0,053 ms` | `0,35 ms` |
| 25 | `0,30 ms` | `0,119 ms` | `0,45 ms` |
| 49 | `0,56 ms` | `0,224 ms` | **`0,64 ms`** |

⇒ ~2,9× o que a semente custava, e **3,8% de um quadro** de 16,7 ms no pior caso medido.

⭐ **E sem a rejeição por CAIXA o mesmo caso custava `4,53 ms` — 30% do quadro, a cada quadro de um
arrasto.** O produto é `faces × amostras × regiões`, e numa grelha cada face toca **uma** região:
cortar o factor das regiões vale **7,1×**. ⚠️ Não é micro-optimização — é o que separa pagável de
inaceitável.

### §10.5 — ⏳ Aberto

- ⏳ **Uma face nova que ninguém pintou fica por pintar** — correcto, mas o artista tem de a clicar.
  O *Live Paint* faz o mesmo.
- ⏳ **A margem de ~7% da votação**: uma grelha mais fina reduz o viés e paga linearmente. Sem
  report, não há número que justifique o preço.
- ⏳ **Uma ilha dentro de uma face** continua a não virar buraco (§9).
