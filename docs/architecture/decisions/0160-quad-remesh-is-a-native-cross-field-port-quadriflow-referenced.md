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
| A4 | a **forma** sobrevive | distância de Hausdorff bilateral ≤ **1 %** da diagonal da bbox |
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

⚠️ **A Q3 fecha com um número de não-quads, não com um zero.** Ele é a medida do
que a Q4 existe para curar — declarar zero antes do fluxo seria declarar que a
técnica base não tem o defeito que a literatura inteira nomeia.

### ✅ Q3 FECHADA — o que ela entregou, MEDIDO (2026-08-19)

| | esfera 48×64 | toro 64×32 |
|---|---|---|
| células (vértices de saída) | 957 | 799 |
| **quads** | **664 de 1 246 faces (53,3 %)** | **454 de 1 144 (39,7 %)** |
| não-quads | 426 | 420 |
| maior ciclo | 4 | 10 |
| χ (alvo) | **5** (2) | **2** (0) |
| Hausdorff bilateral (barra 1 %) | **3,14 %** | — |

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
| 4 | **hierarquia** no caminho do produto | **38,9 %** contra 53,3 % |
| 5 | células pelo **quociente da retícula** | **1** célula na esfera |
| 6 | arestas por escolha **MÚTUA** (valência ≤ 4 por construção) | remove arestas de mais: ciclos de **31** lados, **53,3 % → 35,0 %** |

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
