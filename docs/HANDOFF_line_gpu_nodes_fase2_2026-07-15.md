# HANDOFF — linha `line/gpu-nodes` (GPU/M5 **F1.2 + Fase 2**), 2026-07-15

> **Status:** FECHADA, pronta para integração. **NÃO integrei, NÃO pushei, NÃO rodei `ship.sh`**
> (§0.7 — só por ordem explícita do Enio, via integrador dedicado). Construída em cima da F1.1
> (`e7605cfd`), no MESMO branch `line/gpu-nodes` (stack de commits — o integrador landa
> Fase 0 → F1.1 → F1.2/Fase 2 pela fronteira de commit; ver §Integração).
> **Autor:** o agente da continuação (a pedido do Enio, "linha motion nodes, doc na linha e não no main").
> Briefing que esta linha executou: [`HANDOFF_line_gpu_nodes_fase2_briefing_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase2_briefing_2026-07-15.md).
> Documento central herdado: [`HANDOFF_line_gpu_nodes_fase1_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase1_2026-07-15.md). ADR: [`0122`](architecture/decisions/0122-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md).

---

## O que landou

**A missão inteira do briefing, na ordem — mas as fatias de Fase 2 primeiro** (nós mecânicos,
cada um com seu gate de paridade, menor risco), **depois a F1.2** (cirurgia no pump/shell, com
contexto quente). Reordenar é a decisão do padrão-ouro; a regra "feche cada fatia antes da
próxima" foi honrada (cada nó é uma fatia fechada, gateada).

### Fase 2 — 6 nós hot portados para kernels WGSL (contrato **8/2/1 intocado**)

Cada um: `register_gpu_kernel` no crate do nó (drop-crate isolation), + gate naga (todo
subconjunto de colunas) + gate de paridade ε na RTX. **Ordem = impacto do plano §3.**

| Nó | commit | binding-chave | ε medido (RTX) | nota |
|---|---|---|---|---|
| `motion.transform` | `6325a3a8` | `P` ReadWriteExisting | 3,8e-6 | afim puro `p·s+o` blendado por falloff |
| `motion.rotate` | `6325a3a8` | `rot` ReadWrite (id 0) | bit-exato | soma escalar; a sin/cos é do lowering |
| `motion.scale` | `6325a3a8` | `size` ReadWrite (SIZE_IDENTITY) | bit-exato | `size·(1+(amount-1)·falloff)` |
| `motion.falloff` | `86a2fe35` | `falloff` ReadWrite, `P` Read | 1,9e-6 | field×curve; enums por round-half-away |
| `motion.tint` | `86a2fe35` | `tint` ReadWrite (branco) | bit-exato | **Solid only** (ver Gotchas) |
| `motion.wiggle` | `86a2fe35` | `P` ReadWrite (canal X/Y) | 1,9e-6 | **noise integer-hash BIT-EXATA** |

**twist/bend/look_at NÃO portados** — todos **multi-input** (o `plan().eligible()` já os recusa a
uma fronteira CPU, `inputs.len() > 1`), e twist/bend ainda carregam uma **max-reduction**
(`r_max`/`x_extent`) que não cabe num kernel por-elemento. Deixá-los na CPU é correto (o plano
recua; a F1.2 acelera o sufixo deles se houver). Não inventei um passe de redução (seria extensão
do motor — desenhe antes; ver Aberto).

### F1.2 — o híbrido no shell (CPU-prefixo / GPU-sufixo) — commits `f877b8a0` + `88326d00` + `8c018447`

O MOTOR já cozinhava híbrido (gate `the_hybrid_boundary_chain_matches_the_cpu_within_epsilon`
verde); faltava o shell ligar o caso `plan.boundary == Some(...)`.

- **`ph2d-eval-motion` (foundational):** método novo no pump
  `advance_or_scrub_to_node_scoped(node, tick, …)` + `boundary_stream()`. Cozinha o nó de
  fronteira no `Cook` **persistente** (memo + `pre` feedback + a marcha de ticks — um prefixo
  sequencial sim'a certo e o scrub é bit-exato) e guarda o **stream cru** do nó; **sem lowering**
  (é da GPU agora). A marcha forward/scrub, o ring e o `pre`-advance são o **MESMO caminho** do
  pump de sinks, via um `CookTarget { Sinks | Boundary }` privado — as duas portas não divergem
  ([[feedback_two_doors_to_the_same_question_diverge]]). Os métodos públicos de sink viraram
  wrappers finos; o zero-alloc do frame pausado + os testes de checkpoint/scrub seguem verdes.
- **Shell:** a **decisão de rota** (`Cpu | FullyGpu | Hybrid`) é função **pura** em
  `shells/desktop/src/render_loop/motion_bridge_gpu.rs`, unit-testada headless (briefing §4 — a
  paridade do dispatch já é gateada no motor). `gpu::cook_gpu` faz plano+rota+dispatch; o
  `motion_bridge::dispatch` ficou uma costura de uma linha. O híbrido **marcha o pump até a
  fronteira e RETORNA** (não roda o loop de sinks no mesmo tick — senão o pump early-returna e
  `instances` fica com o lowering da fronteira; uma falha de GPU renderiza nada neste frame em vez
  de corromper o relógio do pump).
- **Fronteira só-lowering recua:** um boundary cujo único stage GPU é o `output` passthrough vira
  CPU (upload do stream de sink só pra baixar — sem ganho de compute).

## Gates verdes no fechamento (rodados 2026-07-15, este worktree)

- `cargo check --workspace --all-targets` ✓
- `architecture_contract_surface` **8/2/1** ✓ (o canal lateral não tocou o contrato congelado)
- `cook_determinism` (Fase 0) + `transform_determinism` (ECS) ✓ — o refactor do pump não regrediu
- **Paridade ε na RTX** (`--release --ignored`): 9/9 — os 6 nós novos + híbrido + full-GPU +
  re-cook byte-idêntico. Máx |Δpos| dos novos ≤ 3,8e-6 (orçamento 2e-3). **naga** valida todo
  subconjunto de colunas de todo kernel (inclui a bitcast/u32 da noise).
- **nextest** nos 6 crates-nó + `ph2d-gpu-cook` + `ph2d-eval-motion` + `ph2d-ecs` + **shell 566** ✓
- `clippy --all-targets` (crates tocados + shell) **zero** · `typos` **zero** · `file_loc_caps` ✓
  (o bloco GPU foi extraído pro módulo `gpu` — `motion_bridge.rs` 628→547) · `cargo machete` limpo
- Perf na RTX (full-GPU, `--release`): 500k ≈ **1,0–1,2 ms/frame**; **2M ≈ 4,0 ms/frame**
  (`gpu_cook_millions_timing`, o demo `DEMO=1`). Domina overhead fixo, não o N.

## Smoke (pronto — rodar e clicar na tool Motion)

**Full-GPU (F1.1, `1250×1600` = 2.000.000 instâncias, 100% GPU):** cozinha em
**~4 ms/frame** na RTX (probe `gpu_cook_millions_timing`) — os "milhões a 60fps"
do roadmap, com folga. Zoom out pra ver o campo inteiro.
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=1 cargo run --release -p ph2d-host-desktop
```

**Híbrido (F1.2, 129k):** `grid → oscillator(Rotation, sem kernel = fronteira CPU) →
oscillator(Y) → scale → output` — o campo **ondula em Y** (GPU) **E gira** (CPU), calculados nos
dois lados da costura CPU↔GPU:
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=2 cargo run --release -p ph2d-host-desktop
```
Sem os flags, nada muda. Gate headless prova que `DEMO=2` **planeja como híbrido** de verdade
(`motion_state_tests::the_hybrid_demo_document_plans_as_a_cpu_boundary_with_a_gpu_suffix`).

Gate de GPU (headless, precisa de adapter):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture
```

## Gotchas (custaram iteração; a próxima fatia VAI esbarrar)

- **`round` CPU↔GPU diverge no meio-ponto** (Rust half-away, WGSL half-even). Todo param-enum
  roteado por round cai nisso. Usei o helper `*_round` (half-away, = `osc_round`) em `falloff`
  (shape/curve); em `wiggle`/oscillator o canal usa `< 0.5` (concorda com round no domínio
  admitido). [[feedback_cpu_gpu_rounding_conventions_diverge]].
- **noise integer-hash na GPU:** `bitcast<u32>(i32)` (== Rust `as u32`, NÃO `u32(x)` que é value-cast
  e diverge em negativos); u32 do WGSL **wrappa mod 2³²** como `wrapping_*`; constantes com sufixo
  `u`. Se o caminho inteiro divergir 1 célula, a noise pula pra um valor pseudo-aleatório TOTALMENTE
  diferente (O(amplitude), não ε) — o gate na RTX é o árbitro (deu 1,9e-6 = portou bit-exato).
- **identidade constante NÃO expressa fallback posicional:** o Gradient do `tint` chaveia por
  `Index/(Count-1)` com fallback `f32(i)`/`f32(count)` quando ausentes — a `ColumnBinding.identity`
  é uma CONSTANTE, não `f32(i)`. Por isso o `tint` é **Solid only** (`applicable` em `mode`), e
  Gradient recua pra CPU. Estender o binding com "identidade posicional" é motor (Aberto).
- **testar com a coluna AUSENTE:** o grid não emite `falloff`/`size`/`rot`/`tint`, então os gates
  já rodam com o alvo ausente (o caso comum). Escolher `ReadWrite` (materializa) vs
  `ReadWriteExisting` (dropa o write) = LER o braço do match da CPU, não chutar
  ([[feedback_a_gate_only_proves_what_its_fixture_contains]]).
- **duas portas divergem:** o pump de sink e o de fronteira compartilham UM `CookTarget` de
  propósito — a marcha/ring/`pre` não podem ter dois donos. Se você adicionar um 3º consumidor do
  cook, estenda o enum, não copie a marcha.

## Aberto (a próxima fatia / journeys futuros)

1. **`motion.integrate`/`spring` — estado na GPU** (a fatia 3 do briefing): o `pre` vira ping-pong
   de buffer ENTRE frames. É **extensão do MOTOR** (o plano hoje refuta `pre` na eligibility), não
   um port de kernel. **Desenhe antes de codar** — pode ser o journey seguinte.
2. **Reduções na GPU** (destrava twist/bend): um passe de redução (`r_max`/`x_extent`) no motor.
   Também extensão; desenhe antes. (Alternativa honesta hoje: ficam na CPU, o híbrido acelera o
   resto.)
3. **Multi-input na GPU** (twist/bend/look_at/combine): o `eligible()` recusa `inputs.len() > 1`.
   Precisa o sequenciador aceitar 2+ streams de entrada.
4. **Gradient do tint + canais Rotation/Size do oscillator/wiggle:** exigem binding de coluna
   selecionável por param OU identidade posicional (tint) — decisão de design pequena, mas é motor.
5. **Readouts/probe do painel no modo GPU** (o caminho GPU não alimenta o memo da CPU) — Fase 4.
6. **Fases 3–4 do plano** (JFA voronoi, spatial-hash boids, renderer consumindo colunas cru) —
   journeys dedicados.

## Integração (pro integrador do Enio)

- **Base:** `line/cook-parallel` (Fase 0) → F1.1 → **esta** (F1.2/Fase 2), tudo em `line/gpu-nodes`,
  em ordem de commit. `e7605cfd` (fim da F1.1) → `8c018447` (HEAD). Fast-forward natural.
- **Foundational tocado:** `ph2d-eval-motion` (método de pump novo + refactor `CookTarget`, aditivo
  — API pública de sink intacta) · 6 node crates (kernel + 1 linha no `register`) · `ph2d-gpu-cook`
  (só testes + 6 dev-deps no `Cargo.toml`) · shell (`motion_bridge` seam + módulo `gpu` novo +
  `motion_state` demo). Contrato **8/2/1** intocado — nenhum escape §1.5.5.
- **Conflitos esperados:** `Cargo.lock` (6 dev-deps novas — regenerar). O resto é aditivo.
- **NÃO precisa** de ADR novo (o ADR-0122 já cobre o canal lateral; nenhum contrato descongelado).
