# PLAN — Quad Remesher estado-da-arte na PH2D

> **Documento VIVO.** Decisão e fronteira jurídica: [ADR-0161](../../architecture/decisions/0161-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md).
> O que o porte local entregou e por quê: [ADR-0160](../../architecture/decisions/0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md).
> ⚠️ Este plano **aguarda aprovação**. Nenhuma linha de código de produto foi escrita para ele.

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
| **F1** | sanitização + **remesh isotrópico** + sizing field | `cube` deixa de devolver malha vazia; a densidade da saída deixa de depender da densidade da entrada (é a lacuna §3-2) |
| **F2** | cross field MIQ-style + streamlines no viewport | ⭐ **vértices irregulares ≤ 2 %** no corpus liso (hoje: 21–49 %). É esta fase que mata os sintomas 1 e 2 |
| **F3** | tracing + patches | nº de patches na mesma ordem do oráculo (15 na `sculpt_hooked`); zero patch não-disco |
| **F4** | ⭐ **solver Bi-MDF** | quads **= 100,0 %** no corpus fechado; χ preservado; **fase crítica** |
| **F5** | quadrangulação por patch + smoothing + a porta no shell | desvio angular médio ≤ oráculo × 1,2; Hausdorff ≤ oráculo × 1,2; ⭐ **`sculpt_hooked` sem aglomerado e sem colapso na feature** (gate de regressão do §9) |
| **F6** | guide strokes: direção → feature → densidade | ⚠️ **a densidade por PRESSÃO depende da camada de tablet, que NÃO existe** (ADR-0161) — F6 entrega direção e feature; a pressão é um projeto irmão |
| **F7** | dois backends (preview BSD + qualidade) partilhando o campo | preview < 1 s até 100 k triângulos; o preview mostra o alinhamento que o modo qualidade honra |
| **F8** | invalidação incremental + pinning de singularidade | editar um stroke não custa o pipeline inteiro; ⚠️ **exige infraestrutura de DAG que o Sculpt não tem** |

⚠️ **F6 e F8 estão precificadas com o defeito de premissa embutido** (ADR-0161): elas não são
"mais uma fase do remesher", são infraestrutura de produto que hoje não existe.

---

## 5 — Risco, por ordem de quanto pode custar

| risco | por que é real | mitigação |
|---|---|---|
| ⭐ **Bi-MDF do zero** (F4) | não há min-cost-flow permissivo em Rust com grafo bi-dirigido e custo convexo; é a fase de que 100 % dos quads depende | atacar F4 **cedo** com um protótipo isolado sobre os patches que o **oráculo** exporta (`.patch`/`.corners` são texto) — assim F4 é medida **antes** de F3 estar pronta |
| **rounding do campo divergir** (F2) | o paper descreve a energia; **a ordem do rounding** decide onde as singularidades caem, e é onde o oráculo tem escolhas não publicadas | comparar contagem+posição de singularidades contra o `.rosy` que o oráculo já escreve |
| **a lei de densidade do estágio 2** | o oráculo dá ~4 500 vértices de qualquer entrada; não sabemos de onde vem | varrer os presets do oráculo e regredir a lei a partir das saídas — é medição, não leitura de fonte |
| **degenerescências em sculpts** | `sculpt_punctured` sai com 88 arestas de borda **no oráculo** | a sanitização é nossa e é F1, não um "detalhe" |
| **`f64` × `Mesh` em `f32`** | determinismo cross-platform tem de ser afirmado onde ele vive | núcleo em `f64`, fronteira explícita, hash da saída no gate 3-OS |
| **custo do Bi-MDF em malha grande** | o oráculo levou minutos em algumas | modo qualidade é **offline com barra**; o preview BSD cobre o laço interativo |
| ⚠️ **ambiguidade dos papers** | inevitável num clean-room | toda dúvida vira uma linha neste plano com o experimento que a resolveu — ⛔ nunca uma olhada na fonte GPL |

---

## 6 — Esforço e ordem de ataque

Ordem recomendada: **F0 → F1 → F2 → (F4 protótipo em paralelo, sobre patches do oráculo) → F3 → F4 → F5 → F7 → F6 → F8.**

⚠️ **F4 sai da ordem natural de propósito.** Ela é a fase de maior risco e a que o produto inteiro
depende; medi-la sobre os patches que o oráculo **já exporta** custa pouco e responde cedo a pergunta
que pode matar o plano. *Descobrir no fim que o solver não fecha seria descobrir tarde demais.*

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

1. ⛔ **A lei de densidade do estágio 2 do oráculo** (§3, §5) — é a primeira medição da F1.
2. ⛔ **O QEx 2013** não foi obtido.
3. ⛔ **Ninguém mediu Hausdorff** ainda: `metrics.py` cobre contagem, topologia e ângulo; a fidelidade
   geométrica é a lacuna do F0.
4. ⛔ **O preset `Mechanical` não foi corrido** — só o `Organic`. Um segundo eixo por medir.
