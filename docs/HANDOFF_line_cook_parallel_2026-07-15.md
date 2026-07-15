# HANDOFF — linha `line/cook-parallel` (GPU/M5 **Fase 0**), 2026-07-15

> **Status:** FECHADA, pronta para integração. **NÃO integrei, NÃO pushei** (§0.7 — só por ordem
> explícita do Enio, via integrador dedicado). Ramificada de `main` (`12ccaecd`).
> Plano: [`docs/plans/2026-07-gpu-resident-node-pipeline.md`](plans/2026-07-gpu-resident-node-pipeline.md).

## O que landou (Fase 0 — paralelizar o cook na CPU, SEM tocar o contrato)

O cook do grafo rodava **CPU single-thread**. Esta linha o paraleliza com rayon, **bit-idêntico**.

- **Slice A — o substrato + retrofit.** `ph2d_nodegraph::attr::par_build(n, f)` — **um** builder
  auditado: acima de `PAR_THRESHOLD` (8192) o map por-elemento roda em `into_par_iter().map().collect()`
  (índice preservado → bit-idêntico ao serial); abaixo, serial (o demo de boot minúsculo não toca o
  pool nem aloca — o `paused_no_alloc` segue verde). **16 nós retrofitados** para usá-lo:
  `motion.{grid,move,transform,oscillator,rotate,scale,twist,bend,look_at}` + `force.{wind,drag,attractor,vortex,curl}`.
  (`grid` é gerador row-major → posição `i` = `(i/cols, i%cols)`.)
- **Slice B — o marshalling.** `lower_to_instances_onto` (Stream→`Vec<RenderInstance>`, um gather +
  `sin_cos` por instância) usa `par_extend` acima do threshold (ordem preservada, sem Vec temporário).
- **Slice C — o golden que faltava.** `crates/ph2d-eval-motion/tests/cook_determinism.rs`: cozinha
  `grid→oscillator→move→transform` a **25.600 instâncias** (acima do threshold → o caminho paralelo
  RODA) e afirma (1) **reprodutibilidade** — dois cooks byte-idênticos (uma redução float paralela
  divergiria run-a-run) e (2) um **FNV pinado** (`0x1aa7…713f`, capturado em Linux; HR-5 → esperado
  cross-OS estável). **Antes desta linha NÃO havia golden no nível do cook** — só o `transform_determinism`
  do ECS, que não vê as colunas do Stream. Uma reordenação do rayon num nó passaria batido; agora não.

## Determinismo — a disciplina (a razão de tudo ser bit-idêntico)

- `par_build` é um **map puro** (sem redução): o rayon escreve o elemento `i` no slot `i`, então o
  resultado é o mesmo do serial. Gate em `attr.rs` (`par_build_is_bit_identical…` + `…preserves_index_order`).
- **Reduções e leituras indexadas ficaram SERIAIS de propósito** (o subagente aplicou a regra certa):
  o `r_max`/`x_extent` (fold-max sobre todas as instâncias) do twist/bend; os passos in-place
  semi-implícitos do `integrate`/`spring` que leem uma linha de estado PAREADA `state[pairing[i]]`
  (indireção de índice, não um map). Esses hot loops **não** paralelizaram — correto e conservador; o
  ganho da sim vem das **forças** (que paralelizam), não do passo do integrador.
- A CPU segue o caminho **canônico**; `transform_determinism` + `c9_replay` intactos.

## Perf (medido em `--release`, mesma cadeia sob 1 vs 32 threads rayon)

**Cook de ~500k instâncias (`grid→oscillator→move→transform`): 16,07 ms (1 thread) → 4,93 ms (32) = 3,3×.**
Põe 500k num chain O(N) **dentro** do orçamento de 60fps (16,67 ms) só no cook. **Não é 10-20×** — os
maps têm pouquíssimo flop por elemento (bandwidth-bound; 32 threads ≠ 32×). O multiplicador grande é a
**Fase 1 (GPU)**, não mais cores. Sonda: `cook_500k_timing` (`#[ignore]`), comando no doc-comment dela.

## Integração (para o integrador do Enio)

- **Foundational:** `ph2d-nodegraph` (`Cargo.toml` +rayon, `attr.rs`) + `ph2d-eval-motion` **não** casam
  o padrão drop-crate → `scripts/foundational-integrate.sh` força **`cargo check --workspace`** (já roda
  verde aqui). Rebase: conflito **só** esperado em `Cargo.lock` (rayon) — resolve regenerando.
- **Contrato congelado INTOCADO** — `architecture_contract_surface` verde (8/2/1). O `attr::par_build`
  é um método/const inerente do substrato, não mexe em `NodeOp`/`NodeManifest`/`OpResolver`.
- **`rayon` no `ph2d-nodegraph`** contraria o comentário "dependency-free" do `Cargo.toml` — **atualizei o
  comentário** (não silenciei a cerca). Decisão do plano aprovado.
- **ADR-0122 número TENTATIVO** — [`0122-gpu-resident-node-engine-appends-the-frozen-contract.md`](architecture/decisions/0122-gpu-resident-node-engine-appends-the-frozen-contract.md).
  Linhas paralelas (audio/vector) podem ter reivindicado 0122; renumerar na integração se colidir.

## Fase 1 — o próximo journey (DESENHADO, aguarda "aprova" do Enio)

**ADR-0122 (PROPOSTO):** o motor GPU-resident estende o contrato de forma **append-only** —
`NodeManifest` ganha `gpu_kernel: Option<GpuKernel>` (contagem **8→9**, o ADR autoriza o bump do gate);
`NodeOp` fica em 2; a **CPU permanece canônica** (replay-hash), a GPU é performance reconciliada por
**tolerância** (float cross-vendor não é bit-reproduzível). **Nada do contrato foi tocado nesta linha.**
Abrir `line/gpu-nodes` (Fase 1) é ordem do Enio, após aprovar o ADR.

## Gates verdes no fechamento

`cargo check --workspace` · `architecture_contract_surface` (8/2/1) · **218 nextest** nos crates tocados
(1 skip = a sonda de perf) · clippy `--all-targets` limpo · typos limpo · o golden `cook_determinism`.

## Deferido (nomeado, não escondido)

- **Cook graph-level** (sinks disjuntos em paralelo) — semanticamente sólido (cache/`rev_counter`
  deriváveis), bloqueado só pela recursão `&mut self`; maior risco + troca o memo cross-sink. Fica para
  depois de a Fase 1 assentar.
- **`integrate`/`spring` step loops** — serial por design (leitura pareada indexada); paralelizar exigiria
  reescrever como scatter/gather, risco de determinismo. As forças já paralelizam.
- **Golden cross-OS** — o FNV foi capturado em Linux; se o CI cross-OS reclamar, é HR-5 real ou re-pin.
