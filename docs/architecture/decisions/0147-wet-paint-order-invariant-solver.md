# ADR-0147 — O solver do Wet Paint é INDEPENDENTE DE ORDEM (e é por isso que ele paraleliza)

- **Status:** ACEITO (Enio, 2026-07-30 — ordem literal: *"GPU do Wet Paint"*).
- **Data:** 2026-07-30.
- **Escopo:** troca o `advect` e o `drying_pass` do Wet Paint por formas **independentes de
  ordem** (`solver::advect_jacobi`, `drying::drying_pass_jacobi`), habilita `rayon` nos dois, e
  **move o pino de fingerprint** do ADR-0134 com justificativa.
- **Não afeta:** nenhum contrato congelado (Nodes [ADR-0039](0039-nodegraph-contract-freeze-w2t4.md),
  Tools [ADR-0040](0040-tool-as-isolated-feature-crate.md), Vector). Nenhum schema
  (`PROJECT_SCHEMA` intocado). Nenhuma dep nova.
- **Precedentes:** [ADR-0109](0109-rayon-exception-watercolor-composite.md) (a 1ª exceção `rayon`),
  [ADR-0145](0145-wet-paint-solver-row-parallel-passes-rayon-exception.md) (a 2ª — cuja **§2 recusa
  explicitamente estes dois passes**, e corretamente), [ADR-0146](0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md)
  (que nomeia os dois como *"93% do passo, e eles MUDAM os números"*).
- **⚠️ O número 0147 é PROVISÓRIO** — linhas paralelas reivindicam ADRs na mesma janela e o valor se
  **CONTA** na integração, não se escolhe (4 precedentes: 0115, 0131, 0134, e o par 30/32-34 de
  physics×FLIP).

## 1. Contexto — a pergunta era sobre a GPU, e a resposta estava na CPU

O ADR-0146 mediu o passo e concluiu: `advect` (70,4%) e `drying_pass` (21,9%) somam ~92%, os dois
são **Gauss-Seidel**, e portá-los para a GPU seria *"um segundo modelo, não o mesmo mais rápido"*.
A recomendação era **não construir**.

O que aquele ADR não perguntou: **o preço de reformular esses dois passes é o preço da GPU, ou o
preço do PARALELISMO?** A máquina tem **32 núcleos**, e o `advect` — 70% do passo — rodava em um.

## 2. A decisão, e o argumento que NÃO é sobre velocidade

Os dois passes passam a ler **o estado do INÍCIO do passe** (forma de Jacobi):

- **`advect`** vira um **gather conservativo**. A relação é simétrica — se o destino `d` puxa a
  fração `w` do canto `c`, então `c` **dá** `w` a `d` — logo a saída de uma célula é ela mesma um
  gather sobre a vizinhança, e a velocidade limitada (`|u| ≤ maxVelocity`) a torna local. **Sem
  atômicos, sem ordem.**
- **`drying_pass`** materializa o **fator de borda** (a única leitura cross-célula) num pré-passe.

⚠️ **A justificativa é CORREÇÃO, não desempenho.** O Gauss-Seidel varre em ordem de raster e lê o
vizinho que a célula anterior já reescreveu; isso não é física, é a direção do laço, e tem
assinatura mensurável. Numa folha **espelhada** — massa simétrica, fluxo antissimétrico, cena cuja
física é simétrica por construção (`tests/solver_symmetry.rs`):

| passe | desvio de espelho, Gauss-Seidel | independente de ordem |
|---|---:|---:|
| `advect` | **1189,29** unidades de massa | **0,000000** |
| `drying_pass` | **554,82** | **0,000000** |

O viés do advect é **mais que uma célula cheia de pigmento** (o platô da fixture é ~900), deslocada
só porque o laço anda da esquerda para a direita.

## 3. O que isso custa — medido, não estimado

**(a) O fingerprint do ADR-0134 se move.** Ele é o contrato de fidelidade do port 1:1, e o
protocolo do doc 23 é honrado: o pino novo é do produto, e **o pino ANTIGO vira um gate
executável** na rota de ablação `Sim::order_invariant = false`
(`the_gauss_seidel_route_still_reproduces_its_own_pin`). É esse gate que torna a troca **auditável
em vez de um número que mudou**: ele prova que nem a secagem, nem o fluxo, nem a projeção, nem o
depósito, nem o `lift_settled` mudaram — **só** os dois passes desta decisão.

**(b) O escorrido corre menos.** Medido pelo deslocamento do **centroide de massa** do filme
(`tests/measure_transport_range.rs`), varrendo `Flow Grid` 1..8: o solver independente de ordem
transporta **0,64–0,96× (média ~0,82×)** do que o Gauss-Seidel transportava, uniformemente, sem
colapso e **sem viés de direção** (o transporte para cima e para baixo é o mesmo nos dois modelos —
a hipótese óbvia, *"a varredura de cima para baixo cascateia com a gravidade"*, foi **refutada por
medição**). O knob **Gravity** cobre a diferença.

**(c) Memória: +25 B por célula do fluido** (o rascunho derivado — fluxo fino materializado, a
saída, o registro de destino e o fator de borda), alocado **preguiçosamente** no primeiro passo.
Numa grade 1:1 a 4096² são +420 MB; o slider **Grid Size** é a resposta, como já era para os 43 B/
célula que o grid custava antes.

**(d) `rayon` nos dois passes.** A cerca de contenção do ADR-0109 exige ADR por uso novo, e é este.
⚠️ Um deles **acumula em ponto flutuante** — o que a condição 3 do ADR-0145 recusaria —, e a
identidade serial×paralelo vale porque a **ordem da soma é fixa e privada da linha**; há gate
afirmando exatamente isso, mais o irmão de escalonamento (repetir a rota paralela dá sempre o mesmo
resultado).

## 4. O ganho, pela porta do PRODUTO

A/B no **mesmo processo e na mesma poça** (a lição do ADR-0145 §4: comparar duas corridas atribui
deriva de máquina ao ganho), `on_canvas_pointer` a 4096², ciclo de 12 passos:

| `Flow Grid` | Gauss-Seidel | independente de ordem | |
|---|---:|---:|---|
| 1 | 60,19 ms (16,6 Hz) | **29,29 ms (34,1 Hz)** | **2,06×** |
| 4 | 52,05 ms (19,2 Hz) | **11,02 ms (90,8 Hz)** | **4,72×** |

⇒ **a água sai do regime work-limited**: a 90,8 Hz ela corre **2,3× o nominal de 40 Hz da SPEC**, e
o teto passa a ser a SPEC, não a máquina.

## 5. E o que isso faz com o ADR-0146 (a GPU)

O ADR-0146 dizia que um port era **all-or-nothing sobre `advect` + `drying_pass`**, porque os dois
exigiam um modelo diferente. **Esse modelo agora existe, roda em produção, e está provado contra a
referência** — logo o port da GPU deixou de ser um redesenho e virou uma tradução:

- os dois passes são **gather puro por célula**, sem atômicos e sem dependência de ordem;
- a identidade **serial × paralelo** já é gate, e é a mesma propriedade que um `dispatch` exige;
- o oráculo deixou de ser um hash byte-exato e passou a ser **simetria + conservação + paridade de
  rota**, que é exatamente o template que o passe de luz na GPU do Painter usa.

⚠️ **O que NÃO mudou, e continua sendo o bloqueador do ADR-0146:** o stamp recebe a **silhueta do
HOST por closure** (o pincel do Painter — Shape image, ramp LUT, footprint), então (B) segue
exigindo o pincel em WGSL ou um round-trip por batch; e a residência dos planos continua sendo
all-or-nothing. **A recomendação do ADR-0146 não é revogada aqui** — o que muda é que a metade cara
dela (*"93% do passo vira outro modelo"*) **já foi paga na CPU**, e o ganho que sobra para a GPU
tem de ser medido contra 11 ms, não contra 52.

## 6. Consequências de aceitar

- o `Sim::order_invariant` (default `true`) é uma **ablação** no molde do `Grid::spans_enabled`: um
  relatório de campo se bissecta com um bool, e o pino antigo continua executável;
- **UM flag para os DOIS passes**, de propósito — eles são a mesma mudança, shipam juntos e o smoke
  os julga juntos; dois bools sugeririam uma combinação que ninguém projetou;
- os dois kernels seriais **ficam no repo** como caminho de referência (CLAUDE.md §0: o caminho de
  referência só precisa computar a mesma resposta);
- ⚠️ **um gate teve o ORÁCULO corrigido, não a barra afrouxada:** o
  `the_water_still_runs_when_the_flow_is_coarse` media a **célula mais extrema acima de um limiar**,
  uma estatística de um valor só e caótica na razão de fluxo (o mesmo motor devolvia 27, 23, 36, 18,
  10, 14, 21 — 3,6× de amplitude *dentro do mesmo modelo*). Pelo **centroide de massa** a varredura é
  lisa e a queda em `rf = 2` aparece **igual nos dois modelos** (0,60 × 0,64) — ela é da grade de
  fluxo, não do solver. A frente amplificava um 0,6 compartilhado em 0,85 contra 0,43, e foi assim
  que o gate reprovou uma mudança de modelo por um motivo que não era o dela.
