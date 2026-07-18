# HANDOFF (briefing de continuação) — `line/gpu-nodes` · ADR-0130 · o emitter na GPU (o gather por `id`)

> **Você é o agente que continua esta linha em contexto fresco.** O ADR já está escrito e as fatias 1-2
> landaram e estão gateadas. **As fatias 3 (o gather) + 4 (os gates) LANDARAM em `76ae0d52` (2026-07-17)
> e estão gateadas na RTX** — ver §1.LANDOU e §3.LANDOU. Falta só a **fatia 5** (`forget_state` na edição
> de param do emitter, D7) — **shell-side e gated por ordem EXPLÍCITA do Enio** (§4 + §6).
> Leia este doc + o [ADR-0130](architecture/decisions/0130-gpu-emitter-the-id-gather-is-arithmetic-because-the-window-is-dense.md) inteiro (é curto). O ADR tem o PORQUÊ; este doc tem o ONDE e o COMO, e os gotchas que a wave de pesquisa expôs.

---

## §0 — Inegociáveis (memorize antes de tocar em nada)

1. **Trabalhe SÓ em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes`. SEMPRE prefixe todo comando com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes &&`.** A cwd escorrega pro repo primário (aconteceu 4× em jornadas anteriores — um `git` no primário deixa o shell lá). Se um `git log` mostrar `main`/`cdc3acc1`, você escorregou.
2. **NÃO integre, NÃO pushe, NÃO rode `ship.sh`.** Feche o trabalho, atualize este handoff, e PARE. Integração/ship só por ordem EXPLÍCITA do Enio, via agente integrador dedicado (§0.7 do CLAUDE.md). Esta linha já foi integrada uma vez e re-preparada; ela **acumula** por cima do `main` integrado (fork em `cdc3acc1`).
3. **Contrato congelado 8/2/1 intocado** (`NodeManifest`/`NodeOp`/`OpResolver`, [ADR-0126](architecture/decisions/0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)). Tudo que você mexe é **metadado lateral**: `GpuKernel`, `SourceCountFn`, `KernelResolver`, o `output_shape` do plano, os kernels. Se sentir vontade de bumpar o `NodeManifest`: PARE — a resposta é sempre o canal lateral.
4. **O gate É o audit.** Verde-de-compilação vale ZERO. Todo kernel novo tem paridade ε contra a CPU (canônica, ADR-0126) + mutação (mate o código, exija vermelho, restaure com `cp` NUNCA `git checkout`). `git commit --no-verify`; crase em msg de commit = execução → use `git commit -F <arquivo>`; um pipe mascara o exit code; `docs/**/*.md` é excluído do typos (rode `typos` sem argumento).
5. **Inner loop = `cargo check -p <crate>`.** Gates 1× no fechamento. Meça em `--release` na RTX (os gates de GPU são `#[ignore]` — precisam de `-- --ignored`).
6. **LOC cap: SPLIT, nunca allowlist.** O de workspace (`crates/*/src`, 700) NÃO roda com `cargo test -p` (mora na `ph2d-editor-core`); o do shell (600) roda. Cheque os dois no fechamento.

---

## §1 — Onde estamos (o que landou e está gateado)

Fork de `main` em `cdc3acc1`. Commits desta fatia (do ADR pra cima):

| Commit | O quê | Gate (verde na RTX) |
|---|---|---|
| `ff1cc9d1` | **ADR-0130** — o desenho (leia inteiro) | — |
| `33b0a8c8` | **Fatia 1:** `SourceCountFn` cresce o playhead (mecânico) | zero mudança de comportamento |
| `d29366fc` | **Kernel do `motion.emitter`** — a lei da contagem | `the_emitter_generator_matches_the_cpu`: janela `[15,81,121,121]` + cap 256; 2 mutações |
| `90e70302` | **Fatia 2:** `dense_window`, a propriedade provável de plano | 5 gates de plano puros; mutação mata 3, deixa o positivo verde |
| `76ae0d52` | **Fatia 3+4:** o gather aritmético + os gates (§3.LANDOU) | 3 gates de sim de emitter na RTX + 2 de plano; 4 mutações RED |

### §1.LANDOU — fatia 3+4 (`76ae0d52`, 2026-07-17)

- **`ColumnAccess::GatherKey`** (nodegraph, `gpu.rs`): a recusa de `id` virou **condicional** — reivindica uma janela densa, recua uma `id` não-densa/improvável. É **legível** (o id corrente alimenta o `gather_row`); o id passa pelo base. `RefuseIfPresent` FICA (fixtures `test.refuser`).
- **`plan::eligible`** ganhou o laço do `GatherKey`: `output_shape` provável + densa → reivindica; presente-mas-não-densa **ou** improvável → recua (nunca um mispair posicional mudo).
- **`codegen`**: gera `gather_row(i)`/`gather_paired(i)` + o uniform `gather_prev_n`. Positional (grid) reduz a `gather_row(i)=i`, então integrate/spring compilam **UM corpo** pros dois modos. **Casts de VALOR** (`u32(max(id,0.0))`, = o CPU `f.max(0.0) as u32`) — os ids são armazenados por valor (`f32(em_id)`), NUNCA `bitcast` (o §3c do handoff dizia bitcast; era um deslize — corrigido). `prev_first = read_<stateport>_id(0u)`, raw.
- **`lib.rs`** (encode): as portas de ESTADO (porta ≠ base) são **desacopladas do dispatch** sob um gather ativo (`prev_n ≠ n` é normal — pareado por id, não posição); o resto segue `count == dispatch`. `gather_prev_n` empacotado APÓS os params (casa o struct gerado).
- **integrate/spring**: `id` GatherKey na porta base + `id` Read na porta de estado (pro `prev_first`); o corpo lê o estado em `gather_row(i)`, guardado por `gather_paired(i)` — a **semente por-elemento** (recém-nascido não pareia) DISTINTA do global `HAS_` ("existe QUALQUER estado?").

**Números medidos (RTX, `--release`):** `emitter → integrate` (balístico, muzzle) e `emitter → drag → integrate` casam a CPU com **max |Δpos| ~6e-8** ao longo de uma janela CAPADA que desliza (rate 400, max 40, 20 nascimentos/tick; cap prende no tick 2). Scrub reproduz a janela do passado. Os 22 gates de sim pré-existentes (grid → integrate posicional) intactos + a validação WGSL exaustiva (todo módulo do gather parseia sob naga).

**Rodar tudo verde hoje** (do worktree):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --release -- --include-ignored   # 16 lib + 2 WGSL + 13 paridade + 14 sim + 5+12 plano
```

### O que o emitter JÁ faz e o que FALTA

- **JÁ (fatia 1-4):** `emitter → [forças] → integrate/spring → output` cozinha **100% na GPU** e casa a CPU: a **contagem** (`n(t)`, cap, bordas) E agora o **gather por id** (`vel`/`id`/`age` viajam e pareiam por `current_id − prev_first`). Kernel do emitter em [`crates/ph2d-node-motion-emitter/src/lib.rs`](../crates/ph2d-node-motion-emitter/src/lib.rs); o gather em `motion.integrate`/`motion.spring` + `ph2d-gpu-cook` (`plan.rs`/`codegen.rs`/`lib.rs`) + `ColumnAccess::GatherKey` em `ph2d-nodegraph::gpu`.
- **FALTA (fatia 5, D7):** `forget_state` na edição de **param** do emitter — arrastar `rate`/`life`/`max` re-numera os ids e o gather mispareia (**igual na CPU** — é o modelo). **Shell-side** (`render_loop/motion_bridge_gpu.rs`) e **gated por ordem EXPLÍCITA do Enio** (§4 + §6). Não-bloqueante pro gather básico.

---

## §2 — O fato decisivo (não re-pesquise; a wave já rodou)

O `motion.emitter` é **stateless**: o conjunto vivo no playhead `t` é uma **janela de ids CONTÍGUA e ascendente** `[first, first+n)` — `n(t)`/`first(t)` são aritmética fechada de `(rate, life, t)` (`emit`, em [`motion-emitter/src/lib.rs`](../crates/ph2d-node-motion-emitter/src/lib.rs) fn `emit`). **Sem free-list, sem morte-no-meio, sem compactação.**

Logo o gather **não é hash/sort** (o padrão Niagara que o ADR-0127 D3 imaginou). É **aritmética**: parear o elemento de id `first+k` à sua linha no `prev` é

```
prev_row = current_id − prev_first            (prev_first = read_forces_id(0), UNSIGNED)
se prev_row ∈ [0, prev_n)  →  lê o estado dessa linha
senão                      →  RECÉM-NASCIDO, semeia (o caminho seed)
```

Isto **iguala** o `pairing` da CPU (`integrate/src/lib.rs`, um `BTreeMap<id,row>`) **exatamente**, porque numa janela densa a linha de id-X *é* `X − prev_first`. Prior-art confirmou: é a forma branch-free de um **alocador ring-buffer de vida-fixa** (heap do Latta), **novo-mas-sólido**. A CPU permanece canônica (ADR-0126); o gate ε é o audit.

---

## §3 — FATIA 3: o gather (D4 do ADR-0130) — a parte arriscada

> **§3.LANDOU (`76ae0d52`, 2026-07-17).** Feito exatamente como abaixo, com uma escolha de projeto:
> em vez de gerar accessors gather-indexados, o **corpo do kernel calcula a linha** (`gather_row(i)`)
> e passa aos accessors RAW (`read_forces_vel(ig_row)`) — mais simples e é o que o §3c já sugeria. Os
> gates (§3d) landaram junto (fatia 4). **4 mutações provadas RED** (§3d.MUTAÇÕES). Não é preciso
> re-implementar; abaixo é o registro do desenho. Os deformers Fase 2 (§3e) **não** foram registrados
> como keepers (segue seguro/recuando — não-bloqueante).
>
> **§3d.MUTAÇÕES (todas restauradas por `cp`, nunca `git checkout`):**
> 1. `gather_row → i` (posicional): os 3 gates de sim de emitter sangram (crash de bind-group — o
>    `read_rest_id` vira código morto e a layout descasa). 2. `gather_row … + 1u` (off-by-one): o
>    **oráculo de paridade** sangra limpo (`cpu 0.47 vs gpu 0.5` — o sobrevivente 0 herda a linha do
>    vizinho). 3. `gather_paired → true` (colapsa o por-elemento no global, D4): sangra onde `prev_n < n`
>    (recém-nascidos lendo estado OOB, `cpu 0.62 vs gpu 0.5`). 4. `plan` sem o `!dense`: `a_reorder…recede`
>    E `a_derivable…id…refuses` (idgen não-densa) ficam RED, enquanto `the_emitters_dense_window_claims`
>    segue VERDE — o par presença/ausência.

**Objetivo:** `emitter → integrate → output` cozinha na GPU e casa com a CPU **com a janela deslizando** (nascimentos E mortes por tick). Não pode quebrar os 22 gates existentes (`grid → integrate` pareia posicional).

### 3a — Flipar a recusa (condicional em `dense_window`)

Hoje `motion.integrate` e `motion.spring` têm um binding `id` **`RefuseIfPresent`** na porta 0 → recusam TODO stream com `id`. A recusa mora em `plan.rs::eligible`, o laço `for b in kernel.bindings.iter().filter(|b| b.access.refuses())` (perto da linha 192). Ele deriva `output_shape(input)`; se `id` presente (ou desconhecido) → recusa.

**Mude para:** recusar `id` presente **exceto** quando `output_dense_window(input) == Some(true)` (a propriedade da fatia 2, já em `plan.rs`). Assim `emitter → integrate` é reivindicado (janela densa), mas `emitter → sort → integrate` (sort quebra a densidade) recua. **Não** remova o `RefuseIfPresent` do binding — ele é o gancho; só a condição no `eligible` muda. Pense se a condição deve ser um novo `ColumnAccess` (ex.: `GatherIfDense`) ou um teste no `eligible` que consulta `output_dense_window` quando vê `RefuseIfPresent id` — a 2ª é menos invasiva.

⚠️ **Só flipe a recusa JUNTO com 3b/3c (o gather correto).** Flipar sozinho cria um caminho reivindicado-MAS-ERRADO (integrate pareia POSICIONAL numa janela que desliza → cada sobrevivente herda a velocidade de um estranho). A fatia 2 foi mantida "só prova a propriedade" exatamente por isso.

### 3b — O modelo de binding do codegen (o blocker estrutural real)

Dois lugares, e este é o coração da fatia:

- **`crates/ph2d-gpu-cook/src/lib.rs:437`** — `encode_kernel_stage` tem a regra de presença `s.count == count` (o dispatch count `n`). Para um conjunto que renasce, o estado (porta `forces`) tem `prev_n ≠ n` quase todo tick → as colunas de estado viram `ReadIdentity` → `HAS_forces_sim_d = false` → **todo elemento pega o seed, a sim nunca acumula**. As colunas de estado (porta 1) têm de ser ligadas no **próprio comprimento `prev_n`**, desacopladas do dispatch `n`.
- **`crates/ph2d-gpu-cook/src/codegen.rs:233`** — `fn read_{c}(i: u32) -> {ty} {{ return in_{c}[i]; }}`. O accessor hard-indexa `i`. Para as colunas da porta 1 (state) num nó com gather, ele tem de indexar **`prev_row`** (calculado no corpo), sob bounds-check.

### 3c — O corpo do kernel (integrate e spring)

- Adicione um `id` **Read na porta 1** (`read_forces_id`) — hoje só existe o `RefuseIfPresent` na porta 0. `prev_first = read_forces_id(0u)`.
- `prev_row = bitcast<u32>(...)` de `read_rest_id(i)` menos `prev_first`, **unsigned** — o underflow `id < prev_first` (possível só fora do caminho monotônico) cai como recém-nascido, não como leitura OOB.
- **O bounds-check por-elemento (`prev_row < prev_n`) é DISTINTO do global `HAS_forces_sim_d`** — são duas perguntas: *"existe algum estado anterior?"* (o global, `pairing().is_some()`) vs *"ESTE elemento tem uma linha?"* (o por-elemento, o `Some(j)` da CPU em `integrate/src/lib.rs:304`). Colar as duas é [[feedback_layered_defenses_need_per_layer_gates]] — cada uma precisa do seu gate.
- Onde hoje o corpo lê `read_forces_vel(i)`/`read_forces_sim_d(i)`/`read_forces_sim_t(i)` (o passo), passe a ler em `prev_row`; recém-nascido (fora do range) → o caminho seed (o `else` que já existe). O `spring` é o mesmo maquinário.

### 3d — Gates (o audit da fatia 3)

- **`one_step_of_the_emitter_sim_matches_the_cpu`** (o principal): `emitter → integrate → output`, **janela DESLIZANDO** — use um emitter CAPADO (`rate·life ≫ max`), porque aí `first` avança TODO tick e o gather é exercitado já no tick 2; um fixture estático parearia posicional e o gate ficaria verde com o gather morto ([[feedback_test_with_product_numbers_not_convenient_ones]]). Compare `vel`→posição integrada contra a CPU. Rode 2-3 ticks. Copie a forma de `cpu_ticks`/`gpu_ticks` em `crates/ph2d-gpu-cook/tests/gpu_cpu_parity_sim.rs`.
- **A recusa condicional:** `emitter → integrate` agora é `is_fully_gpu()` (dense) **com o irmão** `emitter → test.reorder → integrate` recuando ([[feedback_absence_gate_needs_a_presence_sibling]]). Os testes de plano da fatia 2 (`plan_simulation.rs`) já têm o `test.refuser` — estenda.
- **Os dois bounds-checks (D4):** mutar o por-elemento pro global tem de sangrar num fixture onde `prev_n < n` (recém-nascidos no fim da janela — exatamente o capado nos primeiros ticks, ou um emitter crescendo).
- **Scrub (D5):** `emitter → integrate` scrub pra trás reproduz o passado, não a marcha do futuro. O `id` viaja no checkpoint (já corre por `rest→out→pre→forces`); o kernel lê `prev_first` dele. Copie o gate de scrub existente.
- **Mutação obrigatória:** o gather (`prev_row = id − prev_first`) trocado por `i` (posicional) tem de sangrar SÓ no fixture de janela deslizante (num fixture estático `prev_row == i`, então não sangra — por isso a janela TEM de deslizar).

### 3e — Registre os deformers Fase 2 como keepers (quando precisar)

Na fatia 2 registrei só os 9 nós do laço (`register_dense_window`). Os deformers Fase 2 (`transform`/`rotate`/`scale`/`falloff`/`tint`/`wiggle`/`color_ramp`/`oscillator`/`move`) TAMBÉM são per-elemento (keepers), mas **não** os registrei — então `emitter → wiggle → integrate` hoje RECUA (seguro, mas perde a claim). Registre-os (`reg.register_dense_window(MANIFEST.id)` no `register()` de cada) **se/quando** quiser esses grafos reivindicáveis. Não é bloqueante pro gather básico.

---

## §4 — Fatias 4-5

- **Fatia 4 = os gates da fatia 3** (já descritos em 3d; landam junto).
- **Fatia 5 = `forget_state` na edição de PARAM do emitter (D7).** `first` só é monotônico sob params CONSTANTES. Arrastar `rate`/`life`/`max` re-numera os ids → o gather mispareia (igual na CPU — é o modelo, não a GPU). O honesto é invalidar o estado (`forget_state`, `gpu-cook/src/lib.rs:362`) na edição de param do emitter. **Verifique/costure o gatilho** — hoje ele responde a *graph edit*, não a *param change*. É shell-side (`render_loop/motion_bridge_gpu.rs`). Gate: um param-change do emitter zera a sim.

---

## §5 — Gotchas que a wave adversarial expôs (não re-descubra)

1. **A densidade é do emitter NU, não do stream.** `sort`/`cull`/`combine`/`clone`/`mirror`/`trail` quebram (underflow u32 → velocidade zerada em SILÊNCIO). A fatia 2 (`dense_window`) é o guarda — a recusa condicional (3a) DEPENDE dela. Nunca reivindique o gather sem `output_dense_window == Some(true)`.
2. **`age` é re-derivado, nunca acumulado** (`emit` carimba `age = t − id/rate` fresco; integrate copia as não-sim ao vivo do `rest`). NÃO faça o gather carregar `age` do estado — double-conta. ⚠️ O **`sim.step`** (o OUTRO integrador) acumula `age`; os dois não se misturam num stream.
3. **Paridade do playhead:** `params.playhead` no kernel é `clock.playhead as f32` (`lib.rs:456`), o mesmo f32 que a CPU `emit` lê (`ctx.playhead() as f32`). O `source_count` trunca igual. Mantenha isso — é o que faz `newest`/`n`/`first` casarem.
4. **`id` é `f32`, teto 2²⁴** (~16,7M ids ≈ 4,8 dias a rate 40). Compartilhado com a CPU (o BTreeMap keya no mesmo f32) — não é divergência, é teto comum. Fora de escopo.
5. **`bitcast<u32>(i32)` == Rust `as u32`; `u32(x)` é cast de VALOR e diverge em negativos.** Use `bitcast` na fronteira de id se precisar.

## §6 — Convergência com a linha da timeline (sem colisão — medido)

Há um agente na timeline (`line/anim-ajustes`, worktree `line-anim`). **Medido: zero overlap de crate** (timeline = `ph2d-timeline`/`ph2d-panel-timeline`; você = `ph2d-gpu-cook`/`ph2d-nodegraph/gpu.rs`/node-crates). O `Playhead` (`ph2d-core`) é intocado pelos dois; a timeline usa `render_loop/timeline_bridge.rs`, você o `motion_bridge_gpu.rs`. O único arquivo possivelmente compartilhado é `render_loop/mod.rs` (wiring de módulo) — merge de mesma-arquivo-região-diferente, Mergiraf resolve; não é escape §1.5.5. **Podem rodar em paralelo.** (Se o Enio pedir a fatia 5, que toca o shell, cheque `git status` do `motion_bridge_gpu.rs` — mas a timeline não o toca.)

---

## §7 — Ao fechar a fatia 3 (o protocolo)

1. Gates 1× (paridade `--ignored` na RTX + plano sem device + WGSL). Todas as mutações VERMELHAS→restauradas.
2. `cargo fmt` nas crates tocadas · `cargo clippy --all-targets` · `typos` (sem arg) · os 2 LOC caps.
3. Atualize ESTE handoff (mova a fatia 3 pra "landou", registre os hashes + os números medidos).
4. Se salvar lição durável: escreva na memória (`project-memory/`, via symlink → repo PRIMÁRIO — fica sem commit lá, é do Enio commitar).
5. **PARE.** Não integre, não pushe. Reporte ao Enio: fatias fechadas + o que falta.
