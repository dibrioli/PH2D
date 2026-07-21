# HANDOFF — `line/gpu-nodes` · A GRANDE AUDITORIA do Motion Nodes · 2026-07-20

> **Para o agente NOVO que assume esta linha.** Você vai fazer uma **auditoria
> ampla de TODO o sistema Motion Nodes** (bugs · melhorias · performance) ANTES de
> qualquer implementação nova. Depois da auditoria, continua a linha de onde o
> agente anterior parou (§3). **NÃO integre ao main** — a ordem de integração é do
> Enio (CLAUDE.md §0.7).
>
> Este doc é a TAREFA + o mapa. O **estado técnico profundo** (o que já foi
> construído, medido e REPROVADO) mora em
> [`HANDOFF_line_gpu_nodes_continuacao_2026-07-20.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-20.md)
> e nos ADRs [0126](architecture/decisions/0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md)
> · [0127](architecture/decisions/0127-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md)
> · [0130](architecture/decisions/0130-gpu-emitter-the-id-gather-is-arithmetic-because-the-window-is-dense.md)
> · [0134](architecture/decisions/0134-gpu-multi-pass-kernels-neighborhood-sims-build-a-spatial-grid-on-device.md)
> · [0135](architecture/decisions/0135-gpu-sim-zone-is-a-conditional-passthrough-and-a-partial-claim-retreats.md).
> **Leia-os antes de auditar** — muita coisa que "parece bug" já é decisão medida.

---

## ⛔ FASE 0 — ONDE VOCÊ ESTÁ (execute JÁ, antes de ler qualquer código)

Você começa na RAIZ do repo, que está em `main`. Os MESMOS paths relativos existem
aqui e na sua worktree; editar a árvore errada **compila e commita sem um único
erro** e ninguém descobre até a integração ([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

```
cd Worktrees/line-gpu-nodes && pwd && git branch --show-current
```
- `pwd` TEM de terminar em `/Worktrees/line-gpu-nodes`.
- a branch TEM de ser `line/gpu-nodes`. **Deu `main`? PARE — árvore errada.**

```
git log --oneline -6 && git status -sb
```
- HEAD ao escrever isto: **`cf92e934`**, **21 commits à frente do `main`** (⚠️ o
  main **NÃO andou** desde o fork desta jornada — confira com `git rev-list --count
  main..HEAD`; se andou, `git rebase main` primeiro, DIRETRIZ §1.5.2.3).
- Árvore limpa esperada. Suja = trabalho não-commitado (não descarte; commite
  `--no-verify`).

⚠️ **`SEMPRE prefixe todo comando com `cd Worktrees/line-gpu-nodes &&`.** A cwd
escorrega pro repo primário — aconteceu 6× em jornadas anteriores, e uma delas o
agente **leu o `main` achando que era a linha**.

**Depois da FASE 0, leia (nesta ordem):** o handoff de continuação (§link acima) ·
`docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` (inteira, e releia a cada
passo) · as regras A–H de `MODELO_ABERTURA_LINHA.md`.

---

## §0 — Estado da linha (o que existe, tudo em 21 commits locais NÃO integrados)

O pipeline **GPU-resident de Motion Nodes** está maduro: 32 kernels + a grade de
vizinhança (ADR-0134) + a família `sim.zone` na GPU (ADR-0135), GPU como DEFAULT,
o painel lendo uma frame GPU-resident pelo tap. Fases fechadas e **smokadas pelo
Enio**:

| entrega | ADR | smoke |
|---|---|---|
| Fase 1–3: buffers GPU + lowering WGSL + o laço `pre` de sim | 0126/0127 | OK |
| Emitter (id-gather aritmético, janela densa) | 0130 | OK |
| 32 kernels · lei de contagem · broadcast · variantes por-param · tap · GPU default | — | OK (`=6`) |
| A grade de vizinhança + boids/collide a milhões (Fases 1–5) | 0134 | OK (`=7`/`=8`/`=9`) |
| **A família `sim.zone` na GPU** (passthrough condicional + `sim.step`/`sim.collide` + o RECUO do plano) | **0135** | **OK (`=10`, 2026-07-20)** |

⚠️ **A última coisa que aconteceu (e é uma LIÇÃO pra auditoria):** o smoke do `=10`
reportou *"profunda queda de fps"*. A medição mostrou que **NÃO era o cook** (0,5
ms/tick na GPU, `FullyGpu`, 0 fallthrough, ~58 fps no zoom default) — era o
**RENDER** de 262 k instâncias quando todas ficam visíveis (o count é orçamento de
RENDER, não do cook). Detalhe em `the_zone_demo_scale_cook_cost` (RTX). **Esse é
exatamente o tipo de coisa que a auditoria procura: um sintoma cuja causa engana.**

**Rodar tudo verde hoje (na worktree):**
```bash
cargo test --workspace                                   # 7.6k+ verdes
cargo test -p ph2d-gpu-cook --release -- --ignored       # os gates de GPU (RTX)
cargo test -p ph2d-host-desktop --release -- --ignored   # seams do painel
```

---

## §1 — A TAREFA: A GRANDE AUDITORIA DO MOTION NODES (faça ISTO primeiro)

**Escopo:** o sistema Motion Nodes **INTEIRO**, procurando três coisas —
**(a) BUGS** de correção · **(b) MELHORIAS** (código morto, notas latentes, gates
faltando, dívida) · **(c) PERFORMANCE** (o caminho lento, o render, a memória).

⚠️ **Isto é GRANDE.** Não tente ler tudo numa passada linear. **Decomponha por
subsistema, priorize por impacto, e verifique VOCÊ o fato decisivo de cada achado
antes de reportá-lo** ([[feedback_a_research_fanout_recurses_bound_it]]). Se o Enio
autorizar orquestração multi-agente (`ultracode`/`workflow`), esta é uma auditoria
que se beneficia MUITO de fan-out (uma lente por subsistema + verificação
adversarial). Sem essa autorização, faça direto, priorizado.

### §1.1 — O território (onde o Motion Nodes mora)

| subsistema | crates/arquivos | o que auditar |
|---|---|---|
| **Contrato** (congelado) | `ph2d-nodegraph`: `node.rs`, `cook.rs`, `gpu.rs`, `graph.rs`, `time.rs` | invariantes do `NodeManifest`/`NodeOp`; o memo `(NodeId,ScopeKey)`; o `pre`/`advance_tick`; o `checkpoint`/`restore` |
| **Avaliador CPU** (canônico) | `ph2d-eval-motion`: `lib.rs` (pump), `checkpoint.rs`, `lower.rs` | a marcha de ticks; o `CheckpointRing`; o lowering; **o custo O(N) do eval** (21 ms/262 k — dá pra paralelizar?) |
| **Cook GPU** | `ph2d-gpu-cook`: `plan.rs`, `lib.rs`, `count.rs`, `codegen.rs`, `encode.rs`, `gather.rs`, `grid.rs`, `scan.rs`, `tap.rs`, `stream.rs`, `shape.rs`, `ring.rs`, `instances.rs` | o plano (elegibilidade, o RECUO); os kernels; a grade; o tap; a residência de buffers |
| **Os nós** (~70) | `crates/ph2d-node-*` | paridade CPU↔GPU por nó; os nós SEM kernel (a família que muda contagem); clamps só-CPU |
| **Painéis** | `ph2d-panel-motion-graph`, `ph2d-panel-motion-params` | readouts, wire-march, probe, o digest de flow; custo por-frame |
| **Shell** | `shells/desktop/src/motion_state*.rs`, `render_loop/motion_bridge*.rs` | a rota GPU/pump; o gate do tool; o RENDER das instâncias |

### §1.2 — Onde SUSPEITAR (pontos quentes já conhecidos — comece por aqui)

Estes vêm da história acumulada (os handoffs) e da minha jornada. Cada um é uma
**hipótese a MEDIR/refutar**, não um bug confirmado:

**Correção:**
- **A família que MUDA CONTAGEM** (`sim.spawn`/`lifetime`/`cull`/`combine` +
  `value.attribute` + `color_ramp.t`) roda 100% na CPU (o censo mede a neve travada
  em `sim.zone`). É o maior buraco de cobertura E o lugar mais provável de um bug
  latente de correção (contagem dinâmica, id-gather).
- **A mistura de comprimentos NÃO-`1→N`** do `ReadBroadcast` (ex.: campo de 3 vs
  campo de 5): a CPU degrada por `debug_assert`+`0.0`, a GPU lê a identidade em TODO
  índice — **as duas divergem em `[0,min_len)`** e não há mecanismo de recusa nessa
  granularidade (`applicable` só vê params). Documentado, NÃO fechado.
- **A sonda GPU fica UM frame atrás** (o tap lê o cook anterior) enquanto na CPU é
  fresca — assimetria anotada, não resolvida (é ordem-de-publicação).
- **A paridade ε acumula sobre N ticks** (ADR-0127 D4) — os gates de sim provam UM
  passo de um estado semeado. Confira se algum caminho assume bit-exatidão de uma
  trajetória longa.
- **`motion.trail`** foi excluído da GPU (CHANGES_COUNT + feedback) — confira a
  correção dele na CPU.

**Performance:**
- **O eval CPU custa 21 ms/262 k** (medido). A neve de boot roda HYBRID (o interior
  no pump). Vale medir: o eval é O(N) ótimo? Os `force.*` já paralelizam (rayon)? O
  `combine`/`cull`/`spawn` são O(N) ou pior?
- **O RENDER é orçamento** (a lição do `=10`): há frustum-culling por-instância?
  overdraw? o custo por-instância do lowering/upload quando NÃO é GPU-live?
- **O custo por-frame do PAINEL** (flow digest, wire-march, probe) num documento
  grande.
- **A memória** do `CheckpointRing` (GPU e CPU) e do buffer pool.

**Melhoria/higiene:**
- **Notas que prometem waves futuras** ("child bodies land in W2" foi o
  anti-padrão na física — [[feedback_stale_comment_and_dead_code_lie]]). Grepe por
  promessas e confira se apodreceram.
- **Código morto** (o `vec_history` da física; o `_ => GpuRoute::Cpu` que já foi
  código morto aqui antes do tripwire).
- **Sobreviventes de mutação** = gates faltando. Rode mutação sobre os kernels e o
  plano.
- **Os "Aberto" espalhados** nos handoffs (o §2 do handoff de continuação lista
  vários).

### §1.3 — O MÉTODO (a disciplina desta linha — não negocie)

1. **MEÇA antes de concluir performance** (CLAUDE.md §0.0). O `=10` provou: o
   sintoma ("queda de fps") mentia sobre a causa (render, não cook). RENDER-AND-LOOK
   e instrumente o caminho REAL, não um harness que reproduz o mecanismo sem o
   contexto ([[feedback_harness_reproduces_mechanism_not_context]]).
2. **Toda afirmação de bug vira um gate red-first + MUTAÇÃO** (mate o código, exija
   vermelho, restaure com `cp`, NUNCA `git checkout`). Um achado sem gate é um
   palpite ([[reference_topic_mutation_proofs]]).
3. **Verifique contra o HEAD shipado** — um gate vermelho no seu código correto pode
   ser dívida herdada ([[feedback_a_gate_red_on_your_correct_code_may_predate_you]]).
   ⚠️ NESTA jornada o `cargo test --workspace` estava VERMELHO no HEAD em
   `no_tofu_glyphs`/`file_loc_caps`/`typos`/`fmt` (o lane do shell não roda
   `ph2d-editor-core`) — greenei ao fechar; se achar mais dívida assim, é in-scope
   greenar, mas ATRIBUA a quem introduziu.
4. **Auditoria de ≥2 LENTES** (correção + performance, ou dois revisores
   independentes) e **verificação ADVERSARIAL** (tente REFUTAR cada achado antes de
   reportar) — [[reference_topic_audit_protocol]].
5. **O oráculo modela a APARÊNCIA/o produto, nunca a regra do próprio código**
   ([[reference_topic_oracle_discipline]]); a fixture TEM de conter o fenômeno
   ([[reference_topic_fixture_discipline]]).

### §1.4 — O ENTREGÁVEL da auditoria

Um **relatório/handoff** (`docs/HANDOFF_line_gpu_nodes_auditoria_RESULTADO_*.md`)
com, por achado: **o quê · onde (`file:line`) · severidade · repro/medição ·
gate proposto**. Separe: **(a) bugs confirmados** (com repro), **(b) perf medida**
(com números e a tabela), **(c) melhorias priorizadas**. **Conserte os claros na
hora** (gate + mutação); os grandes/decisão-de-produto, NOMEIE e pare pro Enio.
Um catálogo de "pode ser" sem verificação é ruído — **verifique o fato decisivo de
cada um você**.

---

## §2 — DEPOIS da auditoria: as próximas etapas de implementação (ranqueadas)

Continue a linha (§2 do handoff de continuação tem o detalhe e o "o que MEDIR"):

1. **A NEVE de ARTISTA na GPU — a família que MUDA CONTAGEM** (o item estrutural,
   GRANDE). `sim.spawn`/`sim.lifetime`/`motion.cull`/`motion.combine` +
   `value.attribute` + `motion.color_ramp.t`. A fundação (`sim.zone`/`sim.step`/
   `sim.collide` + o recuo) já landou (ADR-0135); falta a classe de **contagem
   dinâmica** — a que a linha adiou 3× (`trail`). O `sim_state_on_gpu` exige o laço
   INTEIRO, então **nada aquém disso move a neve de boot da CPU**. MEÇA: o custo de
   reimplementar spawn/cull (contagem que muda) na GPU — é onde mora o trabalho.
2. **Subir os 2 tetos MEDIDOS** (dispatch 2-D em `grid.rs` p/ >8 M · binding maior
   p/ >11,67 M) — polimento de escala, ambos já nomeados e medidos.
3. **O cull do `motion.boids`** (~20% medido, NÃO aplicado; NÃO é o cull do collide).
4. **Próximo kernel de cobertura** — re-meça o censo (`motion_gpu_coverage.rs`).
5. **Voronoi (JFA) / soft_body-verlet (XPBD)** — território grande, algoritmo GPU
   próprio, NÃO reusa a grade.

⚠️ **A auditoria pode reordenar isto.** Se ela achar um bug de correção ou um
gargalo de perf que morde um documento real, ELE vira o item 1 — o método do censo
manda perguntar o que o documento REAL faz, não a capacidade.

---

## §3 — Inegociáveis + gotchas (o resumo; detalhe no handoff de continuação §0/§3)

- **CPU é CANÔNICA** (o replay-hash nunca roda na GPU); a GPU é paridade ε.
- **Contrato 8/2/1 intocado** (ADR-0126); tudo é metadado lateral (`GpuKernel`,
  `GridSpec`, `StateSelect`, `count_law`, `KernelResolver`).
- **`target`/`out`/`in` são palavras RESERVADAS do WGSL** — o `generated_wgsl_
  validates` (device-free) pega no `cargo test`.
- **Inner loop = SÓ `cargo check -p <crate>`**; gates 1× no fechamento, `--release`
  na RTX (os de GPU são `#[ignore]`).
- **2 LOC caps:** o de workspace (700, `crates/*/src`, mora na `ph2d-editor-core` —
  NÃO roda com `cargo test -p`) + o do shell (600). `cargo fmt` re-expande ⇒ fmt
  ANTES de medir.
- **NÃO integre, NÃO pushe, NÃO rode `ship.sh`.** Feche, atualize o handoff, PARE.

**Reporte a abertura (FASE 0, passo 8):** *"Assumi `line/gpu-nodes` em
`Worktrees/line-gpu-nodes` (HEAD `<sha>`). Pipeline GPU maduro (ADRs 0126–0135),
`=10` smokado. Tarefa: a GRANDE AUDITORIA do Motion Nodes (bugs/melhorias/perf) —
§1. Depois, a família que muda contagem (a neve de artista) — §2. Aguardo."* — e
**PARE**.
