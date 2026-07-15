# HANDOFF — linha `line/gpu-nodes` (GPU/M5 **Fase 1 / F1.1**), 2026-07-15

> **Status:** FECHADA, pronta para integração. **NÃO integrei, NÃO pushei** (§0.7 — só por
> ordem explícita do Enio, via integrador dedicado). Ramificada de **`line/cook-parallel`**
> (`8310d3cb`) — o integrador landa a Fase 0 primeiro, depois esta.
> ADR: [`0122`](architecture/decisions/0122-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md) (ACEITO; agora com stamp de implementação).
> Briefing que esta linha executou: [`HANDOFF_line_gpu_nodes_fase1_briefing_2026-07-15.md`](HANDOFF_line_gpu_nodes_fase1_briefing_2026-07-15.md).

## O que landou (F1.1 — o chain simples 100% na GPU, renderer sem readback)

**O motor existe e o alvo da fatia está batido:** `grid → oscillator → move → output` cozinha
inteiro na GPU (compute passes num **único submit**), o **lowering é um compute pass** que
escreve direto no layout `RenderInstance` (buffer `VERTEX|STORAGE`), e o renderer **binda esse
buffer como instance buffer** — zero readback, zero marshalling CPU.

**Perf medida (RTX, `--release`): cook de 500k instâncias = 1,0 ms/frame** (encode + submit +
espera TOTAL da GPU, por frame). Baseline da Fase 0 na CPU: 4,93 ms (32 threads) — e o número
da GPU ainda inclui o que na CPU seria o marshalling. O orçamento pros milhões da Fase 4 está
aberto: a 500k o custo é dominado por overhead fixo, não pelo N.

### As peças (onde tudo está)

1. **`ph2d_nodegraph::gpu`** (módulo irmão novo; contrato congelado INTOCADO — gate 8/2/1 verde):
   `GpuKernel { wgsl, wgsl_lib, bindings, params, source_count, applicable }` ·
   `ColumnBinding { column, dim, access, identity }` ·
   `ColumnAccess { Read, Write, ReadWrite, ReadWriteExisting }` · trait `KernelResolver`
   (espelho do `OpResolver`) · `GpuKernel::PASSTHROUGH`.
   - **`ReadWrite` materializa** a coluna ausente da identidade (o `base_vec2` do
     `apply_channel_delta`); **`ReadWriteExisting` dropa o write** quando a coluna não existe
     (o pattern-match do `motion.move`) — ausência significa o MESMO nos dois caminhos.
   - `applicable: fn(&params) -> bool` = a cobertura parcial honesta: kernel estático não
     cobre todo o espaço de params (ex.: oscillator só X/Y) → o plano recua pra CPU, nunca
     responde errado.
2. **`NodeRegistry::register_gpu_kernel(id, kernel)`** + `impl KernelResolver` — o canal
   lateral do ADR-0122, idêntico ao `register_ui`. Opt-in; 92 nós intocados.
3. **Kernels registrados (4):** `motion.grid` (gerador; `source_count` espelha o
   `param_as_count` + cap EXATOS — o dispatch tem que igualar o count da CPU) ·
   `motion.oscillator` (X/Y; waveform HR-5 portada no `wgsl_lib` — parábola + correção Capens) ·
   `motion.move` · `motion.output` (= `PASSTHROUGH`, zero pass).
4. **Crate nova `ph2d-gpu-cook`** (foundational, isolada de propósito):
   - `stream.rs` — `GpuStream` (colunas `Arc<wgpu::Buffer>` + `BufferPool` com reclaim por
     refcount: steady state não aloca) · `upload_stream` (a fronteira CPU→GPU, 1 crossing).
     **Colunas são imutáveis**: todo write vai pra buffer novo do pool; pass-through = mesmo
     `Arc`. O ping-pong é implícito, sem barreira manual.
   - `codegen.rs` — gera o módulo WGSL por (kernel × colunas presentes): bindings + uniforms
     (`count`/`playhead`/params) + helpers `read_*`/`write_*` (coluna ausente → função
     constante com a identidade). `plan_bindings` é a MESMA função que monta o bind group —
     módulo e bind group não podem divergir.
   - `lower.rs` — o gêmeo compute do `lower_to_instances_onto`: 46 words por instância via
     `array<u32>` + `bitcast` (o `#[repr(C)]` do `RenderInstance` tem `anchor` no offset 68,
     inalinhável em WGSL struct). Defaults ausentes = os MESMOS da CPU (`default_size`/
     `default_uv` vêm por uniform).
   - `lib.rs` — **`plan()`**: reivindica o maior **sufixo** GPU do chain; o que sobra em cima
     cozinha no `Cook` REAL da CPU (toda a semântica canônica: memo, `pre`, scopes, driven
     params) e sobe **uma vez** pelo `upload_stream`. `GpuPlan { boundary, stages }`,
     `is_fully_gpu()`, `dispatching_stages()` (o assert de "a otimização DISPARA").
     **`GpuCook::cook()`**: pipelines cacheadas por `(tipo, assinatura-de-presença)` — slider
     e playhead são uniforms, NUNCA recompilam; só rewire que muda presença de coluna minta
     pipeline nova. `read_instances()` = o readback DELIBERADO (gates/canônico), fora do hot path.
5. **`SpriteRenderer::render_with_streams(...)`** — `render_with_extra` + `gpu_extra:
   Option<(&wgpu::Buffer, u32)>`: um draw extra no MESMO passe, material do atlas + blend
   default (a run exata que o stream Motion produz na CPU). Só no caminho plain (frame com
   clip/mask ignora, como o subrect).
6. **Shell:** `MotionState.{gpu_cook, gpu_live, gpu_enabled}` + desvio no `motion_bridge`
   (**`PH2D_GPU_COOK=1`**, single-sink, sem time-scopes, plano fully-GPU → cozinha na GPU e
   PULA o pump) + `present.rs` binda `gpu_cook.instances()`. Fallback = caminho CPU intacto,
   byte-idêntico.

### Smoke (pronto — é rodar e clicar na tool Motion)

```
cd Worktrees/line-gpu-nodes && PH2D_GPU_COOK=1 PH2D_GPU_COOK_DEMO=1 cargo run --release
```

`PH2D_GPU_COOK_DEMO=1` troca o boot document por `grid 512×512 (262k) → oscillator (onda Y
viajante) → move → output` — todo coberto por kernel, então cozinha 100% na GPU (auto-play na
entrada da tool; dê zoom out pra ver o campo inteiro). Sem os flags, nada muda (app launch
verificado limpo 15 s com e sem). O demo é denso de propósito: 262k quads unitários lado a
lado lêem como um tecido ondulando.

## Gates verdes no fechamento

- `cargo check --workspace --all-targets` (foundational-integrate vai forçar; já verde aqui).
- `architecture_contract_surface` **8/2/1** — o canal lateral NÃO tocou o contrato.
- `cook_determinism` (golden da Fase 0) + `transform_determinism` — a CPU não regrediu.
- **Paridade GPU-vs-CPU (ε), medida na RTX:** full-GPU max |Δpos| = **4,4e-4** (budget 2e-3,
  derivado: FMA numa fase ≤256 → ~3e-5 no frac × amplitude) · **híbrido bit-exato** (Δ = 0;
  move é aritmética exata) · re-cook mesmo device **byte-idêntico**. O gate AFIRMA
  `is_fully_gpu` + `dispatching_stages == 3` antes de comparar — um fallback silencioso
  compararia CPU com CPU e ficaria verde com o motor morto.
- **naga sobre TODO módulo gerável** (kernel × todo subconjunto de colunas + lowering × 32
  variantes) — typo de WGSL morre em qualquer lane de CI, não no primeiro dispatch.
- `plan_analysis` (5): chain coberto reivindicado inteiro · boundary por param não coberto ·
  boundary por param DIRIGIDO · boundary por nó sem kernel · input solto = chain vazio GPU.
- nextest nos crates tocados (386) + shell (571) + `file_loc_caps` · clippy `--all-targets`
  zero · typos zero.

Comando dos gates de GPU (rodam headless; precisam de adapter):
```
cd Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture
```

## Gotchas (custaram iteração aqui; a Fase 2 VAI esbarrar neles)

- **Arredondamento CPU↔GPU divergem no meio-caminho:** `f32::round` do Rust é half-AWAY
  (2,5→3); `round()` do WGSL é half-EVEN (2,5→2). Um enum-por-param roteado por `round`
  (channel/wave) escolheria RAMOS diferentes nos dois lados. Fix no oscillator: `osc_round`
  (half-away explícito) pro wave + comparação `< 0.5` pro channel (concorda com o round da CPU
  em todo valor que o `applicable` admite). **Todo nó portado na Fase 2 com param-enum precisa
  disso.** (Também em `project-memory/feedback_cpu_gpu_rounding_conventions_diverge.md`.)
- **`array<vec3<f32>>` tem stride 16** — upload de Vec3 apertado (12 B) desindexa a partir do
  elemento 1. O `upload_stream` já padda; se a Fase 2 criar colunas Vec3 na GPU, o
  `element_stride` é a única porta.
- **O layout do `RenderInstance` NÃO é espelhável em struct WGSL** (`anchor` vec2 @ 68). O
  lowering escreve words em `array<u32>` + `bitcast`; o teste
  `instance_words_matches_render_instance_size` pina 46×4 = `size_of`.
- **`source_count` tem que ser o `param_as_count` EXATO** (floor + clamp 2^24 + produto
  saturado) — dispatch ≠ count da CPU mata a paridade na largada. O corpo WGSL do grid repete
  o mesmo floor/clamp pro `cx`/`cy`.
- **Borrow do `entry().or_insert_with()`**: não segure o `&mut` do cache de pipelines através
  de `pool.acquire`/`uniform_slot` — insira primeiro, re-busque depois (2 sítios em `lib.rs`).

## Aberto (F1.2 / Fase 2 — o próximo journey)

1. **Wire do híbrido no shell** — o MOTOR já cozinha prefixo-CPU/sufixo-GPU (gate híbrido
   verde), mas o bridge só ativa em plano fully-GPU. Falta: cozinhar o boundary via
   `pump.cook` (mantém memo/`pre`/ring) e passar o stream pro `GpuCook::cook` — atenção à
   interação com `ticks_owed` quando o prefixo tem nós sequenciais.
2. **Fase 2 — portar os nós hot** (um `register_gpu_kernel` por vez, cada um com paridade ε):
   ordem por impacto: `transform`/`rotate`/`scale`/`twist`/`bend`/`look_at` · `wiggle`/`tint`
   (ramp?) · oscillator canais Rotation/Size (exige binding de coluna selecionável OU 2º
   kernel — decisão de design pequena) · forças + `integrate` (ping-pong de estado ENTRE
   frames — o `pre` na GPU; desenho no plano §3 Fase 1 "Estado").
3. **Readouts/probe do painel no modo GPU** — o caminho GPU não alimenta o memo da CPU; os
   cards ficam sem leitura sob `PH2D_GPU_COOK=1`. Ou readback esparso on-demand (1 nó
   sondado), ou aceitar em preview. Fase 4 decide o default do flag.
4. **Multi-sink + time-scopes no caminho GPU** (hoje recuam pra CPU inteiros — correto, só
   não acelerado).
5. **Zero-alloc do frame GPU**: o pool recicla buffers, mas bind groups são criados por
   frame. Medir antes de otimizar ([[feedback_measure_perf_symptom_scale]]).
6. **Fases 3–4 do plano** (JFA voronoi, spatial hash boids, renderer consumindo colunas
   direto sem o passe de lowering) — journeys futuros.

## Integração (pro integrador do Enio)

- **Base:** `line/cook-parallel` (a Fase 0 vem junto; landar ela primeiro, depois esta —
  fast-forward natural).
- **Foundational tocado:** `ph2d-nodegraph` (+`gpu.rs`, 1 linha no lib.rs) · `ph2d-node-registry`
  (campo+métodos aditivos) · `ph2d-render` (`render_with_streams` aditivo; `render_with_extra`
  delega) · 4 node crates (kernel + 1 linha no `register`) · shell (bridge/present/motion_state,
  +dep `ph2d-gpu-cook`) · crate nova `ph2d-gpu-cook` (drop-in, glob membership).
- **Conflitos esperados:** `Cargo.lock` (crate nova — regenerar) · número do ADR-0122 se outra
  linha reivindicou (renumerar; o stamp de implementação vai junto) · registry-init NÃO mudou
  (register() das crates é o mesmo entry point).
- **Contrato congelado:** intocado (gate 8/2/1 verde) — nenhum escape §1.5.5 nesta linha.
