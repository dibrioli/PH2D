# PLAN — Quad Remesher estado-da-arte na PH2D

> **Documento VIVO.** Decisão e fronteira jurídica: [ADR-0162](../../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md).
> O que o porte local entregou e por quê: [ADR-0160](../../architecture/decisions/0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md).
> ✅ **Aprovado pelo Enio em 2026-08-20** (*"Siga como achar melhor... buscamos o estado da arte independente dos custos"*).
> Estado: **F1..F5 FEITAS** (§4-bis, §4-ter, §4-quater, §4-quinquies, §4-sexies).
> ⭐ **A cadeia devolve MALHA, com código nosso e com os números do pivô**: 100 % de quads, característica
> de Euler exata, e os vértices irregulares de **39,7 % para 0,5 %** — um fator de ~85 sobre o motor que
> ela substitui, e a mesma ordem de grandeza do oráculo (que fica em 0,2 %).
> ⭐ **2026-08-21:** a não-variedade que travava o `cube` era do **flip** da `ph2d-mesh`, curada e gateada
> (§4-septies) — e com ela a cadeia passou a fechar também na esfera **sacudida** (> 20 min → 1,6 s) e na
> **ruidosa**.
> ⛔ **Mas o mesmo trabalho revogou o título do §4-ter:** o campo do F2 só chega ao ótimo em malhas bem
> distribuídas — sobre distribuição irregular ele passa de **8** para **194** singularidades (§4-octies).
> ⭐ **E a RÉGUA estava errada** (§4-nonies): faltava o defeito angular na fórmula do índice, o que só se
> via em malha com triângulos de tamanhos muito diferentes. Corrigida, Poincaré–Hopf passa a valer
> **exactamente** em todo o corpus — incluindo o `cube`, que era a última malha com a soma errada.
> ⭐ **E a proveniência dos irregulares está medida** (§4-decies): **100 % vêm do layout do F3** —
> zero em arco, raio e grade.
> ⭐⭐ **E o F3-bis fechou (§4-undecies): 47 → 14 numa esfera**, contra um chão de 8 e o oráculo em ~7.
> A causa era **cantos-artefacto**: metade dos cantos do layout não tinha estrutura nenhuma por baixo —
> a parede zigue-zagueia sobre arestas de malha e vira 90° sem que nada vire. A lei nova é *um canto só
> existe onde a parede se ramifica*.
> Próximo: os **patches de três lados** (a fusão de separatrizes do QuadWild §6) e a **porta no shell**.
> ⛔ **2026-08-21, o Enio mandou a foto: o produto saiu DESTRUÍDO com os 10.515 gates verdes.** Auditoria
> multi-agente, causa raiz numa linha, três camadas de cura, e a lição que fica: **nenhuma asserção desta
> cadeia olhava uma coordenada** (§4-terdecies).
> ⭐ **E a foto seguinte foi uma grade de verdade a seguir o fluxo de uma orelha** — com **fendas**: faces
> DOBRADAS, que não movem nenhuma outra régua. Metade delas caiu ao trocar a lei de peso do arco
> (§4-quaterdecies). ⛔ **E o que separa isto do estado da arte tem nome, MEDIDO (§4-quindecies):
> PARAMETRIZAÇÃO POR PATCH.** A dobra não é da projeção nem do grão — é do **leque de Coons sobre um
> patch grande**, e onde o layout é bom o alisamento cura até **0,0 %**, onde é mau nem 96 rondas movem a
> agulha.

---

## 1 — Relatório de preparação (Passo 0: FEITO)

### 1.1 O oráculo compila e roda

`cgg-bern/quadwild-bimdf` clonado com submódulos em
`/home/enio/Documentos/Projetos/ph2d-quadbench/oracle` (533 MB), compilado com
`-DSATSUMA_ENABLE_BLOSSOM5=0 -DQUADRETOPOLOGY_WITH_GUROBI=OFF -DCMAKE_BUILD_TYPE=Release`.
**Exit 0.** Binários: `quadwild`, `quad_from_patches`, `cli_trace`, `viz_mesh_results`.

⚠️ **Os dois binários têm de rodar a partir da RAIZ do oráculo.** Os configs referenciam outros
configs por caminho **relativo** (`config/main_config/flow_virtual_simple.json`); de outro `cwd` o
segundo estágio morre com *"Could not open config file"* e **exit 0** — falha silenciosa. O
`run_oracle.sh` já faz o `cd`.

### 1.2 Corpus (10 malhas) e saídas de referência: ARQUIVADAS

Escrito por `shells/desktop/src/sculpt3d_corpus.rs` (`#[cfg(test)]`, `#[ignore]`), porque as fixturas
de escultura são desenhadas com os **verbos do produto** e não existem fora do shell.

| malha | vértices | triângulos | por que está no corpus |
|---|---|---|---|
| `cube` | 8 | 12 | o degenerado: entrada mais grossa que qualquer quad pedido |
| `sphere_uv_96x144` | 13 682 | 27 360 | o controle liso |
| `torus_64x32` | 2 048 | 4 096 | gênero 1 — um remesh que costura o buraco reprova aqui |
| `sphere_sculpt_98k` | 98 306 | 196 608 | a malha que o módulo **abre** |
| `sphere_noisy` | 13 682 | 27 360 | ruído sem feature — o pior caso do campo |
| `sphere_shuffled` | 13 682 | 27 360 | ⚠️ a mesma forma com a **DISTRIBUIÇÃO** torta (jitter tangencial + reprojeção) |
| ⭐ `sculpt_hooked` | 3 386 | 6 768 | **a esfera-com-bico do diagnóstico** — o gate de regressão do §9 |
| `sculpt_wrinkled` | 13 682 | 27 360 | sete sulcos (`Crease`) |
| `sculpt_ridged` | 13 682 | 27 360 | cristas |
| `sculpt_punctured` | 738 | 1 340 | chega **quebrada** (faces arrancadas) — o caso da sanitização |

### 1.3 ⭐ A BASELINE, medida — e é ela que justifica o pivô

Régua: `ph2d-quadbench/metrics.py` (lê OBJ e nada mais; não linka o oráculo).
Nosso: defaults do painel (`detail = 0,50`, `adapt = 0,00`). Oráculo: `basic_setup_Organic` + `flow_noalign_lemon`.

| malha | quads (nosso → oráculo) | **vértices irregulares** (nosso → oráculo) | desvio angular médio |
|---|---|---|---|
| `sphere_uv_96x144` | 68,7 % → **100,0 %** | **39,7 % → 0,2 %** | 11,38° → 5,48° |
| `sculpt_hooked` ⭐ | 70,5 % → **100,0 %** | **40,5 % → 0,3 %** | 11,30° → 7,71° |
| `sculpt_wrinkled` | 83,3 % → **100,0 %** | **23,2 % → 0,2 %** | 8,54° → 4,49° |
| `sculpt_ridged` | 75,3 % → **100,0 %** | **27,6 % → 0,4 %** | 8,28° → 6,41° |
| `torus_64x32` | 64,9 % → **100,0 %** | **48,9 % → 0,0 %** | 9,38° → 2,02° |
| `sphere_sculpt_98k` | 82,7 % → **100,0 %** | **21,2 % → 0,2 %** | 5,68° → 5,20° |
| `sphere_shuffled` | 74,8 % → **100,0 %** | **29,7 % → 0,2 %** | 6,74° → 5,73° |
| `sphere_noisy` | 78,0 % → **100,0 %** | **29,4 % → 4,5 %** | 6,89° → **16,21°** |
| `cube` | **saída vazia** | — | — |
| `sculpt_punctured` | 76,0 % → 100,0 % | 34,4 % → 2,2 % (χ=1, **88 arestas de borda**) | — |

⚠️ **CORREÇÃO (2026-08-21): o `sphere_shuffled` NÃO embaralha ordem de índice.**
[`shapes::uv_sphere_shuffled`] sacode cada vértice **tangencialmente** e reprojeta
— a forma fica exacta e a **distribuição** é que fica torta. ⇒ Duas consequências
que este plano vinha a arrastar:

1. ⛔ **O corpus não tem controle de determinismo nenhum**, e o §4-sexies atribuiu
   a uma dependência de ordem (*"o passeio é guloso e a ordem das sementes decide
   quem pára em quem"*) um defeito que é de **distribuição de triângulos**. *Uma
   fixtura só prova o que ela contém, e o nome dela não é a régua.*
2. ⭐ **O que ela de facto mede é o regime em que o F2 se parte** — ver §4-octies.

⭐ **A grandeza que decide é a coluna do meio.** Uma grade de quads numa esfera admite **oito**
vértices irregulares. Nós entregamos **21 a 49 % de todos os vértices**; o oráculo entrega **0,2 %**.
São **duas ordens de grandeza**, e é exatamente o sintoma 1 do diagnóstico (*"singularidades
espalhadas sem controle"*).

⚠️ **Três leituras honestas que a tabela obriga:**

1. **`cube` devolve malha VAZIA no nosso.** Não é bug: a nossa entrada tem 8 vértices e o piso do
   `edge_for_detail` é 3 arestas de entrada. **O oráculo devolve 4 818 vértices** — porque ele
   **remalha isotropicamente ANTES** (estágio 2 do pipeline dele). *Nós não temos esse estágio, e é
   ele que torna a entrada irrelevante.* Isto sozinho explica por que o oráculo é indiferente à
   densidade e à qualidade da malha que recebe.
2. **`sphere_noisy` é o único ponto em que o oráculo é PIOR** (desvio angular 16,21° contra 6,89°) —
   ruído sem feature é caso difícil para a família global também. Vale como cerca: **não prometer
   superioridade universal**.
3. **`sculpt_punctured` sai com 88 arestas de borda no oráculo** — ele **não** fecha um buraco de
   entrada. A sanitização (estágio 1) é nossa responsabilidade e não é opcional.

### 1.4 Papers em `ph2d-quadbench/docs/papers/`

| paper | estado |
|---|---|
| Pietroni et al., *Reliable Feature-Line Driven Quad-Remeshing*, SIGGRAPH 2021 | ✅ 38 MB |
| Heistermann et al., *Min-Deviation-Flow in Bi-Directed Graphs*, SIGGRAPH 2023 | ✅ 54 MB |
| Bommes et al., *Mixed-Integer Quadrangulation*, 2009 | ✅ 9 MB |
| Jakob et al., *Instant Field-Aligned Meshes*, 2015 | ✅ 113 MB |
| Ebke et al., *QEx: Robust Quad Mesh Extraction*, 2013 | ⛔ **NÃO obtido** — paywall ACM |

⚠️ **O QEx é o menos crítico** (fundo teórico de extração; o F5 quadrangula **por patch**, não por
mapa global), mas ele fica como item aberto: sem ele, as decisões de extração no F5 saem do QuadWild
2021 §6 e de experimentação com o oráculo.

### 1.5 Mapa do remesher atual e a superfície de integração

| peça | onde | o que acontece com ela |
|---|---|---|
| campo + extração | `crates/ph2d-quadflow/` (5 114 LOC) | **FICA** — vira `RemeshBackend::Preview` (BSD, ADR-0160) |
| voxel remesh | `crates/ph2d-sdf/` (3 011 LOC) | **FICA**, é outro produto (arrumação destrutiva) |
| a única porta | `Sculpt3dScene::quad_remesh(detail, adaptive)` em `sculpt3d_history_remesh.rs` | **é a costura inteira** — 5 chamadas a `ph2d_quadflow::*`, num só sítio |
| undo | `StrokeUndo::Remeshed(Box<Mesh>)` | inalterado: um remesh não partilha estrutura com o que estava lá |
| UI | `SCULPT3D_QUAD_REMESH` + `Detail` + `Follow Curvature` | ganha o seletor de backend |
| malha | `crates/ph2d-mesh/` (20 824 LOC) — `Mesh`, octree, `Adjacency` | **reusar**; ⛔ não criar uma segunda half-edge |
| IO | `ph2d_mesh::{import_obj, write_obj}` | já existe — o harness usa esta porta |

⚠️ **A superfície é pequena e isso é a boa notícia**: um backend novo entra por **uma** função.

---

## 2 — Inventário de crates (licença e decisão)

| precisa de | candidata | licença | decisão |
|---|---|---|---|
| Cholesky esparso (campo cruzado) | **`faer`** | MIT | **usar** — é a recomendação do briefing e a única madura em Rust puro |
| álgebra densa pequena (3×3, SVD) | `nalgebra` | Apache-2.0 | **usar** |
| grafos (fluxo, tracing, patches) | `petgraph` | MIT/Apache-2.0 | **usar como ESTRUTURA** |
| min-cost-flow / network simplex | **nenhuma** | — | ⚠️ **implementar do zero** (Bi-MDF é o paper de 2023; LEMON é Boost-licensed e serve de **referência de leitura**, mas é C++ e não se linka a um núcleo Rust sem FFI) |
| half-edge / topologia | **`ph2d-mesh`** (interno) | própria | **reusar** — ⛔ o briefing manda avaliar antes de criar outra, e a resposta é *não crie* |
| paralelismo | `rayon` (já no stack) | MIT/Apache-2.0 | **usar só** em estágios com redução determinística documentada |
| determinismo de mapa | `BTreeMap`/`BTreeSet` (std) | — | **obrigatório** (HR-5, já é lei do repo) |
| ⛔ proibidos no núcleo | CoMISo · vcglib · libQEx · quadwild | **GPL3** | **só como oráculo externo** |

⚠️ **O item de maior risco é a linha vazia**: não existe min-cost-flow permissivo em Rust com a
generalidade que o Bi-MDF pede (grafo **bi-dirigido**, custos convexos por arco). É a F4, e é a fase
crítica.

---

## 3 — Fichamento dos papers, por estágio

| # | estágio | paper-fonte | o núcleo | lacuna que exige o oráculo |
|---|---|---|---|---|
| 1 | sanitização | — | manifold, orientação, degenerados | qual tolerância o oráculo usa para "degenerado" |
| 2 | **remesh isotrópico** | QuadWild 2021 §4 | split/collapse/flip/relax com reprojeção | ⚠️ **o alvo de aresta**: o oráculo devolve ~4 500 vértices de um cubo E de uma esfera de 98 k — a densidade dele **não vem da entrada**; de onde vem? |
| 3 | features | QuadWild 2021 §5 | ângulo diedral + linhas marcadas | o limiar diedral do preset `Organic` vs `Mechanical` |
| 4 | **cross field** | Bommes 2009 (MIQ) + QuadWild 2021 §5 | energia de suavidade + matching inteiro, rounding greedy | ⚠️ a **ordem** do rounding greedy e o critério de parada — o paper descreve, o comportamento é o que decide onde as singularidades caem |
| 5 | tracing / patches | QuadWild 2021 §6 | separatrizes a partir de singularidades e features | tratamento de separatriz que não fecha |
| 6 | ⭐ **quantização Bi-MDF** | Heistermann 2023 | fluxo de desvio mínimo em grafo bi-dirigido | ⚠️ a construção do grafo a partir do layout, e a função de custo por lado |
| 7 | quadrangulação por patch | QuadWild 2021 §7 | padrões conformes por assinatura de lados | o catálogo de padrões para 3/4/5/6 cantos |
| 8 | smoothing | QuadWild 2021 §8 | Laplaciano restrito + reprojeção | quantas iterações, e a restrição de feature |

⚠️ **A lacuna do estágio 2 é a primeira coisa a medir**, e ela é grande: o oráculo produziu
**4 818 vértices de um cubo de 8**. Enquanto essa lei não for entendida, nenhum estágio a jusante é
comparável — nós estaríamos a alimentar o pipeline com outra malha.

---

## 4 — Fases, com critério de aceitação MEDIDO

Cada fase fecha com o benchmark verde sobre o corpus e um sumário curto de desvios.

| fase | o que entrega | ✅ aceita quando |
|---|---|---|
| **F0** | harness na bancada: corpus + oráculo + `metrics.py` + Hausdorff + screenshots | as três colunas (atual · oráculo · nova) saem de **um** comando; ⚠️ a baseline já está medida (§1.3) — falta o Hausdorff e o harness sair do `#[cfg(test)]` do shell |
| **F1** | sanitização + **remesh isotrópico** + sizing field | ✅ **FEITO em 2026-08-20** (`crates/ph2d-remesh-iso`) — ver §4-bis |
| **F2** | cross field MIQ-style + streamlines no viewport | ✅ **O CAMPO FEITO em 2026-08-20** (`crates/ph2d-crossfield`) — ver §4-ter. ⚠️ O critério *«irregulares ≤ 2 %»* era sobre a MALHA, e a malha só melhora no F5 |
| **F3** | tracing + patches | ✅ **FEITO em 2026-08-20** (`crates/ph2d-trace`) — ver §4-quinquies. O layout que ele produz é **quantizado com prova** pelo F4 em esfera e toro; falta a fusão de separatrizes (saem ~2× mais patches que o oráculo) e as *feature lines* |
| **F4** | ⭐ **solver Bi-MDF** | ✅ **PROTÓTIPO FEITO em 2026-08-20** (`crates/ph2d-quantize`) — ver §4-quater. Fecha com o **ótimo demonstrado** em todos os layouts fechados do oráculo; falta só o consumidor (F5) e a válvula de emergência, que **nenhum layout pediu** |
| **F5** | quadrangulação por patch + smoothing + a porta no shell | ✅ **A MALHA FEITA em 2026-08-20** (`crates/ph2d-quadfill`) — ver §4-sexies: 100 % quads, χ exata, irregulares ~85× abaixo do motor local. ⛔ **Falta a porta no shell**, o Hausdorff/desvio angular, e o gate de regressão da `sculpt_hooked` (que precisa das *feature lines* do F3) |
| **F6** | guide strokes: direção → feature → densidade | ⚠️ **a densidade por PRESSÃO depende da camada de tablet, que NÃO existe** (ADR-0162) — F6 entrega direção e feature; a pressão é um projeto irmão |
| **F7** | dois backends (preview BSD + qualidade) partilhando o campo | preview < 1 s até 100 k triângulos; o preview mostra o alinhamento que o modo qualidade honra |
| **F8** | invalidação incremental + pinning de singularidade | editar um stroke não custa o pipeline inteiro; ⚠️ **exige infraestrutura de DAG que o Sculpt não tem** |

⚠️ **F6 e F8 estão precificadas com o defeito de premissa embutido** (ADR-0162): elas não são
"mais uma fase do remesher", são infraestrutura de produto que hoje não existe.

---

## 4-bis — ✅ F1 FEITA, e a medição REFUTOU a esperança que a acompanhava

`crates/ph2d-remesh-iso` — split/collapse/flip/relax-tangencial com reprojeção,
**clean-room** (Botsch & Kobbelt 2004; QuadWild 2021 §4). ⚠️ **Os três primeiros
passos já existiam na engine, testados** (`refine_in_sphere`,
`collapse_in_sphere`, `dyntopo_flip`): o que o crate acrescenta é o passe
**global** e a reprojeção. *O plano mandava conferir antes de construir outra
estrutura de malha, e a resposta foi: não construa.*

**A propriedade entregue** (gate `the_output_density_does_not_depend_on_the_input_density`):

| entrada | vértices antes | depois | aresta / alvo |
|---|---|---|---|
| cubo | **8** | 4 857 | 1,09× |
| esfera 24×36 | 830 | **2 544** | 1,09× |
| esfera 96×144 | **13 682** | **2 608** | 1,09× |

⭐ **830 e 13 682 saem a 2,5 % um do outro.** A densidade da saída deixou de
depender da entrada, e as três chegam ao **mesmo múltiplo do próprio alvo**.

### ⛔ E o que ela NÃO comprou — o número que muda o plano

Corpus inteiro, o nosso remesher **sem** e **com** o F1 na frente:

| malha | quads sem → com | **irregulares sem → com** | oráculo |
|---|---|---|---|
| `cube` | **vazia** → **100,0 %** | — → **3,7 %** | 0,2 % |
| `sculpt_hooked` ⭐ | 70,5 → 80,6 % | 40,5 → **32,0 %** | 0,3 % |
| `sculpt_punctured` | 76,0 → 81,5 % | 34,4 → 29,6 % | 2,2 % |
| `sphere_uv_96x144` | 68,7 → 72,9 % | 39,7 → 35,3 % | 0,2 % |
| `torus_64x32` | 64,9 → 66,2 % | 48,9 → 45,3 % | 0,0 % |
| `sculpt_ridged` | 75,3 → 77,3 % | 27,6 → **28,7 %** ⚠️ | 0,4 % |
| `sphere_sculpt_98k` | 82,7 → 76,5 % | 21,2 → **33,5 %** ⚠️ | 0,2 % |
| `sphere_shuffled` | 74,8 → 74,6 % | 29,7 → **33,1 %** ⚠️ | 0,2 % |

⭐ **O F1 cura exatamente UM caso — o `cube`, que não tinha entrada para
resolver — e nos outros nove move a agulha alguns pontos, para os DOIS lados.**

⛔ **E essa única cura CAIU em 2026-08-20** (§4-quinquies): o `cube` que o F1
devolve tem **18 vértices não-manifold**, medidos pelo [`TraceReport::open_rings`]
do F3. A célula acima foi obtida sobre uma malha quebrada — os 100 % de quads dela
não descrevem nada. *As outras nove linhas não dependem dela e continuam a valer;
a tese do pivô, que é o que esta secção existe para provar, sai reforçada e não
enfraquecida — o F1 cura ainda menos do que se pensava.*

⇒ **A tese do pivô está agora MEDIDA, não afirmada.** As 20–45 % de
singularidades **não são a malha de entrada**: são a **classe** do algoritmo. Se
fossem da entrada, uniformizá-la teria de as derrubar, e ela não derruba.

⚠️ **Consequência de plano, e é a razão de esta seção existir:** ⛔ **não gastar
mais nenhuma jornada a afinar o F1.** Ele é **pré-requisito** do pipeline global
(o oráculo depende dele para ser indiferente à entrada), não uma melhoria do
local. Os levers são **F2** (campo cruzado com rounding inteiro global) e **F4**
(quantização Bi-MDF).

⚠️ **E por isso o F1 NÃO foi ligado ao produto.** Ele piora três das dez malhas
com o extrator local, e ligá-lo agora seria trocar um número por outro sem
ganho: ele entra **junto com o F2**, que é quem o consome.

### O que ficou aberto no F1

1. ⛔ **A metade ADAPTATIVA da lei de densidade.** Medido: sobre o cubo (plano) o
   oráculo bate `alpha × diagonal` quase exato (0,0346 pedido, 0,0356 medido);
   nas fixturas **curvas** ele termina **mais fino** (0,0566 contra 0,0693 na
   esfera). Ele refina abaixo do alvo onde a curvatura pede, e essa metade não
   está portada.
2. A **sanitização** (manifold/orientação/degenerados) ainda não existe como
   estágio próprio — `sculpt_punctured` entra quebrada e sai quebrada.

---

## 4-ter — ⚠️ F2: o campo chega ao ótimo em malhas BEM DISTRIBUÍDAS — e só nelas

> ⛔ **Título REVOGADO em 2026-08-21.** Ele dizia *"o CAMPO chegou ao ótimo
> teórico"*, e isso foi medido **só em grades `uv`**, que são o caso mais fácil que
> existe: a própria grade já é um campo cruzado perfeito. Sobre distribuição
> irregular a contagem passa de **8** para **194**, e em duas malhas do corpus a
> soma dos índices **viola Poincaré–Hopf**. A medição inteira está no §4-octies, e
> ela é o próximo trabalho da linha. *O resto desta secção continua a valer — o que
> cai é a extrapolação do título.*

`crates/ph2d-crossfield` — MIQ (Bommes et al. 2009) **clean-room**: campo por
FACE, saltos de período **inteiros** por aresta dual, gauge de árvore geradora,
rounding guloso em lote.

### ⭐ A régua nova: o índice, medido no CAMPO e não na malha

⚠️ **Todas as medições anteriores desta linha contavam vértices irregulares da
malha extraída — que é o campo MAIS o extrator.** Um número ruim ali não diz de
quem é a culpa. O índice de singularidade sai do campo direto, e tem uma
invariante topológica que serve de oráculo: **`Σ índice = 4·χ`**
(Poincaré–Hopf), `8` numa esfera e `0` num toro. *Ela não depende do solver, da
malha nem dos pesos.*

### ⛔ E a primeira tentativa PASSOU no gate topológico sendo péssima

A alternância ingênua *(resolve `θ` com `p` fixo → arredonda `p` com `θ` fixo)*,
partindo de `p = 0`, **converge na primeira rodada e não faz nada**: os resíduos
ficam todos abaixo de `π/4` e nenhum `p` chega a mudar.

| | alternância | **MIQ com gauge + rounding** | ótimo teórico |
|---|---|---|---|
| esfera 24×36 | **2** singularidades, índices `+4 +4` | **8**, todas `+1` | **8** |
| esfera 48×64 | **2**, `+4 +4` | **8** | **8** |
| cubo subdividido | **2**, `+4 +4` | **8** | **8** |
| toro | 0 | 6 (soma 0) | 0 |
| energia | 7,67 | **4,48** | — |

⚠️ **A soma passava nas duas** (`Σ = 8`, correto) **e uma delas era inútil.** Uma
singularidade de índice `+4` é um ponto onde a cruz dá uma volta inteira; nenhuma
grade de quads a contorna. *A invariante prova que o campo FECHA, não que ele
presta* — foi preciso a segunda régua (a **contagem**) para ver. A alternância
fica como controle em `solve_alternating`.

⭐ **Oito singularidades numa esfera é o número que a topologia pede e o que o
oráculo entrega.** Neste eixo o F2 está em paridade com o estado da arte.

⚠️ **O toro sai com 6 onde o ótimo é 0** — um toro admite campo sem
singularidade. É o item aberto do F2, e é qualidade de rounding (o MIQ congela
**um** inteiro de cada vez; nós congelamos um oitavo por rodada, para trocar 851
resoluções por 48). ⛔ Não mexer no lote sem a tabela qualidade × relógio.

### ⛔ E a MALHA piorou — o que também estava previsto

Alimentando o extrator **local** que já existe com o campo novo:

| malha | atual | +F1 | **+F1+F2** | oráculo |
|---|---|---|---|---|
| `sphere_uv_96x144` | 69 % / 39,7 % | 73 % / 35,3 % | **51 % / 69,9 %** | 100 % / 0,2 % |
| `sculpt_hooked` | 71 % / 40,5 % | 81 % / 32,0 % | **46 % / 75,9 %** | 100 % / 0,3 % |
| `sphere_sculpt_98k` | 83 % / 21,2 % | 76 % / 33,5 % | **49 % / 85,2 %** | 100 % / 0,2 % |

⭐ **A causa está declarada na porta que faz a ponte** (`to_vertex_dirs`): a
conversão para por-vértice **joga fora os saltos de período**, que são onde a
estrutura global mora. O extrator do Instant Meshes não tem como os ler — ele
re-deriva a estrutura dele a partir das direções médias, e a média de um campo
com singularidades REAIS é pior, para ele, que o campo liso que ele mesmo produz.

⇒ **O valor do F2 só se colhe com o F3+F4+F5** (traçado → patches → quantização →
quadrangulação por patch), que é literalmente o que o plano diz. A costura de hoje
mede um **piso**, não o valor.

⚠️ **Por isso NEM o F1 NEM o F2 estão ligados ao produto.** O botão hoje é o que
sempre foi. Ligar qualquer um deles agora pioraria o que o artista vê.

---

## 4-quater — ✅ F4 (protótipo): o solver de quantização FECHA, e prova que fechou

**Entregue:** `crates/ph2d-quantize` — clean-room de Heistermann/Warnett/Bommes,
*Min-Deviation-Flow in Bi-directed Graphs for T-Mesh Quantization* (SIGGRAPH 2023),
§3 e **§4.4** (o caso *polygonal T-mesh*, que é o do QuadWild), medido sobre os
patches que o **oráculo já exporta em texto** — exatamente a mitigação que o §5
prescreve para o risco nº 1.

### ⭐ A lei é UMA, e ela engole os casos que a literatura escreve à parte

Um patch de valência `n` ladrilhado com **um** vértice irregular interior é a
subdivisão regular do leque que liga um ponto de cada lado ao centro. Chamando
`e_j` a aresta interior que vai do centro ao ponto do lado `j`:

```text
    L_i = e_{i-1} + e_{i+1}        (índices módulo n,  e_j >= 1)
```

| valência | no que a lei vira | o nome usual |
|---|---|---|
| 3 | `L_0 = e_1+e_2`, … | paridade + desigualdade triangular |
| **4** | **`L_0 = L_2` e `L_1 = L_3`** | *"lados opostos iguais"* |
| 5, 6 | um sistema cíclico invertível | os padrões de Takayama |

E a condição global de Takayama/Tarini sai de graça: `Σ L_i = 2·Σ e_j` é **par,
sempre**. ⚠️ *Não se testa a paridade — ela não pode ser violada.* Ver
[`corners.rs`](../../../crates/ph2d-quantize/src/corners.rs).

### Por que isto é fluxo, e por que ele é BI-dirigido

Um nó por **lado de patch**; a lei acima é a conservação nesse nó. Cada variável
toca exatamente dois nós com coeficiente `±1` — matriz de incidência de um grafo.
⚠️ Mas **os dois sinais são iguais**: um arco *soma* nos dois patches (`+1,+1`),
uma aresta de leque é *consumida* pelos dois lados (`−1,−1`). Aresta com as duas
pontas no mesmo sentido = **bi-dirigida**, e é isso que tira o problema do
manual de min-cost-flow e o põe no de *matching*.

### ⭐ O que o solver devolve é uma PROVA, não uma opinião

*Dupla cobertura* (§3.6): duplica-se cada nó em `v⁺`/`v⁻` e a rede vira dirigida
comum, que Dijkstra resolve exatamente. Todo fluxo bi-dirigido viável vira um
fluxo **simétrico** ali, de custo exatamente o dobro — logo **o ótimo da dupla
cobertura, a dividir por dois, é um limite inferior do ótimo inteiro**.

O ótimo da dupla cobertura pode ser **assimétrico**; a média com o seu espelho é
simétrica mas **meio-inteira**. Sobre as arestas meio-inteiras corre um
**ramifica-e-limita** cujo limite de cada ramo é a mesma dupla cobertura. Quando a
fila esgota, o custo é o **ótimo inteiro demonstrado** (`prova = sim`).

### A medição, sobre os 10 layouts do oráculo

Régua: `ph2d-quadbench/layout.py` reconstrói o layout de `.patch`/`.corners` e
escreve um ficheiro de **números** (nenhum formato do oráculo entra na engine);
a sonda é [`tests/oracle_bench.rs`](../../../crates/ph2d-quantize/tests/oracle_bench.rs).
Alvo de cada arco = comprimento ÷ aresta média da saída final do oráculo.

| malha | patches (valências) | arcos | meio-inteiras | expansões | fluxos | aumentos | **prova** | custo → limite | ms |
|---|---|---|---|---|---|---|---|---|---|
| `cube` | 8 (3) | 12 | 12 | 3 | 13 | 208 | **sim** | 2,85 → 2,68 | 1 |
| `sphere_uv_96x144` | 8 (3) | 12 | 12 | 11 | 32 | 512 | **sim** | 3,83 → 2,77 | 1 |
| `sphere_sculpt_98k` | 8 (3) | 12 | 12 | 10 | 23 | 343 | **sim** | 4,03 → 2,58 | 1 |
| `sphere_shuffled` | 8 (3) | 12 | 12 | 12 | 30 | 492 | **sim** | 4,06 → 2,85 | 1 |
| `sculpt_wrinkled` | 8 (3) | 12 | 12 | 9 | 48 | 698 | **sim** | 3,80 → 2,72 | 2 |
| `torus_64x32` | 4 (4) | 8 | **0** | 0 | 1 | 12 | **sim** | 20,16 → 20,16 | 0 |
| ⭐ `sculpt_hooked` | 15 (3·11, 4·1, 5·3) | 30 | 14 | 200 | 414 | 30 219 | **sim** | 29,81 → 29,36 | 91 |
| `sculpt_ridged` | 18 (3·12, 4·2, 5·4) | 43 | 18 | 6 | 19 | 1 090 | **sim** | 28,75 → 28,67 | 7 |
| `sculpt_punctured` | — | — | — | — | — | — | — | **layout RECUSADO** | — |
| `sphere_noisy` | 1 318 (3·649, 4·178, 5·489, 6·2) | 3 613 | — | — | — | — | — | **ORÇAMENTO ESGOTADO** | 48 306 |

⭐ **Os oito layouts fechados de tamanho normal quantizam, e TODOS com o ótimo
demonstrado — o mais lento em 91 ms.** É a resposta à pergunta que o §5 diz que
podia matar o plano, e ela chegou **antes** do F3, como o §6 mandava.

⚠️ **Quatro leituras honestas:**

1. **`sculpt_punctured` é recusado de propósito.** 18 dos seus 42 arcos são de
   **bordo** (usados por um patch só), e esta fase pressupõe superfície fechada.
   Devolver um número seria pior que recusar — e ⚠️ **o oráculo também não fecha
   aquele buraco** (§1.3, nota 3). A sanitização é do F1 e não é opcional.
2. ⚠️ **`sphere_noisy` sai por ORÇAMENTO, e a palavra importa.** Ele é a saída do
   próprio oráculo sobre a malha ruidosa — **1 318 patches** onde as outras têm 8
   a 18, que é o oráculo a falhar no traçado, não nós. O solver não diz *"não
   existe"*: diz *"não coube no que me deste"*, e as duas são afirmações sobre
   coisas diferentes (`SolveError::Exhausted` contra `Infeasible`).
   ⛔ **E chegar a dizer isso levou três correções**, porque as primeiras versões
   diziam a coisa errada — ver *"o teto que mentia"*, abaixo.
3. **O `torus` sai com ZERO arestas meio-inteiras.** Ele é o único layout todo de
   valência 4, e nele a estrutura bi-dirigida quase desaparece — o problema cai
   num fluxo comum. *A dificuldade mora nas valências ímpares.*
4. **O limite não é apertado.** Nas esferas o ótimo inteiro custa ~40 % acima do
   limite fracionário: é o preço real da integralidade, não folga do solver — a
   força bruta confirma o mesmo número (abaixo).

### ⭐ E o que custa, medido — o custo NÃO é o tamanho, é a HETEROGENEIDADE

Sonda: [`tests/scaling.rs`](../../../crates/ph2d-quantize/tests/scaling.rs), sobre grelhas toroidais de
`n × m` patches de 4 lados (`2·n·m` arcos). *"Dispersão"* mistura alvos diferentes arco a arco.

| arcos | dispersão | aumentos de fluxo | ms |
|---|---|---|---|
| 2 048 | 0,0 | **0** | **1** |
| **8 192** | **0,0** | **0** | **6** |
| 512 | 0,5 | 2 149 | 65 |
| 512 | 0,9 | 5 107 | 129 |
| 2 048 | 0,9 | 10 692 | 1 710 |

⭐ **Oito mil arcos em 6 ms quando os alvos são uniformes, e ZERO aumentos** — o desequilíbrio inicial
já é nulo. O relógio inteiro desta fase mora na coluna do meio.

⚠️ **Duas curas, e as duas mudaram ordens de grandeza:**

1. **Partida a quente das arestas de leque.** Elas têm custo **zero**, logo nada as empurra: partiam do
   piso `1` enquanto os arcos já se pré-saturavam perto do alvo, e o desequilíbrio que sobrava em cada
   nó era da ordem do **comprimento do lado**. Pôr-lhes fluxo inicial levou a grelha uniforme de 1 024
   patches de **1 581 ms a 1 ms**.
   ⭐ **E é EXATO, não heurístico:** a otimalidade exige que todo arco residual tenha custo `>= 0`; num
   arco de **custo zero** ambos os sentidos custam zero, logo qualquer quantidade inicial preserva a
   condição. *O ótimo é o mesmo — a força bruta continua a bater.*
2. ⭐ **Primal-dual em vez de um caminho de cada vez.** O motor achava **um** caminho mais curto por
   Dijkstra e empurrava o gargalo dele; com desequilíbrio grande isso são centenas de travessias do
   grafo inteiro. Agora cada Dijkstra fixa os potenciais e um **fluxo bloqueante** (Dinic sobre o
   subgrafo de custo reduzido zero) esgota **todos** os caminhos daquela fase de uma vez. A grelha de
   2 048 arcos dispersos passou de **não terminar em meia hora** para **1,7 s**.
   ⚠️ **E o nível NÃO substitui a admissibilidade**: um arco caro pode ligar dois níveis consecutivos
   por acaso. Sem a segunda condição o fluxo escolhia a rota de custo 5 com a de custo 1 aberta — o gate
   `the_cheaper_of_two_parallel_routes_wins` devolvia 32 onde o ótimo é 24.

### ⛔ O teto que MENTIA — a correção que mais importa

⚠️ **Um teto de capacidade que não escala transforma-se numa afirmação falsa sobre o layout.** A
`sphere_noisy` devolvia `Infeasible` — *"não existe quantização regular para esta superfície"* — e o
que não cabia era o **teto da rede**, não a superfície. Um teto inflado à mão fazia a mesma malha
deixar de ser inviável.

⇒ Hoje o teto **escala em degraus** (`CAP_STEPS = [1, 4, 16, 64]`) e só sobe quando o degrau anterior
recusou; o [`Report::cap_step`] diz em qual coube e o `cap_binding` conta os arcos encostados nele
(gate: **zero**). *Um limite que decide a resposta em vez de a limitar é um bug, não uma configuração.*

⚠️ **E os três tetos são três, porque falham por motivos diferentes:** *expansões* limitam a busca (que
é opcional — gastá-las devolve resposta sem prova), *resoluções* limitam também o mergulho (que é
obrigatório), e *aumentos* são os únicos que limitam o relógio de **uma** resolução isolada.
⛔ **Nenhum deles limita segundos**, porque um aumento não custa o mesmo em todo layout — e é por isso
que o orçamento é **parâmetro** ([`Budget`]) e não só constante: quem conhece o tamanho é quem chama.

### A régua que não partilha código com o solver

⭐ Os gates comparam contra **força bruta**: enumerar todas as quantizações de um
layout pequeno e ficar com a mais barata. Sobre um tetraedro (4 patches, 6 arcos,
12 casos) e sobre a esfera mínima (2 patches colados por 3 arcos, 16 casos), o
solver **acerta o ótimo em todos**. *É a única forma de uma afirmação de
otimalidade não ser o solver a avaliar-se a si próprio.*

### E quantos quads sai — com a divergência explicada

| malha | nossos quads | oráculo | Δ |
|---|---|---|---|
| ⭐ `cube` | **4 816** | **4 816** | **0 %** |
| `sculpt_ridged` | 4 147 | 4 109 | +0,9 % |
| `sculpt_hooked` | 4 440 | 4 262 | +4 % |
| `sphere_uv_96x144` | 2 860 | 3 352 | −15 % |
| `sphere_shuffled` | 5 034 | 4 428 | +14 % |
| `sphere_sculpt_98k` | 5 272 | 4 592 | +15 % |
| `sculpt_wrinkled` | 5 470 | 4 696 | +16 % |
| `torus_64x32` | 7 290 | 5 538 | +32 % |

⚠️ **O `cube` bater EXATO e as formas curvas não é diagnóstico, não é ruído.** O
alvo de cada arco foi normalizado pela aresta **média** da saída do oráculo. Num
cubo a densidade dele é uniforme e a média é a lei inteira ⇒ bate exato. Numa
forma curva a densidade dele **segue a curvatura**, e uma média única é o alvo
errado arco a arco. *Isto confirma, por outro caminho, a incógnita nº 1 do §7 — a
metade **adaptativa** da lei de densidade —, e não é um defeito do quantizador.*

### ⛔ E os CINCO bugs, com o que cada um ensinou

1. **Fonte e sorvedouro trocados** no motor de fluxo: o nó com excesso pendurado
   no sorvedouro em vez de na fonte. O fluxo **fechava**, `solve` devolvia `Ok`, e
   a resposta satisfazia a conservação de **outro** problema. Nenhuma inspeção do
   resultado o mostraria; quem o apanhou foi a força bruta, quatro camadas acima.
   Gate próprio hoje: `a_node_with_surplus_sends_it_away_instead_of_doubling_it`.
2. **O piso errado ao remontar a resposta**: uma aresta presa por uma ronda
   anterior guardava o piso da *rede*, não o da *ronda*. Só quebrava nos ramos em
   que a fixação atuou — e a rede não acusa nada. Hoje há `debug_assert` de
   conservação sobre o fluxo final.
3. ⭐ **A busca ramificava em PONTOS**, não em meias-retas — `x = piso` e
   `x = piso+1` em vez de `x <= piso` e `x >= piso+1`. Pontos **não particionam**:
   todo inteiro fora dos dois era descartado em silêncio, e a busca continuava a
   esgotar a fila e a **declarar-se provada**. Sintoma: a `sculpt_hooked` deu
   **29,86**, **29,92** e — já curada — **29,81**, as três com `prova = sim`.
   *Duas provas do mesmo ótimo não podem discordar; uma delas não era prova.*
4. ⭐ **O teto da rede dizia `Infeasible`** — *"esta superfície não admite
   quantização"* — quando o que não cabia era o teto. Ver *"o teto que mentia"*
   acima; hoje ele **escala** e há gate.
5. **O nível de Dinic sem a admissibilidade**: um arco caro que ligue dois níveis
   consecutivos por acaso entra no fluxo bloqueante e paga custo a mais. O gate
   `the_cheaper_of_two_parallel_routes_wins` devolveu **32** onde o ótimo é 24.

⚠️ **E a lição de gate é a parte que fica.** Este terceiro bug **sobreviveu** à
força bruta no tetraedro, no octaedro, no layout com junção em T e num prisma —
instâncias pequenas raramente precisam que uma variável se afaste dois passos do
valor fracionário, então o atalho acerta por acaso. O que o matou foi extrair a
ramificação para uma função de três inteiros e gatear a **propriedade que a
define** — *os dois ramos particionam a faixa* —, com aritmética pura e sem malha
nenhuma (`the_two_branches_partition_the_range_they_came_from`).
*Quando a mutação sobrevive ao gate do RESULTADO, suba um nível: gate a invariante.*

### O que ficou aberto no F4

- ⛔ **O nó de emergência do §4.4.1 NÃO foi construído**, e é decisão medida: ele
  admite patches com mais de um vértice irregular, e com ele uma resposta
  "válida" pode conter patches que o verificador não fecha — o certificado
  deixaria de ser certificado. **Nenhum layout do oráculo precisou dele.** Ligá-lo
  é meia hora no dia em que um layout real o exigir.
- ⭐ **O solver exato por *matching* (§3.7, Blossom) passou a ter justificação
  MEDIDA.** Ele não estava aqui por não haver número que o pedisse; agora há: o
  custo desta fase é a **meia-integralidade**, não o tamanho (secção acima), e é
  exatamente ela que o matching resolve de uma vez em vez de a procurar. *É o
  próximo trabalho desta crate, e já não é especulação.*
- ⛔ **O alinhamento de singularidades** (§4.4, *paired sides*) está fora: o preset
  que o oráculo correu tem `alignSingularities 0`, logo não há nada para comparar.
- A **isometria** é o único termo do custo; a *regularidade* do QuadWild
  (`regularityNonQuadrilateralsWeight 0.9`) ainda não tem análogo.

---

## 4-quinquies — ✅ F3: a CADEIA FECHOU — o layout deixou de vir do oráculo

**Entregue:** `crates/ph2d-trace` — clean-room de Pietroni et al., *QuadWild*
(SIGGRAPH 2021), **§6**. ⭐ **É a peça que faltava para o pipeline correr de ponta
a ponta com código nosso**: até aqui o F4 só tinha layout porque o oráculo o
exportava em texto.

### O que ela faz

De cada **singularidade** que o F2 decidiu partem curvas que seguem o campo — as
*separatrizes* —, elas correm até bater noutra singularidade ou noutra
separatriz, e o que sobra da superfície recortada por elas são os **patches**.

⚠️ **As separatrizes correm sobre ARESTAS da malha**, vértice a vértice, e não
como polilinhas a cortar triângulos. Não é preguiça: é o que faz a fronteira de
cada patch ser um conjunto de arestas existentes, logo o patch é um conjunto de
**faces inteiras** — que é exatamente o formato que o oráculo também produz
(medido: as fronteiras dele, reconstruídas em 2026-08-20, caem **todas** em
arestas da malha remalhada).

### ⭐ As três leis que o traçado precisou, e o preço de cada uma

1. **Uma singularidade emite `4 − índice` separatrizes, não quatro.** Num vértice
   de índice `k` a cruz só fecha depois de `4 − k` quartos de volta, e as saídas
   ficam espaçadas por `Θ/(4−k)` — onde `Θ` é o ângulo total à volta do vértice,
   que **num cone não é `2π`**.
   ⚠️ Emitir sempre quatro parece inofensivo: medido, davam **23 patches** numa
   esfera onde o oráculo dá 8. *A quarta separatriz não tem para onde ir e vai
   bater no meio de outra — e cada batida dessas é um patch a mais.*
2. **A identidade de um arco é o CONJUNTO de arestas, nunca a sequência.** Os dois
   patches que partilham um arco percorrem-no em sentidos **opostos**, então a
   lista de um é a do outro ao contrário e o mesmo arco ganha dois ids. Sintoma:
   o F4 a recusar com `ArcUse { uses: 1 }` — *um arco de bordo numa superfície
   fechada*.
3. **A fronteira percorre-se pivotando DENTRO do patch**, nunca saltando de
   aresta em aresta. Num vértice onde quatro arestas de fronteira se cruzam,
   *"a próxima que ainda não usei"* é ambíguo, e ali o laço salta para o patch
   errado — sem erro nenhum, só com uma fronteira que descreve outra coisa.

### E uma limpeza que não é cosmética

Um patch de **dois lados** é uma lasca — duas separatrizes que correram quase
juntas e prenderam uma tira entre elas. ⚠️ **Ele invalida o layout INTEIRO**: a lei
do F4 pede pelo menos três lados, e um único degenerado derruba tudo. A cura é
dissolver a **parede**, não o patch: removida ela, as faces juntam-se ao vizinho
na inundação seguinte, e nenhuma face muda de sítio.

### A medição

| malha | vértices | sing (soma) | anéis abertos | separ. | patches | dissolv. | arcos | valências | o F4 |
|---|---|---|---|---|---|---|---|---|---|
| esfera 24×36 | 830 | 8 (**8**) | 0 | 22 | 14 | 2 | 45 | 3–9 | ✅ **quantiza com prova** |
| ⭐ esfera 48×72 | 3 386 | 7 (**8**) | 0 | 20 | **15** | 0 | 39 | **3, 4, 5** | ✅ **quantiza com prova** |
| toro 32×16 | 512 | 8 (**0**) | 0 | 27 | 15 | 4 | 53 | 3–9 | ✅ **quantiza com prova** |
| esfera + F1 | 2 525 | 10 (**8**) | 0 | 25 | 17 | 0 | 79 | 3–19 | ✅ **quantiza com prova** |
| cubo + F1 | 4 857 | 33 (**−1**) | ⛔ **18** | 81 | 59 | 0 | 226 | 0–13 | ❌ recusa |

⭐ **Quatro das cinco correm a cadeia inteira e saem com o ótimo demonstrado.** E a
esfera 48×72 sai com **todas as valências entre 3 e 5** — a mesma faixa em que o
oráculo trabalha.

### ⛔ E o achado que o F3 desenterrou no F1

⚠️ **O `cube` remalhado pelo F1 chega com 18 ANÉIS ABERTOS** — vértices
não-manifold. Todas as outras malhas chegam com **zero**. É isso, e não o traçado,
que explica as 33 singularidades falsas, a soma **−1** onde a topologia exige `+8`,
e os 59 patches fragmentados: um anel aberto não tem índice definido, o campo
devolve `0` ali, e a invariante de Poincaré–Hopf deixa de bater **sem que nada no
campo esteja errado**.

⛔ **Isto reabre a única linha do §4-bis que dizia que o F1 tinha curado alguma
coisa.** A célula *"o `cube`: vazio → 100 % / 3,7 %"* foi medida sobre uma malha
não-manifold. As outras linhas do F1 não dependem dela, mas essa cai.
⇒ **O F1 tem um defeito de sanitização nomeado e com número**, e o
[`TraceReport::open_rings`] é o instrumento que o denuncia de agora em diante.
*Uma fase a jusante que expõe um defeito a montante é o pipeline a funcionar.*

✅ **CURADO em 2026-08-21 — e não era do F1: era do FLIP da `ph2d-mesh`.** Ver
§4-septies. ⚠️ **E não era do cubo só:** a mesma colisão atingia a esfera
embaralhada e a ruidosa, que são justamente as duas que a cadeia não fechava.

### O que ficou aberto no F3

- ⚠️ **Ainda saem cerca do DOBRO dos patches do oráculo** (15 contra 8 numa
  esfera). Falta a fusão de separatrizes redundantes do QuadWild §6 — duas que
  correm quase juntas deviam virar uma. Todas as valências ficarem em 3–5 diz que
  a estrutura está certa e a **densidade** é que está grossa.
- ⛔ **Feature lines não entram.** O QuadWild traça também a partir das arestas
  vivas; sem isso uma quina dura não vira fronteira de patch, e o `sculpt_hooked`
  (a esfera-com-bico) não terá a feição respeitada.
- ⚠️ **Uma separatriz que não fecha é descartada** e contada em
  [`TraceReport::dangling`]. Ligá-la à mais próxima é trabalho do dia em que o
  número doer — medido, ele é **0** em quatro das cinco malhas.

---

## 4-sexies — ⭐ F5: A MALHA. 100 % quads, e os irregulares caíram ~85×

**Entregue:** `crates/ph2d-quadfill` — clean-room de QuadWild (SIGGRAPH 2021)
**§7 e §8**. ⭐ **É a primeira fase que devolve MALHA**: as quatro anteriores
entregam estrutura — campo, patches, inteiros — e nenhuma delas produz algo que se
veja.

### O que ela faz

Um patch de valência `n` vira `n` quads em volta de **um** vértice central: sobre
o lado `i` há um corte que o parte em `e_{i-1}` e `e_{i+1}`, e do centro sai um
raio de comprimento `e_j` até o corte do lado `j`. ⭐ **É a mesma lei que o F4
resolveu** (`L_i = e_{i-1} + e_{i+1}`), lida ao contrário: lá ela decidia os
inteiros, aqui diz onde pôr os pontos. *Duas leis diferentes para a mesma coisa
seria a costura que não fecha.*

O interior de cada quad sai por **Coons** (interpolação transfinita), não por média
dos cantos — a média ignora a forma dos quatro bordos e numa quina do patch puxa a
grade para dentro até as linhas se cruzarem. Depois, seis rondas de Laplaciano
**tangencial** com reprojeção na superfície original.

### ⭐ A medição — e é o número que o pivô existiu para produzir

Mesmas malhas do corpus, mesmo alvo de densidade (≈ 4 500 quads), para a linha
ficar comparável com o §1.3:

| malha | motor LOCAL (§1.3) | **F1..F5 (nosso)** | oráculo |
|---|---|---|---|
| `sphere_uv_96x144` | 68,7 % quads · **39,7 %** irreg. | **100,0 % · 0,5 %** (21 vértices) | 100 % · 0,2 % |
| `torus_64x32` | 64,9 % quads · **48,9 %** irreg. | **100,0 % · 0,6 %** (30) | 100 % · 0,0 % |
| `sphere_sculpt_98k` | 82,7 % quads · **21,2 %** irreg. | **100,0 % · 1,1 %** (52) | 100 % · 0,2 % |

⭐ **100 % de quads em todas, zero arestas de bordo, e a característica de Euler
exata** — `2` na esfera, `0` no toro. E os vértices irregulares passaram de
**~1 800** (39,7 % de ~4 500) para **21**: um fator de **~85**.

⚠️ **A leitura honesta do que ainda falta:** o chão topológico de uma esfera é
**8** irregulares. O oráculo fica praticamente nele (0,2 % de ~3 350 ≈ **7**); nós
ficamos em **21**, que são **2,6× o chão**. ⇒ *Mesma ordem de grandeza que o
oráculo, ~85× melhor que o motor que ele substitui, e ainda não é o chão.* O que
sobra vem dos ~2× patches a mais do F3 (§4-quinquies).

### ⚠️ E a percentagem é a régua ERRADA

A mesma malha com o dobro da densidade tem **os mesmos** irregulares e **metade**
da percentagem. Medido: pedir uma grade duas vezes mais fina na mesma esfera dá
**12 012 quads em vez de 3 156** e **18 irregulares nos dois casos** — a
percentagem cai de 0,6 % para 0,1 % sem que nada tenha melhorado.
⇒ **A barra dos gates é a CONTAGEM**, e `a_finer_target_adds_quads_without_adding_irregulars`
é o gate que prova que a estrutura é da topologia e não da densidade.

### As três leis da costura

1. ⭐ **Um ponto de fronteira pertence ao ARCO, nunca ao patch.** Os dois patches
   que o partilham pedem-lhe os mesmos índices, um deles ao contrário. Amostrar
   por patch daria dois conjuntos quase iguais sobre a mesma curva, e a malha
   sairia **rasgada** ao longo de toda fronteira de patch — erro pequeno demais
   para se ver num render e grande demais para a malha servir.
2. **Amostrar por COMPRIMENTO DE ARCO, não por contagem de vértices.** A cadeia de
   malha tem arestas de tamanhos diferentes; dividir por contagem põe os pontos
   onde a triangulação por acaso é densa, e a grade herda a densidade da entrada
   em vez da do alvo — que é exatamente o defeito que o F1 existe para não ter.
3. **A orientação corrige-se em BLOCO ou não se corrige.** Ela sai da fronteira do
   patch, que é a mesma para todos; se estiver ao contrário, está em bloco.
   Inverter face a face por teste local produziria uma malha inconsistente.

### ⭐ O gate que vale por três

`the_euler_characteristic_survives_the_remesh`. Rasgar uma divisa, duplicar uma
face ou montar um patch ao contrário mudam `V − E + F`, e **nenhum deles se vê a
olho**. Um número, três defeitos, e ele não depende de densidade, de alvo nem do
solver.

### O que ficou aberto no F5

- ~~⛔ **`sphere_shuffled` não fecha** … e ele mora no F3 (o passeio é guloso).~~
  ✅ **FECHADO em 2026-08-21, e o diagnóstico estava DUAS FASES ERRADO.** Com o F1
  na frente, ela fecha em **1,6 s** com 100 % de quads. ⚠️ E a atribuição a *"ordem
  de índice"* era falsa duas vezes: a fixtura não embaralha ordem nenhuma (§1.2), e
  a causa era a colisão de diagonal do flip (§4-septies). *Uma nota que nomeia a
  fase errada custa a jornada inteira de quem a for seguir.*
- ⚠️ **O relógio é do LAYOUT, não do tamanho** — e a cura do §4-septies mostrou-o
  pelo outro lado: a esfera de **98 k** desceu de 33 s para **8,5 s** só por o F1
  entrar antes, e a sacudida de **> 20 min** para **1,6 s** sem que o F4 mudasse
  uma linha. *O F4 nunca foi lento; ele estava a receber lixo.*
- ⛔ **Sem *feature lines*** (herdado do F3): uma quina dura não é respeitada, e é
  isso que a `sculpt_hooked` do diagnóstico exige.
- ⛔ **Nem Hausdorff nem desvio angular** foram medidos aqui — a régua do §9 do
  briefing continua por correr sobre a saída nova.
- ⛔ **Nada disto está ligado ao botão.** O `Quad Retopology` continua a chamar a
  `ph2d-quadflow`, e ligar a cadeia nova é o F5.2 (a porta no shell).

---

## 4-septies — ⭐ A não-variedade não era do F1: era do FLIP, e custava uma troca em dez mil

**Entregue:** a **recusa 4** em [`ph2d-mesh/src/dyntopo_flip.rs`], o gate
`a_round_of_flips_never_creates_the_same_diagonal_twice` (na `ph2d-mesh`) e o
gate `the_remesh_returns_a_closed_manifold` (na `ph2d-remesh-iso`).

### O mecanismo — e ele explica por que três guardas corretas não bastavam

O operador de troca de diagonal já recusava **borda**, **diagonal que já existe**
e **dobra**. A segunda pergunta *"o anel de `c` contém `d`?"* — e ⚠️ **esse anel é
o de ANTES da rodada**. Duas trocas da mesma rodada, sobre pares de faces
**disjuntos** (logo fora do alcance do `spent`, que só protege a face), produzem a
mesma diagonal `c—d`: nenhuma das duas vê a outra, e a malha sai com **duas
arestas entre o mesmo par**.

⭐ **A assinatura que provou o mecanismo em vez de o supor:** a aresta ofensora sai
com **quatro** faces. Uma diagonal criada *por cima* de uma que já existia teria
**três** — logo não é a recusa 2 a falhar, é a rodada a colidir consigo mesma.
*Um número, e ele distingue as duas hipóteses.*

### A medição — uma rodada de `relax_valence`, com a recusa 4 desligada

| fixtura | vértices | trocas | arestas de valência ≠ 2 |
|---|---|---|---|
| `uv_sphere` (lisa, qualquer tamanho) | — | **0** | 0 — *não flipa, não prova nada* |
| `uv_sphere_shuffled(48,72)` | 3 386 | 2 390 | 0 |
| ⭐ `uv_sphere_noisy(24,36)` | **830** | 457 | **7** ← a menor que contém o fenómeno |
| `uv_sphere_shuffled(96,144)` | 13 682 | 9 968 | 1 |
| `uv_sphere_noisy(96,144)` | 13 682 | 8 590 | **185** |

⚠️ **A esfera LISA é o controle negativo que quase enganou**: ela não aceita troca
nenhuma, então passaria com o operador inteiro apagado. É o **ruído** que dá ao par
`c,d` a valência alta de que a colisão precisa — e é por isso que a fixtura do gate
é ruidosa e pequena, não lisa e grande.

⭐ **O preço da cura, medido: UMA troca em 9 968.** A recusa 4 é reservada só
depois das guardas de ângulo e de dobra — reservá-la antes faria um candidato
**rejeitado** bloquear uma troca válida com a mesma diagonal.

### O que a cura destravou — e é muito mais do que o cubo

| malha | anéis abertos, antes → depois | o que a cadeia fazia → faz |
|---|---|---|
| `cube` (após F1) | **18 → 0** | ❌ o F4 recusava → ✅ **quantiza com prova**, 2 146 quads |
| `sphere_shuffled` | 2 → 0 | ⛔ **> 20 min sem fechar** → ✅ **1,6 s**, 100 % quads |
| `sphere_noisy` | — | ⛔ orçamento esgotado em 47 s → ✅ **1,7 s**, 100 % quads |
| `sphere_sculpt_98k` | 0 → 0 | 33 s → **8,5 s** |

⭐ **O `sphere_shuffled` era o controle de determinismo do corpus, e o §4-sexies
atribuiu a culpa ao passeio guloso do F3.** ⚠️ **Estava errado.** A causa era duas
fases a montante, e a cura não tocou uma linha do F3. *Uma nota que nomeia a fase
errada custa a jornada inteira de quem a for seguir.*

### As duas provas de mutação

⚠️ **Um gate que ninguém viu VERMELHO não afirma nada.** Com a recusa 4 trocada
por um `insert` cujo valor de retorno se ignora:

| gate | veredito sem a cura |
|---|---|
| `a_round_of_flips_never_creates_the_same_diagonal_twice` | ❌ `left: 7, right: 0` |
| `the_remesh_returns_a_closed_manifold` | ❌ *cubo: borda 0, **não-variedade 3**, dirigida-repetida 16* |
| `the_genus_survives` (com o cubo acrescentado) | ❌ *cubo: o remesh mudou o GÉNERO* |

⚠️ **E o `the_genus_survives` passava, antes, sobre a malha quebrada** — porque ele
conta arestas num `BTreeSet`, que **funde** o par duplicado. *Uma régua que
deduplica não pode denunciar duplicação*, e é por isso que o gate novo conta por
**ocorrência** e faz as três perguntas separadas (borda · não-variedade · aresta
dirigida repetida).

### ⛔ O que a cura NÃO comprou

- ⛔ **O `cube` continua com a soma de índices errada** (`−5`, e a topologia exige
  `+8`) — mas agora **sem um único anel aberto**. ⇒ *A causa mudou de sítio:* o
  campo mente nas 12 arestas vivas do cubo porque **não há feature lines**, que já
  era o item aberto nº 1 do F3. O que a cura fez foi **separar os dois defeitos**,
  que até aqui liam como um.
- ⚠️ **Com o F1 na frente, os irregulares SOBEM** (esfera 21 → 47, toro 30 → 66) —
  ver o parágrafo seguinte. A cadeia ficou **robusta e rápida** e ainda não ficou
  **melhor**.

---

## 4-octies — ⛔ O F2 NÃO chegou ao ótimo: ele chega ao ótimo em malhas BEM DISTRIBUÍDAS

⚠️ **Esta secção REVOGA o título do §4-ter** (*"o CAMPO chegou ao ótimo teórico"*).
Ele foi medido em esferas de grade `uv`, que são o caso mais fácil que existe — a
própria grade **é** um campo cruzado perfeito. Sobre malhas com distribuição de
triângulos irregular, o campo parte-se, e parte-se **com o tamanho**.

### O censo — a régua é a CONTAGEM, e a soma não podia denunciar nada

⭐ **`Σ índice = 4·χ` é forçada pela topologia**: vale `8` numa esfera com um campo
óptimo *e com um campo péssimo*, porque um par `+1/−1` espúrio cancela-se. ⚠️ O
módulo do solver já registava essa lição (a alternância ingénua passava no gate da
soma com singularidades de índice `+4`) — e mesmo assim **nenhum gate media a
contagem contra o refinamento da malha**.

| malha | vértices | **singularidades** | soma |
|---|---|---|---|
| `uv_sphere(24,36)` estruturada | 830 | **8** | 8 |
| `uv_sphere(96,144)` estruturada | **13 682** | **7** | 8 |
| esfera iso `α=0,020` | 2 608 | **8** | 8 |
| esfera iso `α=0,017` | 3 571 | 10 | 8 |
| esfera iso `α=0,014` | 5 291 | **50** | 8 |
| esfera iso `α=0,012` | 7 147 | **110** | 8 |
| esfera iso `α=0,010` | 10 251 | **194** | 8 |
| ⛔ `sphere_shuffled` (crua) | 13 682 | **448** | **−147** |
| ⛔ `sphere_noisy` (crua) | 13 682 | **1 115** | **−288** |

⛔ **As duas últimas linhas eram pior que qualidade: a soma VIOLAVA Poincaré–Hopf**
numa superfície fechada, variedade, com **zero anéis abertos**.
✅ **RESOLVIDO no mesmo dia — e não era nenhuma das duas hipóteses óbvias.** Não
eram os `return 0` silenciosos (a auditoria mostrou **zero** desistências e **zero**
colisões de chave): era a **fórmula**. Ver §4-nonies.

⚠️ **E as linhas das malhas ISO não mudaram um número com a correção** (194
continua 194): ali o resíduo do arredondamento já era `0,002`, e a régua já era
exacta. *A conclusão desta secção sobre o lote sai intacta — o que a correção
mudou foram as duas linhas em que a régua estava a decidir por sorteio.*

### As três hipóteses que a medição REFUTOU — e por que registá-las

⛔ **Cada uma parecia óbvia, e as três estavam erradas.** Sem a medição, qualquer
delas teria comprado uma jornada:

| hipótese | o que a mataria | o que a medição deu |
|---|---|---|
| *"o passe do F1 sai no teto de rodadas e entrega malha torta"* | rodadas gastas e dispersão de aresta | ⛔ **converge em todos os alvos** (7–13 rodadas de 24), e a dispersão é **plana** (0,127–0,140) |
| *"o CG não converge em malha grande"* | resíduo por resolução | ⛔ a esfera **estruturada** de 13 682 tem o **pior** resíduo da tabela (`8,8e-4`, 68 resoluções não-convergidas) — **e sai com 7** |
| *"a reprojeção sobre uma referência facetada faz a curvatura virar ruído"* | referência muito mais fina | ⛔ referência **7× mais fina** (97 922 vértices) move `194 → 168` |

### ⭐ A tabela que o `BATCH_FRACTION` exigia desde que nasceu

O comentário daquela constante dizia, à letra: *"é uma divergência declarada da
referência… ⛔ não a mexa sem a tabela de qualidade × relógio ao lado."* Aqui está,
sobre a esfera iso `α=0,010` (10 251 vértices, o caso que sai com 194):

| política | singularidades | resoluções | relógio |
|---|---|---|---|
| **1/8 por rodada** (em vigor) | **194** | 67 | **1 841 ms** |
| 1/16 | 132 | 126 | 3 360 ms |
| 1/32 | 84 | 232 | 6 335 ms |
| 1/64 | 72 | 423 | 11 407 ms |
| 1/128 | **40** | 760 | 22 852 ms |
| ⭐ 1/256 | **24** | 1 343 | 42 184 ms |
| ⛔ 1/8, só se `|d| ≤ 0,25` | 194 | 439 | 8 497 ms |
| ⛔ 1/8, só se `|d| ≤ 0,15` | 194 | 1 074 | 25 365 ms |
| ⛔ 1/8, só se `|d| ≤ 0,08` | **196** | 2 582 | 72 546 ms |
| ⛔ 1/8, só se `|d| ≤ 0,04` | 192 | 4 186 | **129 348 ms** |

⭐⭐ **A curva NÃO achata: `194 → 24`, oito vezes melhor, e ainda a descer.** ⇒ **O
lote não é uma alavanca, é a CAUSA** — e o chão de 8 está ao alcance da lei da
referência (um inteiro por rodada), que é o limite desta série.

⚠️ **Correção de percurso registada de propósito:** com a série a parar em `1/64`
(72) eu concluí *"há um segundo mecanismo"*. Dois pontos a mais mataram essa
leitura. **Uma curva de três pontos que achata pode ser uma curva de cinco que
não** — e a diferença entre as duas conclusões era uma jornada a procurar um
mecanismo que não existe.

⛔ **E o limiar de CONFIANÇA é uma RECUSA MEDIDA:** `194 → 192` por **70× de
relógio**. *Recusar congelar os genuinamente fracionários não compra nada — a
re-resolução não os move para onde interessa.* ⛔ Não reconstruir.

### ⭐ Por que o lote existe, e qual é a cura real

⚠️ **A lei da referência foi ORÇADA e não corrida**, e o orçamento é o achado: são
`E − F + 1 ≈ 10 250` resoluções, e as primeiras têm quase todos os inteiros livres
(dimensão ~30 700, até 600 iterações de CG cada). Dá **mais de uma hora**, contra
11 s de `1/64`.

⇒ **O lote existe para tapar a falta de um solver INCREMENTAL.** O MIQ da
referência refatoriza (Cholesky esparso) e faz *update* de posto 1 a cada inteiro
congelado, então uma re-resolução custa quase nada e um-de-cada-vez é acessível.
Nós resolvemos por **CG do zero, com o vetor dos inteiros livres a partir de
ZERO**, todas as vezes. *A cura não é escolher um ponto da tabela: é tornar a
re-resolução barata.* Duas rotas, por ordem de preço:

1. **Warm-start do bloco dos inteiros** (barato, mede-se numa tarde) — hoje só o
   `θ` viaja de rodada em rodada; o `q` recomeça em zero, e o CG tem de reconstruir
   uma resposta que quase não mudou. ⚠️ Instrumento já existe:
   [`SolveReport::cg_capped`] diz que **23 de 67** resoluções gastam as 600
   iterações sem convergir.
2. **Fatoração direta com update** (a rota da referência) — é o que torna a lei
   de um-de-cada-vez pagável, e é trabalho de crate.

### O que isto muda no plano

1. ⛔ **A quantização (F4) e a montagem (F5) estão a ser alimentadas por um campo
   mau nas malhas difíceis.** Os 21 irregulares da esfera são o que a cadeia dá
   quando o campo está bom; os 47 (com o F1 na frente) são o que ela dá quando não
   está. *O gargalo mudou de fase.*
2. ⚠️ **~~O F2 volta a ser o próximo trabalho~~ — CORRIGIDO no mesmo dia pelo
   §4-decies.** Na configuração que o produto corre (`α = 0,02`) o campo entrega
   `sing = 8`: ele está bom. Os 194 desta secção só aparecem a `α = 0,010`. O
   próximo trabalho é o **F3**, e a medição de proveniência diz porquê.
3. ⚠️ O `Rounding` já é um parâmetro (`solve_miq_with`), então o compromisso é
   **exposto e medido**, não escondido numa constante. ⛔ **A constante em vigor
   NÃO foi mexida** — mudá-la sem a rota (1) seria trocar 194 singularidades por
   42 segundos de espera, e o pivô não existe para escolher entre dois números
   maus.
4. ✅ **O defeito da régua está corrigido** — §4-nonies.

---

## 4-nonies — ⭐ A régua do F2 estava errada, e o erro escondia-se em malha uniforme

**Entregue:** `+ K_v` na fórmula do índice, o [`IndexReport`] que audita a própria
régua, e duas fixturas novas no gate de Poincaré–Hopf.

### A pista, e por que ela é o método e não sorte

A auditoria da régua respondeu **três** perguntas de uma vez, e foram elas que
apontaram a causa:

| malha | desistiu | colisões de chave | **pior resíduo** | ambíguos |
|---|---|---|---|---|
| `uv_sphere(96,144)` | 0 | 0 | 0,0009 | 0 |
| esfera iso `α=0,010` | 0 | 0 | 0,002 | 0 |
| `torus(32,16)` | 0 | 0 | 0,049 | 0 |
| ⛔ `sphere_shuffled` | **0** | **0** | **0,4999** | **1 468** |
| ⛔ `sphere_noisy` | **0** | **0** | **0,5000** | **4 472** |

⭐ **`0,5` é o máximo possível: um empate — o `round` a decidir por sorteio.** E as
duas primeiras colunas **excluíram** a hipótese óbvia (os `return 0` silenciosos):
a régua não desistiu de vértice nenhum. *Instrumentar as três perguntas ao mesmo
tempo é o que transformou "está errado algures" em "está errado AQUI".*

⚠️ **E o resíduo tinha a ordem de grandeza do defeito angular.** Numa esfera `uv`
de 13 682 vértices, `K_v ≈ 4π/13682 ≈ 0,0009 rad`, que em quartos de volta é
`0,0006` — **exactamente** o resíduo medido. A hipótese escreveu-se sozinha.

### A lei, e o teste que a escolheu

```text
índice_v = ( Σ_anel ±(κ_e + (π/2)·p_e)  +  K_v ) / (π/2),   K_v = 2π − Σ ângulos
```

Os `κ` medem a rotação da **moldura** de face para face; dar a volta ao vértice
roda a moldura pela holonomia da superfície, que é exactamente `K_v`. ⛔ **Nenhuma
das três variantes foi escolhida por dedução** — as três correram lado a lado:

| variante | `uv_sphere` | `torus` | `sphere_shuffled` | `sphere_noisy` |
|---|---|---|---|---|
| só o total (a lei antiga) | 0,0009 | 0,0487 | **0,4999** | **0,5000** |
| `total − K_v` | 0,0018 | 0,0974 | 0,4999 | 0,4999 |
| ⭐ `total + K_v` | **0,0001** | **0,0000** | **0,0001** | **0,0000** |

⚠️ **O controle da sonda é Gauss–Bonnet:** `Σ K_v / 2π` tem de dar `χ`, e dá
`2,000` nas esferas e `0,000` no toro. *Uma sonda que julga uma fórmula tem de
provar primeiro que a grandeza dela está certa.*

### O resultado

| malha | soma antes | soma depois | contagem antes → depois |
|---|---|---|---|
| `sphere_shuffled` | ⛔ **−147** | ✅ **+8** | 448 → **1 124** |
| `sphere_noisy` | ⛔ **−288** | ✅ **+8** | 1 115 → **2 041** |
| `cube` + F1 | ⛔ **−5** | ✅ **+8** | 21 → 23, e **52 → 42 patches** |
| tudo o resto | correcta | correcta | **inalterada** |

⚠️ **As contagens SUBIRAM naquelas duas, e isso é a correcção a funcionar:** os
números antigos eram arredondamentos por sorteio. O campo é mesmo assim tão mau
ali — agora está medido em vez de mascarado.

⭐ **E o `cube` fechou a topologia:** ele era a única malha do corpus com a soma
errada depois da cura do §4-septies, e a causa era esta. Menos 10 patches e o
custo da quantização de 391 para **308**.

### ⭐ A prova de mutação, e o que ela diz sobre as fixturas

Com o `K_v` desligado, e a asserção de resíduo temporariamente afrouxada para se
ver o que a SOMA sozinha apanharia:

| fixtura | soma sem `K_v` | veredito |
|---|---|---|
| esfera 24×36 | 8 (esperado 8) | ✅ **passa** — resíduo 0,0145 |
| esfera 48×64 | 8 | ✅ passa — resíduo 0,0041 |
| toro | 0 | ✅ passa — resíduo 0,0122 |
| cubo subdividido | 8 | ✅ passa — resíduo 0,0062 |
| ⭐ **esfera SACUDIDA** | ⛔ **−4** (esperado 8) | ❌ **cai** |

⚠️ **As quatro fixturas antigas passavam sobre a fórmula errada, e a soma fechava
por CANCELAMENTO.** O gate existia há dois meses e era verde. ⇒ *A fixtura nova é
o gate; o resto era decoração.* E a asserção de **resíduo** cai ainda mais cedo —
já na primeira fixtura fácil —, porque ela julga cada vértice em vez da soma.

---

## 4-decies — ⭐ De onde vêm os irregulares: **100 % do layout**, e o F5 não cria nenhum

**Entregue:** [`FillReport::by_provenance`] e o gate
`this_crate_introduces_no_irregular_of_its_own`.

⚠️ **`47 irregulares` diz que há trabalho e não diz em que FASE.** Cada vértice da
saída passou a carregar de onde veio, e a conta fecha:

| malha | canto (F3) | centro (F3) | **arco** | **raio** | **grade** | total |
|---|---|---|---|---|---|---|
| esfera 96×144 | 32 | 15 | **0** | **0** | **0** | 47 |
| toro 64×32 | 46 | 20 | **0** | **0** | **0** | 66 |
| esfera 98 k | 42 | 21 | **0** | **0** | **0** | 63 |
| esfera sacudida | 44 | 15 | **0** | **0** | **0** | 59 |
| `cube` | 90 | 38 | **0** | **0** | **0** | 128 |

⭐⭐ **Zero em arco, raio e grade, em todas as seis.** O leque e a interpolação de
Coons **não introduzem um único irregular**. ⇒ *A dívida inteira é do traçado, e a
montagem está limpa.*

### A anatomia dos 47, e ela nomeia duas coisas diferentes

1. **15 centros** = um por patch de valência ≠ 4. A esfera sai com **15 patches e
   os 15 têm valência ≠ 4** — nenhum é um quadrilátero. Um patch de 3 ou 5 lados
   **obriga** a um irregular; o que está errado é a valência, não o leque.
2. **32 cantos** = cantos do layout onde não se encontram quatro arcos. O layout
   tem **65 arcos para 15 patches**, logo `2 − 15 + 65 = 52` cantos — para **8**
   singularidades. Os outros 44 são **junções em T**, e cada uma é um vértice de
   valência 3.

⇒ ⭐ **A conta do chão:** 8 singularidades + 0 junções em T + patches todos de
quatro lados dariam **8** irregulares, que é exactamente onde o oráculo fica.

### ⛔ Isto CORRIGE o "próximo trabalho" que o §4-octies escreveu

O §4-octies apontou o **F2**. ⚠️ **Está errado para a configuração do produto:**
com `α = 0,02` o campo entrega `sing = 8` em cinco das seis malhas — ele está
**bom**. Os 194 do §4-octies só aparecem a `α = 0,010`, que não é o que a cadeia
corre. *O F2 é dívida real e é dívida de mais tarde.*

⇒ **O próximo trabalho é o F3**, e tem dois alvos com número:
**(a)** matar junções em T (a fusão de separatrizes do QuadWild §6) — vale **32**
dos 47; **(b)** fazer os patches saírem de quatro lados — vale **15**.

### ⚠️ E uma mutação que SOBREVIVE de propósito

Trocar o rótulo de um vértice de **grade** não muda número nenhum: esses vértices
são sempre regulares e nunca entram na conta. ⛔ **Não é gate a faltar** — é o
alcance honesto da afirmação. As duas mutações que importam (o canto e o centro a
mentirem sobre si) caem, e a do centro **só cai por causa da conferência contra o
layout**, que foi acrescentada precisamente porque as duas primeiras sobreviveram.

---

## 4-undecies — ⭐⭐ F3-bis: **um canto só existe onde a parede se ramifica** — 47 → 13

**Entregue:** a porta estrutural em [`patches::is_corner`], a promoção contada em
[`TraceReport::promoted`], e a sonda `corner_census`.

### O que a medição encontrou

O §4-decies disse que os 47 irregulares eram **32 cantos** + **15 centros**, e que
o alvo era fundir separatrizes (QuadWild §6). ⚠️ **Antes de o fazer, medi de que
espécie eram os 52 cantos** — e a resposta mudou o trabalho:

| malha | cantos | singularidade | junção de paredes | ⛔ **artefacto** |
|---|---|---|---|---|
| esfera 96×144 + F1 | 52 | 6 | 19 | **27** |
| toro 64×32 + F1 | 72 | 8 | 27 | **37** |
| esfera 98 k + F1 | 68 | 6 | 25 | **37** |

⛔ **O maior balde não tinha estrutura nenhuma por baixo** — grau de parede `≤ 2`,
ou seja o **interior de uma separatriz**. E eram **todos** irregulares na malha
final. *Uma jornada a fundir separatrizes não teria tocado neles.*

### A causa, e a lei

O canto era decidido pelo **ângulo interno** do patch no vértice, arredondado a
quartos de volta. ⚠️ Uma parede é uma **polilinha sobre arestas de malha**: ela
zigue-zagueia, e um vértice onde ela vira 60° dá `120°` de ângulo interno, que
arredonda para **1 quarto** — canto. *Quem virou foi a polilinha, não a estrutura.*

⭐ **A lei nova:** *um canto só pode existir onde a parede se ramifica*
(`branching > 2`). No interior de uma separatriz a fronteira do patch passa
**direito** por definição — uma separatriz é uma linha da grade e não vira.

⚠️ **A geometria continua a falar, e é necessária:** numa junção em T os dois
patches que ladeiam o pé têm quina e o terceiro tem a fronteira reta. *A estrutura
diz ONDE pode haver canto; a geometria diz para QUEM ele é.*

### ⛔ E a cerca de Chesterton que a porta sozinha derrubou

Os cantos-artefacto eram **load-bearing**: eram eles que faziam cada laço ter lados
suficientes para ser um patch. Com a porta sozinha, a esfera 24×36 colapsava de 14
patches para **1, com zero arcos** (a limpeza de degenerados cascateia). ⇒ A
promoção existe para isso, e ela é **contada**.

### ⛔ E o piso da promoção: `4` foi construído, MEDIDO e rejeitado

Pedir quatro cantos por patch parecia melhor — um patch de três lados produz um
irregular no centro por construção. Medido:

| malha | piso **4** | piso **3** |
|---|---|---|
| esfera 96×144 | 13 irreg. · 2 623 quads | 14 · **4 922** |
| toro 64×32 | 23 · 3 666 | 24 · **5 071** |
| esfera ruidosa | 20 · 3 949 | 20 · **4 503** |
| `cube` | 48 · 2 778 | 48 · **3 020** |
| esfera 98 k | ⛔ **`Infeasible`** | ✅ **21** · 5 978 |
| esfera sacudida | ⛔ **`Infeasible`** | ✅ **14** · 2 568 |

1. ⛔ **Promover um quarto canto onde a estrutura tem três torna o sistema
   INVIÁVEL** — não é orçamento, é o fluxo do F4 a não fechar. **Duas das seis**
   malhas deixavam de quantizar.
2. ⚠️ **E não comprava qualidade:** a contagem fica dentro de **um**, e o piso 4
   **distorcia a densidade** (2 623 quads onde o alvo pede ~5 000).

⇒ *O que a estrutura não dá, a promoção não inventa.* ⛔ Não reconstruir.

### ⭐ O resultado, na cadeia do produto

| malha | irregulares antes | **depois** | valências dos patches, antes → depois |
|---|---|---|---|
| esfera 96×144 | 47 | **14** | `{3:7, 4:6, 5:2}` → `{3:9, 4:6}` |
| toro 64×32 | 66 | **24** | 3–9 → `{3:8, 4:10, 5:4, 6:1}` |
| esfera 98 k | 63 | **21** | 3–19 → `{3:12, 4:10, 5:1}` |
| esfera sacudida | 59 | **14** | — |
| esfera ruidosa | 61 | **20** | — |
| `cube` | 128 | **48** | — |
| ⭐ *e o F4 passou a provar o ótimo em **todas*** | | | |

⭐ **14 numa esfera, contra um chão topológico de 8 e o oráculo em ~7.** De
**5,9×** o chão para **1,75×**, e a barra do gate desceu de `6×` para `3×` —
medido: **18** nas fixturas do gate, e **o mesmo 18** com densidades de 2 809 a
14 202 quads. *A contagem é estrutural, como tem de ser.*

### O que sobra

- ⚠️ **A dívida mudou de forma:** de `32 cantos + 15 centros` para
  `5 cantos + 9 centros` (esfera). Agora o balde maior são os **patches de três
  lados** — e a cura deles é o traçado dar separatrizes que fechem retângulos, que
  é a fusão do QuadWild §6 que este trabalho **não** fez.
- ⚠️ O `cube` continua em 48, e a causa é a mesma de sempre: **sem feature lines**
  o campo mente nas 12 arestas vivas.

---

## 4-duodecies — ⭐⭐ F5.2: **A PORTA** — o botão passou a chamar a cadeia global

**Entregue:** `shells/desktop/src/sculpt3d_history_retopo_global.rs`
(`Sculpt3dScene::quad_remesh_global`), as três recusas novas, o log com os
irregulares, e a porta de bissecação `PH2D_RETOPO_LEGACY=1`.

⭐ **É a primeira vez que alguma coisa deste pivô é alcançável por um gesto.** Até
aqui a cadeia existia e era medida por sondas; o `Quad Retopology` continuava a
chamar o porte local do ADR-0160.

### As decisões, e por que cada uma

1. **O botão passa a ser a cadeia GLOBAL, e o local FICA.** Não é remoção: o porte
   BSD responde em sub-segundo e este leva segundos — é o *preview* do F7, e o
   ADR-0162 já o dizia. ⛔ `PH2D_RETOPO_LEGACY=1` volta a ele, **para bissecar**:
   um resultado mau só se atribui a esta cadeia depois de se ver o que o outro faz
   com a mesma peça.
2. **O `detail` atravessa pela MESMA lei dos dois** (`edge_for_detail`). *Duas leis
   para o mesmo knob é como um botão passa a precisar de duas explicações.*
3. ⚠️ **O `adapt` NÃO é passado, e o painel diz isso em voz alta.** Esta cadeia não
   tem densidade adaptativa; passá-lo seria um knob que o painel mostra, o artista
   mexe e **nada consome**.
4. **A reprojeção do F5 é contra a malha ORIGINAL, não contra a remalhada** —
   somar os dois erros perderia a silhueta que o artista esculpiu.
5. **Três recusas novas com nome próprio** (`Layout` · `Quantize` · `Fill`) em vez
   de uma. ⭐ **O `match` exaustivo obrigou os três sítios de despacho a decidir** —
   é exactamente o que ele existe para fazer.

### Medido, pelo gesto

| | |
|---|---|
| fixtura | `wrinkled_sphere`, `detail = 0,5` |
| saída | **530 quads · 0 não-quads · 0 arestas de bordo** |
| irregulares | **22** (chão topológico 8) |
| relógio | **874 ms** |
| undo | ✅ Ctrl+Z devolve a malha byte-a-byte |

### ⚠️ E o gate que se pode e o que não se pode ter

O gesto precisa de **GPU** (a cena segura buffers de device), então um gate sobre
ele é `skip` gracioso numa máquina sem adapter — e *skip gracioso não é verde*. ⇒ A
**decisão** de qual motor correr foi separada numa função **pura**
(`legacy_from`), e é ela que corre em toda a máquina. ⚠️ *Esta crate proíbe
`unsafe`, e desde a edição 2024 o `std::env::set_var` é `unsafe`: um gate que
quisesse mexer na variável **não compilava**.* A separação não é estética — é o que
torna a lei testável.

### ⛔ O que a porta NÃO resolve

- ⛔ **Sem feature lines**: uma quina dura não é respeitada, e é isso que a
  `sculpt_hooked` do diagnóstico exige. A peça com bico ainda sai com a feição
  arredondada.
- ⚠️ **Sem barra de progresso.** Segundos de espera sem sinal na tela leem-se como
  travamento. O log da consola diz o relógio; a UI não diz nada.
- ⚠️ **`adapt` continua no painel sem consumidor** neste backend — hoje um aviso,
  amanhã ou uma lei ou um knob desligado.

---

## 4-terdecies — ⛔⛔ O PRODUTO SAIU DESTRUÍDO COM 10.515 GATES VERDES

**Achado por:** o Enio, numa foto. **Diagnosticado por:** auditoria multi-agente
(6 lentes, 29 achados, 21 sobreviveram a dois céticos cada).

### O que o artista viu

Clicou no botão numa esfera esculpida. A malha voltou em **lascas finas** com um
emaranhado de **linhas retas a atravessar a peça de lado a lado**, com a silhueta
da esfera ainda a adivinhar-se por baixo.

E o log dizia: `100 % quads · casca FECHADA · 22 irregulares`. **As duas coisas
eram verdade ao mesmo tempo.**

### A causa raiz — uma linha, e um raciocínio CORRETO que a escreveu

[`fill`] tinha **um** parâmetro, `reference`, a servir **dois papéis**:

1. a tabela de posições que os índices do `layout` indexam
2. a superfície sobre a qual reprojetar

A porta do shell passou-lhe a malha **original**, raciocinando — corretamente —
sobre o papel (2). Mas o layout foi traçado sobre a saída do **F1**, que tem espaço
de índice próprio. ⇒ Cada `arc_chain[i]` foi ler a posição de um vértice
**arbitrário**.

⚠️ **O F1 quase sempre REDUZ a contagem** (98 306 → 2 679), então todo índice caía
**dentro** do alcance: sem panic, sem erro, leitura silenciosa.

| | quads | não-quads | bordo | irreg. | aresta mediana | aresta MAX |
|---|---|---|---|---|---|---|
| o que o produto fez | 5 978 | 0 | 0 | 21 | **0,2315** (4,6× o alvo) | **2,0141** |
| o que devia fazer | 5 978 | 0 | 0 | 21 | 0,0454 (= o alvo) | 0,4077 |

⭐ **Os quatro números que os gates leem são bit-a-bit os mesmos.** E `2,0141` numa
peça de raio 1,0 é o **diâmetro** — uma aresta a atravessar a esfera.

⚠️ **E a última operação do F5 é `project_onto(surface, …)`**, que repõe cada
vértice sobre a superfície original: é literalmente por isso que a foto mostra
escombro **colado à silhueta certa**.

### ⛔ Por que 10.515 gates ficaram verdes — e não foi azar

1. ⭐ **O `FillReport` é função pura dos ÍNDICES, por construção.** `quads` e
   `non_quads` saem da aridade das faces; `boundary_edges` de um mapa de pares;
   `irregular` da valência. As faces saem de `layout` + `quant`. **Nenhuma posição
   escolhe um índice** ⇒ embaralhar posições deixa o relatório **byte-idêntico**.
2. **O gate do gesto era LITERALMENTE invariante sob o defeito.** As suas seis
   asserções: quatro do relatório, um `assert_ne!(positions, before)` que passa
   trivialmente (comprimentos diferentes), e o round-trip do undo — que mede a
   malha **de antes**. *Não passou por sorte: não tinha como reprovar.*
3. **Nenhum teste da árvore corria a ordem do produto.** Em 100 % da cobertura, os
   dois papéis colapsavam na **mesma variável** — costura não-testada clássica
   (`DIRETIVA_IMPLEMENTACAO`, causa nº 1).
4. ⚠️ **E o gate nem correu:** é `#[ignore]` + precisa de GPU. Os 10.515 verdes
   nunca o incluíram.

### A cura — três camadas, e a primeira torna o erro INEXPRIMÍVEL

1. ⭐⭐ **Partir o parâmetro:** `fill(indexed, surface, …)`. ⛔ *Trocar o argumento
   para `&work` teria consertado a geometria e apagado, em silêncio, uma intenção
   declarada e legítima.* **Um erro que a assinatura torna inexprimível não precisa
   de gate.**
2. **Uma pré-condição dentro do `fill`** — o comprimento da polilinha de cada arco,
   medido na malha que se vai amostrar, contra o `arc_length` que o F3 declarou.
   Barra `1e-3`; medido: coerente **1,000 exacto**, destruído **5,40×** (pior arco
   9,04×). Três ordens de grandeza de margem. *E o `get` no lugar do `[]` converte
   de graça o panic do §seguinte numa recusa nomeada.*
3. **Duas asserções GEOMÉTRICAS** no relatório e no gate: aresta **máxima** ≤ 4× o
   alvo e **mediana** na faixa 0,5×–2,0×.
   ⚠️ **As duas, e não uma:** com o defeito reintroduzido, nesta fixtura a máxima
   saiu **1,64× — debaixo da barra** — e quem apanhou foi a mediana (0,27×); na
   fixtura de 98 k a auditoria mediu o inverso (máxima 18×). *O dano geométrico não
   escolhe sempre a mesma régua.*

### ⛔ O segundo defeito, da mesma raiz: o SEGUNDO CLIQUE era panic certo

`remesh_isotropic` leva a malha ao alvo `α × diagonal`, logo **densifica** toda
entrada mais grossa que ~2 500 vértices. O clique 1 deixava ~500 vértices; o clique
2 tomava-os como `reference` e o layout tinha índices até 2 887 contra `len` 492 ⇒
`index out of bounds`, **a janela morria com a peça por gravar** (`catch_unwind` não
cobre este caminho). Reproduzido também com `torus`, `cube`, e com qualquer OBJ
low-poly importado.

### ⛔ E os gates que apontavam para o motor errado

`two_clicks_without_undo_still_return_a_piece` e
`every_point_of_the_detail_slider_returns_a_piece` chamam o irmão **local**. Quando
o botão foi repontado para a cadeia global, **nenhum foi repontado**: a
`quad_remesh_global` ficou com **um** chamador de teste em toda a workspace, com um
clique, `detail = 0,5`, sobre a única fixtura que caía do lado que não partia.
⇒ Os dois ganharam irmão global em `sculpt3d_global_retopo_tests.rs`.

### ⛔ O defeito que o gate novo encontrou, e que fica ABERTO

O **terceiro clique** recusa com `Broken { patch: 6, side: 12 }` — a fronteira de um
patch não fecha. A cadeia nunca tinha corrido sobre uma malha que ela própria
produziu, e o F1 grosseira a peça a cada clique (270 vértices ao clique 2) até o
traçado sair degenerado. ⚠️ **O gate afirma o contrato que de facto importa —
*recusar sem destruir*** — e nomeia o defeito em vez de o esconder.

### ⭐ O que NÃO é problema — não repetir

1. **A reprojeção sobre a malha ORIGINAL está CERTA.** ⛔ Não reverta: o defeito era
   o parâmetro único servir também a indexação.
2. **F1, F2, F3 e F4 estão inocentes** — provado por controlo: os dois ramos
   partilham o **mesmo `layout` e o mesmo `quant`**; um sai bom e o outro sai a foto.
3. ⛔ **Hausdorff vértice→superfície numa direção só é TAUTOLÓGICO aqui**: a malha
   destruída pontua **0,0000** contra 0,0015 da correta, porque `project_onto` é a
   última operação do F5. *A régua destruída pontuava melhor.*

---

## 4-quaterdecies — ⭐ O ARTEFACTO É A FACE DOBRADA, e ela não move nenhuma outra régua

**Achado por:** a segunda foto do Enio — *"o melhor resultado até agora, mas com
artefactos"*: uma grade boa a seguir o fluxo de uma orelha, com **fendas escuras**
ao longo do vinco. E o relatório dizia `casca FECHADA`.

### A espécie, e o detector VALIDADO antes de se acreditar nele

A fenda **não é buraco topológico** — zero arestas de bordo, χ exacta. É **face
dobrada**: a normal aponta contra a superfície por baixo. Ela renderiza como fenda e
**não move nenhuma das réguas que existem**: nem contagem, nem bordo, nem valência,
nem comprimento de aresta.

⚠️ **O detector foi validado primeiro**, e é por isso que os números valem: sobre a
esfera crua (13 824 faces) e sobre a saída do F1 (5 212) ele acusa **zero**; e sobre
a saída da cadeia dois detectores independentes — normal contra a face de referência
mais próxima, e o teste **radial** num sólido estrelado — dão **exactamente o mesmo
número**.

### As três hipóteses, e as duas primeiras caíram

| hipótese | como se testou | veredito |
|---|---|---|
| *o alisamento causa* | varrer 0·1·3·6·12 rondas | ⛔ **refutada**: 405·403·289·205·135 — ele **repara** monotonicamente e nunca cura. *Um remédio que melhora e não cura está a tratar o sintoma.* |
| *a projeção tardia causa* | pousar TODO ponto de interior na superfície ao nascer | ⛔ **quase nada**: 405 → 389 |
| *o alvo esmagado de um arco causa* | varrer 4 leis de peso | ⛔ **refutada, e ao contrário** — ver abaixo |

### ⭐ A varredura das leis de peso do arco, e o que ela mostrou

O custo de um arco desviado era `1 · |x − alvo|` — deviação **absoluta e
uniforme**. ⇒ O solver é **indiferente** entre esmagar um arco que pedia 24
segmentos até 4 e esticar vinte curtos em 1 cada. Medido: o arco `#17` pedia **24,3
e recebeu 4** — uma aresta de 6× o alvo.

| lei de peso | quads | **dobradas** | pior arco |
|---|---|---|---|
| uniforme `\|x−t\|` | 4 922 | 180 | 6,1× |
| ⭐ **relativa `/t`** | 4 099 | **85** | 8,1× |
| proporcional `×t` | 6 887 | 379 | **2,4×** |
| raiz `×√t` | 5 516 | 236 | 3,5× |

⭐⭐ **A que dá o MELHOR pior-arco dá o PIOR número de dobras**, e a que ganha tem o
pior arco de todos. ⇒ *O comprimento de aresta não é o que causa a dobra* — a minha
hipótese estava errada, e só a varredura o mostrou.

⭐ **Entregue: `ArcSpec::relative` no `to_layout`** — dobras a **metade** nas duas
fixturas (180→85 na lisa, 83→43 na esculpida). Prova de mutação: com a lei uniforme
de volta, o gate novo dá **10,5 %** contra a barra de 6 %.

⚠️ **`ArcSpec::new` (peso 1) FICA**, e não é resíduo: o oráculo de força bruta dos
gates do F4 precisa do custo mais simples possível para as duas respostas serem
comparáveis.

### O que a dobra de facto acompanha: o GRÃO

| fixtura (a mesma esfera) | quads | dobradas |
|---|---|---|
| alvo 0,08 | 663 | **31 (4,7 %)** |
| alvo 0,06 | 2 958 | **2 (0,1 %)** |

⇒ *Quanto mais grossa a grade, mais curvatura cada quad atravessa, e o interior
interpolado por Coons afunda para dentro da forma.*

### ⛔ A dívida que fica NOMEADA

1. ⛔ **O leque é a construção errada para um patch de QUATRO lados.** Ele põe um
   centro e `n` sub-grades onde uma grade `L₀ × L₁` simples bastaria — e é entre as
   sub-grades que a dobra aparece. *Próxima medição: trocar o leque por grade plana
   nos patches de valência 4 e recontar.*
2. ⛔ **Sem feature lines** (herdado do F3): a grade não se alinha ao vinco da
   orelha, ela **corre por cima dele**. É o item que separa isto do estado da arte,
   e nenhuma das medições desta secção o toca.
3. ⚠️ **Sem densidade adaptativa**: a grade tem o mesmo passo no vinco e na parte
   lisa. O oráculo também não adapta por omissão, mas o estado da arte adapta.
4. ⚠️ O gate da dobra tem a barra em **6 %** — ⛔ **não é o alvo, é o
   anti-retrocesso.** O alvo é zero.

---

## 4-sexdecies — ⭐⭐ AS LICENÇAS, VERIFICADAS — e o coração do QuadWild é MIT

**Pedido do Enio (2026-08-21):** *"não quero um armengo na engine. Quero portar o
estado da arte. QuadWild Bi-MDF, Instant Meshes, AutoRemesher."*

⚠️ **O veredito do produto é dele e está aceite:** a foto seguinte à do vinco
mostrou a topologia e o seguimento de curvatura **piores**. As secções anteriores
são medição honesta de peças que, somadas, não deram um produto melhor.

### O que se pode portar, verificado ficheiro a ficheiro no clone do oráculo

| peça | onde vive | licença | pode entrar? |
|---|---|---|---|
| ⭐⭐ **Bi-MDF — a quantização** | `libs/satsuma` (**libSatsuma**) | **MIT** | ✅ **SIM** |
| grafo de fluxo | `libs/lemon` | Boost | ✅ |
| malha | `libs/OpenMesh` | BSD-3 | ✅ |
| ⛔ campo cruzado (MIQ) | `libs/CoMISo` | **GPL-3** | ⛔ |
| ⛔ traçado de separatrizes | `libs/xfield_tracer` | **GPL-3** | ⛔ |
| ⛔ estruturas + cola | `libs/vcglib`, `quadwild` | **GPL-3** | ⛔ |
| ✅ **Instant Meshes** | já portado (`ph2d-quadflow`, 5 117 LOC) | BSD-3 | ✅ **já cá está** |
| ⛔ **AutoRemesher** | código próprio MIT, **mas** embute **CoMISo (GPL-3)** e **CGAL** | ⛔ | ⛔ |

⭐⭐ **A peça que faz o QuadWild ser estado da arte em QUALIDADE — o solver Bi-MDF —
é MIT, do próprio autor do paper, e são 3 884 linhas com uma dependência
permissiva.** ⚠️ Isto **muda o ADR-0162**: ele diz *"clean-room a partir dos
papers"* porque presumia que a fonte inteira era GPL. Para esta peça o clean-room
**não é necessário** — um porte fiel com atribuição é legal, e é o mesmo padrão que
o `ph2d-quadflow` (BSD) e o sculpt (MIT) já usam nesta árvore.

⛔ **AutoRemesher está FORA e não é discutível:** o código dele é MIT mas os
`ACKNOWLEDGEMENTS` listam **CoMISo (GPL-3)** e **CGAL** (GPL ou licença comercial
paga). *Um binário MIT que embute GPL é um binário GPL.*

### ⭐⭐ A fixtura que reproduz a FOTO, e que nunca esteve num gate

Os dois backends, no mesmo gesto, na mesma peça:

| fixtura | backend | quads | **bordo** | dobradas | relógio |
|---|---|---|---|---|---|
| amassada, `d=0,5` | GLOBAL | 540 (100 %) | 0 | **0,0 %** | 830 ms |
| amassada, `d=0,5` | local | 378 (83 %) | 3 | 0,8 % | 267 ms |
| ⛔ **com BICO**, `d=0,5` | **GLOBAL** | 260 (100 %) | ⛔ **17** | ⛔ **6,5 %** | 314 ms |
| ⛔ **com BICO**, `d=0,5` | local | 177 (63 %) | **0** | **0,0 %** | 66 ms |
| com CRISTAS, `d=1,0` | GLOBAL | 987 (100 %) | 0 | 0,0 % | 843 ms |

⛔⛔ **Na `hooked_sphere` — a esfera-com-bico, a malha de DIAGNÓSTICO do corpus — a
cadeia global devolve a casca ABERTA**: 11 a 22 arestas de bordo. Não são dobras:
são **buracos**. E é exactamente o que a foto do Enio mostra.

⛔ **E o slider quase não move a contagem ali:** 247 · 260 · 287 · 403 em todo o
curso, contra 429 · 540 · 802 · 1 298 na amassada.

⚠️ **A `hooked_sphere` está no corpus desde o §1.2 como *"o gate de regressão do
§9"* — e nunca entrou num gate.** Todos os gates do gesto usam a `wrinkled_sphere`,
onde tudo dá zero. *A fixtura que contém o fenómeno estava escrita no plano e não
estava no código.*

### O plano que substitui o improviso

| # | peça | fonte | licença |
|---|---|---|---|
| 1 | ⛔ **os buracos na `hooked_sphere`** | nosso defeito | — |
| 2 | ⭐ **Instant Meshes como backend de PRIMEIRA CLASSE** | já portado | BSD-3 |
| 3 | ⭐ **porte fiel do libSatsuma** (Bi-MDF exacto) | `libs/satsuma` | **MIT** |
| 4 | **parametrização por patch** (§4-quindecies) | clássica, sem dono | — |
| 5 | **campo**: o do Instant Meshes no lugar do meu MIQ | já portado | BSD-3 |
| 6 | ⛔ traçado do QuadWild | GPL | **não entra** |

⚠️ **A quantização não é o buraco:** a minha prova o ótimo em todas as seis malhas
do corpus. O porte do libSatsuma vale pela garantia da referência e pelo relógio,
não por corrigir um defeito medido. ⇒ **A ordem acima é por VALOR MEDIDO, não por
prestígio da peça.**

---

## 4-septdecies — ⛔⛔ **NÃO HAVIA BURACO NENHUM**, e o custo do F4 é LINEAR onde a referência o faz QUADRÁTICO

> ⚠️ **Esta secção corrige a anterior em dois pontos, e o segundo muda a ordem
> de ataque inteira.**

### 1. ⛔ A casca nunca esteve aberta — eu li a coluna errada

A sonda do §4-sexdecies imprimia seis números sem rótulo por coluna:

```text
com BICO d=0.50 | GLOBAL 260 quads (100% quads) 19   irreg 0    bordo 17   dobradas (6.5 %)
                                                ^^irreg          ^^bordo    ^^DOBRADAS
```

Eu reportei **«17 buracos»** onde a linha diz `0 bordo` e `17 dobradas`, e abri o
plano com *«os buracos na hooked_sphere»* em primeiro lugar. Medido agora por uma
porta que **não precisa de GPU** (`the_two_engines_on_the_same_piece_without_a_device`,
[`sculpt3d_global_retopo_tests.rs`](../../../shells/desktop/src/sculpt3d_global_retopo_tests.rs)):

| fixtura | arcos com uso ≠ 2 | `boundary_edges` | `open_edges` da saída |
|---|---|---|---|
| com BICO, as 4 densidades | **0 de 50** | **0** | **0** |
| amassada, as 4 densidades | **0 de 67** | **0** | **0** |

*Um arco é partilhado por exactamente dois patches; o censo diz `{2: 50}` em toda
densidade, e é isso que prova a casca fechada — não a ausência de queixa.*

⇒ **Item 1 do plano anterior SAI.** *Seis números lado a lado sem rótulo em cada
um é uma sonda que se pode ler ao contrário — e eu li.*

### ⚠️ E a régua RADIAL, que eu ia acusar, estava certa

Trocá-la por [`ph2d_quadfill::folded_against`] (normal da face da referência mais
próxima) não mudou nada de material — `11·17·19·22` contra `12·17·19·23`. **A troca
fica** por outro motivo: o radial só é válido num sólido **estrelado**, e a
`hooked_sphere` não é um; e agora os **dois backends** são medidos pela mesma régua,
que vive na fase e entra no [`FillReport::folded`] e no `QuadRemeshReport`, não numa
sonda. ⛔ *Não escrevi «a régua mentiu» porque a medição não o disse.*

### 2. ⭐⭐ O defeito REAL, e o mecanismo, com o número

Com a régua no relatório, o estado medido (`detail = 0,5`):

| fixtura | motor | quads | **dobradas** | aresta med | **aresta MAX** |
|---|---|---|---|---|---|
| amassada | GLOBAL | 540 | **0 (0,0 %)** | 0,88× | 1,92× |
| com CRISTAS | GLOBAL | 314 | **0 (0,0 %)** | 1,04× | 2,70× |
| ⛔ **com BICO** | GLOBAL | 260 | ⛔ **17 (6,5 %)** | 0,93× | ⛔ **4,08×** |
| com BICO | local (Instant Meshes) | 177 | 0 (0,0 %) | — | — |

**Dois experimentos controlados** (`whose_fold_is_it_the_construction_or_the_projection`):

| experimento | hipótese que ele mata | resultado |
|---|---|---|
| projetar na **remalhada** em vez de na ORIGINAL | *a projeção salta o pescoço fino do gancho* | ⛔ **REFUTADA**: 17 → 15 (alis=6), 18 → 18 (alis=24) |
| varrer o alisamento `0 · 6 · 24` | *o alisamento causa* | ⛔ **REFUTADA**: 25 → 17 → 18. Ele **repara** e estagna |

⭐ **O que sobra, e o que a terceira coluna mostra:** o alisamento leva a amassada
de 11 dobras a **ZERO** e a com-bico de 25 a **17**. A diferença é a **fronteira**:
o alisamento move o interior do patch e **não desfaz um lado errado**.

E o lado errado tem nome. O défice de cada arco — o que o comprimento pedia contra
o que o F4 lhe deu:

| fixtura | arcos com alvo > 2× o dado | o pior |
|---|---|---|
| ⛔ **com BICO** | **6 de 50** | `#21` pedia **4,1** e recebeu **1** (len 1,105 numa peça de raio 1) |
| amassada | 3 de 67 | `#58` pedia 6,4 e recebeu **2** |

⭐⭐ **Um arco com UM segmento é uma corda recta de canto a canto.** Sobre o gancho
ela atravessa a forma; a grade de Coons construída sobre ela nasce do lado errado, e
nenhum alisamento a traz de volta porque a corda **é** a fronteira.

### ⭐⭐ Por que o F4 faz isso — e é o custo, não o solver

[`ArcSpec::cost`](../../../crates/ph2d-quantize/src/lib.rs) e
[`BiEdge::cost`](../../../crates/ph2d-quantize/src/network.rs) são **`w·|x − t|`**.
A derivada é **constante** de cada lado do alvo, então `segments()` fatia o arco em
**três** degraus de custo plano — e mover uma unidade custa o mesmo quer seja a
primeira ou a décima. ⇒ **esmagar um arco e espalhar o erro custam exactamente o
mesmo**, e o ótimo escolhe livremente.

⛔ **E a `ArcSpec::relative` piora isso, ao contrário do que o doc dela diz.** Com
`weight = 1/alvo`:

| escolha | custo com `w·|x−t|`, `w = 1/t` | custo com `w·(x−t)²`, `w` uniforme |
|---|---|---|
| esmagar o arco `t=4,1` até `1` | `3,1/4,1 =` **0,76** | `(1−4,1)² =` **9,6** |
| espalhar: 3 arcos `t=1` esticados em 1 | `3 × 1,0 =` **3,0** | `3 × 1 =` **3,0** |
| **quem o ótimo escolhe** | ⛔ **esmagar (4× mais barato)** | ✅ **espalhar (3× mais barato)** |

*O raciocínio do doc — «a qualidade de uma grade é uma RAZÃO» — está certo; sobre um
custo LINEAR a implementação fica com o sinal invertido no regime de deviação grande.*

### ⭐ A referência, lida no clone (MIT, `libs/satsuma` + `libs/quadretopology`)

| ficheiro | o que diz |
|---|---|
| `CostFunction.hh` | três custos convexos: `AbsDeviation` `w|x−t|` · **`QuadDeviation` `w(x−t)²`** · `ScaleFactor` `w·max((x+ε)/(t+ε),(t+ε)/(x+ε))` |
| `qr_flow.cpp:178` | `add_subside_edge(..., ObjectiveKind obj = ObjectiveKind::QuadraticDeviation, int lower=1)` — ⭐ **o default de TODA aresta de sub-lado é o QUADRÁTICO** |
| `qr_flow.cpp:477` | `iso_weight / n_subsides` — ⭐ o peso **não** depende do alvo. *A relatividade vem do quadrático, não do peso.* |
| `BiMDF_to_BiMCF.cc:120` | a escada: `arc_cost = (energy(guess+dev) − energy(guess+dev−1))` — **marginais**, `max_deviation = 2`, mais **um arco sem teto** com a marginal média dos 10 seguintes |
| `Highlevel.cc:69` | dupla cobertura → **refinamento por matching**, repetido até o custo parar de descer; `refinement_maxdev_max = 2`, com o comentário *«2 always suffices for an exact solution»* |

⭐⭐ **A máquina para consumir um custo convexo JÁ ESTÁ NA NOSSA CRATE.**
[`BiEdge::step_cost`](../../../crates/ph2d-quantize/src/network.rs) e
[`segments()`](../../../crates/ph2d-quantize/src/solve.rs) fatiam a marginal em
degraus de custo constante — que é exactamente o `consolidate` do `add_edge` da
referência. *Só o custo ficou linear.* ⇒ **O primeiro passo do porte é uma função,
não uma crate.**

### O plano, reordenado por VALOR MEDIDO

| # | passo | onde | o que fecha | como se mede |
|---|---|---|---|---|
| **1** | **custo QUADRÁTICO** (`QuadDeviation` da referência) + peso da referência | `ph2d-quantize`, `ph2d-trace::to_layout` | os arcos de 1 segmento ⇒ as dobras | `> 2×` cai de 6/50 · `folded` · `edge_max` |
| **2** | a **escada limitada** como a referência (`max_deviation = 2` + arco de cauda sem teto) | `solve.rs::segments` | a rede não explodir com custo estritamente convexo | tamanho da rede · relógio |
| **3** | **refinamento** dupla-cobertura → matching, iterado | `ph2d-quantize` | o `gap` sob o custo novo | `Report::gap` |
| **4** | **Instant Meshes como backend de PRODUTO** (hoje só por `PH2D_RETOPO_LEGACY=1`) | shell | o artista poder escolher o rápido e robusto | ambos no mesmo gate |
| **5** | **parametrização por patch** no lugar do Coons/leque | `ph2d-quadfill` | a dobra que sobrar do passo 1 | `folded` |
| **6** | **campo** do Instant Meshes no lugar do MIQ | F2 | os 19-22 irregulares contra o chão de 8 | `irregular` |
| ⛔ | traçado do QuadWild | — | — | **GPL, não entra** |

⚠️ **O que muda em relação ao §4-sexdecies:** lá o porte do libSatsuma valia *«pela
garantia e pelo relógio, não por corrigir um defeito medido»*. **Isso estava errado
por uma razão que eu não tinha medido:** o nosso `gap = 0` é ótimo **do custo
linear**, e é o custo que produz o arco de um segmento. *Uma resposta provadamente
óptima para o objectivo errado.*

### ✅ O passo 1 está FEITO — e o que ele comprou, medido

A varredura das **cinco leis** (`which_arc_weight_law_protects_the_grid`, com
orçamento de busca grande para separar *inviável* de *acabou o tempo*):

| lei | dobras 48×72 | 96×144 | esculpida | **pior arco** | pior relógio |
|---|---|---|---|---|---|
| `abs · 1/t` (a anterior) | 30,4 % | 1,9 % | 0,2 % | 3,2 / **8,1** / 2,6 | 25 ms |
| `abs · 1` | 33,3 % | 3,4 % | 0,1 % | 1,7 / 6,1 / 1,3 | 156 ms |
| `quad · 1` — **o default da referência** | 37,5 % | 2,8 % | 0,1 % | 2,8 / 2,4 / 1,7 | ⛔ **2 744 ms** |
| ⭐ **`quad · 1/t²` — ADOTADA** | **26,2 %** | **1,8 %** | 0,1 % | 2,9 / 6,1 / 2,6 | 76 ms |
| ⏸️ `scale` — o `ScaleFactor` da referência | ⛔ 36,3 % | 2,2 % | **0,0 %** | ⭐ **1,8 / 3,0 / 1,7** | 20 ms |

⭐ **`quad · 1/t²` é `((x−t)/t)²`, o quadrado do erro RELATIVO** — a forma da
referência com o peso que torna o custo uma razão. Ela **domina a lei anterior em
todas as colunas medidas**, então foi adoptada sem tocar em barra nenhuma:

| gate `no_face_folds_back_on_itself` | antes | depois | barra |
|---|---|---|---|
| esfera 48×72 | 33,2 % | ✅ **25,9 %** | 33 |
| esfera 24×36 | — | 12,2 % | 14 |
| esfera esculpida | — | 0,1 % | 0,5 |

⏸️ **A `Deviation::Scale` ganha na grandeza que nomeia o defeito** (o pior arco:
melhor das cinco nas três malhas) e dá **zero** dobras na esculpida. Não foi
escolhida porque reprova a 48×72 — ⛔ e **a barra não se afrouxa**. ⚠️ Mas nessa
fixtura a aresta máxima é **33 a 40× o alvo em TODAS as leis**: ela está no regime
do artefacto de grão (§4-quaterdecies), logo não arbitra entre custos. *Reabrir esta
escolha depois de o grão estar curado é trabalho pendente, não uma recusa.*

### ⛔ E o que o passo 1 NÃO comprou

| fixtura | dobras antes | dobras depois |
|---|---|---|
| ⛔ **com BICO**, `d=0,5` | 17 (6,5 %) | **18 (6,9 %)** |
| amassada, as 4 densidades | 0 | 0 |
| com CRISTAS, as 4 densidades | 0 | 0 |

**A esfera com bico não se mexeu.** O pior arco caiu (`#21 pedia 4,1 e recebeu 1`
desapareceu; os `> 2×` foram de **6/50 para 2/50**) e as dobras ficaram. ⇒ *na peça
que reproduz a foto, a dívida dominante não é a quantização* — são **19 patches**
para uma forma com um gancho, e um leque/Coons sobre um patch grande e curvo. Passo
5 (parametrização por patch) e passo 6 (o campo) são os que a atacam.

### ⛔⛔ E um VERMELHO PRÉ-EXISTENTE que só apareceu porque alguém correu o gate

`three_clicks_in_a_row_still_return_a_usable_piece`, aresta mediana do 3.º clique:

| lei de custo | clique 1 | clique 2 | **clique 3** | barra |
|---|---|---|---|---|
| `abs · 1/t` (a de `5ec438e17`) | 1,05× | 0,76× | ⛔ **0,32×** | 0,5–2,0 |
| `quad · 1/t²` (hoje) | 1,04× | 0,64× | ⛔ **0,43×** | 0,5–2,0 |

⚠️ **O `assert` nasceu em `3d51cf18c` e o `5ec438e17` partiu-o em silêncio**, porque
este gate é `#[ignore]` + GPU e o lote do `ship.sh` **nunca o corre**. *Um gate que
só corre à mão fica verde na memória de quem o escreveu.*

⭐ **A causa, medida:** o `edge_for_detail` deriva o alvo da malha de **entrada**, e
ao 3.º clique ela é a saída grosseira do 2.º (275 vértices). O F1 remalha para
`α × diagonal` e portanto **refina** essa peça ⇒ o layout fica **mais fino que a
densidade pedida**, o piso `ArcSpec::min = 1` morde em quase todo arco, e quem
escolhe o passo da grade passa a ser o piso, não o alvo.

⇒ **passo 7: grosseirar o layout quando ele é mais fino do que o alvo pede** — o
mesmo trabalho que a contagem de patches ~2× acima do necessário já pedia. ⛔ Não é
a barra, e não é a lei de custo (a medição controlada acima prova as duas coisas).

---

## 4-duodevicies — ⭐⭐ A GRADE PASSA A NASCER **NA SUPERFÍCIE**, e o defeito da foto cai 18 → 1

### O que se construiu

Um patch deixa de ser preenchido por interpolação em `ℝ³` seguida de *agarra à face
mais próxima*, e passa a ser **achatado** sobre um polígono convexo — embutimento
baricêntrico de **Tutte** (1963) com **coordenadas de valor médio** (Floater, 2003)
—, com a grade construída **no domínio** e devolvida à malha pela triangulação.
Ficheiro novo: [`param.rs`](../../../crates/ph2d-quadfill/src/param.rs); a montagem
de um patch saiu para [`patch.rs`](../../../crates/ph2d-quadfill/src/patch.rs) (teto
de LOC: 817 contra 700).

⭐ **Tutte é uma GARANTIA e não uma esperança:** fronteira convexa + pesos positivos
⇒ embutimento sem dobras. As coordenadas de valor médio são sempre positivas (a
cotangente **não** é, num triângulo obtuso), então a garantia sobrevive à troca de
pesos.

### O resultado, nas duas réguas

| fixtura | dobras ANTES | **DEPOIS** | aresta máx ANTES | **DEPOIS** |
|---|---|---|---|---|
| ⛔ **com BICO** `d=0,50` | 18 (6,9 %) | ⭐ **1 (0,4 %)** | 5,29× | ⭐ **1,86×** |
| ⛔ **com BICO** `d=1,00` | 22 (6,2 %) | ⭐ **1 (0,3 %)** | 8,16× | ⭐ **2,57×** |
| esfera 48×72 (o gate) | 25,9 % | ⭐ **0,0 %** | 24,81× | ⭐ **4,24×** |
| esfera 24×36 | 12,2 % | ⭐ **1,8 %** | 8,72× | ⭐ **2,58×** |
| esfera esculpida | 0,1 % | ⭐ **0,0 %** | 11,87× | ⭐ **4,69×** |
| amassada · com cristas | 0 | 0 | 3,78× · 5,68× | **2,71× · 5,55×** |

⚠️ **Nenhuma barra foi tocada.** As do gate `no_face_folds_back_on_itself`
continuam `33 / 14 / 0,5`.

### ⛔⛔ E eu quase rejeitei a cura certa — três vezes

| # | o que eu media | o que concluí | por que estava errado |
|---|---|---|---|
| 1 | a `hooked_sphere` sozinha | *"o achatamento não cura"* | ⛔ nessa peça só **1 de 28** pontas de face dobrada é do interior de grade; ela **não contém** o fenómeno que a cura ataca |
| 2 | o polígono **regular** | *"o domínio tem um esticão embutido"* | ⭐ verdade — mas o polígono **proporcional ao comprimento** degenera e **313 pontos** caíram fora do achatamento (dobras 2,1 % → 13,2 %). **Medido e rejeitado** |
| 3 | o centro do leque na **origem** | — | ⭐⭐ era **aqui**: o centro tem de ser o **centróide dos cortes no domínio**. Com essa linha, 48×72 foi de `2,1 %` a **`0,0 %`** e a aresta máxima de `24,81×` a **`4,24×`** |

⇒ ⚠️ **Uma cura pode ser boa e a medição dela ser feita numa fixtura que não a
contém.** O que salvou foi correr o **gate da crate** (que mede outras três malhas)
em vez de só a sonda do shell. *Julgar uma cura pela peça mais atípica é o mesmo
erro que julgá-la pela mais fácil, com o sinal trocado.*

### Os instrumentos que decidiram, e ficam

| instrumento | o que responde |
|---|---|
| [`folded_by_neighbours`] | a **2.ª régua**, que não consulta a referência — piso de ruído `3/3 566` contra `24/3 566` da 1.ª na `hooked_sphere` |
| [`FillReport::folded_prov`] | **de que FASE** são os vértices das faces dobradas — foi ela que disse *"1 de 28 é grade"* |
| `flattened` · `sampled` · `sample_misses` | se a cura **correu** e se ela **colocou** ponto — `19/19` patches com `0` pontos fora |
| censo de arcos, agora **asserção** | todo arco citado por exactamente **2** lados: é a prova de que a casca fecha |

⚠️ **O controlo da 1.ª régua na peça do diagnóstico:** ela acusa **24 de 3 566**
faces da malha **remalhada isotropicamente**, que não tem dobra nenhuma. *Um piso de
0,7 % debaixo de um sinal de 7 % ainda deixa o sinal — mas não se optimiza contra uma
régua sem se conhecer o piso dela.*

### ⛔ E o que continua ABERTO, com a causa afiada

O 3.º clique seguido: mediana `0,16×` o alvo. ⭐ **A causa agora tem nome:** ao 3.º
clique a entrada é a saída grosseira do 2.º, o alvo do slider fica grande, o F1
devolve um `work` fino, e os arcos ficam muito mais curtos que o alvo — com
`ArcSpec::min = 1` cada um leva **um** segmento, e **a densidade da saída passa a ser
a do LAYOUT, não a do slider**.

⛔ **Derivar o alvo do `work` foi construído e MEDIDO:** os cinco pontos do slider
passaram a dar `405 · 405 · 406 · 406 · 451` quads contra `405 … 1 336`, porque o
extremo fino do curso passava a ancorar-se numa constante (`ALPHA × diagonal`).
*Trocar um defeito no 3.º clique por um slider morto não é uma troca.* ⇒ a cura é
**grosseirar o layout**, e continua a ser o passo 7.

---

## 4-undevicies — ⭐⭐ O TETO ERA DO MOTOR ERRADO: `1 336` → `20 039` quads, e o `Follow Curvature` estava MORTO NA ORIGEM

> Dois reports do artista, e os dois eram defeitos reais: *"o máximo de Detail ainda
> é de muito baixa resolução"* e *"Follow Curvature não funciona"*.

### 1. ⛔ O piso do slider era do motor LOCAL

A [`edge_for_detail`](../../../crates/ph2d-quadflow/src/scale.rs) ancora o extremo
fino em `FLOOR_IN_INPUT_EDGES = 3,0` **arestas da malha de entrada**, e esse número
foi medido para a extração por retícula do porte local: ali um quad mais fino que o
triângulo de entrada devolve um ciclo de 352 lados com 58 % do volume perdido (a
foto de 2026-08-19). ⚠️ **A cadeia global não extrai de retícula nenhuma** — ela
reamostra arcos por comprimento e amostra o interior dentro de um triângulo
achatado. *O caminho mais limitado definia o teto do outro* (CLAUDE.md §0.0).

Medido **sem tocar no mapa de patches** (28 patches, 67 arcos, `work` de 5 136
triângulos), só descendo o alvo:

| alvo | quads | dobras | mediana | **máx / diagonal** | F4 |
|---|---|---|---|---|---|
| `3,00×` (o piso local) | 1 336 | 0,00 % | 1,03× | 7,2 % | 24 ms |
| `1,50×` | 4 885 | 0,02 % | 1,05× | 6,4 % | 148 ms |
| ⭐ **`0,75×` — ADOTADO** | **20 039** | **0,03 %** | **1,03×** | **5,1 %** | **2,7 s** |
| `0,54×` | 38 315 | 0,08 % | 1,03× | 4,5 % | 1,5 s |
| ⛔ `0,375×` | 78 883 | 0,09 % | 1,04× | 4,8 % | ⛔ **50 s, sem prova** |

⭐ **O detalhe não se perde ao pedir quads mais finos que o `work`:** todo ponto de
interior é reprojectado sobre a malha **ORIGINAL** do artista, não sobre a remalhada.

⚠️ **De que recurso é o novo teto:** do **relógio da quantização**, e de mais nada.
As dobras ficam em 0,03 %, a mediana em 1,03× e a aresta máxima **em fração da peça
até melhora**. O que explode é a busca do F4.

O slider inteiro, medido pelo gesto: `405 · 496 · 1 336 · 4 885 · **20 039**` quads.

### ⭐ E a barra da aresta máxima media a grandeza errada

O `assert` dizia *"alguma coisa na malha atravessa a peça"* e comparava
`edge_max / alvo`. Isso é uma razão a um denominador que o slider encolhe:

| alvo | quads | `máx / alvo` | **`máx / diagonal`** |
|---|---|---|---|
| `3,00×` | 1 336 | 2,71× | **7,2 %** |
| `0,75×` | 20 039 | 7,71× | **5,1 %** |
| `0,54×` | 38 315 | 9,48× | **4,5 %** |

⭐ **A razão triplica e a fração melhora** — não há defeito nenhum, muda o
denominador. E o defeito que a barra existe para apanhar (a geometria montada sobre
índices de outra malha) media **2,01 numa peça de diagonal 3,46 = 58 %**. ⇒ a barra
passa a ser **20 % da diagonal**, com onze vezes de margem, e deixa de apertar
sozinha. ⛔ *Isto não é afrouxar: é medir a grandeza que a asserção afirma.*

⭐⭐ **E com ela o `three_clicks_in_a_row_still_return_a_usable_piece` passou** —
vermelho desde `5ec438e17`, agora `20 039 · 41 619 · 19 411` quads com mediana
`1,03 · 0,99 · 1,13`.

### 2. ⛔⛔ O `Follow Curvature` estava morto — e não ficava neutro, MANDAVA

Duas causas, uma em cada motor:

**(a) A cadeia global não o lia.** O knob era consumido só pelo porte local; o motor
por omissão é o global, e o log limitava-se a avisar. *Um aviso no terminal não é
uma feature.* ⇒ a densidade entra agora pelo
[`PatchLayout::grade`](../../../crates/ph2d-trace/src/lib.rs), que gradua o **`τ`** —
o comprimento efectivo cumulativo de cada arco. ⭐ **Um só número, lido pelas duas
fases:** o F4 tira dele o alvo de quads e o F5 reamostra a cadeia por ele, então *é
inexprimível* o alvo dizer uma densidade e a amostragem realizar outra.

**(b) O campo de tamanho COLAPSAVA numa constante.** O `lo`/`hi` da
[`ScaleField::adaptive`](../../../crates/ph2d-quadflow/src/scale.rs) são recortados
pelo piso — e o piso era sempre o do motor local. Com um alvo mais fino que ele,
`lo == hi` e **todo vértice recebia o mesmo tamanho**. Medido: `min = mediana = max
= 0,2301` com alvo `0,0910`, em `adapt = 0,5` **e** em `adapt = 1,0`.

⚠️ **E o sintoma não era "não adapta":** o campo constante valia `2,5×` o alvo,
então o knob **grosseirava a peça** — 451 quads em vez de 1 336. *Um knob que
colapsa não fica neutro; ele passa a mandar.*

### ⚠️ Quanto do adensamento chega à saída — e o CONTROLO que o mede

| campo | contraste do campo | dispersão realizada nos quads |
|---|---|---|
| `adapt = 0` | 1,0× | 1,32× (o basal) |
| `adapt = 1` (curvatura) | 3,2× | 1,30× |
| ⭐ **sintético, 9× de contraste** | 9,0× | **2,21×** |

⭐⭐ **O controlo sintético é o que torna a linha do meio legível.** Ele não é um
modo do produto: é a metade que separa *"a graduação não chega à saída"* de *"a
curvatura desta peça não pede muito"*. Com 9× o mecanismo entrega 2,2× ⇒ **a
máquina funciona e o LAYOUT comprime**: a lei do patch (`L_i = e_{i-1} + e_{i+1}`)
obriga lados opostos a bater, e com 28 patches numa esfera a adaptação quase não é
exprimível.

⇒ **Isto é a mesma dívida do passo 7** (grosseirar/afinar o layout), agora com um
segundo sintoma a apontá-la. *A adaptação não se afina no campo: ela precisa de mais
singularidades para caber.*

### ⛔ Medido e rejeitado neste passo

| tentativa | por quê parecia certo | o que a medição disse |
|---|---|---|
| pesos **cotangente** no achatamento | é o mapa quase-conforme, e a distorção de área era o suspeito das arestas longas | ganho marginal (máx `8,14×` → `7,71×`) e **pior no gate** (`0,0/1,9/0,0` contra `0,0/1,8/0,0`). ⇒ **as arestas longas não são distorção do mapa** |
| acoplar a remalha do F1 ao alvo | pedir quads mais finos que o triângulo do F1 é pedir detalhe já deitado fora | ⛔ **271 patches, 778 arcos, quantização em 176 SEGUNDOS**. *Mais patches é o eixo caro; mais segmentos no mesmo mapa é o barato* |

### ⛔ O que fica ABERTO, com o número

⚠️ **A quantização recusa em alvos isolados** (`Exhausted { solves: 512 }` em dois
dos sete pontos medidos), e a escala dela é brutal: **28 patches = 1 ms · 72 = 52 ms
· 271 = 176 s**. É o mesmo dono das três dívidas abertas — o teto da resolução, a
recusa, e a compressão da adaptação.

⇒ ⭐⭐ **O porte do solver do libSatsuma deixa de ser "pela garantia e pelo
relógio" e passa a ser o desbloqueador medido.** A referência não faz
ramifica-e-limita: ela aproxima por **dupla cobertura** e depois **refina por
matching** com `max_deviation = 2`, iterando até o custo parar de descer
(`Highlevel.cc:69`). *É o passo 3 do plano, e agora ele é o passo 1.*

---

## 4-vicies — ⭐⭐ A ORELHA: a fixtura que faltava, e o dilema em DUAS LINHAS

As fotos de 2026-08-22 trouxeram uma feição que **nenhuma fixtura tinha**: uma
borda saliente com um **vinco fundo e côncavo** colado a ela. A `wrinkled` tem
sulcos rasos, a `ridged` relevo convexo, a `hooked` uma protuberância esticada.

⇒ [`eared_sphere`](../../../shells/desktop/src/sculpt3d_fixtures.rs) + a cena de
smoke **`=36`**, esculpidas com os verbos do produto.

### ⭐ A régua que faltava, e o SENTIDO dela

[`detail_lost(referência, saída)`](../../../crates/ph2d-quadfill/src/report.rs):
para cada vértice da **original**, a distância ao ponto mais próximo da saída.

⛔ **O contrário é tautológico.** A última coisa que a montagem faz é pousar cada
ponto na referência ⇒ `saída → referência` dá ~zero **mesmo com a peça destruída**
(medido a 21/08: `0,0000` na destruída contra `0,0015` na boa — *a destruída
pontuava melhor*). Nenhum campo do relatório via uma orelha achatada: 100 % de
quads, casca fechada, irregulares em ordem.

### ⭐⭐ O dilema, medido — e não há α bom hoje

| `α` do F1 | `work` | **o F1 sozinho já perdeu** | patches | quads | irreg | dobras | perdeu (p95 / máx) |
|---|---|---|---|---|---|---|---|
| `0,020` (o de hoje) | 5 104 tris | máx **0,562 %** da diagonal | ⛔ **6** | 16 489 | 6 | 8 (0,05 %) | 0,097 % / 0,430 % |
| `0,010` | 20 718 tris | máx 0,317 % | **218** | 32 335 | ⛔ **296** | 118 (0,36 %) | 0,038 % / **0,275 %** |

⛔ **SEIS patches numa esfera com uma orelha inteira.** O campo mal a vê: o F1
remalha para `α × diagonal = 0,04` e a peça do artista tem aresta média `~0,024`
— *a primeira coisa que a cadeia faz é ficar mais grosseira que a malha que ele
entregou*. Com 6 patches o layout é um cubo-mapa e a grade não tem nada que siga o
vinco: é a **3.ª foto**, a orelha lida como um calombo liso.

⭐ **O `α` fino resolve a forma** — a perda cai para metade — e **paga com 296
irregulares, 0,36 % de dobras e minutos de relógio**. ⇒ **não existe `α` bom hoje**,
e não porque o valor esteja mal escolhido: porque o eixo que ele move (mais
patches) está bloqueado pelo mesmo sítio que tudo o resto.

### ⭐ O rasgo da 2.ª foto: a projeção atravessava o vinco

`project_onto` é o **ponto mais próximo**, e dentro de uma concavidade o pé mais
próximo pode estar do **outro lado da dobra** — o eixo medial encosta na
superfície, então dois pontos a milímetros um do outro têm pés opostos e a face
entre eles vira uma lasca.

⇒ [`project_facing`](../../../crates/ph2d-remesh-iso/src/lib.rs): uma face
candidata só entra se a normal dela concordar, com **queda para o mais próximo**
quando nenhuma concorda. ⭐ A normal viaja com o ponto desde o
`PatchParam::sample` — *ele sabe de que lado veio, e essa informação estava a ser
deitada fora uma linha antes de quem precisava dela.*

⛔ **E pô-la no ALISAMENTO foi medido e rejeitado.** Parece a irmã e não é: lá a
normal é um **facto** (o ponto nasceu sobre uma face concreta); no alisamento seria
a normal de vértice da malha **que o próprio alisamento está a mexer**. Medido na
esfera 24×36: dobras de **1 para 10**, aresta máxima de `2,58×` para `5,85×`.
*Uma estimativa que se realimenta é pior que nenhuma.*

### ⇒ As QUATRO dívidas abertas têm um dono só

| dívida | o número |
|---|---|
| o teto da resolução | 50 s e sem prova um degrau abaixo do teto actual |
| as recusas isoladas | `Exhausted` em 2 de 7 alvos medidos |
| a adaptação comprimida | 9× de campo ⇒ 2,2× na saída |
| ⛔ **a orelha achatada** | 6 patches; o `α` que a resolve custa **176 s** |

**Todas** esperam a mesma coisa: um F4 que aguente 200+ patches. ⇒ o porte do
solver do libSatsuma (dupla cobertura + refinamento por matching, `max_deviation =
2`) é a **única** peça na frente das quatro.

---

## 4-vicies-semel — ⭐⭐ **O CAMPO NÃO TEM COMO OBEDECER AO RELEVO**, e agora há um número

> Report do artista, terceira vez com a mesma palavra: *"sem nenhuma obediência ao
> relevo, à topologia"*.

### A causa está escrita na energia

```text
E = Σ_e w_e · ( θ_f − θ_g + κ_e + (π/2)·p_e )²
```

⛔ **É SÓ suavidade.** Não existe um único termo que puxe a cruz para a direção em
que a superfície dobra. O campo mais suave sobre uma esfera com duas orelhas é o
campo de uma esfera lisa — *ele não tem como ver as orelhas*. **Um alinhamento é um
TERMO, não uma afinação.**

### ⭐ A régua que transforma o report em número

[`follows_relief(referência, saída)`](../../../crates/ph2d-quadfill/src/report.rs):
o desvio médio, em graus, entre cada aresta da saída e a direção principal de
curvatura da peça ali — **4-RoSy** (dobrado em `[0°, 45°]`, porque uma grade rodada
90° está alinhada) e **ponderado pela anisotropia** (numa esfera não há direção
preferida, e o desvio ali é ruído).

⚠️ **O ponto de comparação é `22,5°`** — a média de um ângulo uniforme, ou seja
**uma grade que ignora o relevo por completo**.

| fixtura | **cadeia GLOBAL** | porte do Instant Meshes |
|---|---|---|
| ⭐ **com CRISTAS** (a mais anisotrópica) | ⛔ **25,7°** — *pior que aleatório* | ⭐ **13,7°** |
| com BICO | 24,6° | 22,3° |
| amassada | 22,9° | 22,9° |
| ORELHA | 22,9° | 21,6° |

⭐⭐ **O controlo positivo é o que torna a tabela conclusiva:** o porte do Instant
Meshes — que semeia o campo **na superfície** — dá `13,7°` na fixtura com relevo
direcional, e a nossa cadeia dá `25,7°`. *Se os dois dessem o mesmo número, ou a
régua não media nada ou o diagnóstico estava errado.*

### O que se construiu

1. [`ph2d_mesh::principal_dirs`](../../../crates/ph2d-mesh/src/curvature_dirs.rs) —
   a segunda forma fundamental por face, clean-room de **Rusinkiewicz 2004**, com a
   **anisotropia** normalizada como confiança.
2. `Dual::align` — a direção principal de cada face, na moldura dela, reduzida ao
   4-RoSy.
3. O termo `λ·c_f·(θ_f − α_f)²` na energia — diagonal em `A`, `α` no `b`, com o
   representante 4-RoSy escolhido pelo `θ` corrente.

### ⛔⛔ E ele NÃO SHIPA hoje — `ALIGN_WEIGHT = 0`, por medição

| peso | desvio ao relevo | patches | maior valência | a cadeia fecha? |
|---|---|---|---|---|
| **`0` (hoje)** | 25,7° | **21** | **5** | ✅ |
| `0,01` | ⭐ **20,4°** | ⛔ 104 | ⛔ 15 | ⚠️ fecha, com 139 irregulares e 23 dobras |
| `0,03` | — | 134 | ⛔ **60** | ⛔ a montagem recusa |
| `1,0` | — | 98 | ⛔ **98** | ⛔ recusa |

⭐ **O termo move a agulha certa** e ⛔ **o layout explode de 21 para 104 patches
com um peso de `0,01`**. A causa não é o termo: é que **uma perturbação minúscula de
`θ` troca quais inteiros o arredondamento guloso congela**, e cada troca é uma
singularidade a mais. *Um patch de 60 lados não é «mais detalhe»: é traçado partido.*

⛔ **Suavizar o guia foi construído, medido e REJEITADO no mesmo dia** — média
4-RoSy sobre o grafo dual, transportada pelo `κ`, 32 rondas: o desvio **piorou**
(`20,4° → 24,5°`) e o layout continuou a explodir. *A causa não é a qualidade do
guia; é a fragilidade do arredondamento.*

⇒ ⭐⭐ **O próximo passo tem nome e é o mesmo dono de sempre:** o arredondamento do
MIQ tem de aguentar o termo. A referência não congela guloso sobre um `θ` que
acabou de mudar — e é a mesma família de cura que o F4 acabou de receber
(§4-undevicies): *aproximar, re-centrar, repetir*, em vez de decidir de uma vez.

---

## 5 — Risco, por ordem de quanto pode custar

| risco | por que é real | mitigação |
|---|---|---|
| ~~⭐ **Bi-MDF do zero** (F4)~~ | ~~não há min-cost-flow permissivo em Rust com grafo bi-dirigido e custo convexo~~ | ✅ **RISCO FECHADO em 2026-08-20** pela mitigação que esta linha prescrevia: protótipo medido sobre os patches exportados pelo oráculo, **antes** do F3. Fecha com ótimo demonstrado (§4-quater) |
| **rounding do campo divergir** (F2) | o paper descreve a energia; **a ordem do rounding** decide onde as singularidades caem, e é onde o oráculo tem escolhas não publicadas | comparar contagem+posição de singularidades contra o `.rosy` que o oráculo já escreve |
| **a lei de densidade do estágio 2** | o oráculo dá ~4 500 vértices de qualquer entrada; não sabemos de onde vem | varrer os presets do oráculo e regredir a lei a partir das saídas — é medição, não leitura de fonte |
| **degenerescências em sculpts** | `sculpt_punctured` sai com 88 arestas de borda **no oráculo** | a sanitização é nossa e é F1, não um "detalhe" |
| **`f64` × `Mesh` em `f32`** | determinismo cross-platform tem de ser afirmado onde ele vive | núcleo em `f64`, fronteira explícita, hash da saída no gate 3-OS |
| **custo do Bi-MDF em malha grande** | o oráculo levou minutos em algumas | modo qualidade é **offline com barra**; o preview BSD cobre o laço interativo |
| ⚠️ **ambiguidade dos papers** | inevitável num clean-room | toda dúvida vira uma linha neste plano com o experimento que a resolveu — ⛔ nunca uma olhada na fonte GPL |

---

## 6 — Esforço e ordem de ataque

Ordem recomendada: ~~F0 → F1 → F2 → (F4 protótipo) → F3 → F5 → F2-bis → F3-bis~~ **→ F5.2 (a porta) → F3-ter (fusão de separatrizes) → F2-bis → F7 → F6 → F8**.

⚠️ **A ordem mudou em 2026-08-21, e a razão é uma medição.** A sanitização entrou
à frente do F5 como este parágrafo mandava, e ⭐ **a causa não era o F1**: era uma
colisão de diagonal no flip da `ph2d-mesh` (§4-septies). Curá-la destravou três
malhas do corpus — e **expôs o gargalo seguinte**, que é o campo (§4-octies).

⛔ **A porta no shell (F5.2) desce na fila de propósito.** Ligar o botão hoje é
ligar o caso mau: sobre malhas com distribuição irregular — que é o que sai de um
sculpt real — o campo dá **194 singularidades** onde a topologia admite 8. *Uma
porta aberta para o pior caso é pior que porta nenhuma, porque o veredito do Enio
seria sobre o defeito e não sobre a cadeia.*

⚠️ **F4 saiu da ordem natural de propósito, e a aposta pagou.** Ela era a fase de maior risco e a que o
produto inteiro depende; medi-la sobre os patches que o oráculo **já exporta** custou uma jornada e
respondeu cedo a pergunta que podia matar o plano. *Descobrir no fim que o solver não fecha seria
descobrir tarde demais.* ⭐ **O que sobra agora é uma cadeia sem furos de risco: o F3 produz o layout
que o F4 já sabe consumir, e o F5 consome o que o F4 já sabe produzir.**

| fase | esforço (jornadas de agente) | observação |
|---|---|---|
| F0 | 0,5 | o grosso já está feito (§1) |
| F1 | 1,5 | remesh isotrópico é bem documentado |
| F2 | 2–3 | a fase que mais move a métrica |
| F3 | 2 | |
| F4 | **3–5** | ⭐ a crítica; a variância está aqui |
| F5 | 2 | |
| F7 | 1 | o porte BSD já existe |
| F6 | 2 + tablet | ⚠️ a pressão é outro projeto |
| F8 | 3 + DAG | ⚠️ idem |

---

## 7 — O que este plano ainda NÃO sabe

1. ⛔ **A metade ADAPTATIVA da lei de densidade** — a uniforme está medida e portada (§4-bis); a que
   segue a curvatura, não. ⚠️ **Subiu para o primeiro lugar em 2026-08-20**: o F4 mostrou que, com um
   alvo por arco vindo de uma aresta **média**, o `cube` bate o oráculo **exato** (0 %) e as formas
   curvas erram entre −15 % e +24 % (§4-quater). *A incógnita deixou de ser teórica — ela é o maior
   termo de erro que resta na contagem de quads.*
2. ⛔ **O QEx 2013** não foi obtido.
3. ⛔ **Ninguém mediu Hausdorff** ainda: `metrics.py` cobre contagem, topologia e ângulo; a fidelidade
   geométrica é a lacuna do F0.
4. ⛔ **O preset `Mechanical` não foi corrido** — só o `Organic`. Um segundo eixo por medir.

---

## 4-duovicies — ⭐⭐ **A EXPLOSÃO NUNCA FOI DO TERMO: ERA DO `θ = 0`**

> O §4-vicies-semel fechou com um passo *nomeado*: *"o arredondamento do MIQ tem de
> aguentar o termo — a referência não congela guloso sobre um `θ` que acabou de
> mudar."* Este passo executa-o, e a hipótese estava certa.

### O mecanismo, escrito por extenso

O arredondamento guloso congela `livres/8` inteiros **depois da primeira
resolução** e nunca os revisita. Sem alinhamento isso é benigno: o `θ` da primeira
resolução **já é** o campo suave, e as resoluções seguintes só o afinam.

⛔ **Com alinhamento, a primeira resolução parte de `θ = 0`.** O representante
4-RoSy de cada face é escolhido *pelo `θ` corrente* — `k = round((θ_f − α_f)/(π/2))`
— então a `θ = 0` o alvo de toda face é `α_f` dobrado para `[−π/4, π/4]`, que **não
é** o braço que o campo convergido quer. O guloso congela um oitavo dos inteiros
sobre esse alvo errado, e **cada inteiro errado é uma singularidade a mais**.

⇒ *Não era o termo que partia o traçado: era decidir sobre um `θ` que ainda ia
mudar.* A mesma doença que o F4 tinha (§4-undevicies), no outro solver.

### As três curas construídas, e a tabela que escolheu uma

`ph2d-crossfield/src/continuation.rs` (irmão novo, o `solve.rs` bateu no teto da
HR-18 a 703 linhas). Sonda `does_the_continuation_save_the_alignment_term`, fixtura
**com cristas**, peso `0,01`. A base sem alinhamento é *21 patches · valência 5 · 14
irregulares · 0 dobras · **25,7°***:

| lei | resoluções | patches | valência | irreg | dobras | ⭐ relevo |
|---|---|---|---|---|---|---|
| `cru` (a lei antiga) | 56 | ⛔ **104** | ⛔ 15 | ⛔ 139 | ⛔ 23 | 20,4° |
| ⭐⭐ **`warm`** (só a semente) | 112 | **17** | **4** | **10** | **0** | ⭐ **16,7°** |
| `warm` + 4 re-centragens | 168 | 17 | 4 | 10 | 0 | 16,7° |
| `rampa3` sem semente | 224 | 18 | 7 | 26 | 2 | 16,6° |
| `warm` + `rampa3` | 280 | 17 | 5 | 14 | 3 | 17,5° |
| `warm` + `rampa3` + re4 | 448 | 6 | 4 | 11 | 0 | ⛔ 24,2° |

⭐ **A semente sozinha ganha em todas as colunas — e ganha à própria base.** Menos
patches (17 contra 21), menos irregulares (10 contra 14), zero dobras, e o relevo de
**25,7° para 16,7°** (a grade que não olha vale 22,5°; o porte do Instant Meshes
mede 13,7°). Ela é **uma passagem inteira com `align = 0` cujo único produto que
sobrevive é o `θ`** — os inteiros dela vão para o lixo de propósito.

⚠️ **E a semente cura a explosão em TODA a varredura, não só a `0,01`.** Onde a lei
antiga dava 104 / 132 / 166 patches (cristas) e 133 / 180 / 167 (orelha), com
recusas de montagem a `0,05` e `0,2`, a semente devolve **15 a 21 patches e valência
4 a 6 em todas as células medidas**.

### ⛔ E duas das três curas foram MEDIDAS E REJEITADAS — o mecanismo tem nome

A escada de pesos (`ramp_steps`) e as re-centragens não são caras: são **erradas ao
peso que interessa**. A última linha custa **8× as resoluções** e devolve `24,2°` —
praticamente uma grade que não olhou.

⇒ **O árbitro entre passagens é o `objective`, que soma suavidade + alinhamento.** A
`0,01` a **suavidade domina**, então "melhor objectivo" ≈ "mais liso" — e o campo
mais liso é exatamente aquele que ignora o relevo. *Um laço que melhora o número que
mede não melhora o número que importa, quando são dois números.*

⚠️ **Ficam como parâmetros, não como código morto** — são eles que tornam esta
tabela reproduzível, e a rejeição é do **default**, não da existência.

### ⛔⛔ E O PESO **NÃO SHIPA**, por uma razão que não é o alinhamento

A varredura fina (`how_much_alignment_can_the_field_take`, cinco fixturas × oito
pesos, agora com a semente ligada) elegeu **`0,03`**: ele leva o desvio ao relevo de
`25,7°` para **`13,7°`** na fixtura com cristas — *o número do Instant Meshes* — e
faz o mesmo na enrugada, com as dobras em zero-ou-uma nas cinco peças.

⛔ **E ele reprovou no gate de topologia, que nasceu para o julgar.** Característica
de Euler da saída, onde um toro exige `0`:

| fixtura | peso `0` | peso `0,03` |
|---|---|---|
| toro 32×16 | ✓ `0` | ⛔ **`2`** |
| toro 48×24 | ⛔ **`2`** | ✓ `0` |
| toro 64×32 | ✓ `0` | ⛔ **`2`** |

⇒ `0,03` parte **dois** dos três; o zero parte **um**. Ligá-lo hoje é trocar relevo
por regressão de topologia, e topologia não se negoceia. **`ALIGN_WEIGHT` fica em
`0`**, agora com a tabela do relevo **e** a da topologia ao lado.

---

## ⛔⛔ O ACHADO: **O TRAÇADO PERDE ASAS**, e é anterior a tudo isto

⚠️⚠️ **A linha do meio da tabela é a notícia.** O toro 48×24 sai errado **hoje, no
produto, sem alinhamento nenhum** — e passa em **todas** as outras réguas: 100 % de
quads, zero arestas de bordo, **zero não-manifold**, contagem de irregulares na ordem
certa. *Uma peça pode passar em toda asserção e ter deixado de ser um toro.*

⛔ **Ninguém o tinha medido porque nenhuma fixtura gateada o continha:** o único toro
do corpus era o 32×16, e nele o defeito não aparece
(`reference_topic_fixture_discipline`).

### ⭐ E a sonda já disse ONDE — `where_is_the_genus_lost`

Ela compara três `χ`: o da malha de entrada, o do **complexo de patches**
(`V − E + F` sobre *cantos · arcos · patches*) e o da malha de saída.

| fixtura | peso | entrada | **complexo (F3)** | saída (F5) |
|---|---|---|---|---|
| toro 32×16 | `0` | `0` | ✓ `0` | ✓ `0` |
| toro 32×16 | `0,03` | `0` | ⛔ **`2`** | ⛔ `2` |
| toro 48×24 | `0` | `0` | ⛔ **`2`** | ⛔ `2` |
| toro 64×32 | `0,03` | `0` | ⛔ **`2`** | ⛔ `2` |

⭐⭐ **O complexo já sai errado antes de a montagem começar**, com **todos os arcos
usados exactamente duas vezes** e **nenhum patch não-disco** — ou seja, a colagem é
uma superfície fechada legítima, só que **de género errado**. A montagem realiza
fielmente o que o layout manda.

⇒ **A asa perde-se no F3.** O F1 entrega `χ = 0`, o F5 é fiel: sobra o traçado.

### O que ficou construído, e o endereço do bloqueio

| peça | onde | estado |
|---|---|---|
| a semente do `θ` (a cura da explosão) | `ph2d-crossfield/src/continuation.rs` | ⭐ medida e no lugar, **inerte** enquanto o peso for `0` |
| o termo de alinhamento + a varredura | `ALIGN_WEIGHT` | ⭐ funciona (`25,7° → 13,7°`), **desligado** pelo bloqueio |
| ⭐⭐ o gate do género | `ph2d-quadfill/tests/alignment_topology.rs` | varre `[0, ALIGN_WEIGHT]` — **liga-se sozinho** quando a constante sair de zero |
| ⛔ o vermelho pré-existente | `the_genus_survives_on_every_torus` | `#[ignore]`, com o mecanismo no doc |
| a rede da porta (`aligned` no relatório) | `quad_remesh_global` | guardada por `ALIGN_WEIGHT == 0`, para não pagar a cadeia duas vezes por nada |

⇒ ⭐⭐ **O próximo passo tem nome e mudou de dono:** *o traçado (F3) tem de preservar
o género*. O arredondamento do MIQ — o bloqueio de ontem — **está resolvido**; o que
está na frente agora é um defeito de correção que já existia e que ninguém via.


---

## 4-tervicies — ⭐⭐ **A ASA CABIA NUM PATCH, e a cerca que existia era CEGA a ela**

> O §4-duovicies fechou com o passo nomeado: *"o traçado (F3) tem de preservar o
> género"*. Este passo mede **onde exactamente** ele o perde, e põe a cerca.

### ⭐ O mecanismo, com a aritmética a bater

Sonda `is_every_patch_a_disk` — o `χ` da **região de faces** de cada patch:

| fixtura | patches | discos | anéis | pior | o traçado diz não-disco |
|---|---|---|---|---|---|
| toro 32×16 (bom) | 27 | 27 | 0 | 0 | 0 |
| ⛔ **toro 48×24 (mau)** | 9 | 8 | 0 | **1** | **0** |
| toro 64×32 (bom) | 23 | 23 | 0 | 0 | 0 |

⛔ **Patch 0 do toro mau tem `χ = −1` e `loops_per_patch = 1`.** Ele **engoliu a asa
inteira** — e tem **uma** fronteira, então a única cerca que existia (*"a fronteira é
um laço só"*) deixava-o passar.

⭐⭐ **A régua completa é `χ = 2 − 2g − b`.** Contar fronteiras dá o `b` e apanha o
anel (`b = 2 ⇒ χ = 0`); **o género só aparece quando se mede o `χ` também**. E a
aritmética fecha: um patch com `χ = −1` contado como disco (`+1`) sobre-estima o
complexo em exactamente `2` — que é a diferença medida (`2` em vez de `0`).

### ⛔ E a cura óbvia foi construída, MEDIDA e REJEITADA no mesmo passo

Pôr `χ ≠ 1` no `degenerate()` parecia a linha de uma só palavra. O laço de limpeza
cura degenerados com `dissolve`, que **apaga uma parede** e faz o patch **crescer** —
a direcção errada para uma asa. Medido: **27 patches viraram 1**, e a decomposição
inteira foi comida antes de a cadeia recusar.

⚠️ **E `χ(região) == 1` nem sequer é a condição certa para o corte.** Uma asa cortada
por um laço não-separante continua a ser a **mesma região de faces** (o `flood` não
duplica vértices), logo o `χ` dela não muda — a condição seria inalcançável, e o laço
nunca convergiria. *A estrutura CW mínima de um toro é UM patch com uma aresta dupla,
e ela é perfeitamente válida.*

### ⭐⭐ A cerca certa é a do COMPLEXO, e ela é exacta

```text
    V(cantos) − E(arcos) + F(patches)  ==  χ(malha)
```

`PatchLayout::complex_euler()`, comparada com `PatchLayout::mesh_chi` dentro do
`to_layout`, que recusa com `LayoutError::GenusLost { complex, surface }`.

⭐ **Medido sobre 3 toros × 9 pesos + esfera: zero falsos positivos e zero falsos
negativos.** Toda célula que produzia uma malha de género errado passa a recusar;
toda célula que produzia uma malha correcta continua a produzi-la, bit a bit.

| | antes | depois |
|---|---|---|
| toro que perde o buraco | malha com `χ = 2`, 100 % quads, zero bordo | ⭐ **recusa com nome** |
| toro que fecha | malha correcta | malha correcta |

⚠️ **A frase da recusa é PRÓPRIA**, e não a genérica: *"tente outro Detail"* mandaria
o artista para o sítio errado — a decomposição não depende do alvo de densidade
nenhum, e o mesmo toro falha em **todos** os pesos enquanto o do lado passa em todos.

### O que fica ABERTO — e agora tem uma pergunta pequena

⛔ **O F3 continua sem saber CORTAR a asa.** A cura é acrescentar um laço de parede
(um gerador de `H₁` do patch, por *tree-cotree*) em vez de apagar uma — e é trabalho
novo, não uma linha. O gate `#[ignore]` `the_genus_survives_on_every_torus` exige uma
**malha**, não uma recusa: ele é o endereço do bloqueio.

⇒ **E é isso que trava o `ALIGN_WEIGHT`**, que funciona (`25,7° → 13,7°`).

---

## 4-quatervicies — ⭐⭐ **A CURA AGRAVAVA E ESCONDIA**, e dissolver não alcança uma asa

> O §4-tervicies pôs a cerca e deixou o passo nomeado: *"o F3 tem de saber cortar a
> asa"*. Antes de construir o corte, três medições — e elas mudaram o culpado duas
> vezes.

### 1. ⛔ O traçado está INOCENTE

Sonda `what_does_the_trace_say`, os três toros:

| fixtura | singularidades | separatrizes | `dangling` | patches | **dissolvidas / rondas** |
|---|---|---|---|---|---|
| toro 32×16 ✓ | 10 | 36 | 0 | 27 | **1 / 1** |
| ⛔ toro 48×24 | 10 | 30 | 0 | 9 | ⛔ **10 / 10** |
| toro 64×32 ✓ | 8 | 30 | 0 | 23 | **0 / 0** |

⭐ **A rede de paredes é tão densa no toro que falha como nos que passam**, e ele não
descarta separatriz nenhuma. O que ele tem a mais é a **limpeza**.

### 2. ⛔⛔ A limpeza agravava — e escondia ao mesmo tempo

Sonda `where_does_the_cleanup_break_it`, ronda a ronda. Na **ronda 0**, os dois toros
têm um patch com `χ = 0` — **um ANEL**, e ele *é* sinalizado (duas fronteiras).

| | toro 32×16 | toro 48×24 |
|---|---|---|
| ronda 0 | 28 patches, distância `1`, degenerados `[12]` | 20 patches, distância `1`, degenerados `[0]` |
| depois | ⭐ ronda 1 fecha em `0` ✓ | ⛔ rondas 1–9 **idênticas**; a ronda 10 vai para `2` e a lista fica **vazia** |

⇒ ⛔⛔ **A última ronda trocava «um anel, sinalizado» por «uma asa dentro de um
patch, não sinalizada»** — `χ = −1` com **uma** fronteira, que o `degenerate()` não
apanha. *Ela agravava e apagava o aviso no mesmo passo*, e é assim que a malha de
género errado escapava com 100 % de quads e zero arestas de bordo.

### 3. ⛔ E dissolver NÃO ALCANÇA esta asa — 6 lados e 15 pares, zero curas

O `dissolve` escolhe sempre o **lado mais curto**, ou seja, escolhe por sorte. Sondas
`would_any_wall_cure_the_handle` e `would_a_pair_of_walls_cure_it`, a partir do mesmo
estado da ronda 0:

| fixtura | lados que curam | pares que curam |
|---|---|---|
| toro 32×16 | ⭐ **3 de 6** | — |
| ⛔ toro 48×24 | **0 de 6** | ⛔ **0 de 15** |

⇒ No toro que passa a cura existe e o *"mais curto"* acertou **por acaso**; no que
falha **não existe cura por dissolução nenhuma**. *A operação está errada para esta
classe: uma asa cura-se acrescentando parede, não apagando.*

### ⭐ A guarda que ficou, e a que foi medida e rejeitada

⭐⭐ **FICA:** *uma ronda que aumenta a distância topológica é recusada, sempre.* Ela
corre sobre uma cópia das paredes e só é adoptada se passar — assim a recusa não tem
nada para desfazer. Efeito medido:

| fixtura | antes | depois |
|---|---|---|
| toro 48×24 | 10 rondas, 9 patches, distância **2**, aviso **apagado** | 9 rondas, 10 patches, distância **1**, `não-disco = 1` ⭐ |
| esfera 24×36 · 48×72 · toro 32×16 · 64×32 | — | **byte-idêntico** |

⛔ **REJEITADO: um teto de «rondas paradas».** A ideia era cortar as nove rondas
inúteis. Ela reprova porque a paciência é do **fenómeno**:

| fixtura | rondas com `(distância, degenerados)` idêntico | e depois |
|---|---|---|
| ⭐ esfera 48×72 | `(1, 1)` da ronda 0 à 4 — **cinco** | a ronda 5 fecha em `(0, 0)` ✓ |
| ⛔ toro 48×24 | `(1, 1)` da ronda 0 à 9 — **dez** | a ronda 10 vai para `(2, 0)` ⛔ |

⇒ **Indistinguíveis enquanto correm.** Um teto de `2` matava a esfera; um de `9`
deixava o toro passar na mesma. *Uma paciência que decide certo num caso e errado no
outro não é uma constante — é um palpite.* (Construído, medido, revertido: a esfera
48×72 reprovou o gate `every_face_is_a_quad_and_the_mesh_is_watertight`.)

### ⛔ O que fica ABERTO — e a precondição do corte está medida

O corte de uma asa é **acrescentar** uma parede: para um **anel**, uma ponte de
caminho mais curto entre as duas fronteiras, ligando **canto a canto** (a aritmética
só fecha assim: `V += 0`, `E += 1`, `F += 0` ⇒ a distância cai `1`).

⚠️⚠️ **E ele tem uma precondição medida:** o `boundary_loops` exige que a face do
outro lado da parede **não** seja do mesmo patch —

```rust
walls.blocks(a, b) && half.get(&(b, a)).is_none_or(|&g| face_patch[g] != p)
```

⇒ **uma ponte interior a um patch é invisível para ele.** Cortar exige primeiro que o
passeio da fronteira saiba percorrer uma parede **dos dois lados** — que é
exactamente a representação «cortar e abrir», e mexe numa rotina com dez gates em
cima. *É esse o trabalho, e agora ele tem tamanho.*

---

## 4-quinvicies — ⭐ **A PISTA DO CORTE, medida — e três tentativas rejeitadas**

> O §4-quatervicies deixou a precondição: *"o `boundary_loops` exige que a face do
> outro lado da parede não seja do mesmo patch, logo uma ponte interior é
> invisível"*. Este passo mede quanto custaria mudá-la. ⛔ **Não fecha, e é honesto
> dizê-lo: o que fica é o instrumento e a pista, não o conserto.**

### ⭐ O instrumento: `TraceReport::interior_walls`

Quantas arestas de parede têm o **mesmo patch dos dois lados** — invisíveis para o
passeio da fronteira. Medido dentro do `decompose`, que é o único sítio onde as
paredes e a decomposição que elas produziram existem ao mesmo tempo:

| fixtura | paredes | **interiores (cru)** | **interiores (limpo)** | rondas |
|---|---|---|---|---|
| esfera 24×36 | 362 | 0 | 0 | 0 |
| esfera 48×72 | 442 | 2 | ⛔ **172** | 5 |
| toro 32×16 | 532 | 8 | ⛔ **30** | 1 |
| toro 48×24 | 511 | 18 | ⛔ **184** | 9 |
| toro 64×32 | 466 | 0 | 0 | 0 |
| cubo subdividido | 646 | 0 | 0 | 0 |

⭐⭐ **A limpeza deixa 172 a 184 arestas de parede mortas.** O `dissolve` apaga as
arestas de **um lado** do patch degenerado; o resto da separatriz fica lá dentro,
bloqueando o `flood` e não produzindo fronteira nenhuma. *São cortes que já existem e
que ninguém percorre.*

⛔ **E a primeira versão desta sonda contou errado** — media as paredes **cruas**
contra os patches **limpos**, que não se correspondem porque a limpeza dissolve
paredes. Dava `204` onde o número é `18`. *Uma contagem sobre dois estados que não se
correspondem não é uma medição.*

### ⭐ A pista: honrar as paredes interiores **corrige o toro mau**

Experiência: tirar a segunda condição do `outside`, isto é, tratar toda parede como
fronteira e percorrê-la **dos dois lados** (a representação *"cortar e abrir"*).

| fixtura | antes | com a mudança |
|---|---|---|
| ⛔ toro 48×24 | `χ = 1` (recusa) | ⭐ **`χ = 0`** ✓, 20 patches, **zero** rondas de limpeza |
| ⭐ toro 32×16 | `χ = 0` ✓ | ⛔ **`χ = −1`** |
| toro 64×32 · esfera 24×36 | ✓ | ✓ inalterados |

⇒ **A direcção está certa e a regra é larga demais.** O mecanismo da quebra tem nome:
o passeio ganha **arco** sem ganhar **canto**, e a conta desce `1` por cada um.

### ⛔ As três tentativas, medidas e rejeitadas

| tentativa | resultado |
|---|---|
| honrar em **todo** patch | toro 48×24 fecha ✓, toro 32×16 parte (`0 → −1`) ⛔ |
| honrar só no patch **doente** (`χ ≠ 1`), com o `χ` calculado antes das fronteiras | ⛔ **idêntico** — o patch que parte é o próprio doente |
| ⭐ **a ponta da fenda é canto** (`ramificação == 1`, sem perguntar o ângulo) | ⛔ **não move a agulha** — as paredes interiores daquele patch são **laços fechados**, sem ponta livre |

⇒ ⚠️ **E a terceira revelou o resto do mecanismo:** um laço de parede fechado sem
canto nenhum é **descartado inteiro** pelo `decompose` (`if cuts.is_empty() {
continue; }`). Cortar com um laço exige **promover um canto** nele — e é aí que a
conta `V += 1, E += 1` fecha em `0` em vez de descer.

⇒ ⭐⭐ **O próximo passo tem nome e é pequeno:** honrar as paredes interiores **e**
promover um canto em cada laço de parede fechado que passe a ser fronteira. As três
tentativas acima dizem exactamente por que as duas coisas têm de vir juntas.

---

## 4-sexvicies — ⭐⭐⭐ **A PONTE JÁ ESTAVA TRAÇADA** — o toro fechou o buraco

> O §4-quinvicies deixou a pista (184 paredes mortas dentro dos patches) e três
> tentativas rejeitadas. Este passo mede a **estrutura** dessas paredes, e a resposta
> torna o conserto pequeno.

### ⭐ O que elas são, medido

Sonda `are_the_interior_walls_slits_or_loops` — componentes ligadas das paredes
interiores, e a ramificação dos vértices delas:

| fixtura | arestas interiores | componentes | pontas livres | **em que patch** |
|---|---|---|---|---|
| toro 32×16 | 8 | 1 | **0** | `χ = 0` · 2 fronteiras · 6 lados |
| toro 48×24 | 18 | 1 | **0** | `χ = 0` · 2 fronteiras · 6 lados |
| esfera 48×72 | 2 | 1 | **0** | `χ = 0` · 2 fronteiras · 6 lados |

⭐⭐ **Nas três, sem excepção: um único caminho entre duas junções, dentro do patch
ANEL.** Ou seja — *a ponte que abre o anel em disco já está traçada.* O passeio da
fronteira é que se recusava a percorrê-la, porque exigia que a face do outro lado
fosse de **outro** patch.

⇒ Não era preciso construir um algoritmo de corte. Era preciso **olhar**.

### A cura, e o critério que a governa

`decompose_with(..., cut_open)` deixa cair a segunda condição do `outside` — a
parede passa a ser percorrida **dos dois lados** (a representação *"cortar e
abrir"*), e só em patches que não são disco. A adopção é decidida pela **mesma
guarda** do §4-quatervicies.

⛔ **E o critério errado foi construído e medido primeiro:** comparar a saúde inteira
`(distância, degenerados)` deixava entrar um movimento **lateral** — no toro 32×16 a
ponte empatava na distância (`1`) e ganhava nos degenerados (`0` contra `1`), era
adoptada, e o laço parava num complexo de **`−1`**, pior que o `0` a que a dissolução
chegava. *Um critério que aceita empates deixa a cura barata expulsar a cura certa.*
⇒ A melhoria tem de ser **estrita e da distância**.

### ⭐⭐⭐ O resultado

| fixtura | antes | depois |
|---|---|---|
| ⛔ **toro 48×24** | recusava (`χ` do complexo `1`) | ⭐ **3 096 quads · χ = 0 · 0 bordo · 0 não-manifold · 23 irregulares**, e **zero** rondas de dissolução |
| toro 32×16 | 27 patches, 1 ronda | **idêntico** (a ponte empata e é recusada) |
| toro 64×32 · esfera 24×36 · 48×72 | ✓ | **idênticos** |

⭐ **`the_genus_survives_on_every_torus` deixou de ser `#[ignore]`** — o vermelho
pré-existente que abriu o §4-tervicies está **fechado**.

### ⚠️ E o `ALIGN_WEIGHT` continua a zero — mas por outra razão

Re-medida a varredura com a topologia curada (3 toros × 9 pesos + esfera): ⭐ **não
há uma única malha de género errado em nenhuma célula** — cada uma é ✓ ou uma recusa
honesta. ⛔ Mas o toro 32×16 passa a **recusar** em `0,005 · 0,01 · 0,02 · 0,03 · 0,1
· 0,2`, onde a peso `0` ele entrega malha.

⇒ **O bloqueio mudou de espécie**: era *"o alinhamento produz malha errada"*, é agora
*"o alinhamento faz algumas peças recusarem"*. ⚠️ E há um segundo número a explicar:
a esfera 24×36 a `0,03` entrega **357 quads** contra 1 997 a peso `0` — um colapso de
densidade que nada nesta secção explica.

---

## 4-septvicies — ⭐⭐⭐ **A GRADE VINHA 3× MAIS GROSSA DO QUE SE PEDIA**, e a régua faltava

> O §4-sexvicies fechou com um número por explicar: a esfera 24×36 a `0,03` entregava
> **357 quads** contra 1 997 a peso `0`, com o **mesmo** alvo de aresta.

### ⭐ Onde a densidade se perdia

Sonda `where_does_the_density_go` — segue a densidade pelas quatro fases:

| fixtura · peso | `Σtau/alvo` (o F3 pede) | `Σquant` (o F4 concede) | razão | aresta mediana | quads |
|---|---|---|---|---|---|
| esfera · `0` | 348 | 342 | 0,98× | 1,03× | 1 997 |
| ⛔ esfera · `0,03` | 438 | **169** | ⛔ **0,39×** | ⛔ **2,85×** | ⛔ 357 |
| ⛔ **toro · `0`** | 725 | **503** | ⛔ **0,69×** | 1,11× | 3 096 |
| ⛔ toro · `0,02` | 697 | **276** | ⛔ **0,40×** | 1,32× | 918 |

⭐⭐ **O F3 pede a densidade certa e o F4 devolve 39 a 98 % dela — a peso ZERO
também.** Não era do alinhamento; ⛔ **e não era o piso** (`min = 1` toca em 2 a 4
arcos). A mediana de `2,85×` prova que a perda chega à malha: *uma grade quase três
vezes mais grossa do que o artista pediu.*

### ⭐⭐ A régua que faltava, e ela mudou a escolha da lei

A varredura que escolheu o custo do arco (2026-08-21) mediu **dobras** e **pior
arco**. ⛔ **Nenhuma das duas vê uma grade uniformemente grossa** — e a coluna que a
vê, `Σquant / Σalvo`, não existia. Acrescentada, sobre as mesmas três fixturas
(`densidade / dobras / pior arco`):

| lei | esfera 48×72 | esfera 96×144 | esculpida | pior relógio |
|---|---|---|---|---|
| `abs · 1` | 0,95 / 1 / 1,7 | 0,99 / 5 / 6,1 | 1,06 / 0 / 1,3 | 415 ms |
| `abs · 1/t` | 0,85 / 0 / 3,2 | 0,90 / 6 / 8,1 | 0,94 / 0 / 2,6 | 19 ms |
| `abs · t` | 1,14 / ⛔ 28 / 4,2 | 1,07 / ⛔ 21 / 2,4 | 1,09 / 0 / 2,0 | ⛔ 3 864 ms |
| `abs · √t` | 1,11 / ⛔ 18 / 4,2 | 1,03 / ⛔ 24 / 3,5 | 1,07 / 2 / 1,3 | 75 ms |
| `quad · 1` (o default da referência) | 1,03 / 5 / 2,8 | 1,00 / 8 / 2,4 | 1,02 / 0 / 1,7 | ⛔ 1 551 ms |
| `quad · 1/t` | 0,90 / 0 / 1,8 | 0,95 / 3 / 4,9 | 0,97 / 0 / 2,6 | 18 ms |
| ⛔ `quad · 1/t²` (a que shipava) | ⛔ **0,84** / 0 / 2,9 | 0,93 / 3 / ⛔ 6,1 | 0,94 / 0 / 2,6 | 393 ms |
| ⭐⭐ **`scale`** | ⭐ **0,99** / 1 / ⭐ **1,5** | ⭐ **0,99** / 7 / 3,0 | ⭐ **0,98** / 0 / 1,7 | ⭐ **14 ms** |

⭐ **Ganha a densidade nas três, o pior arco em duas, e é a mais rápida das oito.**

### ⛔ E a recusa que a barrava DISSOLVEU — o doc anterior tinha escrito a condição

A `scale` foi rejeitada em 2026-08-21 por reprovar o gate da 48×72 com **36,3 %** de
faces dobradas contra a barra de 33 %. ⚠️ **Hoje ela dá `0,0 %`** (uma dobra em
4 066) na mesma fixtura — e as outras sete caíram junto, a maior em `0,6 %`.

⇒ **O que mudou não foi a lei: foi a montagem.** A parametrização por patch
(§4-duodevicies) curou o grão que punha todas as leis naquele regime. E o doc da lei
antiga já dizia: *"reabrir esta escolha depois de o grão estar curado é trabalho
pendente, não uma recusa."* ⭐ *Uma recusa medida que nomeia a sua própria condição de
reabertura é a única que não envelhece em silêncio.*

### ⭐⭐⭐ O efeito no produto

| fixtura (peso `0`, mesmo alvo) | antes | ⭐ depois |
|---|---|---|
| **toro 48×24** | 3 096 quads (densidade `0,69×`) | ⭐ **6 221** (`1,00×`) |
| toro 32×16 | 3 600 | **4 011** |
| esfera 24×36 | 1 997 | **2 080** |
| esfera 24×36 · `0,03` | ⛔ 357 (mediana `2,85×`) | **1 508** (mediana `1,23×`) |

⚠️ **Não é cura completa:** sobram células a `0,73–0,83×`, e a **aresta máxima**
continua em `12–24×` o alvo em várias — um defeito geométrico separado, que nenhuma
coluna desta secção explica e que o `edge_max_span` do relatório já sabe medir.

---

## 4-duodetricies — ⛔⛔ **AS TRÊS FOTOS: uma aresta de 56 % DA PEÇA, e nenhuma régua piscou**

> *"muito ruim"* — o artista, 2026-08-22, com três fotos e duas setas verdes.

### ⛔ O que a saída marcava quando ele tirou a foto

100 % de quads · casca **fechada** · característica de Euler **exacta** · densidade
no alvo (mediana `1,02×`) · **15** irregulares · valência máxima **6** · e a forma
preservada a `0,085 %` da diagonal. **Todas verdes.**

⇒ *Uma peça pode passar em toda asserção e estar visivelmente destruída* — e desta
vez a régua **existia**: `QuadRemeshReport::edge_max_span`, barra `≤ 0,20`. Ela só
corria sobre a `wrinkled_sphere`, que mede `6 %`.

### ⭐ O número, e ele é exclusivo da orelha

| fixtura | aresta máxima em fração da peça (d = 0,25 · 0,5 · 1,0) | dobras (d = 1,0) |
|---|---|---|
| ⛔ **orelha** | ⛔ **56,5 % · 56,3 % · 57,0 %** | ⛔ **2 204** (4,85 %) |
| gancho | 9,2 % · 8,0 % · 11,6 % | 19 |
| enrugada | 10,0 % · 6,4 % · 6,1 % | 70 |

⭐⭐ **O que se sabe da aresta:** ela liga dois pontos **ANTIPODAIS**, ambos a raio
`1,00` — na parte **lisa** da esfera, não no vinco da orelha — e mede `1,98`, o
**diâmetro**. ⚠️ **Ela não encolhe** quando o alvo encolhe 10×: *não é falta de
resolução, é um ponto no sítio errado.*

⭐ **E o controlo ilibou as fases de montante:** a maior aresta da **fixtura** é
`1,8 %`; a da saída do **F1** é `3,3 %`. A cadeia cria-a.

⚠️ **O que ela NÃO é**, medido: não é valência (o máximo é `6`, e não há um único
vértice `≥ 7`) · não é o leque (`patch max 6` lados) · não é perda de forma
(`detail_lost` p95 `0,085 %`).

### ⛔ Uma hipótese construída, MEDIDA e REJEITADA

O achatamento falhava **3 329 amostras** na orelha (`8,4 %`) e **zero** nas outras
duas fixturas — um sinal limpo. O recurso de uma falha é um ponto de Coons no
**espaço 3D**, que num patch grande passa perto do centro da peça e é depois
projectado para o lado que estiver mais perto. *Parecia a causa exacta da aresta
antipodal.*

Curei-as com um recuo do `uv` para dentro do domínio (o polígono é estrelado a
partir de `(0,0)`, então encolher o raio chega sempre lá dentro):

| | antes | com o recuo |
|---|---|---|
| falhas de amostragem | 3 329 | ⭐ **0** |
| ⛔ aresta máxima | 57,0 % | ⛔ **57,0 %** |
| ⛔ dobras | 2 204 | ⛔ **2 768** (+25 %) |

⇒ **As falhas acompanham o defeito naquela peça; não o causam.** E curá-las sozinhas
**piora** a saída — o ponto puxado para o centro do patch é pior vizinho que o de
espaço. Revertido, com o mecanismo no doc do `Domain::place`.

### ⇒ O que fica

⭐ O gate `the_ear_does_not_ship_an_edge_across_the_piece` (`#[ignore]`, **vermelho**)
é o endereço do defeito, com a barra que o relatório já definia. ⛔ **A causa
continua por achar** — e as três coisas que ela **não** é já estão eliminadas.

---

## 4-undetricies — ⭐⭐⭐ **A CAUSA DA FOTO: um patch que dá TRÊS VOLTAS à peça**

> O §4-duodetricies deixou a aresta de 56 % com três explicações eliminadas e a causa
> por achar. Ela estava num único patch, e o número que o denuncia tem margem.

### ⭐ O patch

`patch 1` da esfera com orelha, **seis lados**, comprimentos:

```text
    6,78 · 0,27 · 2,15 · 6,80 · 0,09 · 2,15      perímetro 18,2
```

⛔ **Perímetro = 520 % da diagonal da peça.** Os dois lados de `6,8` são quase a
circunferência inteira (`6,28`); os de `0,09` e `0,27` são lascas.

⭐⭐ **E o número separa-o de tudo o resto:** em três fixturas e dezenas de patches, o
**segundo** maior perímetro é `230 %`. *Um contra 2,3× de folga não é a cauda de uma
distribuição — é outra coisa.*

| | perímetro / diagonal |
|---|---|
| ⛔ **orelha, `patch 1`** | ⛔ **520 %** |
| o segundo maior (todas as fixturas) | 230 % |
| a mediana | ~110 % |

### ⭐⭐ O mecanismo, do perímetro à aresta

As lascas recebem **2** segmentos e os lados longos **40**. A lei do leque
(`L_i = e_{i−1} + e_{i+1}`) resolve isso com raios **`[1, 39, 1, 1, 39, 1]`** —
quatro dos seis valem `1`.

⇒ Um raio de `1` faz o sector ter **uma célula de fundo**, e o quad dessa célula liga
um ponto lá longe de um lado a um ponto lá longe do seguinte. Num patch que dá três
voltas à peça, **esse quad atravessa-a** — e as duas pontas saem antipodais, ambas a
raio `1,00`. *Exactamente o que a foto mostra, e exactamente o que a face medida
diz:* `[135, 136, 283, 282]`, dois pares vizinhos em lados opostos da esfera.

⚠️ **E é por isso que ela não encolhe com o slider:** a `d = 1,0` os lados longos
passam a `384` segmentos e os raios ficam `[6, 384, 10, 1, 379, 4]` — **o sector
continua a ter `1` de fundo**.

### ⇒ Onde o conserto mora

⛔ **O F3 não devia emitir este patch**, e ⛔ **`dissolve` não serve** — ele *junta*
patches, e este já é grande demais. As duas direcções abertas:

| direcção | o que exige |
|---|---|
| **cortar** o patch gigante | o mesmo trabalho da asa: acrescentar parede, não apagar |
| **piso do raio** no F5/F4 | um `min` de arco que olhe o comprimento **geométrico** do lado, em vez do `1` fixo |

⚠️ **A segunda mexe nas restrições do F4**, cujo ótimo é demonstrado — e mudar o
`min` muda o problema que ele prova. *Não é uma linha.*

---

## 4-tricies — ⭐⭐⭐ **A CADEIA CAUSAL FECHA: um ANEL CORTADO preenchido como LEQUE**

> O §4-undetricies parou no patch de perímetro `520 %`. Ele é o anel que a ponte
> abriu — e a ponte não é opcional.

### ⭐ O patch, pelos arcos

```text
    lado:   [16,17,4,5]   [6]   [7]   [8..14]   [15]   [7]
    comp:      6,78      0,27  2,15    6,80     0,09   2,15
    canto:      74°       77°   14°     118°     70°    26°
```

⭐⭐ **O arco `7` aparece DUAS vezes** — os dois lados de `2,15` são ele, ida e volta.
Os dois *hairpins* (`14°` e `26°`) caem exactamente nas pontas dele. ⇒ **Este patch é
o anel que a ponte do §4-sexvicies abriu**, e os lados de `6,78`/`6,80` são as duas
fronteiras dele, quase a circunferência da esfera cada uma.

### ⭐ E a ponte NÃO é opcional — o controlo

| | orelha |
|---|---|
| com a ponte | malha, com a aresta de **56 %** |
| ⛔ **sem a ponte** | ⛔ **RECUSA nos três níveis** (`GenusLost`) |

⇒ *A ponte é o que a faz existir; o defeito é como o patch dela é preenchido.*

### ⇒ Onde o conserto mora, por ELIMINAÇÃO

| candidato | porque **não** |
|---|---|
| dar mais segmentos à lasca (F4) | ⛔ a densidade dela **está certa** — `0,27` com `2` segmentos é `0,135` cada, contra um alvo de `0,167` |
| `dissolve` o patch (F3) | ⛔ ele **junta** patches, e este já dá três voltas à peça |
| desligar a ponte (F3) | ⛔ a orelha deixa de fechar |
| ⭐ **o preenchimento (F5)** | ⭐ **é o que sobra, e é onde a premissa quebra** |

⭐⭐ **A lei do leque (`L_i = e_{i−1} + e_{i+1}`) é a da referência e está certa para
um `n`-gono de verdade.** ⛔ **Um anel cortado não é um hexágono:** duas das seis
arestas dele são a *mesma* curva. Alimentada com `L = [40, 2, 40, 40, 2, 40]`, a lei
devolve `e = [1, 39, 1, 1, 39, 1]` — e é **forçada**, não escolhida: para `n` par o
sistema tem solução única. Quatro raios a `1` são quatro sectores com **uma célula de
fundo**, e o quad dessa célula atravessa a peça.

⇒ ⭐⭐⭐ **O preenchimento certo é uma FAIXA** — uma grade entre as duas fronteiras,
com a ponte como costura. *É o que o `fill_rectangle` já faz para `n = 4`; falta
reconhecer este caso e mandá-lo para lá.* ⚠️ **E o reconhecimento é barato e
inequívoco:** o patch tem um **arco repetido na própria lista de lados**.

---

## 4-untricies — ⛔ **A FAIXA foi construída, MEDIDA e REJEITADA** — e a peça continua a ser o problema

> O §4-tricies concluiu, por eliminação, que o conserto morava no **preenchimento**:
> um anel cortado não é um hexágono, e a lei do leque não o descreve. Construí-o.

### O que se construiu

`regroup_cut_annulus`: reconhecer o **arco repetido** na lista de lados do patch —
inequívoco, e não uma heurística — e reagrupar os seis lados em quatro:

```text
    antes:  [16,17,4,5]   [6]   [7]   [8..14]   [15]   [7]
              40           2     40      40       2     40
    depois: [7]   [8..14, 15]   [7]   [16,17,4,5, 6]
             40        42        40          42
```

⭐ **Lados opostos com contagens iguais** — um retângulo, que o F5 já preenche sem
leque nenhum, e cuja lei o F4 passa a impor.

### ⭐ Estruturalmente funcionou

| | antes | com a faixa |
|---|---|---|
| maior valência de patch | 6 | ⭐ **4** |
| valência máxima de vértice | 6 | ⭐ **5** |
| quads (d = 0,25 · 1,0) | 603 · 45 407 | ⭐ **1 054 · 88 090** |

### ⛔ E a agulha que importa não se mexeu — e a outra piorou

| | antes | com a faixa |
|---|---|---|
| ⛔ **aresta máxima** | 56,5 % · 57,0 % | ⛔ **55,7 % · 58,6 %** |
| ⛔ **dobras** (d = 1,0) | 2 204 (**4,85 %**) | ⛔ **7 795** (**8,85 %**) |

⇒ **Revertido.** *Uma cura que acerta a forma do patch e piora a malha não é uma
cura* — e é a segunda deste passo (a primeira foi o recuo do `uv`, §4-duodetricies).

### ⇒ O que as duas rejeições ensinam, juntas

⭐⭐ **O problema não é COMO o patch é preenchido: é o patch.** Ele dá três voltas à
peça (perímetro `520 %` da diagonal) — e nem o leque nem a faixa fazem uma grade
decente sobre uma banda que envolve uma esfera inteira, porque o achatamento tem de a
esmagar num quadrado.

⛔ **E as saídas fáceis já estão todas medidas e fechadas:**

| saída | porque não |
|---|---|
| mais segmentos na lasca (F4) | a densidade dela **está certa** |
| `dissolve` o patch (F3) | ele **junta**, e este já é o maior |
| desligar a ponte (F3) | a orelha deixa de fechar (`GenusLost`) |
| recuo do `uv` no achatamento (F5) | falhas `3 329 → 0`, aresta **inalterada**, dobras **+25 %** |
| faixa em vez de leque (F3/F5) | forma do patch corrigida, aresta **inalterada**, dobras **+82 %** |

⇒ ⭐⭐⭐ **O que resta é o traçado CORTAR aquele anel em vez de o abrir** — dois ou
três patches em vez de uma banda que envolve a peça. É o mesmo trabalho que a asa
pedia (§4-quatervicies) e que a ponte só **adiou**: ela tornou o layout *válido* sem
o tornar *bom*.

---

## 4-duotricies — ⭐⭐⭐ **O GABARITO ESTAVA EM DISCO** — o oráculo grava as fases intermédias

> **Pergunta do Enio (2026-08-22):** *"só me interessa o estado da arte. Os códigos
> não abertos não podem ser estudados e adaptados?"*

### A posição legal, em três linhas

| | |
|---|---|
| **estudar** o código GPL | ✅ permitido — o algoritmo é ideia, e ideia não tem direito autoral |
| ⛔ **adaptar / traduzir** | ⛔ obra derivada. Um porte C++→Rust de fonte GPL **é** GPL |
| ⚠️ o risco prático | quem **lê** a fonte contamina tudo o que escreve depois; é por isso que existe sala limpa, e é por isso que o ADR-0162 partiu dos **papers** |

### ⭐⭐⭐ E o que torna a pergunta quase irrelevante

**O binário do oráculo grava as fases intermédias dele.** Em
`ph2d-quadbench/ref/<peça>/`, por peça do corpus:

| ficheiro | o que é | a nossa fase |
|---|---|---|
| `*_rem.obj` | a malha remalhada dele | **F1** |
| ⭐⭐ `*_rem.rosy` | **o campo cruzado dele** — `9 464` direções para `9 464` faces | **F2** |
| ⭐⭐ `*_rem_p0.patch` | **a decomposição dele** — o patch dono de cada face | **F3** |
| `*_rem_p0.corners` | os cantos | F3 |
| `*_rem.sharp` · `*.feature` | as quinas duras | (não temos) |
| `*_quadrangulation.obj` | a malha final | F5 |

⇒ ⭐⭐⭐ **As duas fases cujo código de referência é GPL — o campo (CoMISo) e o
traçado (xfield_tracer) — têm o resultado delas em disco, na mesma malha, ficheiro a
ficheiro.**

⚠️ **E ler a SAÍDA de um programa não é obra derivada.** É legal, é o padrão, e é
**mais forte** que ler o código: em vez de interpretar intenção, compara-se número
com número.

### ⛔ O que se estava a fazer errado

A bancada compara o **resultado final** (`65–83 %` de quads contra `100 %`). ⛔ **O
campo e a decomposição nunca foram comparados com o oráculo** — e são exactamente as
duas fases em que esta linha está encalhada há dias. *Redescobrir às cegas com a
resposta no disco.*

### ⇒ O que isto destrava, em perguntas que hoje são chute

| pergunta | hoje | com o gabarito |
|---|---|---|
| o campo dele obedece ao relevo (`13,7°`) e o meu não (`25,7°`) | *não sei onde* | **em que faces**, uma a uma |
| o patch que dá três voltas à peça | *não sei se ele o tem* | ele corta ali, ou não |
| singularidades na orelha | *não sei* | quantas e **onde** |

⚠️ **A comparação corre sobre a malha DELE** (`*_rem.obj`), não sobre a nossa: assim
a única diferença entre as duas colunas é o solver, e não o F1.

### ⚠️ E uma terceira via que custa um e-mail

O autor do solver de contagem **já libertou a parte dele em MIT**. Autores académicos
relicenciam a pedido com frequência. Pedir licença permissiva para o `CoMISo` e o
`xfield_tracer` ao grupo do QuadWild é barato e resolveria de vez. ⚠️ **É decisão do
Enio, não da linha.**

---

## 4-tritricies — ⭐⭐⭐ **O CAMPO É O CULPADO, E AGORA HÁ UM ALVO: `12°`**

> Primeira colheita do gabarito (§4-duotricies). Sonda `my_field_against_the_oracle`,
> **sobre a malha remalhada DELE** — a única diferença entre as colunas é o solver.

### ⛔ O nosso campo é praticamente aleatório onde a peça tem relevo

| peça | ⛔ o NOSSO | ⭐ o do ORÁCULO | aleatório | discordância (faces `≥ 30°`) |
|---|---|---|---|---|
| **com cristas** | ⛔ **24,3°** | ⭐ **12,1°** | 22,5° | 43,2 % |
| **com bico** | ⛔ **22,9°** | ⭐ **11,4°** | 22,5° | 38,2 % |
| enrugada | 14,8° | 12,4° | 22,5° | 34,6 % |
| esfera lisa *(conf 0,04 — sem sinal)* | 23,0° | 21,6° | — | 29,6 % |

⭐⭐ **Duas das peças ficam do lado errado do aleatório.** E isto é medido **à saída
da própria fase**, sem traçado, sem quantização, sem montagem — *o diagnóstico deixa
de depender de quatro fases a jusante.*

⚠️ **A régua é a mesma da `follows_relief`, aplicada ao CAMPO** em vez da malha.
⛔ E a primeira versão dela estava errada: dobrava com `45 − |45 − x|`, que vai a
**negativo** acima de `90°` (o `acos` chega a `180°`). O sintoma foi um desvio médio
de **`−33,9°`**. *Uma régua que sai do próprio contradomínio está errada antes de
dizer o que quer que seja.*

### ⭐⭐⭐ E com o alvo à mão, o peso do alinhamento escolhe-se sozinho

Sonda `which_alignment_weight_matches_the_oracle_field` — o **relevo do nosso campo**
contra o do oráculo, na malha dele:

| peso | cristas (alvo 12,1°) | bico (alvo 11,4°) | enrugada (alvo 12,4°) | singularidades (cristas · bico · enrugada) |
|---|---|---|---|---|
| **`0`** (o que shipa) | ⛔ 24,3° | ⛔ 22,9° | 14,8° | 10 · 26 · 8 |
| `0,01` | 15,6° | 15,0° | 12,0° | 12 · 22 · 8 |
| `0,03` | 14,7° | 13,9° | 11,7° | 12 · 22 · 8 |
| ⭐⭐ **`0,1`** | ⭐ **13,2°** | ⭐ **12,5°** | ⭐ **11,4°** | **12 · 29 · 8** |
| `0,3` | 11,3° | 11,2° | 11,0° | 22 · 33 · 10 |
| `1,0` | 9,7° | 10,5° | 9,9° | ⛔ 48 · 50 · 8 |
| `3,0` | 8,1° | 7,9° | 8,1° | ⛔ 74 · 126 · 24 |

⭐⭐ **`0,1` alcança o oráculo nas três, e o preço em singularidades é quase nulo**
(`10 → 12`, `26 → 29`, `8 → 8`), com a soma dos índices sempre em `8`.

⚠️ **E isto contradiz a varredura anterior**, que rejeitou pesos muito menores. A
diferença é onde se mede: aquela media o **relevo da malha montada**, onde o campo, o
traçado, a quantização e a montagem estão todos misturados — e por isso dava tabelas
não-monótonas. *O peso de uma energia escolhe-se à saída da fase em que ela vive.*

⇒ **O passo seguinte é correr a cadeia INTEIRA a `0,1`** nas nossas fixturas, com o
que hoje já está curado (a semente do `θ`, a ponte do anel, a lei `scale`) — e ver se
os gates do género e da aresta máxima aguentam o que o campo agora sabe fazer.

---

## 4-quattuortricies — ⭐⭐⭐ **O CAMPO ALINHADO SHIPA, e a aresta da FOTO desaparece**

> `ALIGN_WEIGHT` deixou de ser zero. O número veio do gabarito do oráculo
> (§4-tritricies), não de uma varredura no fim da cadeia.

### ⭐⭐⭐ O que o artista vê

| | peso `0` | ⭐ **`0,03`** |
|---|---|---|
| ⛔ **orelha — aresta máxima** | ⛔ **56,5 % · 56,3 % · 57,0 %** da peça | ⭐ **12,4 % · 7,8 % · 5,5 %** |
| ⛔ **orelha — dobras** | ⛔ 42 · 134 · **2 204** | ⭐ **0 · 0 · 171** |
| gancho — aresta / dobras (d=1) | 11,6 % / 19 | ⭐ **5,3 % / 18** |
| enrugada — aresta / dobras (d=1) | 6,1 % / 70 | ⭐ **3,6 % / 11** |
| com cristas — relevo | 24,2° | ⭐ **13,7°** |
| enrugada — relevo | 23,1° | ⭐ **13,8°** |
| com bico — relevo | 23,9° | ⭐ **19,0°** |

⭐⭐ **É a aresta que o artista fotografou três vezes, e ela desaparece.** O gate
`the_ear_does_not_ship_an_edge_across_the_piece` deixou de ser `#[ignore]`.

⚠️ **`0,1` é melhor no campo e recusa na cadeia** (a quantização do gancho não fecha);
`0,03` é o maior peso que fecha nas cinco fixturas de escultura.

### ⚠️ O preço, dito por extenso

⛔ **O toro 32×16 passa a RECUSAR** — e pior: com o campo alinhado o **traçado**
produz ali uma fronteira malformada (*"o lado 5 acaba em 81 e o lado 6 começa em
200"*, um patch de nove lados).

⭐ **O produto está protegido por duas coisas construídas no mesmo dia:** a cerca
`LayoutError::GenusLost` recusa o layout, e a porta cai para o campo só-suavidade
(`quad_remesh_global`), com o relatório a dizer qual correu. *A rede foi construída de
manhã e é agora que ela ganha o lugar dela.*

⛔ **E a fragilidade tem gate próprio e vermelho:**
`the_tracer_survives_the_aligned_field` (`#[ignore]`, em `ph2d-trace/tests/trace.rs`).

### ⭐ Um gate mais severo que o produto não mede o produto

Ao ligar o peso, **seis gates reprovaram** — e nenhum por o produto estar mau:

| gate | porque reprovou | o conserto |
|---|---|---|
| 3 × `ph2d-quadfill::quadfill` | o helper `chain()` **não tinha a rede** que a porta tem | passou a fazer as duas tentativas, como o produto |
| 3 × `ph2d-trace` | afirmam coisas sobre o **traçado** e corriam sobre o campo do **produto** | passaram a correr sobre o campo só-suavidade |

⚠️ **A segunda linha é a lição:** um gate do F3 que muda de cor quando uma constante
do F2 se move está a testar **duas** fases, e no dia em que reprova não diz qual
quebrou. *Um gate, uma afirmação* — e a afirmação sobre a fragilidade passou a ter
gate próprio em vez de contaminar seis.

---

## 4-quinquiestricies — ⛔⛔ **«PÉSSIMO» outra vez, e a régua verde estava a olhar para o lado errado**

> **2026-08-22, a quarta foto.** No mesmo dia em que a `edge_max` da orelha caiu de
> **57 % da peça para 5,5 %** (§4-quattuortricies), o artista mandou outra foto da
> mesma peça com a palavra **«péssimo»**.

### ⛔ O que estava errado com as réguas — e é a lição desta secção

**Todas** as grandezas geométricas desta linha mediam **um extremo global**:
`edge_max` é a aresta mais longa da malha inteira, `edge_median` a mediana de todas.
⚠️ *Um quad de `0,02 × 0,30` não move nenhuma das duas* — a longa dele está muito
abaixo da máxima e a curta afunda-se na mediana de dezenas de milhares.

⇒ **O defeito da foto é POR-FACE**, e nenhuma asserção do repo olhava um quad de
cada vez. A régua nova ([`ph2d_quadfill::QuadShape`](../../../crates/ph2d-quadfill/src/shape.rs)) tem três colunas, e as três são
precisas **juntas**:

| grandeza | o que apanha | a que é cega |
|---|---|---|
| **aspecto** (longa ÷ curta do mesmo quad) | o rectângulo `1 × 10` | o losango, que tem aspecto `1` |
| ⭐ **enviesamento** (desvio de 90° no pior canto) | o losango de 30° | o rectângulo, que tem cantos rectos |
| **área** (p99 ÷ p50) | a orelha grossa ao lado da calota fina | as duas de cima |

### ⭐⭐⭐ A barra saiu do ORÁCULO, não de uma opinião

⚠️ **A `sculpt_eared` não estava no corpus da bancada** — nove peças de que ninguém
se queixou, e não a única de que alguém se queixou. Foi acrescentada e o oráculo
correu sobre ela. Medido com **o mesmo código** nos dois lados:

| `d = 1,0` | faces | aspecto p50 | p99 | `> 4×` | ⭐ **env. p50** | p99 | ⭐ **`> 60°`** |
|---|---|---|---|---|---|---|---|
| ⭐ **oráculo, orelha** | 4 658 | **`1,08`** | `1,4` | **0** | **`6°`** | `20°` | **`0`** |
| ⛔ nós, orelha | 78 403 | `1,98` | `7,4` | 3 558 | `27°` | `79°` | **9 159 (12 %)** |
| ⛔ nós, gancho | 8 772 | `1,62` | `6,6` | 581 | `28°` | `86°` | 1 872 (21 %) |
| ⛔ nós, enrugada | 29 468 | `1,28` | `3,1` | 152 | `18°` | `87°` | 8 281 (28 %) |

⭐ **A orelha é a peça MAIS LIMPA do corpus dele** (`1,08 / 1,4 / 1,6 / zero`). Ela é
fácil para o oráculo; é a nossa cadeia que a destrói.

⚠️ **E o CONTROLO ilibou a entrada:** a malha do F1 mede aspecto máximo `2,7` e
**zero** faces acima de `4×`. A cadeia cria tudo isto.

### ⛔ Três hipóteses construídas, medidas e REFUTADAS

| hipótese | como morreu |
|---|---|
| «o alisador do oráculo é a cura» | ⛔ a saída **crua** dele já mede `1,11 / 28°`; o alisamento só a leva a `1,08 / 20°`. *A qualidade está no layout.* |
| «os nossos patches são piores» | ⛔ a decomposição dele da orelha são **10 triângulos e 2 pentágonos**, com espalhamento de tamanho de **18×** — tão «má» como a nossa, e entrega `6°` |
| «a nossa grade não segue o campo» | ⛔ medindo **uma** família de linhas, a nossa segue-o *melhor* que a dele (`5,5°` contra `7,3°` na enrugada) |

### ⛔ E uma CURA construída, medida e rejeitada — [`SQUARE_ROUNDS = 0`](../../../crates/ph2d-quadfill/src/relax.rs)

O alisador que temos é um **Laplaciano**: trata a malha como um grafo, iguala
comprimentos de aresta e é cego ao ângulo — *um losango perfeito é ponto fixo dele*.
Construí a relaxação que falta: cada face pede o **quadrado mais próximo de si**
(forma fechada — o primeiro harmónico da DFT de quatro pontos), cada vértice vai
para a média dos pedidos. Orelha, `d = 1,0`:

| rondas | aspecto max | `> 4×` | ⭐ **env. p50** | ⛔ **dobras** | ms |
|---|---|---|---|---|---|
| **0** | `122,7` | 3 558 | **`27°`** | **171** | 5 063 |
| 16 | `30,3` | 2 143 | **`26°`** | **576** | 16 009 |

⭐ **A cauda melhora 4×; a mediana não se mexe.** Preço: `3,4×` as dobras.

### ⭐⭐⭐ O que essa rejeição PROVA — e vale mais que a feature

**Uma relaxação move vértices e mais nada.** Se dezasseis rondas de um método cuja
função-objectivo *é* a esquadria não movem a mediana, então **endireitar um quad
desendireita o vizinho** ⇒ o esmagamento está na **CONECTIVIDADE**, não nas
posições, e nenhum alisador lhe toca.

⚠️ *E há mecanismo para as dobras a mais:* num vértice irregular o pedido é
**contraditório** — três quads a pedir 90° somam 270° e têm de fechar 360°.

### ⭐⭐⭐ A causa, nomeada: **a SEGUNDA família de linhas**

A sonda de uma família não discriminava. Medindo **as duas** ([`sculpt3d_field_follow.rs`](../../../shells/desktop/src/sculpt3d_field_follow.rs)):

| | só a família `u` | ⭐ **as duas famílias** |
|---|---|---|
| ⭐ oráculo, gancho | `5,1°` | **`7,6°`** — mal se move |
| ⛔ nós, gancho | `9,9°` | ⛔ **`19,2°`** — mais do dobro |
| ⭐ oráculo, orelha | `6,0°` | **`7,8°`** |
| ⛔ nós, orelha | `16,0°` | ⛔ **`21,7°`** |

⇒ **A nossa primeira família segue o campo; a segunda não fica ORTOGONAL a ela.** É
a assinatura da interpolação transfinita: ela casa com a **fronteira** do patch e
enviesa no **meio**. ⚠️ E [`fill_with`](../../../crates/ph2d-quadfill/src/stitch.rs) nem sequer **recebe** o campo — uma fase que
não o tem entre os argumentos não o pode seguir no interior.

### ⇒ O próximo passo, e é ele que fecha o defeito da foto

**O interior de um patch tem de nascer de uma parametrização alinhada ao campo**, não
de uma interpolação da fronteira. O gate `the_quads_are_as_square_as_the_oracles`
está **vermelho com esse endereço** e a barra é a do oráculo (`p50 ≤ 10°`, `> 60°`
abaixo de `0,1 %`).

### ⚠️ E a régua MUDOU-SE para o caminho do produto no mesmo dia

⛔ Ela nasceu numa sonda `#[ignore]`, que é onde uma régua **não existe**. Hoje o
[`FillReport::shape`](../../../crates/ph2d-quadfill/src/report.rs) mede-a na cadeia, o `QuadRemeshReport` carrega-a, e a linha que o
artista lê **diz o enviesamento**. Dois gates verdes guardam isso —
`the_report_carries_the_shape_of_every_quad` e uma asserção nova dentro de
`the_button_delivers_the_global_chain` —, e **os dois foram provados por mutação**
(apagar `shape: r.shape` na porta, e tirar a palavra da linha).

### ⛔ Recusas MEDIDAS nesta secção

| o quê | porquê não | onde |
|---|---|---|
| `SQUARE_ROUNDS > 0` | cauda melhora, mediana não; `3,4×` dobras | [`relax.rs`](../../../crates/ph2d-quadfill/src/relax.rs) |
| alisador de quads como cura | a saída **crua** do oráculo já é boa | esta secção |
| culpar a forma dos nossos patches | os dele são 10 triângulos com espalhamento `18×` | esta secção |
| sonda de campo com **uma** família | um quad esmagado **passa** nela | [`sculpt3d_field_follow.rs`](../../../shells/desktop/src/sculpt3d_field_follow.rs) |
| `a_rhombus_becomes_a_square` como prova da lei | é **tautologia**: `h·iᵏ` é quadrado para qualquer `h` | [`relax_tests.rs`](../../../crates/ph2d-quadfill/src/relax_tests.rs) |

---

## 4-sexiestricies — ⛔⛔ **DUAS CURAS CERTAS, ZERO MOVIMENTO — e a régua que finalmente localizou o defeito**

> **2026-08-23.** A secção anterior nomeou a causa do enviesamento: *o interior de um
> patch nasce de interpolar a fronteira, e a 2.ª família de linhas não fica ortogonal
> à 1.ª.* Esta secção implementa a cura que essa frase pede — e regista que ela **não
> funcionou**, com o mecanismo.

### ⭐ O que foi construído

**1. O interior segue o campo** ([`aligned.rs`](../../../crates/ph2d-quadfill/src/aligned.rs)).
O achatamento de Tutte pede que cada `uv` interior seja a média ponderada dos
vizinhos — um mapa **harmónico**, que conhece a fronteira e mais nada. A lei nova
pede que **cada passo até um vizinho valha o que o campo diz que ele vale**:

```text
    minimizar  Σ w_ij · | (uv_j − uv_i) − c·d_ij |²   ⇒   uv_i = Σ w_ij (uv_j − c·d_ij) / Σ w_ij
```

⭐ Com `d = 0` isto é a lei antiga **termo a termo** — a inércia é demonstrável, não
prometida. Os pesos continuam os de valor médio (sempre positivos, garantia de Tutte
intacta), e a rede é a **contagem de triângulos virados** no domínio.

⚠️ **Nenhuma constante mágica:** a escala e a rotação entre a cruz e os eixos do
domínio saem em forma fechada da **fronteira já presa** (`c = Σ conj(d)·Δuv / Σ |d|²`).

⭐⭐ **E o campo passou a CHEGAR ao F5** — [`PatchLayout::face_dir`](../../../crates/ph2d-trace/src/patches.rs).
Ele viaja no layout e não num parâmetro novo, porque *um parâmetro pode ser esquecido
em qualquer um dos dezoito sítios que chamam o F5; quem tem o layout tem, por
construção, o campo que o gerou.*

**2. O domínio na proporção dos segmentos** ([`corners_for_sides`](../../../crates/ph2d-quadfill/src/param.rs)).
O polígono era **regular**: todo lado com o mesmo comprimento, independentemente de
quantos quads carrega. Um patch de 4 lados com `13 × 6` segmentos recebia `13`
divisões numa direcção e `6` na outra sobre um quadrado `1 × 1` — **toda célula
nascia com aspecto `2,17` antes de tocar na superfície.**

### ⛔ A tabela — orelha, `d = 1,0`, 78 403 quads

| | enviesamento p50 | `> 60°` | dobras | detalhe p95 |
|---|---|---|---|---|
| fronteira (o que shipa) | **`27°`** | 9 146 | 170 | `0,219 %` |
| ⭐ interior pelo campo | **`27°`** | 9 062 | 161 | `0,189 %` |
| domínio proporcional | **`27°`** | 9 146 | 170 | `0,219 %` |

⛔ **Nem uma nem outra move o alvo.** E a proporcional **piora a cauda do gancho**
(aspecto máximo `22,5 → 49,0`; `> 4×` de `581 → 658`). ⇒ as duas ficam no código,
**desligadas, com a tabela ao lado**.

### ⭐⭐⭐ A régua que faltava: **onde** mora o enviesamento

Duas hipóteses boas falharam ⇒ o modelo estava errado. A resposta é parar de supor e
perguntar à malha. [`skew_by_provenance`](../../../crates/ph2d-quadfill/src/shape.rs)
dá a mediana por **fase de origem** dos cantos de cada face — orelha, `d = 1,0`:

```text
    canto (F3) 0°    arco 26°    centro (F3) 0°    raio 56°    grade 26°
```

⭐⭐ **Está em TODA a parte, e a grade interior mede o mesmo que o resto.** Não é o
leque, não é a costura, não é um caso raro. ⇒ *isso exclui de uma vez toda a família
de hipóteses «uma construção local está errada»* — que é a família a que as duas
curas acima pertenciam.

### ⭐ E o alisamento foi ILIBADO, com número

A hipótese «é o Laplaciano que enviesa» é natural (ele move vértices para o centróide
e podia cisalhar). Medido na orelha:

| rondas | grade | raio | dobras | aspecto p99 |
|---|---|---|---|---|
| 0 | `27°` | `75°` | 118 | `13,9` |
| 6 (o que shipa) | `26°` | `56°` | 161 | `7,5` |
| 20 | `25°` | `37°` | 57 | **`4,7`** |

⇒ **ele REPARA e não causa** — a mesma conclusão que o `param.rs` já registava para
as dobras, agora também para o ângulo. ⚠️ E `20` rondas são interessantes por si
(dobras `161 → 57`, `> 4×` de `3 665 → 2 299`) ao preço de `14 s` contra `5 s`.
**Não ligado: o alvo continua parado, e o relógio é do artista.**

### ⭐⭐⭐ O achado que aponta a próxima fase: **a HOLONOMIA**

[`Aligned::holonomy_deg`](../../../crates/ph2d-quadfill/src/aligned.rs) mede o
desacordo que sobra ao **pentear** o campo dentro de um patch. Se o patch não contém
singularidade — que é o que o traçado promete —, ele é ~`0°`. Medido:

| fixtura | holonomia |
|---|---|
| orelha | **`29°`** |
| gancho | **`44°`** |
| enrugada | **`16°`** |

⛔ **O campo dentro dos nossos patches NÃO é combável.** ⇒ pedir ao interior que siga
um campo inconsistente não podia funcionar, e a dívida é do **F3** — o traçado está a
deixar singularidades **dentro** dos patches em vez de as pôr nos cantos.

⚠️ **É um MAX sobre arestas, não uma mediana** — um único ponto mau dá `29°`. A
próxima medição tem de dar a distribuição e dizer **quantos** patches estão sujos.

### ⇒ O próximo passo, com endereço

**Os patches têm de conter as singularidades nos CANTOS.** Enquanto isso não for
verdade, nenhuma lei sobre o interior de um patch pode ser aplicada — o campo que ela
seguiria não existe lá dentro de forma consistente. É o mesmo F3 que a §4-quinquiestricies
já tinha na fila pelo patch de perímetro 520 %.

### ⛔ Recusas MEDIDAS nesta secção

| o quê | porquê não | onde |
|---|---|---|
| interior alinhado ao campo | não move o alvo; holonomia explica porquê | [`aligned.rs`](../../../crates/ph2d-quadfill/src/aligned.rs) |
| domínio ∝ segmentos | não move o alvo e piora a cauda do gancho | [`param.rs`](../../../crates/ph2d-quadfill/src/param.rs) |
| «é o alisamento que enviesa» | a `0` rondas é **pior** | esta secção |
| `SMOOTHING_ROUNDS = 20` | compra cauda e dobras, paga `2,8×` o relógio, alvo parado | esta secção |

---

## 4-septiestricies — ⛔⛔ **A HOLONOMIA foi ACUSADA E ILIBADA no mesmo dia, pelo CONTROLO**

> **2026-08-23, poucas horas depois da §4-sexiestricies.** Aquela secção fechou com um
> achado: *«pentear o campo dentro de um patch deixa 29° (orelha) e 44° (gancho) de
> desacordo ⇒ há singularidade DENTRO dos nossos patches, e a dívida é do F3».* Ele foi
> escrito no `CLAUDE.md`, no `PLAN.md` e numa mensagem de commit. **Estava errado.**

### ⭐ O que o levantou

A própria nota dizia o que faltava: *«é um MAX sobre arestas — a próxima medição tem de
dar a distribuição e dizer QUANTOS patches estão sujos»*. Foi o que se fez, e com a
peça que decide: **a decomposição do ORÁCULO, medida com o mesmo código**
(`*_rem_p0.patch`, o dono de cada face, que a bancada já tinha em disco).

### ⛔ A tabela

| | patches | `max` do pior | ⭐ **p50 mediano** | ⭐ **p95 mediano** |
|---|---|---|---|---|
| nós, orelha | 17 | `29,3°` | `0,479°` | `2,52°` |
| ⭐ **oráculo, orelha** | 12 | **`18,6°`** | **`0,470°`** | **`3,19°`** |
| nós, gancho | 26 | `44,1°` | `0,892°` | `3,70°` |
| ⭐ **oráculo, gancho** | 15 | **`38,4°`** | **`0,726°`** | **`3,63°`** |
| nós, enrugada | 14 | `15,6°` | `0,525°` | `2,62°` |
| ⭐ **oráculo, enrugada** | 8 | **`16,6°`** | **`0,437°`** | **`2,16°`** |

⇒ **A referência tem exactamente a mesma coisa**, e no `p95` chega a ter **mais**. Os
nossos patches são tão penteáveis quanto os dele. ⛔ *A hipótese está morta, e com ela a
barra `CLEAN_DEG = 1,0` que eu tinha escrito — ela classificava **12 de 12** patches do
oráculo como sujos.*

### ⚠️ E o controlo quase não correu — a armadilha dentro da armadilha

A primeira versão de `comb` devolvia `None` ao primeiro triângulo degenerado. Sobre a
malha do oráculo isso deu `None` nos **12 patches de 12**, e a sonda imprimiu:

```text
    ⭐ORACULO: 12 patches · ⭐0 SUJOS (resíduo > 1°) · 12 sem resposta
```

⛔ **«0 sujos» sobre ZERO patches medidos**, e «0 sujos» lê-se como *limpo* — teria
«confirmado» a acusação com o oposto exacto do que os dados diziam. ⭐ **O que salvou foi
a coluna `sem resposta`**, que estava lá porque *skip gracioso não é verde* (`CLAUDE.md`
§5.0). A cura: a face impossível fica **de fora e CONTADA** (`Holonomy::skipped`), e a
região continua a ser medida pelo resto.

### ⇒ O que fica excluído, e o que sobra

Cinco hipóteses medidas e mortas para o enviesamento:

| # | hipótese | como morreu |
|---|---|---|
| 1 | a relaxação por ajuste de quadrado | 16 rondas: `27° → 26°` (§4-quinquiestricies) |
| 2 | o interior alinhado ao campo | `27° → 27°` (§4-sexiestricies) |
| 3 | o domínio ∝ segmentos | `27° → 27°`, e piora o gancho (§4-sexiestricies) |
| 4 | «é o alisamento que enviesa» | a `0` rondas é **pior** (§4-sexiestricies) |
| 5 | ⭐ **«o campo não é combável dentro dos patches»** | **o oráculo tem o mesmo** (esta secção) |

⭐ **E o `skew_by_provenance` já tinha dito onde ele mora:** `arco 26° · grade 26° ·
raio 56°` — *em toda a parte, com a grade interior igual ao resto*. Com a hipótese 5
morta, o F3 fica **ilibado por esta via** (os patches dele são tão penteáveis quanto os
do oráculo), e o que sobra por testar é a **densidade**: na orelha o oráculo entrega
`4 658` quads e nós `78 403` a `d = 1,0`; a `d = 0,5` entregamos `2 868` e o
enviesamento ainda é `21°` contra `6°` dele. *A próxima medição compara as duas saídas
à MESMA contagem de quads, e isso ainda não foi feito.*

### ⛔ Recusas MEDIDAS nesta secção

| o quê | porquê não | onde |
|---|---|---|
| `Holonomy::CLEAN_DEG = 1,0` | reprova 12 de 12 patches do **oráculo** | [`comb.rs`](../../../crates/ph2d-crossfield/src/comb.rs) |
| «a dívida do enviesamento é do F3, por combabilidade» | a distribuição dele é igual à nossa | esta secção |
| `comb` devolver `None` à primeira face má | dá «0 sujos» sobre zero medidos | [`comb.rs`](../../../crates/ph2d-crossfield/src/comb.rs) |

---

## 4-duodequadragies — ⭐⭐⭐ **A ESFERA LISA — a reprodução mais barata, e sete hipóteses mortas**

> **2026-08-23.** Sete curas e explicações foram construídas e medidas em duas
> jornadas, **todas sobre esculturas**. Esta secção regista as duas últimas a morrer e
> a pergunta que ninguém tinha feito.

### ⛔ As duas que morreram aqui

**1. A densidade.** Todas as tabelas comparavam a nossa saída a `d = 1,0` (`78 403`
quads na orelha) com a do oráculo (`4 658`). ⚠️ *Duas malhas 17× diferentes não são
comparáveis.* Varrendo o `detail` até à contagem dele:

| orelha | quads | aspecto p50 | ⭐ **enviesamento p50** |
|---|---|---|---|
| nós, `d = 0,30` | 942 | `1,60` | `18°` |
| nós, `d = 0,55` | 4 162 | `1,76` | `22°` |
| nós, `d = 1,00` | 78 403 | `1,98` | `27°` |
| ⭐ **oráculo** | **4 658** | **`1,08`** | **`6°`** |

⇒ **À contagem dele continuamos em `22°` contra `6°`.** A densidade move o número de
`18°` a `27°` — nunca para perto de `6°`. **Morta.**

**2. O mapa conforme.** O achatamento usa pesos de **valor médio**, e a nota ao lado
deles dizia: *«cotangente seria harmónico e admite peso negativo num triângulo obtuso —
é aí que a garantia de Tutte se perde»*. ⚠️ **Verdadeiro, e responde à pergunta
errada**: troca conformalidade por validade, e o preço em **enviesamento** nunca foi
medido. Construído ([`cotangent_weights`](../../../crates/ph2d-quadfill/src/param.rs))
com a rede de triângulos virados:

| esfera lisa, `d = 0,55` | valor médio | cotangente |
|---|---|---|
| enviesamento p50 | `18°` | `18°` |
| faces `> 60°` | 141 | 142 |
| ⭐ **recuos** | — | **`0/16`** |

⚠️ **`0/16` recuos é o que torna a medição honesta:** o mapa conforme **sobreviveu em
todos os patches** e mesmo assim não mudou nada. ⛔ *Dois operadores de Laplace
fundamentalmente diferentes dão o mesmo número* ⇒ **o interior do achatamento não é o
que decide o enviesamento**; o que os dois partilham é a **fronteira presa**. **Morta**,
e fica desligada com esta tabela (`CONFORMAL_MAP = false`).

### ⭐⭐⭐ A pergunta que ninguém tinha feito

**O que a cadeia faz com uma esfera LISA?** Sem relevo, sem vinco, sem bico.

| esfera lisa, `d = 0,55` | nós | ⭐ oráculo |
|---|---|---|
| quads | 2 006 | 3 352 |
| aspecto p50 | `1,26` | **`1,22`** |
| ⛔ **enviesamento p50** | **`18°`** | **`6°`** |
| ⛔ faces `> 60°` | **141** | **0** |

⭐⭐ **As células têm as PROPORÇÕES certas e os ÂNGULOS errados**, numa peça que não
tem defeito nenhum para expor. ⇒ **o defeito é do NÚCLEO**, e esta é a reprodução mais
barata que esta linha alguma vez teve. **Gate vermelho:**
`a_plain_sphere_is_as_square_as_the_oracles`, com a barra do oráculo **na mesma peça**
(ela está no corpus da bancada).

⛔ **Toda hipótese nova mede-se AQUI primeiro** — sete morreram sobre esculturas, onde
o sinal chega misturado com dez patologias.

### ⛔⛔ E dois DEFEITOS NOVOS que só apareceram nos casos triviais

| peça | o quê |
|---|---|
| ⛔ **toro `64×32`** | a **quantização RECUSA** nos três níveis de detalhe. O oráculo entrega `5 538` quads a `2°` de enviesamento |
| ⛔ **esfera `24×36`** (grossa ⇒ o F1 **refina**) | aspecto p50 **`4,38`**, enviesamento **`52°`**, **50 dobras** — e a densidade é **não-monótona**: `d = 0,55` dá `823` quads e `d = 0,80` dá `643` |

⚠️ **A rota em que o F1 REFINA nunca tinha sido medida**, e ela está muito pior que a
que grosseiraria. *Uma peça mais grossa que ~2 500 vértices entra por aí.*

### ⭐ A pista por perseguir — hipótese, NÃO medição

Dos 16 patches da esfera lisa: **8 triângulos, 5 quadriláteros, 3 pentágonos.** Onze
passam pelo **leque**, e um sector de leque é um *papagaio* no domínio, não um
rectângulo — uma grade construída dentro dele nasce enviesada. ⛔ **Não medido**, e o
`skew_prov` complica-a: ele diz que a `grade` está tão torta quanto o `raio`.

### ⛔ Recusas MEDIDAS nesta secção

| o quê | porquê não | onde |
|---|---|---|
| «é a densidade» | à contagem dele, `22°` contra `6°` | esta secção |
| `CONFORMAL_MAP` (cotangente) | `18° → 18°` com **`0/16`** recuos | [`param.rs`](../../../crates/ph2d-quadfill/src/param.rs) |
| medir curas na ORELHA | sete hipóteses morreram lá; a esfera lisa dá o mesmo sinal limpo | esta secção |
