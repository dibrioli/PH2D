# HANDOFF DE INTEGRAÇÃO — `line/gpu-nodes` (ADR-0130 + emenda 1) · 2026-07-18


> ⚠️ **HISTÓRICO — INTEGRADO À `main` EM 2026-07-18.** Este doc conta o briefing do integrador. Quem continua a linha começa em [`HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md).

> **Para o agente INTEGRADOR do Enio** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). Esta linha
> está **FECHADA**. Ela não integrou, não pushou e não rodou `ship.sh` — por
> protocolo (§0.7 do CLAUDE.md).
>
> **Ordem do Enio:** *"vamos integrar ao main"* (2026-07-18).

---

## §0 — TL;DR do integrador (leia isto e o §2)

| | |
|---|---|
| **Branch** | `line/gpu-nodes`, HEAD **`b0c2d2e7`** |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes` |
| **Fork** | `cdc3acc1` — **que É a `main` de hoje**, então o merge é `--ff-only` limpo |
| **Tamanho** | 24 commits · 50 arquivos · +3651 / −450 |
| **Contrato congelado** | **INTOCADO** (`NodeManifest=8`/`NodeOp=2`/`OpResolver=1`) — gate verde |
| **Conflito com linhas em dia** | **3 arquivos, todos append-friendly** (§2) |
| **Risco alto** | **1**, e é mecânico + pego pelo compilador (§3.1) |

**A `main` não andou desde o fork desta linha.** Se isso ainda for verdade quando
você rodar, a integração é `git merge --ff-only line/gpu-nodes` e acabou. Confira
antes: `git -C <primário> rev-parse main` tem de dar `cdc3acc1`.

---

## §1 — O que a linha entrega

**ADR-0130** (o emitter na GPU: o gather por `id` é *aritmético* porque a janela é
densa) + **emenda 1** (a identidade é exata em qualquer rate).

Em produto: `emitter → [forças] → integrate/spring → tint → output` cozinha
**100% na GPU** e casa a CPU dentro do ε. Medido na RTX:

| janela | GPU ms/tick | CPU ms/tick |
|---:|---:|---:|
| 262.144 | 0,277 | 13,060 |
| 1.048.576 | 0,984 | 52,608 |
| **4.194.304** | **3,636** | 227,800 |

Peças, na ordem em que um leitor as encontra:

1. **`ColumnAccess::GatherKey`** (`ph2d-nodegraph::gpu`) — recusa **condicional**:
   reivindica uma janela densa, recua uma `id` não-densa/improvável.
2. **`dense_window`** — propriedade **provável** de plano, default-false, opt-in
   pelo `register_dense_window`. Nunca uma allowlist dos nós que a quebram.
3. **O gather aritmético** — `prev_row = current_id − prev_first`, com
   bounds-check **por-elemento** (`gather_paired`) distinto do global `HAS_*`.
4. **`SourceCountFn` → `SourceWindowFn`** (emenda 1) — um gerador dependente de
   playhead emite uma **janela** `{count, first, age_first}`, não uma contagem.
   Lei de contagem **única**, em `f64`; identidade com **wrap em `ID_WRAP`**;
   `MAX_ALIVE` = **4.194.304** (orçamento de MEMÓRIA, ~370 MB).
5. **`motion.tint` Gradient na GPU** — o `HAS_<col>` fecha o fallback posicional
   que mantinha o modo na CPU.
6. **Edit ao vivo** (D7) — só `rate` re-numera ⇒ `reseed_from_next_tick` (O(1));
   `life`/`max` são **live** e exatos.
7. **Limites soft/hard de param** (`ParamHardMax`, `ph2d-node-registry`) — o
   slider arrasta até o soft, a caixa digita até o hard.

**Doc:** CLAUDE.md **§0.0** (inegociável novo — o alvo é o extraordinário, o teto
é do hardware), memória `feedback_the_ceiling_is_the_hardwares_never_the_fallbacks`,
ADR-0130 emenda 1, o plano da GPU, e o handoff da linha.

---

## §2 — Conflitos: MEDIDOS, não estimados

**Três linhas estão ATRASADAS** e a sobreposição aparente delas comigo é
**artefato disso** — o trabalho delas já está na `main`, só o worktree ficou pra
trás. **Não as trate como paralelas às minhas:**

| linha | estado |
|---|---|
| `line/cook-parallel` | ⚠️ **230 commits atrás** da main |
| `line/motion-value` | ⚠️ **235 commits atrás** |
| `line/audio` | ⚠️ **95 commits atrás** |
| `anim-ajustes` · `Painter` · `Vector` · `FLIP` · `physics` · `gpu-nodes` | em dia |

Contra as linhas **em dia**, a sobreposição total desta linha é:

| arquivo | com quem | natureza |
|---|---|---|
| `CLAUDE.md` | `line/Painter` | ambos **acrescentam** (eu no §0, ele no §5) — merge textual |
| `project-memory/MEMORY.md` | `line/anim-ajustes` | **SÓ ADICIONE**; remover linha é operação de integração ([[feedback_a_shared_list_is_merged_against_todays_main]]) |
| `Cargo.lock` | `Painter`, `physics` | regenere (`cargo check --workspace`), não resolva à mão |

**Nenhuma outra linha em dia toca `crates/ph2d-nodegraph`, `ph2d-gpu-cook`,
`ph2d-node-*` ou o `motion_bridge`.** Verificado por `git diff --name-only`
cruzado, não por leitura de handoff.

---

## §3 — Riscos, do maior pro menor

### §3.1 — ⚠️ O rename `source_count` → `source_window` (ALTO, mas mecânico)

Toquei **30 literais em 23 crates de nó**. O campo do `GpuKernel` mudou de nome
**e de tipo** (`Option<SourceCountFn>` → `Option<SourceWindowFn>`).

**Por que é seguro:** medi que **nenhuma outra linha em dia cria literais
`source_count:`** (`git diff cdc3acc1..<linha> | grep '^+.*source_count:'` = 0 em
todas). E se alguma criar depois, **o compilador pega** — não existe caminho
silencioso: um campo com nome errado é erro, e um tipo de retorno errado é erro.

**O que fazer:** se o merge trouxer um nó novo de outra linha com
`source_count: None`, troque por `source_window: None`. Se ele for um **gerador**
(`Some(...)`), envolva o retorno em `SourceWindow::of_count(...)`.

**O que NÃO fazer:** não "conserte" reintroduzindo `source_count`. O rename é o
conteúdo da emenda 1, não cosmética — o campo agora carrega `first`/`age_first`,
que é o que torna a identidade exata.

### §3.2 — Foundational tocado (protocolo ADR-0107)

- `crates/ph2d-nodegraph/src/gpu.rs` — `GatherKey`, `SourceWindow`, `ID_WRAP`,
  o campo renomeado. **Extensão append-only** exceto o rename.
- `crates/ph2d-node-registry/src/{lib,ui}.rs` — `ParamHardMax` + a tabela lateral.
  **Puramente aditivo** (nenhum `ParamUiHint` existente muda de forma).

⇒ **Rode `scripts/foundational-integrate.sh`** (gate da árvore combinada) depois
de fundir tudo, não só o `cargo check` da minha linha isolada
([[feedback_clean_text_merge_can_be_semantically_broken]]).

### §3.3 — Mudanças de comportamento que um smoke poderia estranhar

| o quê | por quê é intencional |
|---|---|
| **`age` mudou de fórmula** (`age_first − offset`, não `t − id/rate`) | o antigo é subtração de dois números grandes pra obter um pequeno — já era ruído em rate alto. **`cook_determinism` NÃO usa emitter** (verificado), então nenhum golden se move. |
| **`DEMO=5` foi de 3.000 → 1,2 MILHÃO de partículas** | cada número anterior era um teto que outra preocupação escolheu. Grão 0,012 pra não virar borrão. |
| **`MAX_ALIVE` 4096 → 4.194.304** | orçamento de memória medido, não frame time da CPU. |
| **Slider de `rate` 200 → 12.000** (caixa até 4.000.000) | pedido explícito do Enio, 2026-07-18. |

### §3.4 — Dev-deps novas (rode `cargo machete`)

- `ph2d-gpu-cook` ← `ph2d-node-motion-cull` (fronteira CPU do gate posicional)
- `ph2d-panel-motion-params` ← `ph2d-ui-testkit` (os 3 gates de seam que CLICAM)

Ambas verificadas com `cargo machete` (limpo).

---

## §4 — Gate: o que rodar, e o que tem de sair

Do worktree (⚠️ **sempre com o `cd`** — a cwd escorrega pro primário):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && \
  cargo test -p ph2d-gpu-cook --release -- --include-ignored
```

Esperado (**exige a RTX** — os gates de GPU são `#[ignore]`):

| suíte | verdes |
|---|---:|
| `ph2d-gpu-cook` lib | 16 |
| `generated_wgsl_validates` | 2 |
| `gpu_cpu_parity` | 15 |
| `gpu_cpu_parity_sim` | 17 |
| `plan_analysis` | 5 |
| `plan_simulation` | 12 |
| `sim_invalidation` | 4 |

Mais: `ph2d-node-motion-emitter` 16 · `ph2d-panel-motion-params` 14 ·
`ph2d-host-desktop --bins` 691 · `ph2d-nodegraph` 84+3+6 · os 2 LOC caps
(`architecture_workspace_file_loc_cap` mora na `ph2d-editor-core` e **não** roda
com `cargo test -p`).

**Os dois gates que mais importam se algo der errado no merge:**

- `the_emitter_sim_is_exact_past_the_old_id_cliff` — marcha uma sim real por
  **1,72×10⁷ spawns** e compara CPU↔GPU elemento a elemento. Se a emenda 1 for
  parcialmente perdida no merge, é ele que grita.
- `identity_is_exact_at_any_rate_because_it_wraps` — a mesma coisa sem device.

---

## §5 — Smoke pro Enio (depois do merge, antes do push)

```bash
PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=5 cargo run -p ph2d-host-desktop --release
```

- **1,2 milhão de partículas**, coloridas por idade (branco quente no bico → azul
  nas pontas), arcando e caindo. Fluida (~1 ms/tick medido).
- Arraste **`rate`** → a fonte reinicia do tick da tela, **e o arrasto fica
  fluido** (1 cook/frame). Arraste **`life`/`max`/`speed`/`size`** → a sim viva
  **não pisca**.
- **Max Particles** arrasta até 4.194.304; **Rate** arrasta até 12.000 e aceita
  4.000.000 digitado.

**Pendente de smoke** (landou depois do último OK do Enio): a emenda 1 inteira
(§1.4) e o `DEMO=5` a 1,2 M.

---

## §6 — Depois de integrar

O ship é seu (DIRETRIZ §1.5.4) e **só por ordem do Enio**. Dois avisos que já
custaram caro neste repo:

1. **`./scripts/ship.sh` do integrador drena latentes** — orce **2-4 iterações**;
   gate por-linha não basta ([[project_integrator_ship_catches_latents_budget_iterations]]).
2. **Um pipe mascara o exit code** — verifique o ESTADO, não a saída do `grep`
   ([[feedback_pipe_masks_script_exit_code]]).

**Aberto nesta linha** (nada bloqueia a integração): reduções na GPU
(destrava twist/bend) · multi-input real (`look_at`/`combine` — o motor já
suporta, falta o kernel) · os bloqueios de motor do `t` do `color_ramp` /
`value.*` · N fronteiras no shell · readouts/probe no modo GPU (Fase 4).
Detalhe no §9 do [handoff da Fase 3](HANDOFF_line_gpu_nodes_fase3_2026-07-16.md)
e no [handoff desta linha](HANDOFF_line_gpu_nodes_emitter_ADR0130_2026-07-17.md).
