# ADR-0135 — A família `sim.zone` na GPU: o contêiner de laço de estado é um passthrough CONDICIONAL, e um claim parcial RECUA

- **Status:** PROPOSTA. Continua a linha `line/gpu-nodes` (ADR-0126 contrato · 0127 laço de sim · 0130 emitter · 0140 grade de vizinhança). **Número provisório**, escolhido nesta linha; o integrador reconcilia se colidir ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **NÃO toca o contrato congelado** (ADR-0126): `NodeOp`/`NodeManifest`/`OpResolver` intactos. Tudo aqui é **metadado lateral** append-only do sequenciador (`StateSelect` + `KernelResolver::state_select`, ao lado de `GridSpec`/`grid`) e dois kernels de nó.
- **Método:** o **censo medido** (`shells/desktop/src/motion_gpu_coverage.rs`) apontou o gargalo ANTES de qualquer desenho ([[feedback_a_frontier_is_not_a_census]]).

---

## Contexto

O único documento de ARTISTA que a engine abre — a **neve** (`sim.zone`, doc 52: neve caindo no mar raso) — cozinha quase toda na CPU. O censo mede: **`HYBRID`, fronteira em `sim.zone`, 2 stages despacham** (só a cadeia de render), 16 tipos de nó do interior no prefixo CPU. A família `sim.*` (`sim.zone`/`sim.step`/`sim.collide`) **não tinha handling nenhum na GPU**.

Dois fatos do fonte enquadram o desenho:

1. **`sim.zone` não é caso especial em lugar nenhum.** É um nó `Temporal` comum cujo "laço" é 100% o mecanismo genérico de aresta `pre` (`Cook::prev_outputs` + `advance_tick`) mais a marcha de ticks do pump. O eval é `if ctx.started() { input(state) } else { input(init) }`, com os transientes (`accel`/`falloff`) removidos. A GPU **já tem** o mecanismo `pre` (`GpuSource::Prev` + `GpuCook::prev`, ADR-0127 D1).
2. **`sim.step`/`sim.collide` não têm `pre` próprio** — o relógio vem da zona. `sim.step` deriva `dt` de uma COLUNA por-elemento `sim_t` que ele mesmo carimba e a zona carrega de volta; fora de uma zona `sim_t` nunca volta ⇒ `dt ≡ 0` ⇒ o integrador é no-op. `sim.collide` reflete `vel`, que só existe dentro do laço.

Ou seja: os pré-requisitos (estado que sobrevive ao tick, kernels por-elemento) **já existem**. Falta a zona ser reconhecida, e os dois kernels serem escritos.

---

## Decisões

### D1 — A zona é um PASSTHROUGH CONDICIONAL (metadado lateral `StateSelect`)

`sim.zone` **não é um kernel por-elemento** — ele SELECIONA um de dois streams de entrada de contagens diferentes, o que é uma operação de **nível de stream no host**, não uma computação por-elemento. Então: a zona registra `GpuKernel::PASSTHROUGH` (o plano a reivindica, nenhum passe é emitido) **mais** um `StateSelect` lateral — `{init_port, state_port, transients}` — registrado como o `GridSpec` (`register_state_select`/`KernelResolver::state_select`, default `None`). No cook, a zona emite `started ? input(state) : input(init)`, tirando os transientes.

- **"Started" é `GpuCook::prev.contains(zone)`** — a zona alimenta uma aresta `pre`, então depois do tick 0 ela sempre tem `prev`. É o espelho exato do `prev_outputs.contains_key` da CPU.
- **A lista de transientes é UMA** — a MESMA `TRANSIENTS` que o `store()` da CPU tira (`&["accel","falloff"]`, referenciada por `&TRANSIENTS`), então as duas metades não podem divergir.
- **Por que não um campo no `GpuKernel`:** ele é construído por literal em ~34 crates-nó; apendar um campo custa O(sítios) e recorre ([[feedback_widely_constructed_type_favors_optional_component_over_appended_field]]). O metadado lateral custa zero para quem não o declara.

### D2 — `sim.step`/`sim.collide` são kernels por-elemento comuns, DESTRAVADOS pela zona

São transcrições de porta única (Fase 2), o `motion.integrate`/`force.buoyancy` como gabarito. `sim.step` lê a coluna-relógio `sim_t` **por elemento** (`HAS_sim_t` ausente ⇒ `dt = 0`, um elemento novo COMEÇA em vez de saltar); a guarda de finitude mantém o valor VELHO (não zero). `sim.collide` reflete `vel` contra a forma estática (Floor/Disc/Bowl), com os clamps de param do kernel (um clamp só na CPU é uma divergência esperando um slider na borda).

### D3 — Um claim PARCIAL do laço RECUA; não refuta o plano inteiro

Uma zona cujo interior tem um nó **sem kernel** não pode ser reivindicada inteira (a regra `sim_state_on_gpu` do ADR-0127: uma fronteira dentro do laço faria o pump re-cozinhar a sim com o `prev` DELE — duas simulações de um estado). A neve tem exatamente isso: a família que MUDA CONTAGEM (`sim.spawn`/`lifetime`/`cull`/`combine`) não tem kernel.

⚠️ **A regra antiga refutava o plano INTEIRO** (`boundaries = [(sink,0)]`, 0 stages) — e isso, com a zona agora elegível, **REGREDIRIA a neve de `HYBRID`(2 stages de render) para `CPU`(0)**, jogando fora a cadeia de render que sempre esteve válida na GPU, à toa. A cadeia de render a JUSANTE da zona é trabalho por-elemento comum e fica na GPU.

Então o plano **RECUA**: proíbe os `pre`-sources reivindicados e RE-PLANEJA (`plan_forbidding` com um conjunto `forbidden`), fazendo a zona virar uma FRONTEIRA — exatamente o que ela era antes de ter kernel. O laço de sim recua ao pump; o sufixo de render fica na GPU. Um laço **totalmente** coberto (o demo `=10`) é reivindicado inteiro. Medido pelo censo: a neve fica **idêntica** (`HYBRID`, 2 stages, fronteira `sim.zone`) — a única mudança é o rótulo (`[no-kernel]` → `[refused-despite-kernel]`).

### D4 — CPU canônica; paridade ε

O replay-hash nunca roda na GPU (ADR-0126). O seed do tick 0 é bit-exato; os passos diferem só por FMA. **Medido na RTX**, 40 ticks, 1600 elementos: floor **1,7e-6** · disc **5,7e-6** · bowl **2,1e-6** · sea+bed (a física exata do demo) — todos sob o `EPS_POS` de 2e-3. Cada colisor é comparado contra uma linha de **queda livre** (um colisor que nunca contata), senão um ramo morto na fixture passaria vacuamente.

---

## Consequências

- **A neve NÃO fica GPU-residente ainda.** A família que muda contagem (`spawn`/`lifetime`/`cull`/`combine`) + o text-param `value.attribute` seguem no pump — a **próxima fatia**, um território maior (a classe que muda contagem, adiada repetidamente na linha). O que ESTA fatia entrega é a CAPACIDADE (`sim.zone`/`sim.step`/`sim.collide` na GPU) + um demo `=10` (uma neve de população fixa, 100% na GPU) que a prova.
- **Superfície append-only:** `StateSelect` + `KernelResolver::state_select` + `plan_forbidding`. Nenhum kernel existente muda; um nó sem `StateSelect` é um passthrough/kernel comum.
- **Número de ADR provisório** (como 0130→0131 na física).

## Alternativas rejeitadas

- **Refutar o plano inteiro num claim parcial** (o comportamento antigo). Regride o sufixo de render da neve à CPU à toa. O recuo preserva o híbrido de hoje E habilita o full-GPU quando o laço inteiro é coberto.
- **A zona como um `NodeOp` de "select" genérico.** Ela é o único nó dessa forma; um metadado lateral responde a única pergunta que o cook tem, sem inflar o contrato.
- **Expressar a zona como um kernel WGSL.** Ela seleciona entre streams de CONTAGENS diferentes (init vs state) — uma operação de host, não um mapa por-elemento. Um kernel opera sobre uma contagem fixa.
- **Portar `sim.step`/`sim.collide` ANTES da zona.** Foi tentado e adiado na linha: fora de uma zona `dt ≡ 0`, então os kernels nunca rodariam — motor sem consumidor.
