# Handoff MESTRE de integração — `line/motion-value` (2026-08-04)

**Branch:** `line/motion-value` · **Base (merge-base com `main`):** `dc0587cbe` ·
**HEAD:** `7bd6c0e0c` · **20 commits** · **87 arquivos, +6811 / −719**

Este é o handoff **guarda-chuva** da janela. O batch tem TRÊS frentes; duas já têm
sub-handoff dedicado (referenciados abaixo — não re-documentados aqui), e a terceira
(o diagnoser ADR-0155 + a fila de fixes de hoje) é detalhada neste doc.

---

## As três frentes

### (A) Doc 86 item 3 — filho vetor/flip de grupo SEM NOME carimba pelo drawing id
Commits `54f2f9ffc` · `0a11a4f0d` · `0e0935ca7`. **Aprovado** ("smoke =4: filho vetor
sem nome carimba nas 16 cópias"). **Sub-handoff:**
[`docs/Motion Nodes/HANDOFF_INTEGRACAO_line_motion_value_item3_2026-08-03.md`](Motion%20Nodes/HANDOFF_INTEGRACAO_line_motion_value_item3_2026-08-03.md).
Smoke: **`PH2D_MOTION_OBJ_SMOKE`**.

### (B) ADR-0154 — `source.shape`: formas vetoriais VIVAS na GPU
Commits `97b318f98` · `ecfa6bba0` (handoff) · `debcb2cef` (size 0.05–10) · `69153641a`
(flicker/per-shape params/gear). **Aprovado.** Crate nova **`ph2d-node-motion-shape`**.
`geometry_id` é **CONVENÇÃO DE STREAM** (o gêmeo do `texture_id`), ausente ⇒ byte-idêntico.
**Sub-handoff:**
[`docs/HANDOFF_INTEGRACAO_line_motion_value_shape_2026-08-03.md`](HANDOFF_INTEGRACAO_line_motion_value_shape_2026-08-03.md).
**ADR:** [0154](architecture/decisions/0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md).
Smoke: **`PH2D_SHAPE_SMOKE=1`** (Star → Gear ao vivo).

### (C) ADR-0155 — o setup do grafo é DIAGNOSTICADO e CURADO, não recusado
Commits `1d9afbd2f` (W1) · `dd768c81b` · `400799700` (W2) · `8937202c2` (W2b) ·
`871d6df16` (W3) · `480b0840f` (W4) · `f3d69031e` (fmt) · `d44ea9521` (W4b/2a) ·
`d0808e5f4` · `e909e2712` (W4d/2b) — **mais a fila de fixes de HOJE** (§C.3 abaixo:
`bc4dc688d`, `10796c2ae`, `7bd6c0e0c`). **Plano:**
[`docs/Motion Nodes/87_plano_correcao_automatica_setup.md`](Motion%20Nodes/87_plano_correcao_automatica_setup.md).
**ADR:** [0155](architecture/decisions/0155-motion-graph-setup-is-diagnosed-and-healed-not-refused.md).

**O problema:** o grafo de Motion tem uma classe de erro que **não produz erro** — uma
`force.*` (Pure, acumula `accel`) ligada à saída SEM integrador escreve `accel`, nada
consome, e a cena fica estática, sem erro nem aviso. `Graph::validate` só confere tipos
de porta e membranas.

**A resposta** (crate nova **`ph2d-motion-diagnose`**): `diagnose(graph, reg)` **DERIVA**
os papéis produce/consume da **própria declaração do nó** (a binding de GPU +
side-channel `Coupling`), nunca de uma tabela paralela à mão. Três eixos de defeito:
- **`InertProducer`** (W1/W2/W3/W4) — produz `accel`/`falloff`/`inv_mass` que nada
  consome. Cura: **auto-cura no gesto** (insere `motion.integrate`/`sim.step`, W2) ·
  **reorder reusando o integrador** que já existe (W2b) · ou **badge ⚠ + quick-fix**
  (W3: clica o pip → cura canônica onde existe, EXPLICA + seleciona onde não —
  *nunca adivinha uma escolha criativa*). A família **`falloff`** (W4) vem 100% por
  DERIVAÇÃO (a binding de GPU do `field.*`), zero anotação.
- **`MissingSource("P")`** (W4b/2a) — um deformer/força que LÊ `P` sem nada a montante.
  Aviso (Offer): QUAL fonte é escolha criativa.
- **`MissingInput(porta)`** (W4d/2b) — porta obrigatória sem aresta (o `points` do
  `motion.duplicator`). Único eixo **DECLARADO** (`register_required_inputs`), porque
  required-vs-opcional é semântico.
- **Toggle "Node Help"** — liga/desliga o sistema inteiro (a liberdade do artista).

#### (C.3) A fila de fixes de HOJE (2026-08-04) — sem sub-handoff próprio, detalhada aqui

**`bc4dc688d` — smoke `=6` usa o nó "Shape" DE VERDADE na porta `shape`.** A cena do
aviso `MissingInput` ligava um `motion.grid` na porta `shape` do duplicator (grid são
PONTOS, não forma) — a demo mentia. Agora a `shape` recebe `source.shape` (estrela) e o
grid fica solto para o artista ligar em `points`. O ⚠ da porta `points` continua
disparando; a estrela RENDERIZA por passthrough (sem `points`, o duplicator PASSA a
forma adiante). Smoke: **`PH2D_AUTOFIX_SMOKE=6`**.

**`10796c2ae` — um documento com fonte de aparência (Shape/Object) RECUSA o cook GPU
(o bug dos retângulos brancos).** ⚠️ **Mudança de comportamento, correctness > speed.**
Report do Enio: *"depois do duplicador coloquei rotate e retângulos (do grid)
sobrepõem as estrelas de Shape"* + *"o rot e o grid movem os retângulos, não as
estrelas"* + *"ocorre com os outros sources também"*.
- **Causa:** o cook GPU-resident (ADR-0126, **ON por default**) tem lowering
  **sprite-only** — `LOWER_COLUMNS = ["P","size","rot","tint","uv_rect"]`, **hardcoda
  `texture_id = 0`** (word 41), **sem rota `geometry_id`/vetor**. No instante em que um
  estágio GPU (o `rotate`) roda depois da fonte (Hybrid), tanto o `source.shape`
  (`geometry_id`) quanto o `source.object` (`texture_id`) desenham como **quads brancos
  do atlas**; as estrelas estáticas eram o `vector_instances` da CPU **obsoleto** — por
  isso o rotate/grid moviam os retângulos e não as estrelas (dois produtores lendo dados
  diferentes).
- **A cura, uma recusa no PLANO:** `motion_bridge_gpu::graph_has_appearance_source` — se
  o grafo tem uma fonte de aparência, `cook_gpu` retorna `FellThrough` **antes** de
  `ph2d_gpu_cook::plan(...)`, e o render cai na CPU (que desenha o vetor). Um flag no
  registry: **`register_appearance_source`/`is_appearance_source`** (o `source.object`
  carrega `texture_id`, o `source.shape` carrega `geometry_id` — ambos declaram).
- **Trade honesto:** grafos com fonte de objeto/forma perdem a aceleração de GPU. O
  `source.shape` **não tem** rota de GPU vetorial; o `source.object` **poderia** ser
  acelerado depois se a lowering lesse `texture_id` (otimização futura NOMEADA, não
  contrabandeada).
- **Gates:** unit (`a_document_bringing_in_an_object_or_shape_recuses_from_the_gpu`,
  mutation-proven) + **arch-gate de shell**
  (`shells/desktop/tests/the_gpu_cook_recuses_an_appearance_source.rs`: afirma a ORDEM
  — recusa ANTES de planejar —, não uma distância em bytes; o `cook_gpu` exige
  janela+GPU, nenhum unit test o alcança). Smoke: **`PH2D_SHAPE_SMOKE=2`** (Shape → dup
  ← grid → rotate → output: **16 estrelas nítidas giradas, SE VIR RETÂNGULOS BRANCOS
  PARE**).

**`7bd6c0e0c` — a FONTE COM ESTADO semeia o próprio `P`, não é source-less (o ⚠ falso
do Boids).** ✅ **Aprovado no smoke (`PH2D_AUTOFIX_SMOKE=7`).** Report do Enio, com foto:
*"por que o alerta no Boids se funciona bem?"*.
- **Causa:** o `MissingSource("P")` do W4b era **falso positivo** no `motion.boids`. O
  diagnoser deriva "lê `P`" da binding de GPU (`ReadWrite` em `P`, na porta `state`), e o
  `has_input` **ignora arestas delayed de propósito** — então um Boids com só o `pre`
  self-loop (`out --pre--> state`, delayed) e sem `target` ligado parecia cabeça
  flutuante sem stream. Mas o Boids é uma **FONTE COM ESTADO**: lê o próprio `P` do frame
  anterior pelo self-loop e SEMEIA a nuvem sozinho (`seed` no tick 0).
- **A isenção `seeds_own_state`:** um nó cujo output alimenta uma entrada própria por
  aresta **DELAYED** (o `pre` self-loop de boids/verlet/soft-body/spring/integrate) NÃO
  é source-less. Um deformer nunca carrega self-loop ⇒ sinal derivável, não toca o eixo
  do deformer-sem-pontos. **O doc do módulo foi corrigido** — a afirmação *"zero false
  positives"* **era o próprio bug**.
- **Gates (mutation-proven):**
  `a_stateful_source_that_seeds_its_own_state_is_not_source_less` (isento com self-loop
  + controle positivo sem self-loop; mutação `false` ⇒ falso positivo volta RED, mutação
  `true` ⇒ controle positivo falha RED) · `the_appropriate_flock_stamp_graph_is_clean`
  (pina a cena `=7` inteira headless, 0 avisos — prova que shape/dup/oscillator também
  ficam limpos).
- **A cena `=7` (a forma apropriada da foto):** `The Shape (estrela) × Boids →
  Oscillator → Output`, 48 estrelas nítidas voando, **zero badge**. Corpo no irmão
  `motion_autofix_smoke_appropriate.rs` (o pai cruzou o teto de 600 LOC — split por
  responsabilidade: o pai heala gestos, o irmão é a forma correta).

---

## Contrato / schema — INTACTOS (conferir por grep na árvore combinada)

- **Contrato de nós congelado (§6, ADR-0039): `NodeOp=2` / `OpResolver=1` /
  `NodeManifest=8` — INTACTO (3/3).** Os canais novos do `NodeRegistry`
  (`register_couplings`/`couplings` · `register_required_inputs`/`required_inputs` ·
  `register_appearance_source`/`is_appearance_source`) são **side-metadata** em
  `ph2d-node-registry/src/{lib.rs,ui.rs}` — `BTreeMap`/`BTreeSet` a mais, com par
  `register_*`/getter, **exatamente como `param_gates`/`reduces`/`luts`**. Nenhum toca
  `ph2d-nodegraph/src/node.rs` nem `cook.rs` (os únicos arquivos que o gate
  `architecture_contract_surface` conta por `include_str!`) ⇒ os contadores não se movem,
  por construção. `geometry_id`/`texture_id` são **convenção de stream**, não campo do
  manifest.
- **`PROJECT_SCHEMA` / `VEC_SCENE_SCHEMA` / `DOC_VERSION` — INTOCADOS.** A auto-cura
  produz nós e arestas NORMAIS (o `ProjectState` já os serializa); diagnósticos/badges
  são **view-state transiente** (não salvos); `source.shape` é nó + convenção de stream
  (o grafo viaja como TEXTO e carrega a própria versão). **`git diff main..HEAD` não toca
  `project.rs`.** ⇒ a linha fica **FORA** de qualquer disputa de número de schema da
  janela.

## Crates novas e deps

- **`ph2d-motion-diagnose`** (ADR-0155) — leaf; deps **só internas** (`ph2d-nodegraph` +
  `ph2d-node-registry`). A lib nunca depende dos crates-nó; **só os testes** montam o
  registry cheio (`ph2d-node-registry-init` em dev-deps).
- **`ph2d-node-motion-shape`** (ADR-0154) — leaf; deps **só internas** (mesmas duas).
- **ZERO dep externa nova, ZERO dep git.** (`git diff main..HEAD -- '*Cargo.toml'` só
  traz path-deps internas + comentários.) Registrados via
  `ph2d-node-registry-init::register_all_nodes` (+1 linha) e o membro glob do workspace.

## Toques foundational (todos ADITIVOS, projetados para isolamento)

- `ph2d-node-registry/src/{lib.rs,ui.rs}` — os 3 canais side-metadata acima (+119/+61).
- `ph2d-editor-core/src/interaction/types.rs` — variant **`GraphHitKind::InertBadge`**
  (apendado; ⚠️ enum de interação **NÃO-congelado**, evolui livre — sem gate de contagem).
- `ph2d-eval-motion/src/lower.rs` — a convenção `geometry_id` +
  `lower_to_vector_instances_onto` (ADR-0154, front B; ver sub-handoff).
- Os nós `force.*` / deformers / `motion.integrate` / `sim.step` etc. (+5 linhas cada) —
  `register_couplings(...)` declarando `Produces("accel")`/`Consumes`/`Requires`, para o
  diagnoser. Aditivo, não muda comportamento de cook.

## ⚠️ Colisão de número de ADR — 0154 / 0155 são PROVISÓRIOS

O `main` do dia da base estava em **0153** (Vector auto-layout, 2026-08-02). 0154/0155
foram escolhidos como os próximos livres, mas **um número escolhido numa linha paralela é
PROVISÓRIO** — se outra linha integrar 0154/0155 ANTES desta, **renumere** (git nunca
conflita: os nomes de arquivo diferem). Ao renumerar, o rewrite do token é **escopado aos
arquivos que a LINHA mudou** (`git grep ADR-0154`/`ADR-0155` na interseção linha ∩
citações), **nunca** o número nu sobre a árvore. Conferir na hora:
`git ls-tree main docs/architecture/decisions/ | grep -E '015[4-9]'`.

## Verificação (gates + smokes)

**Gates (por crate, rodei verdes localmente — o integrador re-roda na árvore combinada):**
- `cargo test -p ph2d-motion-diagnose` — 21 (4 unit + 17 integração + 0 doc), incl. os
  gates mutation-proven de hoje.
- `cargo test -p ph2d-node-motion-shape` · `-p ph2d-node-registry` · `-p ph2d-eval-motion`.
- Shell: `cargo test -p ph2d-host-desktop --test file_loc_caps`,
  `--test the_gpu_cook_recuses_an_appearance_source`; `ph2d-editor-core --test no_tofu_glyphs`.
- `architecture_contract_surface` (3/3) · `architecture_workspace_file_loc_cap` (700) ·
  node-sync staleness. **Rodar a suíte também em DEBUG** (precedente da linha: o
  `voronoi.rs` do flip-colorize panicava só em debug).
- clippy nas crates tocadas — limpo. LOC: `motion-diagnose` 591<700; `motion_autofix_smoke`
  519<600; irmão `_appropriate` 123<600.

**Smokes (todos `--release`):**
- `PH2D_AUTOFIX_SMOKE=1..7` — a auto-cura/reorder/badge (1/2/3), a família falloff (4), o
  requisito-a-montante (5), a porta obrigatória (6), **a forma apropriada / fonte-com-estado (7)**.
- `PH2D_SHAPE_SMOKE=1` (Star→Gear vivo) · `=2` (a recusa de GPU: 16 estrelas giradas, sem
  retângulos brancos).
- `PH2D_MOTION_OBJ_SMOKE` — o item 3 do doc 86 (ver sub-handoff).

## Aberto (não bloqueia a integração)

- **ADR-0155 badges advisory já pintados, o clique explica** (waves futuras, plano 87 §fim):
  **1c** (pin→solver), **3a** (falloff a montante composto), **3b** (dupla integração),
  **1b indireto** (nó não-integrador entre integrador e força).
- **Aceleração de GPU para `source.object`** — a lowering sprite-only PODERIA ler
  `texture_id` e evitar a recusa CPU só para objetos (o `source.shape` é vetorial e não
  tem rota de GPU). Otimização NOMEADA, não construída.
- Os itens abertos dos fronts A/B estão nos seus sub-handoffs.
