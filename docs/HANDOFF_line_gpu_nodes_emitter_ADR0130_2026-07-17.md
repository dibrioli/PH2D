# HANDOFF (briefing de continuação) — `line/gpu-nodes` · ADR-0130 · o emitter na GPU (o gather por `id`)


> ⚠️ **HISTÓRICO — INTEGRADO À `main` EM 2026-07-18.** Este doc conta as fatias 1-5 + a emenda 1. Quem continua a linha começa em [`HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md).

> ⚠️ **O ALVO DESTA LINHA É O EXTRAORDINÁRIO** (CLAUDE.md §0.0, [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]). Medido na RTX: **4,19 M partículas simulam em 3,6 ms**. Se você for escrever um limite — cap, teto, faixa de slider — **meça primeiro** e escreva o número que a medição deu. Nunca deixe a CPU (o caminho de REFERÊNCIA) definir o teto do dispositivo.
>
> ⚠️ **A LINHA ESTÁ FECHADA E ENTREGUE PARA INTEGRAÇÃO** (2026-07-18, ordem do Enio) — o briefing do integrador é [`HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md`](HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-18.md). Este doc segue sendo o ONDE/COMO técnico; o de integração tem os conflitos MEDIDOS e os gates.
>
> **Você é o agente que continua esta linha em contexto fresco.** O ADR já está escrito e **TODAS as
> fatias (1-5) LANDARAM** (fatias 3+4 em `76ae0d52`, fatia 5 em `49829843`, 2026-07-17) e estão gateadas
> (parity na RTX + policy headless) — ver §1.LANDOU, §3.LANDOU e §4.LANDOU. **Pendente de smoke do Enio**
> (§7). Leia este doc + o [ADR-0130](architecture/decisions/0130-gpu-emitter-the-id-gather-is-arithmetic-because-the-window-is-dense.md) inteiro (é curto). O ADR tem o PORQUÊ; este doc tem o ONDE e o COMO, e os gotchas que a wave de pesquisa expôs.

---

## §0 — Inegociáveis (memorize antes de tocar em nada)

1. **Trabalhe SÓ em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes`. SEMPRE prefixe todo comando com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes &&`.** A cwd escorrega pro repo primário (aconteceu 4× em jornadas anteriores — um `git` no primário deixa o shell lá). Se um `git log` mostrar `main`/`cdc3acc1`, você escorregou.
2. **NÃO integre, NÃO pushe, NÃO rode `ship.sh`.** Feche o trabalho, atualize este handoff, e PARE. Integração/ship só por ordem EXPLÍCITA do Enio, via agente integrador dedicado (§0.7 do CLAUDE.md). Esta linha já foi integrada uma vez e re-preparada; ela **acumula** por cima do `main` integrado (fork em `cdc3acc1`).
3. **Contrato congelado 8/2/1 intocado** (`NodeManifest`/`NodeOp`/`OpResolver`, [ADR-0126](architecture/decisions/0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)). Tudo que você mexe é **metadado lateral**: `GpuKernel`, `SourceWindowFn`, `KernelResolver`, o `output_shape` do plano, os kernels. Se sentir vontade de bumpar o `NodeManifest`: PARE — a resposta é sempre o canal lateral.
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
| *(§4.EXTRAORDINÁRIO)* | **`SourceCountFn` → `SourceWindowFn`**, lei de contagem única em `f64`, id com wrap, `MAX_ALIVE` 16.384 → **4.194.304** | paridade além de 1,72e7 spawns + mutação RED |
| `d29366fc` | **Kernel do `motion.emitter`** — a lei da contagem | `the_emitter_generator_matches_the_cpu`: janela `[15,81,121,121]` + cap 256; 2 mutações |
| `90e70302` | **Fatia 2:** `dense_window`, a propriedade provável de plano | 5 gates de plano puros; mutação mata 3, deixa o positivo verde |
| `76ae0d52` | **Fatia 3+4:** o gather aritmético + os gates (§3.LANDOU) | 3 gates de sim de emitter na RTX + 2 de plano; 4 mutações RED |
| `49829843` | **Fatia 5:** `forget_state` no re-number do emitter (D7, §4.LANDOU) | policy headless (todo param) + node real; 1 mutação RED |

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

- **Fatia 4 = os gates da fatia 3** (já descritos em 3d; landaram junto em `76ae0d52`).
- **Fatia 5 = `forget_state` na edição de PARAM do emitter (D7).** `first` só é monotônico sob params CONSTANTES. Arrastar `rate`/`life`/`max` re-numera os ids → o gather mispareia (igual na CPU — é o modelo, não a GPU). O honesto é invalidar o estado (`forget_state`, `gpu-cook/src/lib.rs`) na edição de param do emitter.

### §4.LANDOU — fatia 5 (`49829843`, 2026-07-17)

- **A policy é uma função PURA**, `motion_bridge_gpu::edit_renumbers_emitter(type_name, param)`: só `motion.emitter` × `{rate, life, max}` re-numera (os params do `emit`/`source_count` que definem a janela viva). `speed`/`angle`/`spread`/`seed` e `x`/`y`/`size` **NÃO** re-numeram — o gather ainda pareia id-a-id, então a sim VIVA continua e só os recém-nascidos pegam o novo lançamento (forgetar ali seria um pop à toa). Allowlist minúscula presa à lei-de-contagem de UM nó, não uma regra que apodrece.
- **Costurado no seam de param** (`motion_bridge_params.rs`, arm `SetParam`): decide ANTES do `set_param` consumir `param`, `forget_state()` depois do edit pousar. E **`install()` (load de projeto) também forgeta** — as colunas `pre` do doc velho são keyed por node id; um grafo novo reusando esses ids leria estado de um estranho (mesma classe do D7, bug latente corrigido; espelha o reset do pump logo acima).
- **Gates (headless, sem device):** `only_the_emitters_renumbering_params_invalidate_the_gpu_sim` (todo param do emitter + 4 tipos não-emitter) + `a_real_emitter_nodes_type_name_drives_the_policy` (o `type_name` real do nó casa a policy). Mutação: dropar o guard `motion.emitter` → RED. (O EFEITO do `forget_state` é inobservável headless — precisa de estado de device; o mecanismo já é exercido pelos gates de sim/scrub. Gateamos a DECISÃO, que é a parte arriscada/rot-prone.)
- ⚠️ **O 1º corte usava `forget_state` e o smoke REPROVOU: *"re-bake travado"*. CORRIGIDO em `b104d98d`** — ver §4.RESEED.
- **Divergência GPU↔CPU** conhecida e aceita: a GPU reinicia a sim (limpo), o pump CPU mantém o `pre` (mispair parcial) — a GPU é o caminho EXIBIDO (preview), o CPU alimenta readouts do painel; os gates de paridade (1-passo-de-seed) não cobrem edit ao vivo. Consistência total (reiniciar os dois) é follow-up se o Enio quiser.

### §4.RESEED — o freeze do 1º corte e o fix (`b104d98d`, 2026-07-17)

**O bug era escolher a invalidação errada.** `forget_state` significa *"este estado é inválido, RE-DERIVE"*, e o `rewind_for` honra isso ancorando o ring vazio no **tick 0** (`ring.rs`, `unwrap_or_default`) ⇒ o chamador re-cozinha `0..=target`. Pra uma mudança discreta é **um bake honesto**; pra um param que o artista está **SEGURANDO e arrastando** é **O(tick atual) re-simulado a CADA FRAME** — não é bake, é freeze.

*Um artista arrastando o `rate` do emitter pergunta "como fica com ESTE rate?", não "reproduza os últimos quarenta segundos com ele".* Então agora há **DUAS** invalidações, e o seam escolhe:

| método | significado | custo | quem usa |
|---|---|---|---|
| `forget_state()` | re-derive do seed (ancora no tick 0) | O(tick) | load de projeto (`install`, o relógio rebobina junto) |
| `reseed_from_next_tick()` | **reinicia AO tick da tela** — `rewind_for` devolve o próprio `target`, então a marcha é **UM** cook, e sem estado anterior esse cook É o seed | **O(1)** | o edit ao vivo (D7) |

O seam também só reinicia quando o valor **de fato MUDA** — um slider re-emite o intent todo frame do gesto, e reiniciar num valor inalterado prenderia a sim no seed.

**Gates (headless — `rewind_for` é aritmética de estado pura, então a diferença é observável sem adapter nenhum):** `tests/sim_invalidation.rs` — forget→0 vs reseed→target **no tick 600** (longe de 0, onde as duas concordam por acidente), a flag é consumida UMA vez e não gruda, e um forget cancela um reseed pendente. **Mutação** (`rewind_for` ignora a flag = o freeze pré-fix) → 2 RED, e os gates de forget seguem VERDES (corretamente — não dependem do ramo novo).

⚠️ **`crates/ph2d-gpu-cook/src/lib.rs` está em 691/700** — a próxima adição orça um split.

### §4.LIVE — "por que reiniciar?" e o defeito que a pergunta expôs (`7316bb43`, 2026-07-17)

Enio: *"o que impede que as atualizações sejam feitas em tempo real, sem travamentos e sem reiniciar? Godot faz assim"*. Responder honestamente achou um **defeito na policy que eu shipei**: ela reiniciava por `rate`, `life` **E** `max`, e **dois dos três não re-numeram nada**.

**A lei é uma linha do `emit`: `birth(k) = k / rate`.** Logo:

| param | efeito na janela viva | re-numera? | política |
|---|---|---|---|
| **`rate`** | muda `newest` **e o nascimento de todo id** | **SIM** | reinicia (`reseed_from_next_tick`) — não há estado a carregar, porque *"o estado do id k"* deixa de se referir à mesma coisa |
| `life` | move só a borda ESQUERDA (`oldest`) | não | **VIVO** |
| `max` | move só a borda ESQUERDA (o cap) | não | **VIVO** |
| `speed`/`angle`/`spread`/`seed`/`x`/`y`/`size` | nada na contagem | não | vivo (já era) |

E o vivo de `life`/`max` é **exato dos dois lados**, sem maquinário novo:

- **Encolher** (life↓ / max↓): todo sobrevivente mantém id, mantém linha, e sua trajetória é **bit-idêntica** a uma execução que nunca mudou — nada na física de uma partícula depende de quantas partículas mais velhas foram podadas ao lado dela.
- **Crescer** (life↑ / max↑): a janela revela ids que o frame anterior não carregava; esses são **semeados pelo bounds-check por-elemento `gather_paired` que já existe para os recém-nascidos** (D4). Sem reinício, e sem ler uma linha que não está lá — a defesa em camadas do D4 fazendo exatamente o trabalho dela.

**Gate (GPU, RTX):** `shrinking_the_life_of_a_live_emitter_leaves_the_survivors_untouched` — marcha a sim, encolhe `life` no meio do voo **sem invalidação nenhuma**, e exige que o frame editado seja um **SUFIXO literal** do não-editado, **bit-idêntico** (drift `== 0.0` exato, não dentro de um ε; os ids sobem oldest-first, então o oráculo não precisa de id no stream de instâncias). Mutação (gather de volta a posicional) → **RED no 1º tick editado**.

⚠️ **O 1º fixture era VACUOSO e o disse:** `FIXED_DT` é **0,05**, não 1/60 — com `life = 0.05` cada partícula vivia **UM tick**, nada pareava, e *"os sobreviventes casam"* era uma afirmação sobre o **conjunto vazio**. Quem pegou foi a **pré-condição de drift** (`moved > MUST_MOVE`), que existe exatamente pra isso. Hoje `life` cobre 4 ticks e a encolhida 2.

**O que AINDA reinicia, e por quê (a resposta ao "Godot faz assim"):** as partículas do Godot são **stateful** — cada uma tem estado próprio num buffer de GPU, então trocar um uniforme não mexe em quem já está vivo. O nosso emitter é **stateless de propósito** (o conjunto vivo é função pura do playhead, ADR-0130), e é isso que compra o **scrub bit-exato** — o Godot não tem (o `seek`/`preprocess` dele re-simula do zero, que é literalmente o re-bake O(tick) que o §4.RESEED deletou). O `rate` é o único ponto onde os dois modelos divergem de fato: ele redefine a IDENTIDADE, e o Godot também reinicia quando o `amount` muda.

**Se um dia o `rate` também tiver de ser vivo** (não construído, é decisão do Enio): o pareamento não precisa ser por id, pode ser por **tempo de nascimento** — `id_velho ≈ round((k/rate_novo) · rate_velho)`, ainda aritmética pura, um uniforme a mais (`prev_rate`). Vira vizinho-mais-próximo no tempo, o que dá exatamente o *look* do Godot (o jato fica mais denso/ralo sem reiniciar). Custo: 1 uniforme + a linha do `gather_row`; o risco é que o pareamento passa a ser aproximado, então o gate de paridade CPU↔GPU precisa de uma política nova (a CPU não pareia nada).

### §4.TINT — o Gradient do `motion.tint` na GPU (`6f761704`, 2026-07-17)

Item **#6 do §Aberto da Fase 3**, e ele pertencia a ESTA linha: o doc do próprio emitter promete *"ids ascend oldest-first, so `motion.tint` in Gradient mode paints the stream by AGE"* — e esse modo **derrubava a fonte inteira pra CPU**. O `applicable` recusava o Gradient porque o ramp keya em `Index/(Count−1)` com fallback **posicional** (`i`/`n`), e `ColumnBinding.identity` é uma **CONSTANTE** — `read_Index` não conseguia devolver `f32(i)`. O **`HAS_<col>` gerado** fecha, exatamente como já tinha fechado pro `motion.color_ramp`.

**A `DEMO=5` agora colore por idade** (branco quente no bico → azul profundo nas pontas) **e segue planejando 100% GPU**. É o fix em forma visível.

⚠️ **O perigo de curto-circuito do `color_ramp` NÃO existe aqui, e a diferença é ESTRUTURAL, não sorte:** o `t` dele chega em OUTRA porta, onde uma cadeia enraizada noutro gerador pode ter outro comprimento, e este motor chama coluna de comprimento errado de **ausente** — o que ali significaria em silêncio *"use a chave posicional"* enquanto a CPU **preenche**. `Index`/`Count` viajam na **MESMA** porta da base, e **`Stream::set` ASSERE `col.len() == count`** ⇒ presente ⟺ existe, e presente tem exatamente `n`. O `unwrap_or(default)` da CPU é **inalcançável** pros dois — defensivo, não um ramo semântico.

**O gate do ramo posicional precisou de um fixture que pudesse CONTRADIZÊ-lo:** todo gerador GPU-enraizável emite `Index`/`Count` e todo nó de transformação carrega a base ⇒ numa cadeia normal `HAS_Index` é **sempre true** e o fallback seria código não-testado atrás de suíte verde. **O oráculo é a DEFINIÇÃO do fallback**: a mesma boundary stream cozida **duas vezes**, uma com `Index`/`Count` retiradas, tem de bater **BYTE a byte** — com pré-condições de que as colunas retiradas eram mesmo `0..n−1` e `n` (senão a igualdade é coincidência sobre aquela stream) e de que o ramp de fato **varre**.

⚠️ **Duas armadilhas que caíram no caminho, as duas pegas por pré-condição de fixture:**
1. **`motion.cull` NÃO descarta colunas.** Meu `grep` "provou" que sim — estava lendo o **fixture de teste dentro do próprio crate**, não o `eval` (que copia `for (name, col) in input.columns()`). [[feedback_a_negative_search_needs_a_positive_control]]. Foi a pré-condição do fixture que pegou. O `cull` ficou no gate só como **fronteira CPU** (não tem kernel), que é o que permite entregar ao sufixo GPU uma stream escolhida pelo teste.
2. **`str.replace(old, new, 1)` com `assert old in s` acertou a demo ERRADA** — havia 3 ocorrências de `let ig = g.add_node("motion.integrate");`. *Assert de presença não é assert de unicidade* — ancore em algo **único** (`assert s.count(anchor) == 1`).

**`NodeManifest.lowerings` intocado** (`&[LoweringKind::Cpu]`) — o kernel é side metadata, o contrato congelado não se move.

### §4.TETO — a sim medida, e o `MAX_ALIVE` 4096 → 16384 (`46ae13bd`, 2026-07-17)

Item **#3 do §Aberto da Fase 3** (a sonda de sim). Construí-la achou o que valia achar: **a linha inteira existe pra levar partículas a milhões, e o `motion.emitter` estava capado em 4096.** A `DEMO=5` pedia 3.000 e eu a descrevi como *"até 3.000 partículas"* sem nunca ter lido o teto a que ela encostava.

**Medido (RTX, `emitter_sim_ceiling_probe`, `emitter → wind → drag → integrate`):**

| janela | GPU ms/tick | CPU ms/tick |
|---:|---:|---:|
| 4.096 | 0,085 | 0,243 |
| 16.384 | 0,105 | 1,044 |
| 65.536 | 0,166 | 2,989 |
| 262.144 | **0,285** | **12,982** |

A GPU é **plana** (64× as partículas = 3,35× o custo; 262k = **1,7%** de um frame de 60 fps). A CPU é linear e a 262k come **78% do frame inteiro**.

**⇒ O teto é dado pelo FALLBACK da CPU, não pelo renderer nem pela GPU** — e o `PH2D_GPU_COOK` é **opt-in**, então a CPU é o que todo build default de fato roda. O comentário antigo justificava o guard como proteção contra *"a stream no renderer would draw anyway"*, e essa metade é hoje **plainly false**: o renderer desenha 2M instâncias a 4,02 ms. O número foi escolhido antes de qualquer lado ser mensurável; a **cerca** (limitar um `rate 1e9` escrito à mão) continua de pé.

**16.384** = 4× de folga, fallback em ~6% do frame, < 1 MB de transiente por frame (44 B/partícula × 8 colunas). **65.536 é a tentação seguinte** — 3 ms/tick mordidos de um *fallback*, para **um** nó de um grafo: **decisão do Enio**, não default a se enfiar de contrabando. Uma palavra e eu mudo.

⚠️ **O teto NUNCA pode ser dependente de caminho**, por mais tentador: a paridade do ADR-0130 vale *por construção* porque `eval` e `source_count` derivam `newest`/`n`/`first` de **UMA** lei de contagem compartilhada. Um teto só-GPU daria `n` diferente pros dois lados do MESMO documento, e o scrub, a emenda híbrida e o gather passariam a ler uma janela que a outra metade nunca teve.

**Gate:** o teto duro agora tem um (o gate de lei-de-contagem existente só alcançava o **param** `max`). É **determinístico, não cronometrado** — peça um bilhão de partículas e exija que os dois caminhos pousem no MESMO `n`, enunciado como **invariante** pra não precisar de edição quando o número mudar. Mutação (metade do teto só no caminho GPU) → RED nessa asserção (16384 vs 8192). *(A 1ª mutação — remover o teto da GPU — matava o gate por **crash de alocação da wgpu**, não pela asserção: prova fraca, troquei pela regressão realista.)*

⚠️ **Dois fixtures MENTIRAM antes de dizer a verdade, os dois pegos por pré-condição:**
1. **A sonda imprimiu `601` nas quatro linhas** — o `emitter_sim` fixa `rate 400`, então em `TICKS × FIXED_DT` segundos só 601 partículas chegam a **NASCER**. Tabela de perf que imprime o mesmo número quatro vezes é pior que tabela nenhuma.
2. **O gate do teto pediu um bilhão, recebeu 401 e PASSOU** (`emitter_graph` fixa `rate 40`). *Um gate de teto tem de superar o teto em NASCIMENTOS, não só em PEDIDO.*

**`DEMO=5` agora roda 4000/s × 3 s = 12.000 vivas.** Os números velhos pediam 3.000 mas estavam de fato limitados em **4.200** por `rate × life`, sob o teto antigo — a fonte não conseguia ser uma fonte. ~0,1 ms/tick na GPU.

### §4.EXTRAORDINÁRIO — a identidade exata, e o teto que virou do hardware (`b4b7…`, 2026-07-17)

**A cobrança do Enio:** *"não estamos levando o motion nodes para o GPU para alcançarmos resultados extraordinários?"* — e estava certo. Medido (RTX, `emitter_sim_ceiling_probe`):

| janela | GPU ms/tick | CPU ms/tick |
|---:|---:|---:|
| 262.144 | 0,277 | 13,060 |
| 1.048.576 | 0,984 | 52,608 |
| **4.194.304** | **3,636** | 227,800 |

**4,19 M partículas em 3,6 ms** (22% de um frame de 60 fps) — e o teto estava em **16.384**, porque a **CPU** seria lenta. Duas vezes seguidas (4096, depois 16.384) o teto foi um número que outra preocupação escolheu, nunca o dispositivo. Lição durável: [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]] + **CLAUDE.md §0.0**.

**O bloqueio real era UM, e estava arquivado como nota de rodapé:** ids são `f32`, então índice de spawn acima de 2²⁴ **colide** com o vizinho, e os DOIS pareamentos entregam o mesmo estado a duas partículas (o `BTreeMap<id,row>` da CPU guarda uma; o `id − prev_first` da GPU lê a mesma linha duas vezes). Em silêncio. O §5.4 deste handoff dizia *"≈4,8 dias a rate 40 — fora de escopo"*: verdade a rate 200 (23 h), **4 segundos** a 4e6.

**O conserto deixou o motor MAIS simples:**

| antes | depois |
|---|---|
| `SourceCountFn -> usize` | **`SourceWindowFn -> SourceWindow {count, first, age_first}`** — um gerador dependente de playhead emite uma JANELA, não uma contagem |
| `emit` e `source_window` calculavam a lei de contagem **separadamente em f32** | **UMA** `window()`, em `f64` — as duas portas viraram uma |
| o kernel re-derivava `floor(t·rate)` em f32 | o kernel é **INFORMADO** da janela; a paridade deixou de valer "porque os dois leem o mesmo f32" e passa a valer porque **há um número e ele é INTEIRO** |
| id absoluto (estoura 2²⁴) | id com **wrap em `ID_WRAP`** — todo consumidor lê identidade como DIFERENÇA dentro de uma janela, e a janela é ordens de grandeza menor que o período. Abaixo de 2²⁴ é a identidade ⇒ **byte-idêntico** pra toda cena que funciona hoje |
| `age = t − id/rate` (cancelamento catastrófico) | `age_first − offset`, e a CPU roda os **mesmos dois passos** do kernel |
| `MAX_ALIVE = 16.384` (frame time da CPU) | **4.194.304**, um orçamento de **MEMÓRIA** (~370 MB de residência GPU) |

⚠️ **O teto continua COMPARTILHADO** entre os caminhos: a paridade do ADR-0130 vale por construção a partir de uma lei só. CPU lenta demais pra TOCAR 4M é fato de performance — referência só precisa computar a mesma resposta.

**Gates:** `identity_is_exact_at_any_rate_because_it_wraps` (escrito RED como o próprio oposto — *"past 2²⁴ o espaço de ids tem de aparecer COLAPSANDO"* — e falhou com essa exata mensagem quando o fix pousou; hoje afirma distinção a rate 4e6 uma HORA adentro, 1,4e10 spawns, ~858 wraps) · **`the_emitter_sim_is_exact_past_the_old_id_cliff`** (sim real marchada por 1,72e7 spawns, CPU vs GPU elemento a elemento; **mutação**: devolver `floor(t·rate)` ao kernel → RED, `cpu 0.3278 vs gpu 0.3362`).

⚠️ **O fixture desse gate levou TRÊS tentativas e cada falha foi a pré-condição trabalhando:** a rate 4e6 uma janela de 4096 é **1 ms** de história (nada se moveu ⇒ comparar dois campos de zeros) **e** ela vira **48× por tick** (nada SOBREVIVE a um tick ⇒ o gather nunca é exercitado). Sobreviventes exigem `life > FIXED_DT`, o que capa a rate, o que força marcha longa — daí guardar só o frame FINAL (2100 × 16.384 instâncias seriam gigabytes).

**O gate de teto duro MUDOU DE LUGAR** (do seam de paridade pra suíte do emitter): com uma lei de contagem só, *"os dois caminhos concordam no n"* deixou de ser afirmação que duas implementações podem falsificar e virou **propriedade de haver uma**. Gateie onde ainda pode ser contradito.

**LOC:** testes do emitter → `lib_tests.rs` (711→423); helpers de identidade do gpu-cook → `gather.rs` (701→649).

---

## §5 — Gotchas que a wave adversarial expôs (não re-descubra)

1. **A densidade é do emitter NU, não do stream.** `sort`/`cull`/`combine`/`clone`/`mirror`/`trail` quebram (underflow u32 → velocidade zerada em SILÊNCIO). A fatia 2 (`dense_window`) é o guarda — a recusa condicional (3a) DEPENDE dela. Nunca reivindique o gather sem `output_dense_window == Some(true)`.
2. **`age` é re-derivado, nunca acumulado** (`emit` carimba `age = t − id/rate` fresco; integrate copia as não-sim ao vivo do `rest`). NÃO faça o gather carregar `age` do estado — double-conta. ⚠️ O **`sim.step`** (o OUTRO integrador) acumula `age`; os dois não se misturam num stream.
3. **Paridade do playhead:** `params.playhead` no kernel é `clock.playhead as f32` (`lib.rs:456`), o mesmo f32 que a CPU `emit` lê (`ctx.playhead() as f32`). O `source_count` trunca igual. Mantenha isso — é o que faz `newest`/`n`/`first` casarem.
4. ~~**`id` é `f32`, teto 2²⁴** … Fora de escopo.~~ **RESOLVIDO e a nota estava ERRADA** (§4.EXTRAORDINÁRIO): o *"fora de escopo"* valia só enquanto o slider de rate parava em 200. Hoje a identidade **tem wrap em `ID_WRAP`** e é exata em qualquer rate. **Quem move o número que tornava algo inalcançável tem de reconferir a nota.**
5. **`bitcast<u32>(i32)` == Rust `as u32`; `u32(x)` é cast de VALOR e diverge em negativos.** Use `bitcast` na fronteira de id se precisar.

## §6 — Convergência com a linha da timeline (sem colisão — medido)

Há um agente na timeline (`line/anim-ajustes`, worktree `line-anim`). **Medido: zero overlap de crate** (timeline = `ph2d-timeline`/`ph2d-panel-timeline`; você = `ph2d-gpu-cook`/`ph2d-nodegraph/gpu.rs`/node-crates). O `Playhead` (`ph2d-core`) é intocado pelos dois; a timeline usa `render_loop/timeline_bridge.rs`, você o `motion_bridge_gpu.rs`. O único arquivo possivelmente compartilhado é `render_loop/mod.rs` (wiring de módulo) — merge de mesma-arquivo-região-diferente, Mergiraf resolve; não é escape §1.5.5. **Podem rodar em paralelo.** (Se o Enio pedir a fatia 5, que toca o shell, cheque `git status` do `motion_bridge_gpu.rs` — mas a timeline não o toca.)

---

## §7.SMOKE — como o Enio confere (a linha inteira LANDOU)

O gate É o audit (parity na RTX + policy headless, todos verdes). Há uma **cena ready-to-smoke dedicada** (`PH2D_GPU_COOK_DEMO=5`, a **fonte de emitter** — auto-play, gateada em `the_emitter_fountain_demo_plans_as_a_fully_gpu_id_gather_loop`). **O comando completo, com o `cd` da worktree:**

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=5 cargo run -p ph2d-host-desktop --release
```

- **O gather** (fatias 3-4): a fonte lança até 3.000 partículas que arcam e caem, cada uma com sua velocidade de bico (por-id, cone de `spread`) — **os arcos são limpos**; uma janela deslizante mispareada faria a fonte "ferver" (cada sobrevivente herdando a velocidade de um estranho). Antes da fatia 3, `emitter → integrate` caía na CPU; agora cozinha 100% na GPU (`emitter → integrate` reivindicado pela janela densa). *(As cenas `=3`/`=4` são GRID = pareamento posicional; só a `=5` exercita o gather.)*
- **Fatia 5** (a invalidação do edit ao vivo): com a fonte rodando, **arraste `rate`** no painel de params → a fonte **REINICIA do tick da tela** e volta a crescer com o novo numeramento — **o arrasto tem de ficar fluido** (é 1 cook por frame, igual playback normal; se travar, é regressão do §4.RESEED). *(O 1º corte usava `forget_state` e re-simulava a história inteira por frame — o "re-bake travado" que o Enio pegou.)*
- **§4.EXTRAORDINÁRIO — a fonte agora é 1,2 MILHÃO de partículas** (400.000/s × 3 s, grão 0,012; teto 4,19 M). Tem de parecer uma massa de grãos com estrutura, não um borrão, e continuar fluida: ~1 ms/tick medido. *(Foi 3.000 → 4.200 → 12.000 → 1,2 M; cada número anterior era um teto que alguma outra preocupação escolheu.)*
- **§4.TINT — a fonte agora é COLORIDA por idade** (branco quente no bico → azul nas pontas): é o `motion.tint` em **Gradient** rodando na GPU. O gradiente tem de **varrer ao longo do jato** (as mais velhas, no alto do arco, mais quentes) e a fonte tem de continuar fluida — se ela ficar de uma cor só, ou se o FPS cair, o Gradient recuou pra CPU.
- **§4.LIVE — o que NÃO reinicia mais:** arraste **`life`** e **`max`** → a fonte **não pisca**: as partículas que continuam vivas seguem exatamente o voo delas (encolher é bit-idêntico; crescer só semeia as que a janela revela). Arraste `speed`/`size`/`angle`/`spread` → idem, a sim viva continua e só os recém-nascidos pegam o novo lançamento. **Se `life`/`max` reiniciarem a fonte, é regressão do §4.LIVE.**

## §7 — Ao fechar a fatia (o protocolo)

1. Gates 1× (paridade `--ignored` na RTX + plano sem device + WGSL). Todas as mutações VERMELHAS→restauradas.
2. `cargo fmt` nas crates tocadas · `cargo clippy --all-targets` · `typos` (sem arg) · os 2 LOC caps.
3. Atualize ESTE handoff (mova a fatia 3 pra "landou", registre os hashes + os números medidos).
4. Se salvar lição durável: escreva na memória (`project-memory/`, via symlink → repo PRIMÁRIO — fica sem commit lá, é do Enio commitar).
5. **PARE.** Não integre, não pushe. Reporte ao Enio: fatias fechadas + o que falta.
