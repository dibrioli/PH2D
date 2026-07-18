# ADR-0130 — O emitter na GPU: o gather por `id` é ARITMÉTICO porque a janela é densa (e uma propriedade PROVÁVEL de plano é o que separa isso de um bug mudo)

- **Status:** PROPOSTA (a fatia que o [ADR-0127 D3](0127-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md) adiou — *"o pareamento por `id` é um gather → fora do 1º corte"*). Resolve o D3; **não** o amenda. Sucede a Fase 3 (`line/gpu-nodes`, integrada 2026-07-16).
- **Não toca o contrato congelado** ([ADR-0126](0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)): `NodeManifest`/`NodeOp`/`OpResolver` intactos. O que cresce (`SourceCountFn`, `GpuKernel`, o `output_shape` do plano) é **metadado lateral** — foundational, mas append-only por design (§0.2).
- **Método:** wave de pesquisa (2 agentes de prior-art + 1 adversarial sobre o nosso código) + aterramento próprio. O fato decisivo foi verificado **antes** da fanout ([[feedback_a_research_fanout_recurses_bound_it]]): sem isso os agentes teriam pesquisado free-list + prefix-sum, o problema errado.

---

## Contexto

Hoje o `motion.emitter` **força a simulação inteira pra CPU**: o `motion.integrate` (e o `motion.spring`) carregam um binding `id` **`RefuseIfPresent`** na porta 0 — um stream com `id` recua. O ADR-0127 D3 deixou isso pra "a fatia seguinte", imaginando o gather como *"hash/sort na GPU"* — o padrão free-list do Niagara/Unity VFX Graph.

**O fato decisivo, verificado no código, colapsa esse problema.** O emitter é **stateless** (`crates/ph2d-node-motion-emitter/src/lib.rs`, fn `emit`): no playhead `t` o conjunto vivo é uma **janela de ids CONTÍGUA e ASCENDENTE** `[first, first+n)`, onde `n(t)` e `first(t)` são aritmética fechada de `(rate, life, t)` — a partícula `k` nasce em `k/rate` e vive enquanto `0 ≤ t − k/rate < life`. **Não há free-list, não há morte-no-meio, não há compactação.** A velocidade de lançamento é `hash(seed, id)`, determinística.

Logo o gather **não é** hash/sort. Parear o elemento de id `first+k` à sua linha no `prev` é **`prev_row = id − prev_first`** (com `prev_first = prev_id[0]`) — um deslocamento de índice, `O(1)` por elemento, sem mapa.

A pesquisa de prior-art confirmou o desenho e nomeou os invariantes:

- **É a forma branch-free de um alocador ring-buffer de vida-fixa** (o heap do Latta, *Building a Million-Particle System*, GPU Gems 2 — "packed in the first portion"). **Novo-mas-sólido**: ninguém o usa como feature nomeada porque os outros engines precisam tolerar **morte fora de ordem** (colisão, vida variável, kill na GPU) → free-list / ID→index table. O nosso invariante "sem morte-no-meio" é o que torna a aritmética válida.
- **O híbrido stateless-emitter + stateful-integrator é o desenho certo** (Unity VFX Graph, Niagara e Hanabi separam init de update). Stateless **puro** não reage a forças (Latta/Drone SIGGRAPH'07) — por isso o integrador continua stateful.
- Válido **iff**: vida uniforme · cursores (`first`) monotônicos · sem morte-no-meio · recém-nascido vai por **init (seed)**, não pelo gather deslocado.

O agente adversarial confirmou os quatro invariantes contra o `emit` real (cap mantém os mais novos → contíguo; bordas seguras; `first` não-decrescente sob params constantes; scrub reproduz porque `first` é `pure(t)`) — **e achou onde o desenho quebra.** É disso que trata este ADR.

---

## Decisões

### D1 — O gather é aritmético, e a CPU continua o oráculo

O kernel do `integrate`/`spring` lê `prev_first = read_forces_id(0)`, computa `prev_row = id − prev_first` (**unsigned**), e faz bounds-check `prev_row < prev_n`: dentro → lê o estado dessa linha; fora → **recém-nascido, semeia**. Isto **iguala** o `pairing` da CPU (`integrate/src/lib.rs:353` — um `BTreeMap<id,row>` que casa id-X com a linha de id-X) **exatamente**, porque numa janela densa a linha de id-X *é* `X − prev_first`. A paridade não é aproximada: é a mesma função, uma via mapa e outra via aritmética.

**A CPU permanece canônica** (ADR-0126). O ganho é performance/preview; o gate de paridade ε é o audit.

### D2 — A janela densa é uma propriedade PROVÁVEL de plano, DEFAULT-LIMPA — nunca uma allowlist dos nós que a quebram

**Este é o miolo do ADR, e o que separa a aritmética de um bug mudo.** A contiguidade é propriedade do **emitter NU**, não do stream. O adversarial enumerou os nós que a destroem, todos com kernel na fila: **`sort`** (reordena todas as colunas, `id` incluso — `id[0]` deixa de ser o mínimo), **`cull`** (filtra → buracos), **`combine`** (concatena), **`clone`/`mirror`/`kaleidoscope`** (duplicam `id`), **`trail`** (duplica `id`). Atrás de qualquer um deles, `id − prev_first` **subtrai errado** — em `sort`, um id menor que `prev_first` faz **underflow u32 → tratado como recém-nascido → velocidade zerada em silêncio** (pop visível); em `cull`, um id além do buraco vira recém-nascido **todo tick → congela na velocidade de bico**. A CPU (BTreeMap) acerta todos; a aritmética não.

Hoje esses nós são **fronteira de CPU** (nenhum tem kernel), então a recusa das duas-simulações (`plan.rs`, `sim_state_on_gpu && !boundaries.is_empty()`) resgata **por coincidência**. Tirar o `RefuseIfPresent` tira o único guarda, e o `output_shape` rastreia só **presença de coluna** — não distingue um `id` de emitter nu de um `id` sorteado.

**A recusa cega vira condicional, e a condição é uma propriedade positiva que só o emitter cria:**

- O `plan::output_shape` carrega um bit **`dense_window`** ao lado do conjunto de colunas.
- O **emitter** o SETA (é a fonte da janela).
- Um estágio o **PRESERVA** só se o kernel **declarar** que é um mapa **por-elemento que não reordena, não filtra, não duplica e não reescreve `id`** (`preserves_identity`, default **false**). `integrate`, `spring` e as 6 forças declaram `true` (são endomorfismos de identidade). **Todo o resto default-limpa.**
- O gather é reivindicado **só** quando `dense_window` vale na entrada do `integrate`; senão o plano **recua** (como hoje, mas *provado*, não por sorte).

**Por que default-limpo e não uma lista dos que quebram:** uma allowlist dos nós estruturais **apodrece** — o próximo nó que reordena, adicionado daqui a seis meses sem entrar na lista, mispareia em silêncio ([[feedback_a_condition_that_enumerates_its_readers_rots]], [[feedback_convention_vs_inertia]] — default = mais isolamento). Com default-false, um nó novo é seguro por construção: ele só preserva a janela se **afirmar** que a preserva.

### D3 — O count depende do playhead; `SourceCountFn` cresce o clock; o dispatch é host-sized (não indirect)

`SourceCountFn = fn(&dyn Fn(&str)->f32) -> usize` recebe params mas **não** o playhead (`gpu.rs:145`), e `n(t)` do emitter é função de `t`. `plan::eligible` recusa gerador sem `source_count` → **o emitter não é GPU-elegível até o tipo crescer**: `fn(&dyn Fn(&str)->f32, f64) -> usize`, com `clock.playhead` costurado no call-site (o playhead **já** está lá, `lib.rs:277`) e o único implementador atual (`motion.grid`) ignorando o 2º arg.

**Dispatch continua host-sized** — não indirect. A pesquisa foi categórica: `dispatchWorkgroupsIndirect` existe **só** pra evitar o readback GPU→CPU quando o count vive na GPU (append/consume + contador atômico). O nosso count é fechado e `O(1)` na CPU — indirect não compra nada. **A única exposição é a cauda do buffer** (grow-never-shrink deixa dados mortos além de `n`): todo passe tem de ler `n` vivo, não a capacidade. O passe de lowering já faz isso (`instances.rs:49`, `uni[0..4]=stream.count`); os kernels idem.

`SourceCountFn`/`GpuKernel` são metadado lateral (ADR-0126), não o `NodeManifest` congelado — **sem ADR de contrato**. Mas é tipo foundational tocado por todo kernel de gerador: append-only, projetado pra isolamento.

### D4 — O modelo de binding do codegen: estado no PRÓPRIO comprimento, accessor por `prev_row`, bounds-check por-elemento distinto do global

O blocker estrutural real (não o D3 mecânico): a regra de presença `s.count == count` (`lib.rs:434`) exige que a coluna de estado tenha o **mesmo** comprimento do dispatch. Num conjunto que renasce, `prev_n ≠ n` quase todo tick → as colunas de estado viram `ReadIdentity` → `HAS_forces_sim_d=false` → **todo elemento pega o seed, a sim nunca acumula**. (É por isso que o pareamento posicional não faz partículas.)

O gather reescreve isso:
- As colunas de estado (porta `forces`) são ligadas no **próprio comprimento `prev_n`**, desacopladas do dispatch `n`.
- O accessor gerado `read_forces_vel(i)` — que hoje hard-indexa `i` (`codegen.rs:231`) — passa a indexar **`prev_row`**, sob bounds-check.
- O bounds-check por-elemento (`prev_row < prev_n`) é **distinto do global `HAS_forces_sim_d`**: são duas perguntas — *"existe algum estado anterior?"* (o global, `pairing().is_some()`) vs *"ESTE elemento tem uma linha?"* (o por-elemento, o `Some(j)` da CPU em `integrate/src/lib.rs:304`). Colar as duas numa só é [[feedback_layered_defenses_need_per_layer_gates]] esperando pra acontecer — cada uma precisa do seu gate.

### D5 — Determinismo e scrub: inalterados, com duas dependências a honrar

O scrub do D5 (ADR-0127) não ganha descontinuidade: `first` é `pure(t)`, então o emitter re-simulado reproduz **exatamente** os ids da marcha original, e o `prev` restaurado é um passado genuíno cujo `id[0]` é o `first` daquele tick. O gather no restore é idêntico ao gather pra frente. Duas dependências:

1. **O `id` tem de viajar no estado em checkpoint** (ele já corre por `rest→out→pre→forces`) — e o kernel precisa de um `id` **Read na porta 1** pra ler `prev_first` (hoje só existe o `RefuseIfPresent` na porta 0).
2. **O bounds-check é unsigned** — o underflow `id < prev_first` (possível só fora do caminho monotônico) cai como "recém-nascido", não como leitura OOB. O anchor de seed (target mais velho que o ring) entrega estado vazio ⇒ tudo recém-nascido = o tick 0.

### D6 — `age` é re-derivado (analítico), nunca acumulado — mas `sim.step` acumula, então os dois integradores não se misturam

O emitter carimba `age = t − id/rate` fresco a cada tick; o `integrate::step` copia as não-sim ao vivo do `rest` (exclui só `accel|vel|sim_d|sim_t`), então `age` vem do emitter e o `age` velho do estado é ignorado — **sem double-count**, na CPU e na GPU. A pesquisa confirma que derivar `age` analiticamente é o certo pro modelo stateless (vs `sim.step`, o **outro** integrador, que **acumula** `age_prev + dt`). Corolário a documentar: *"age é stateless"* **não** é invariante do módulo — alimentar o `age` derivado do emitter num `sim.step` double-conta; os dois não se misturam num stream (o `sim.spawn` já cerca o emitter das zonas).

### D7 — Mudança de param em voo re-numera os ids → invalidar o estado; é o modelo, não a GPU

Arrastar `rate`/`life`/`max` no meio da sim muda o mapa `id ↔ partícula` (o `k/rate` se move), então `prev_first` (params velhos) e `current_first` (params novos) discordam e o gather mispareia. **Isto é igual na CPU** (o BTreeMap re-numera junto) — é propriedade do modelo, não da GPU. O honesto é **invalidar o estado da sim** (`forget_state`, `lib.rs:362`) na edição de param do emitter, senão os dois caminhos glitcham igual. **Ação:** verificar/costurar o gatilho pra *param change* (hoje ele responde a *graph edit*).

---

## Consequências

- **+** O regime mais interessante — partículas com nascimento e morte — roda 100% na GPU, e o gather é mais barato que qualquer free-list/sort/prefix-sum.
- **+** A recusa deixa de ser sorte (a coincidência das duas-simulações) e vira uma propriedade **provada** e default-segura.
- **−/limite** Só o grafo `emitter → [forças] → integrate/spring` é reivindicado; `sort`/`cull`/`combine`/`clone`/`mirror`/`trail` no meio recuam (correto — a janela deixa de ser densa). Portá-los pra GPU **não** basta; eles teriam de resolver o gather geral (hash/sort), que é outra fatia e outro ADR.
- **⚪ teto conhecido:** o `id` é `f32` (`emitter/src/lib.rs:193`) → acima de 2²⁴ (~16,7M ids ≈ 4,8 dias a rate 40) ids consecutivos não são distintos e a aritmética perde precisão. **Compartilhado com a CPU** (o BTreeMap keya no mesmo `f32`) — não é divergência GPU-vs-CPU, é um teto comum. Fora de escopo; anotado.

---

## Fatias (nesta ordem)

1. **`SourceCountFn` + playhead** (mecânico): cresce o tipo, costura `clock.playhead`, `grid` ignora. O emitter vira gerador GPU-elegível.
2. **A propriedade `dense_window`** no `output_shape` (default-false, setada pelo emitter, preservada só por quem declara `preserves_identity`) + a **recusa condicional** que substitui o `RefuseIfPresent`.
3. **O modelo de binding** (D4): estado no `prev_n`, accessor por `prev_row`, bounds-check unsigned por-elemento + o `id` Read na porta 1. Mesmo maquinário no `integrate` e no `spring`.
4. **Os gates** (o audit).
5. `forget_state` na edição de param do emitter (D7).

## Gates (o audit — é ele que vale, não o verde-de-compilação)

- **Paridade `emitter → integrate` UM passo == CPU**, com a **janela DESLIZANDO** no tick (nascimentos **e** mortes) — senão o gather nunca é exercitado (um count estático parearia posicionalmente e o gate ficaria verde com a aritmética morta; [[feedback_test_with_product_numbers_not_convenient_ones]]).
- **A recusa:** `emitter → sort → integrate` (e `→ cull →`) **recua** (plano não-fully-GPU), **com o irmão de presença** `emitter → integrate` reivindicado ([[feedback_absence_gate_needs_a_presence_sibling]]). **Mutação:** tirar o clear do `dense_window` num nó estrutural → a recusa fica verde **com o mispair de volta** (é exatamente por isso que o gate existe).
- **Scrub** de partículas reproduz o passado (D5) com a janela deslizante — não a marcha do futuro.
- **Os dois bounds-checks** (global vs por-elemento) têm gate cada (D4): mutar o por-elemento pro global tem de sangrar num fixture onde `prev_n < n` (recém-nascidos no fim da janela).
