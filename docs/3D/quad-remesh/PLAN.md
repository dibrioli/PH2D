# PLAN — Quad Remesher estado-da-arte na PH2D

> **Documento VIVO.** Decisão e fronteira jurídica: [ADR-0161](../../architecture/decisions/0161-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md).
> O que o porte local entregou e por quê: [ADR-0160](../../architecture/decisions/0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md).
> ✅ **Aprovado pelo Enio em 2026-08-20** (*"Siga como achar melhor... buscamos o estado da arte independente dos custos"*).
> Estado: **F1, F2 e o PROTÓTIPO do F4 FEITOS** (§4-bis, §4-ter, §4-quater). O CAMPO está em paridade com o
> estado da arte (8 singularidades numa esfera, o ótimo teórico) e o **quantizador fecha com o ótimo
> demonstrado** em todos os layouts fechados do oráculo. Próximo: **F3** — o traçado é a peça que falta
> para o pipeline correr de ponta a ponta com código nosso.
> ⚠️ Nada disto está ligado ao produto: ligar F1 ou F2 hoje pioraria o que o artista vê (§4-ter).

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
| `sphere_shuffled` | 13 682 | 27 360 | a mesma forma com ordem de índice embaralhada (**controle de determinismo**) |
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
| **F3** | tracing + patches | nº de patches na mesma ordem do oráculo (15 na `sculpt_hooked`); zero patch não-disco. ⚠️ **É a peça que falta**: o F4 já consome layout, e o F3 é quem o produzirá sem o oráculo |
| **F4** | ⭐ **solver Bi-MDF** | ✅ **PROTÓTIPO FEITO em 2026-08-20** (`crates/ph2d-quantize`) — ver §4-quater. Fecha com o **ótimo demonstrado** em todos os layouts fechados do oráculo; falta só o consumidor (F5) e a válvula de emergência, que **nenhum layout pediu** |
| **F5** | quadrangulação por patch + smoothing + a porta no shell | desvio angular médio ≤ oráculo × 1,2; Hausdorff ≤ oráculo × 1,2; ⭐ **`sculpt_hooked` sem aglomerado e sem colapso na feature** (gate de regressão do §9) |
| **F6** | guide strokes: direção → feature → densidade | ⚠️ **a densidade por PRESSÃO depende da camada de tablet, que NÃO existe** (ADR-0161) — F6 entrega direção e feature; a pressão é um projeto irmão |
| **F7** | dois backends (preview BSD + qualidade) partilhando o campo | preview < 1 s até 100 k triângulos; o preview mostra o alinhamento que o modo qualidade honra |
| **F8** | invalidação incremental + pinning de singularidade | editar um stroke não custa o pipeline inteiro; ⚠️ **exige infraestrutura de DAG que o Sculpt não tem** |

⚠️ **F6 e F8 estão precificadas com o defeito de premissa embutido** (ADR-0161): elas não são
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

## 4-ter — ✅ F2: o CAMPO chegou ao ótimo teórico. A MALHA não, e a razão é o F5

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

Ordem recomendada: ~~F0 → F1 → F2 → (F4 protótipo em paralelo, sobre patches do oráculo)~~ **→ F3 → F5 → F7 → F6 → F8.**

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
