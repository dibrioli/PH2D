# ADR-0160 — O quad remesh é um porte NATIVO de campo cruzado, QuadriFlow referenciado

- **Status:** aceito (ordem do Enio, 2026-08-19: *"investigue o melhor algoritmo
  de quad remesh com adaptação correta à topologia e implemente"*).
- **Escopo:** crate-folha nova `ph2d-quadflow` (só `ph2d-mesh` como dependência
  de domínio). **Não** substitui o `ph2d-sdf::surface_nets` — os dois respondem a
  perguntas diferentes, e o §3 mede a diferença.
- **Número:** ⚠️ **PROVISÓRIO** — contado contra o `main` do dia (o último é o
  0159) numa linha paralela. Um número escolhido assim **se re-conta na
  integração**; já aconteceu oito vezes neste repo.

---

## §1 — O problema, com o que já existe medido ao lado

Este módulo **já tem um remesh que devolve quads**: o
[`ph2d-sdf::surface_nets`](../../../crates/ph2d-sdf/src/surface_nets.rs), portado
do SculptGL (MIT). Ele põe **um vértice por célula** que a superfície cruza e liga
os vizinhos ⇒ a saída é uma **grade deformada, valência 4 quase em toda parte**.

⚠️ **Ele não é o que este ADR substitui, e a diferença é a pergunta:**

| | `surface_nets` (existe) | campo cruzado (este ADR) |
|---|---|---|
| a malha vem de | um **campo de voxels** | a **superfície de entrada** |
| a grade se alinha a | os **eixos do voxel** | as **direções principais da FORMA** |
| topologia de entrada | descartada (re-amostrada) | **preservada** (gênero, bordas) |
| densidade | uniforme (a do voxel) | **adaptativa** (curvatura) |
| detalhe fino | perdido abaixo do voxel | preservado onde a escala mandar |
| custo | O(voxels da casca) | O(V) por iteração de suavização |

⇒ O `surface_nets` é o **cavalo de batalha destrutivo** (arrumar uma malha que a
escultura destruiu, fundir booleanas — [`04.3-Topologia`](../../3D/04-Ferramentas/04.3-Topologia.md)
§1). O que falta, e o que o Enio pediu, é a **retopologia**: os quads correndo
*ao longo da forma*, que é o que torna a malha subdivisível, animável e editável.

⚠️ **E é o que o `surface_nets` NÃO pode dar, por construção**: os quads dele se
alinham à grade do voxel, então uma feição diagonal sai em escada. Isto não é um
defeito dele — é a pergunta que ele responde.

---

## §2 — A PESQUISA: o campo, em 2026

Três famílias, e a escolha entre elas é por **onde a consistência é imposta**.

### (a) Parametrização global com inteiros — MIQ, QuadWild

*Mixed-Integer Quadrangulation* (Bommes et al., 2009) e o
**QuadWild** (Pietroni et al., SIGGRAPH 2021, *"Reliable feature-line driven
quad-remeshing"*): resolvem uma parametrização global cujas isolinhas são as
arestas do quad, com as junções forçadas a inteiros.

- ✅ **A melhor qualidade que existe** — alinhamento a linhas de feição, cones de
  singularidade prescritos, controle direto da borda.
- ⛔ **Precisa de um solver de programação inteira mista.** O QuadWild usa
  libigl + um MIP externo. Trazer um MIP para dentro deste app é uma dependência
  de porte e de licença que o [HR-1](../../../SKILL_Stack_PH2D_Definitiva.md) não
  aceitaria sem uma wave só para ela — e o custo é de **minutos**, não do
  *"sob comando"* que o [`04.3`](../../3D/04-Ferramentas/04.3-Topologia.md) exige.

### (b) Campo cruzado local — Instant Meshes

*Instant Field-Aligned Meshes* (Jakob, Tarini, Panozzo, Sorkine-Hornung,
SIGGRAPH Asia 2015). **BSD-3-Clause** (verificado no `LICENSE.txt` do
`wjakob/instant-meshes`). Três passos:

1. um **campo de ORIENTAÇÃO** 4-RoSy (simetria de 4 dobras) por vértice,
   suavizado por atualizações locais sobre uma **hierarquia multirresolução**
   (é a hierarquia que faz a suavização convergir sem um solver global);
2. um **campo de POSIÇÃO** — uma retícula local por vértice, com a mesma simetria
   de 4 dobras, suavizada do mesmo jeito. ⚠️ **A escala-alvo entra AQUI**, e ela
   pode ser um campo em vez de um número: é este o ponto de entrada da
   *adaptação*;
3. a **EXTRAÇÃO** da malha a partir dos dois campos.

- ✅ Rápido, paralelo, sem solver global; escala para milhões de vértices.
- ⛔ **A consistência das singularidades é apenas LOCAL** ⇒ a extração emite
  elementos não-quad e configurações degeneradas em regiões onde os índices não
  fecham. É o defeito conhecido da técnica, e é exatamente o que a (c) cura.

### (c) Campo cruzado + consistência GLOBAL — QuadriFlow ⭐

*QuadriFlow: A Scalable and Robust Method for Quadrangulation* (Huang, Zhou,
Nießner, Shewchuk, Guibas, SGP 2018). **Permissiva** (o README diz MIT, o
`LICENSE.txt` é BSD-3-Clause — as duas servem, e a atribuição cobre ambas). É o
Instant Meshes **mais um passo global** que força a consistência dos índices de
singularidade, formulado como um **fluxo de custo mínimo** sobre o dual.

- ✅ Herda a velocidade da (b) — o fluxo é um passe, não um solver de otimização
  contínua.
- ✅ **Elimina os não-quads e os elementos invertidos** da (b): é a diferença
  entre *"quase todo quad"* e **all-quad manifold**, que é a propriedade que
  torna a saída subdivisível.
- ✅ É o que o **Blender** ships como *Quadriflow Remesh* — ou seja, é o que o
  artista que o Enio é já conhece por esse nome.
- ⛔ O upstream depende de Boost, Eigen e **lemon** (o simplex de rede). Nada
  disso entra aqui: o fluxo de custo mínimo é um algoritmo, e nós o escrevemos.

### A ESCOLHA

**(c), portado NATIVAMENTE em Rust, referenciado — não vendorizado.** É o mesmo
desenho do [ADR-0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)
(SculptGL) e do [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md)
(Rive): a licença **permite** copiar, e mesmo assim não copiamos — porque o que
viaja mal não é o código, é a dependência (Boost/Eigen/lemon) e o modelo de
memória. O que se porta é a **LEI**, medida contra a referência.

⚠️ **A licença é BSD/MIT, então — ao contrário do Blender (GPL) — a citação de
FONTE é permitida.** Os doc-comments podem citar `optimizer.cpp:NNN` como os do
SculptGL fazem, e não só descrever comportamento.

---

## §3 — O que "adaptação correta à topologia" quer dizer, em asserções

O pedido do Enio tem duas metades, e elas são medidas por gates diferentes:

**(i) Topologia PRESERVADA** — a saída descreve a mesma superfície da entrada:
mesmo gênero, mesmas bordas, sem costurar alças que não existiam nem abrir as que
existiam. ⚠️ É aqui que o `surface_nets` perde de propósito (ele re-amostra um
campo, e uma alça mais fina que o voxel some).

**(ii) Densidade ADAPTATIVA** — quads menores onde a curvatura é alta, maiores
onde a forma é chapada, com a razão entre os dois sendo um knob e **não** um
efeito colateral. Entra pelo campo de escala do passo 2.

⚠️ **Uma terceira que NÃO foi pedida e que este ADR recusa por ora:** o
alinhamento a **linhas de feição** autoradas (a quina que o artista marca). É o
que a família (a) compra, custa um MIP, e o gatilho dela é um report — não uma
suposição.

---

## §4 — Conjunto de ACEITAÇÃO (concreto, congelado ANTES do build)

Um alvo irrefutável (*"paridade"*, *"o melhor"*) **não é** definição de pronto —
[DIRETIVA §5](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md). Estes são os
números:

| # | asserção | oráculo |
|---|---|---|
| A1 | a saída é **all-quad** | contagem de lados por face |
| A2 | a saída é **manifold** e orientável | toda aresta tem 1 ou 2 faces; vizinhança de vértice é um disco/leque |
| A3 | o **gênero** da saída é o da entrada | característica de Euler `V − E + F` |
| A4 | a **forma** sobrevive | distância de Hausdorff bilateral ≤ **um lado de quad** (⚠️ **emendada em 2026-08-19** — ver §5-septies) |
| A5 | as singularidades são **isoladas e de valência 3 ou 5** | histograma de valência |
| A6 | a densidade **responde à curvatura** | razão entre a aresta média na região de maior e de menor curvatura ≥ 2× no modo adaptativo, e **1,0×** no modo uniforme |
| A7 | é **determinístico** | duas corridas ⇒ malha byte-idêntica (HR-5) |
| A8 | ⚠️ o campo de orientação tem a **simetria de 4 dobras** | girar a semente de 90° não muda o campo |

**KILL-CRITERION, escrito antes do build:** se sobre a esfera de 196 608
triângulos que este módulo abre o passe custar **> 3 s** depois da segunda
tentativa de otimização, a feature **não existe nesta forma** — ela vira offline
(fora do laço interativo, com barra de progresso) ou não entra. *O remesh é
"sob comando", mas um artista de Nomad o usa dezenas de vezes por sessão*
([`04.3`](../../3D/04-Ferramentas/04.3-Topologia.md) §1).

⚠️ **A4 é bilateral de propósito.** Uma distância só de ida premia uma malha que
encolhe para dentro da original, e foi assim que a primeira sonda de remesh deste
repo quase declarou sucesso sobre uma casca murcha.

---

## §5 — O PLANO, em ondas que fecham sozinhas

| onda | entrega | gate |
|---|---|---|
| **Q1** | a crate-folha + o **campo de ORIENTAÇÃO** 4-RoSy, suavizado | A8 + convergência + determinismo |
| **Q2** | o **campo de POSIÇÃO** + a escala **adaptativa** por curvatura | A6 |

> ⚠️ **CORREÇÃO MEDIDA (Q2, 2026-08-19) — a nota da hierarquia estava errada para
> metade dos campos.** Este ADR dizia que a hierarquia multirresolução é *"um
> acelerador de convergência, não uma lei diferente"*. Para o campo de
> **orientação** é verdade (a tabela de convergência está no doc-comment do
> `solve_orientation`). Para o de **posição** a frase é **vacuosa**: o campo
> atinge um ponto FIXO em dezenas de varreduras — medido imóvel entre **32 e
> 2 048** —, e o resíduo entre vizinhos fica em **0,205 célula**. Não há
> convergência lenta a acelerar; o que a hierarquia compra ali é **coerência de
> longo alcance**, e é a Q3 que a mede.
>
> ⚠️ **E o contrato do campo de posição é mais fraco do que este ADR sugeria:**
> ele coloca uma origem de retícula por vértice, consistente **a menos de passos
> inteiros** — quem quocienta pela retícula e forma os platôs é a **EXTRAÇÃO**. O
> invariante que a Q3 pode usar é `|o_v − p_v| ≤ s/√2` (a meia-diagonal da
> célula), derivado da construção e gateado.
| **Q3** | a **EXTRAÇÃO** ingênua (Instant Meshes) | A1..A4, com os não-quads CONTADOS e nomeados |
| **Q4** | o **fluxo de custo mínimo** (QuadriFlow) | A1 vira exato, A5 |
| **Q5** | a costura no shell: verbo, painel, undo, smoke | o smoke do Enio |

---

## §5-octies — ⛔ EU ESTAVA INVENTANDO. O código da referência foi BAIXADO e PORTADO

> *"Uma bosta!!! Acho que vc está inventando!!! Pare de inventar. Identifique o
> melhor algoritmo do mundo. Baixe o código. E tentar portar."* — Enio,
> 2026-08-19, com foto de uma esfera coberta de leques.

**Ele estava certo.** Este ADR dizia *"porte nativo"* desde o primeiro dia, e o
que existia era uma **reconstrução de memória**: os operadores estavam certos
(conferidos agora, linha a linha), e **tudo o que os consome era invenção
minha** — o agrupamento, o grafo, o passeio de faces, a hierarquia, os pesos e os
dois núcleos de suavização.

### O que foi baixado

| repo | licença | o que se usou |
|---|---|---|
| `wjakob/instant-meshes` @ `7b31608` | **BSD-3-Clause** | `extract.cpp`, `field.cpp`, `hierarchy.cpp`, `adjacency.cpp`, `meshstats.cpp`, `cleanup.cpp` |
| `hjwdzh/QuadriFlow` | **BSD-3-Clause** | lido para a Q4 (fluxo de custo mínimo); **não** portado ainda |

⚠️ **As duas são permissivas**, então a citação de fonte é permitida e os
doc-comments do porte a usam — ao contrário do Blender (GPL), de quem só se pode
descrever comportamento.

### O que estava errado, peça por peça

| peça | a minha invenção | a referência |
|---|---|---|
| **grafo** | cone de 45° + janela `[0,5s, 1,7s]` sobre a geometria | o **passo inteiro** entre as duas retículas, com a cruz de `j` **rotacionada** para a moldura de `i` primeiro; `(1,1)` (a diagonal) é **recusada** |
| **agrupamento** | `union-find` sobre "os campos estão perto" | colapso por **erro crescente**, com teste de conflito e união das vizinhanças |
| **posição do nó** | média simples das origens | média **ponderada** por `exp(−9·\|O−V\|²/s²)` |
| **limpeza** | *não existia* | encolher todo triângulo de altura `< 0,3 s` e **remover toda diagonal de quad** — é o passo que faz a grade ser quad |
| **faces** | um passeio, aceitando o ciclo que saísse (3, 8, **44** lados) | **seis passadas** pedindo comprimento EXATO (3..8); o que não fecha é **desfeito** |
| **n-gon** | leque a partir do vértice 0 (`n−2` agulhas) | corte pelo **melhor ângulo** (o quad mais próximo de 90°), repetido |
| **hierarquia** | primeiro vizinho livre, por ordem de índice | emparelhamento por `(n_i·n_j)·razão_de_área`, decrescente; vértice grosso = média **ponderada pela área** |
| **pesos** | `1` em toda aresta | **cotangente** `½(cot α + cot β)`, carregados para cima somados |
| **núcleos** | acumulador começando com peso `1` no valor próprio | `weight_sum` começa em **zero** — o valor próprio é só a MOLDURA |
| **prolongação** | arredondava à retícula do filho | só **projeta** no plano tangente; quem arredonda é o núcleo |
| **varreduras** | 2 | **6** (`levelIterations` da referência) |
| **não-manifold** | *não existia* | **passo 11**: larga gulosamente toda face que reivindique uma aresta dirigida já com dono |

### O estado MEDIDO depois do porte, na malha que o módulo abre (98 306 vértices)

| | antes (invenção) | **depois (porte)** |
|---|---|---|
| `χ` | 2 | **2** |
| arestas não-manifold | 0 | **0** |
| arestas dirigidas repetidas | 0 | **0** |
| **maior face** | 5 | **5** |
| quads | 96,4 %* | **89,4 %** |
| volume | +4,42 | **+4,42** |
| relógio | 2,12 s | **2,12 s** |

\* ⚠️ **Os 96,4 % eram FABRICADOS.** Aquele número vinha de emparelhar
triângulos e fechar n-gons com um nó no meio — operações que **criam** quads a
partir de faces que a referência nunca emite. Era por isso que a métrica subia
enquanto a foto piorava: *uma régua que conta o que o próprio código inventou
não mede nada.*

### ⛔ Recusas MEDIDAS desta wave

| # | o que foi tentado | o que a medição disse |
|---|---|---|
| 13 | **a relaxação (Q3.6), que eu tinha construído** | com a extração portada ela **piora as três fixturas em tudo**: o Hausdorff da malha da cena vai de **0,60 para 1,49** quad. A grade do campo cruzado já está alinhada; um Laplaciano por cima briga com ela. **REMOVIDA** |
| 14 | fechar buracos só até 6 lados (a barra da referência) | sobravam laços de **12 a 40** lados e **7 arestas de borda** numa esfera — a peça vaza. Nós fechamos **todos**, e a divergência é de produto: a saída dela vai para um ficheiro, a nossa vai para as mãos do artista |
| 15 | mais varreduras para curar as singularidades | de 2 para 64 varreduras: **108 → 89** irregulares, 25× o relógio. O problema não era o número de varreduras |

### ⚠️ O que este porte NÃO cura, e é a Q4

O campo ainda produz **~100 nós irregulares** onde uma esfera admite **8**. Isso
é o defeito que a literatura nomeia e que o **fluxo de custo mínimo do
QuadriFlow** existe para curar — o código está baixado (`hjwdzh/QuadriFlow`,
`parametrizer-int.cpp` + `flow.hpp`) e **não** foi portado. *Enquanto ele não
for, a saída é quad-DOMINANTE, não all-quad, e este ADR não deve dizer outra
coisa.*

---

## §5-septies — ⚠️ O SLIDER OFERECIA O IMPOSSÍVEL, e a foto do smoke cobrou

> *"valores baixos de resolution destroem o objeto. a qualidade é em geral muito
> baixa"* — Enio, 2026-08-19, com foto: uma peça esfarrapada em cima e uma casca
> **espetada de raios** em baixo.

### O que a medição achou

Uma sonda que percorre o slider inteiro (`measure_where_quality_collapses`), com
o lado do quad em múltiplos da **aresta média da entrada**:

| razão | esfera 48×64 | uv 96×144 | uv 96×144 **amassada** (a cena `=35`) |
|---|---|---|---|
| 1,00× | **malha vazia** | **malha vazia** | **malha vazia** |
| 1,25× | **malha vazia** | **malha vazia** | **malha vazia** |
| 1,50× | ciclo de **62** | ciclo de **147** | ciclo de **352**, volume **1,59** de 3,78 |
| 1,75× | ciclo de 10 | ciclo de 20 | ciclo de **50** |
| 2,00× | ciclo de 7 | ciclo de 9 | ciclo de **39** |
| **3,00×** | ciclo de 6 | ciclo de 7 | **ciclo de 8** |
| 4,00× | 5 | 5 | 6 |

⭐ **A lei é de RECURSO e não de gosto: uma retopologia não pode resolver uma
grade mais fina que a malha que ela lê.** Cada vértice da saída é a média de um
punhado de vértices da entrada; peça um quad menor que isso e a célula fica com
um vértice, o campo não tem o que quantizar, o grafo sai com buracos, e o passeio
de faces os contorna em ciclos gigantes.

⚠️ **O painel oferecia `0,02` como mínimo, em unidades de OBJETO.** Sobre a malha
daquela cena isso é **0,66×** — fundo do poço. E o mesmo `0,02` seria conservador
numa malha fina: *o número não era da malha, e por isso queria dizer coisas
opostas em dois modelos.*

### As quatro correções

1. **O knob deixou de ser um tamanho.** `Quad Size` (`0,02 … 1,00`, unidades de
   objeto) virou **`Detail`** (`0 … 1`, fração do curso), e a
   `scale::edge_for_detail` converte **a partir da malha**, entre o piso
   (`FLOOR_IN_INPUT_EDGES = 3,0` arestas de entrada) e o teto
   (`MIN_QUADS = 100` faces, medido: 96 faces guardam 91 % do volume, 40 guardam
   68 %, 23 guardam 50 %). Interpolação **geométrica** — um knob de tamanho anda
   em razão constante. **Todo ponto do curso é legal por construção**, e há gate
   (`every_point_of_the_detail_slider_is_legal`, que já achou um `NaN` a escapar
   do `clamp`).
2. **O LEQUE morreu.** Um ciclo de `n` lados virava `n − 2` triângulos ancorados
   no vértice `0` — 42 agulhas a irradiar de um ponto, que é a casca espetada da
   foto. Agora o fecho é o padrão da indústria: um nó no centroide e
   `(v[i], v[i+1], v[i+2], centro)` de dois em dois ⇒ `n/2` **quads** (par) ou
   `(n−1)/2` quads + 1 triângulo (ímpar), com `Δχ = 0` por construção.
3. **O grafo passou a ser o da REFERÊNCIA.** `Linking::LatticeStep`: duas células
   são vizinhas quando alguma aresta da entrada as separa por **um** passo da
   retícula — o mesmo arredondamento que já decide o agrupamento (passo `(0,0)`),
   lido um degrau adiante. Fora o cone de 45° e a janela `[0,5 s, 1,7 s]`, que
   eram **os dois limiares onde a adivinha errava**. Medido dentro da faixa legal:
   fração de quads empatada (±1 pp) e **maior ciclo pela metade** (13→6, 12→5,
   10→5). E de brinde a extração passou de **~1,4 s para 0,02 s** na malha de
   98 306 vértices.
4. **A relaxação (Q3.6).** Laplaciano **tangente** + projeção de volta à
   superfície de entrada, 2 passadas (medido — ver `RELAX_PASSES`). Ganho
   **modesto e declarado**: −6 a −10 % de irregularidade, −8 % de Hausdorff.

### O estado depois, na malha que o módulo abre (98 306 vértices, no piso)

| | |
|---|---|
| quads | **97,6 %** |
| maior ciclo | **5** |
| `χ` | **2** · arestas não-manifold **0** · dirigidas repetidas **0** |
| volume com sinal | **+4,42** (para fora) |
| desvio de aresta | **0,067** |
| forma (Hausdorff) | **0,020** de um quad |
| relógio | **0,86 s** (kill-criterion: 3 s) |

### ⚠️ A A4 foi EMENDADA, e a emenda é a lição

A barra era **1 % da diagonal da bbox**. Ela media o **slider**, não o algoritmo:
a distância de Hausdorff de uma grade de lado `s` sobre uma superfície de raio `R`
não pode ser menor que a flecha `s²/8R` — ela **cresce com o quad pedido**, por
geometria. O mesmo código passava a `0,18` e reprovava a `0,25` no toro (`0,0131`
contra `0,01`). A barra passa a ser **um lado de quad**, que vale em todo ponto do
curso; medido, o pior caso usa **52 %** dela.

### ⛔ Recusas MEDIDAS desta wave

| # | o que foi tentado | o que a medição disse |
|---|---|---|
| 9 | ligar as células só pelo **passo da retícula**, também abaixo do piso | pior que o cone lá (ciclos de 77 e 109 a 1,5×) — a lei só vale onde a entrada resolve |
| 10 | subir `SWEEPS_PER_LEVEL` agora que o relógio sobrou | **satura em 2** (97,6 % · 97,5 % · 97,4 % · 97,5 % para 2/4/8/16), e 16 custa 5× |
| 11 | curar os triângulos que sobram com mais emparelhamento | **todos são isolados** — o guloso está esgotado, o resto é a Q4 |
| 12 | 4+ passadas de relaxação | esfera e toro **pioram** o desvio depois de 2 (0,160→0,166 · 0,163→0,179) |

### ⚠️ E uma nota que envelheceu, encontrada ao mexer aqui

O `SWEEPS_PER_LEVEL = 2` estava justificado por *"o último degrau que cabe no
kill-criterion de 3 s"*. A correção (3) cortou a extração 70×, o teto afastou-se,
e **o número ficou de pé por um motivo que já não existia**. A re-medição manteve
o 2 — agora por **qualidade**. *Quem move o número que tornava algo inalcançável
tem de reconferir a nota* (`CLAUDE.md` §0.0).

---

## §5-sexies — ⚠️ O GATE DA A2 ESTAVA PELA METADE, e o SMOKE achou

**Report do Enio (2026-08-19, duas fotos): a peça sai com BURACOS, e depois com
LEQUES.** O diagnóstico dele — *"provavelmente as normais estão invertidas"* —
estava certo, e o defeito passou por **todos** os gates.

⚠️ **Porque o gate da A2 media a coisa errada.** Ele contava **faces por
aresta** e chamava a isso *manifold*. A contagem **não vê a orientação**: duas
faces podem partilhar uma aresta e percorrê-la no **mesmo sentido** — a contagem
dá 2, o `χ` fecha, e as duas normais apontam para lados opostos. Do lado do
artista isso é a peça com buracos, porque o *backface culling* apaga metade dela.

**O gate novo afirma DUAS propriedades, e a segunda não decorre da primeira:**

1. **COERÊNCIA** — toda aresta DIRIGIDA aparece no máximo uma vez;
2. **SENTIDO** — o volume com sinal (teorema da divergência) é **positivo**. Uma
   malha perfeitamente coerente pode estar inteira do avesso.

Ele achou **dois** defeitos independentes, um por propriedade:

| # | defeito | medição |
|---|---|---|
| 1 | o **emparelhamento** montava o quad sempre como `(c, a, d, b)` — certo quando o triângulo vai de `a` para `b`, **invertido** quando vai de `b` para `a`. O sentido nunca era perguntado | **54** arestas dirigidas em duas faces |
| 2 | o passeio de faces tomava o vizinho **sucessor** na ordem angular; com `e2 = n × e1` (anti-horária vista de FORA) quem delimita a face é o **antecessor** | volume **−4,15** numa esfera, ou seja a malha inteira do avesso |

**Depois das duas curas, na malha do PRODUTO (98 306 vértices):** `χ = 2` · 0
arestas não-manifold · **0 arestas dirigidas repetidas** · volume **+4,43** ·
96,4 % de quads.

⚠️ **A lição é a mesma da régua, na quarta vez:** *um gate que conta não vê o
sentido.* E as fixtures dos gates (48×64) são menores que a malha do produto
(98 306) — o `measure_the_kill_criterion` passou a medir as quatro propriedades
**na malha que o artista de facto vê**, porque foi lá que o smoke achou.

---

## §5-quinquies — ✅ Q5 FECHADA, e o KILL-CRITERION cobrado

O botão **`Quad Retopology`** vive na seção *Topology*, ao lado do `Remesh`, com
duas pistas próprias (`Quad Size`, `Follow Curvature`). Ele entra na história
pela **mesma** entrada do voxel remesh (`StrokeUndo::Remeshed`), recusa com a
pilha de multires montada, e tem cena de smoke própria: **`=35`**.

⚠️ **O kill-criterion do §4 disparou, e a cura foi MEDIR** em vez de afrouxar.
Sobre a malha que o módulo abre (`sculpt_sphere`, **98 306 vértices**),
`edge = 0,05`:

| varreduras por nível | 1 | **2** | 4 | 8 |
|---|---|---|---|---|
| tempo | 1,10 s | **2,04 s** | 3,87 s | 7,52 s |
| quads | 95,9 % | **96,4 %** | 96,7 % | 96,5 % |

⇒ A qualidade **satura na primeira varredura** — a hierarquia entrega a cada
nível um campo já quase certo. Oito custavam **7×** por **+0,6 pp**, e a 8 o
número é *pior* que a 4 (ruído a dizer que ali não há sinal). **`SWEEPS_PER_LEVEL`
passou de 8 para 2**, e o passe custa **2,06 s** — o último degrau que cabe nos
3 s que este ADR congelou **antes** do build.

⚠️ **E o produto mede MELHOR que as fixtures dos gates:** **96,4 %** de quads na
malha real contra 85,3 % na esfera 24×36. As fixtures pequenas dão poucas células
por feição e o resíduo de borda pesa mais — o piso do gate sai da pior delas, de
propósito.

### ⛔ Uma oitava recusa MEDIDA

A contagem de quads era acumulada durante a construção das faces **e** corrigida
depois do emparelhamento (`non_quads -= pares * 2`). As duas grandezas nunca
foram a mesma — `non_quads` contava **ciclos** e os pares consomem **triângulos**
—, e o `usize` deu a volta: **18 446 744 073 709 551 613** não-quads num gate.
*Duas contagens da mesma coisa divergem no dia em que uma ganha um consumidor
novo; uma contagem derivada da fonte, não.* A contagem passou a sair da lista
final de faces.

⚠️ **A Q3 fecha com um número de não-quads, não com um zero.** Ele é a medida do
que a Q4 existe para curar — declarar zero antes do fluxo seria declarar que a
técnica base não tem o defeito que a literatura inteira nomeia.

### ⭐ O CONJUNTO DE ACEITAÇÃO DO §4, MEDIDO NO FIM DA JORNADA (2026-08-19)

| # | asserção | esfera 48×64 | toro 64×32 | estado |
|---|---|---|---|---|
| **A1** | all-quad | **89,0 %** | **92,6 %** | ⏳ **a única aberta** — o resto é o alvo do fluxo (Q4) |
| **A2** | manifold **e ORIENTÁVEL** | ✓ | ✓ | ✅ (⚠️ ver §5-sexies) |
| **A3** | gênero preservado | **χ = 2** (alvo 2) | **χ = 0** (alvo 0) | ✅ |
| **A4** | forma ≤ 1 % da diagonal | **0,23 %** | **0,24 %** | ✅ |
| **A6** | densidade adaptativa | ≥ 2× | — | ✅ |
| **A7** | determinístico | ✓ | ✓ | ✅ |
| **A8** | simetria 4-RoSy | ✓ | — | ✅ |

**As QUATRO peças que fecharam A2/A3/A4 e levaram a A1 a ~90 %, todas achadas
por medição:**

1. **A PODA DOS PENDENTES.** Um nó de grau 1 faz o passeio ir `a→b` e voltar
   `b→a` — um ciclo de DOIS, que não delimita área. Ele é descartado, mas as duas
   arestas dirigidas já foram consumidas e passam a não pertencer a face nenhuma:
   a soma dos lados deixa de ser `2E` e **χ sai 12 numa esfera**, onde o máximo de
   uma superfície conexa é 2. Poda iterativa (podar um pendente cria outro).
2. **A PARTIÇÃO DOS PINÇAMENTOS.** Um ciclo que visita o mesmo vértice duas vezes
   não é um polígono; leque-triangulá-lo põe a mesma aresta em três faces —
   medido: **6** arestas assim na esfera, e a soma dos lados a passar `2E` em 20.
   Partir o ciclo no vértice repetido é a leitura correta: o pinçamento é um
   vértice **não-manifold** no grafo de células, e cada folha fica com o seu
   polígono.
3. **A RÉGUA DA A4 ERA PONTO-A-VÉRTICE.** Um remesh devolve uma malha mais
   GROSSA, então um vértice da entrada está sempre a meia célula do vértice de
   saída mais próximo — **por construção**. A régua media a densidade da saída, não
   a fidelidade da forma. Com a distância **ponto-a-SUPERFÍCIE** (as sete regiões
   de Voronoi do triângulo), a mesma malha mede **0,23 %** em vez de 4,22 %.

4. **O EMPARELHAMENTO DE TRIÂNGULOS.** Os não-quads que sobravam **não são
   singularidades do campo** — as singularidades de uma grade vivem na VALÊNCIA
   dos vértices (3 ou 5), nunca no número de lados de uma face. Eles são resíduo
   da extração, e dois triângulos vizinhos **são** um quad com uma diagonal a
   mais. A operação preserva χ por construção (some uma aresta e some uma face),
   e a recusa por **normal** (cos 45°) impede um quad dobrado — o limiar diz de
   que recurso é: *a planaridade que um quad promete a quem o subdivide*.
   Medido: **80,6 % → 89,0 %** na esfera, **90,5 % → 92,6 %** no toro.

⚠️ **Três vezes nesta jornada a RÉGUA se corrigiu antes do algoritmo** (a fração
de quads por ciclos · a convergência do campo de posição · a Hausdorff por
vértices). *Um instrumento errado acusa o produto do defeito que ele próprio tem.*

### ✅ Q3 FECHADA — o que ela entregou, MEDIDO (2026-08-19)

| | esfera 48×64 | toro 64×32 |
|---|---|---|
| células (vértices de saída) | 957 | 799 |
| **quads** | **664 de 1 246 faces (53,3 %)** | **454 de 1 144 (39,7 %)** |
| não-quads | 426 | 420 |
| maior ciclo | 4 | 10 |
| χ (alvo) | **12** (2) | — |
| Hausdorff bilateral (barra 1 %) | **4,22 %** | — |

⇒ **A1 e A7 verdes. A2, A3 e A4 NÃO.** Os três gates ficam no repo com a barra
do §4 intacta e um `#[ignore]` que carrega **o número medido** no motivo —
⛔ **não os afrouxe**: eles são a definição de pronto do remesh, e afrouxá-los
trocaria o alvo pela medição de hoje.

### ⚠️ E a Q3 REFUTOU o plano de ondas deste ADR

**A hierarquia multirresolução não é um enfeite da Q2: ela é PRÉ-REQUISITO da
extração.** O Instant Meshes quocienta pela retícula porque o campo de posição
dele forma **platôs**; sem hierarquia o campo varia continuamente (medido:
`the_field_never_leaves_its_own_cell`), não há retícula partilhada a que agarrar,
e a Q3 teve de crescer células **por semente** — o que dá um resultado
quad-dominante mas não all-quad.

⚠️ **Dois caminhos morreram medidos pelo caminho, e ficam registados:**

| tentativa | medição que a matou |
|---|---|
| células por **`union-find` sobre um limiar de distância** | união é **transitiva**: uma corrente de arestas curtas fundiu a esfera de 3 072 vértices em **4 células**. E a corrente sempre existe quando o quad pedido é maior que a aresta da malha — o caso normal de um remesh |
| arestas de saída = **arestas da entrada que atravessam células** | a entrada é uma triangulação ⇒ ~6 vizinhas por célula ⇒ o passeio devolve **triângulos**: **7,2 %** de quads. A grade de quads precisa das **quatro** direções da cruz |

**O plano corrigido:**

| onda | conteúdo |
|---|---|
| **Q3.5** | a **HIERARQUIA** multirresolução (colapso de arestas + prolongação), e os campos resolvidos de cima para baixo |
| **Q4** | o **fluxo de custo mínimo** — e só então A2/A3/A4 são exigíveis |
| **Q5** | a costura no shell |

---

## §5-bis — ⛔ Q3.5 CONSTRUÍDA, MEDIDA e REJEITADA do caminho do produto

A hierarquia foi construída (`hierarchy.rs` + `solve.rs`, 3 gates verdes) e **a
medição refutou a conclusão da Q3 que a pediu**.

| campo × células | esfera | toro |
|---|---|---|
| **plano + semente** (o produto) | **53,3 %** | **39,7 %** |
| hierarquia + semente | 38,9 % | 37,2 % |
| plano + **retícula** | 0 % (**1** célula) | 0 % (5 células) |
| hierarquia + **retícula** | 0 % (**2** células) | 0 % (9 células) |

E a varredura de `(topo × varreduras)` — **24 combinações** — nunca passou de
**52,3 %**: nenhum ajuste faz a hierarquia ganhar.

⚠️ **Por que a conclusão da Q3 estava errada, e a lição vale mais que a wave:**
ela supôs que a extração **usava** a retícula, e que por isso um campo com platôs
a ajudaria. Ela **não usa** — o crescimento por semente lê o campo como uma
DISTÂNCIA, e um campo mais suave não muda distância nenhuma. *A hipótese não era
sobre a hierarquia: era sobre a extração, e a Q3.5 testou a metade errada.*

⚠️ **E o quociente pela retícula — a leitura NATURAL da referência — colapsa por
aritmética**, não por afinação: o campo fica a menos de `s/√2` do seu vértice
(gate) e os vértices da entrada distam **muito menos que uma célula** (0,098
contra 0,18 na fixture), então o passo inteiro entre duas retículas vizinhas é
`(0,0)` em toda parte e o `union-find` funde tudo.

⇒ **O lever da Q4 é a EXTRAÇÃO, não os campos.** A hierarquia fica no repo
gateada e correta — ela é o andaime de que uma extração baseada em retícula vai
precisar —, **fora do caminho do produto**, e o gate
`the_hierarchy_does_not_pay_yet_and_the_gate_says_so` impede que alguém a ligue
sem re-medir.

### ⛔ Recusas MEDIDAS desta wave

| # | recusa | número |
|---|---|---|
| 1 | células por `union-find` sobre limiar de distância | esfera → **4 células** |
| 2 | arestas de saída = arestas da entrada | **7,2 %** de quads |
| 3 | coarsening por **média** de posição/normal | encolhe o modelo; perde em **24/24** combinações |
| 4 | ~~**hierarquia** no caminho do produto~~ | ⚠️ **REFUTADA pelo §5-quater** — com o porte fiel ela GANHA (76,9 % vs 42,9 %) |
| 5 | ~~células pelo **quociente da retícula**~~ | ⚠️ **REFUTADA pelo §5-quater** — com o porte fiel não colapsa, e é o PRODUTO |
| 6 | arestas por escolha **MÚTUA** (valência ≤ 4 por construção) | remove arestas de mais: ciclos de **31** lados, **53,3 % → 35,0 %** |
| 7 | semeadura de **POISSON** (ponto mais distante) | células regulares ≠ grade melhor: **53,3 % → 48,9 %** |

---

## §5-quater — ⭐ O PORTE FIEL VIROU A MESA, e ele REFUTOU DUAS RECUSAS MINHAS

**MEDIDO 2026-08-19, depois de ler o `src/field.cpp` da referência:**

| campo × células | esfera | toro |
|---|---|---|
| plano + semente (o que a Q3 entregava) | 42,9 % | 43,4 % |
| **hierarquia + retícula** (o produto agora) | **76,9 %** | **86,6 %** |

### A peça que faltava

O `compat_position_extrinsic_4` desta crate **não era o da referência**. Ele
arredondava **cada lado ao ponto médio, independentemente**. O da referência
(`field.cpp`) faz outra coisa:

1. `position_floor_index_4` — o **`floor`** dá a CÉLULA em que o ponto médio caiu;
2. as **quatro quinas** dessa célula são enumeradas **de cada lado**;
3. escolhe-se o **PAR** (de 16) que minimiza a distância **entre si**.

⇒ É o passo 3 que **puxa uma retícula até à outra**: a quina escolhida pode estar
a um passo inteiro do nó mais próximo do próprio vértice, e é esse passo que vira
o **degrau**. Sem degraus não há platôs.

### ⚠️ E isto REFUTA as recusas 4 e 5 deste ADR

As duas — *"a hierarquia não paga"* e *"o quociente pela retícula colapsa"* —
eram consequência de **UM operador mal portado**. Com o porte fiel a hierarquia
ganha por **1,8×** e o quociente não colapsa (433 células, não 1).

**A lição, e ela vale mais que a wave:** *uma medição só refuta o que ela de facto
exercitou* — e o que aquelas exercitavam era a **minha aproximação**, não a lei da
referência. É exatamente por isso que a
[DIRETIVA §1](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) manda
**portar** o algoritmo publicado antes de escrever a própria versão.

⚠️ **O gate `the_hierarchy_pays_and_the_number_is_here` foi escrito a afirmar o
CONTRÁRIO**, com a mensagem *"NÃO apague este gate: mude-o, com o número novo ao
lado"* — e foi assim que a virada foi apanhada em vez de passar despercebida.

### ⚠️ Uma divergência DECLARADA que fica

O critério de colapso da referência compara os **índices inteiros**
(`extract_graph`: `(shift.first − shift.second).abs().sum() == 0`). O porte
literal existe (`compat_position_extrinsic_index_4`, gateado) e **MEDIDO dá
19,3 %** de quads, com χ = 280 e ciclos de 63 lados. O que shipa é o mesmo
critério expresso **num referencial só** (*o campo de `w`, trazido à retícula de
`v`, anda?*) — **76,9 %**. Os índices de cada lado vivem na retícula do seu
vértice, e igualá-los pressupõe um referencial partilhado que a nossa otimização
— sem o limiar de `error` e sem os passes de limpeza do `extract_graph` — ainda
não garante. **Ligar o critério literal é trabalho da Q4.**

---

## §5-ter — ⚠️ A RÉGUA MENTIU, e ela se corrigiu antes do algoritmo

A `quad_fraction` media `quads / (quads + ciclos não-quad)` — e um ciclo de **31
lados** contava como **UM** não-quad enquanto virava **29 triângulos** na malha.

⇒ **Ela melhorava quando as falhas ficavam maiores.** A tentativa da escolha
mútua trocou 582 triângulos por 918 e a métrica **subiu de 60,9 % para 71,9 %**.
Sob a régua honesta — sobre as faces EMITIDAS — aquela mudança era **53,3 % →
35,0 %**.

⚠️ **Todos os números deste ADR anteriores a esta nota foram re-medidos.** O que
a Q3 entregou não é 60,9 %/51,9 %: é **53,3 %/39,7 %**. A direção das cinco
recusas não mudou (as duas pontas de cada comparação usavam a mesma régua), mas
os valores absolutos sim.

*Uma régua que premeia o defeito por ele ser grande é pior que nenhuma.*

---

## §6 — O que este ADR NÃO decide

- ⛔ **Não substitui o `surface_nets`** (§1): as duas operações ficam, e a UI as
  oferece separadas. Fundir as duas num botão só é a pergunta *"que remesh?"*
  respondida por omissão.
- ⛔ **Não abre `rayon`** na crate nova. A suavização é um map disjunto por
  vértice e vai querer paralelismo — mas isso é ADR próprio, como o
  [0156](0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md) e o
  [0159](0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md)
  já foram cada um por si.
- ⛔ **Não decide a UI.** Onde o botão mora, e se ele substitui a peça ou cria uma
  nova, é da Q5 e do Enio.

---

## §7 — Fontes

- [QuadriFlow (hjwdzh/QuadriFlow)](https://github.com/hjwdzh/QuadriFlow) — licença, pipeline, dependências.
- [Instant Meshes (wjakob/instant-meshes)](https://github.com/wjakob/instant-meshes) — `LICENSE.txt` = BSD-3-Clause.
- [Compilação de trabalhos em quad meshing](https://github.com/Bigger-and-Stronger/quad-meshing-survey) — o levantamento contínuo do campo.
- [Reliable feature-line driven quad-remeshing (QuadWild)](https://www.researchgate.net/publication/353626734_Reliable_feature-line_driven_quad-remeshing) — a família (a).
- [Quad Remesh do Houdini](https://www.sidefx.com/docs/houdini/nodes/sop/quadremesh.html) — a formulação de produto do sizing adaptativo (*"more smaller quads in regions with many local features"*).
