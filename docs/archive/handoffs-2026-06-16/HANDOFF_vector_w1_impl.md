═══════════════════════════════════════════════════════════════════
HANDOFF — Vector Module · RETOMADA W1 (2026-05-29)
Foco: FINALIZAR A FUNDAÇÃO do Vector → vira drop-crate isolado (ADR-0075)
═══════════════════════════════════════════════════════════════════

CONTEXTO (mudou desde o handoff anterior):
- **Sprite W1 FECHADO** → `crates/ph2d-render/` está **LIBERADO** (não mais reservado).
  Isso DESBLOQUEIA o H5/M3 — a última costura foundational do Vector.
- **T1.4 (Levien cubic fit) DONE** (`ae64a0f` + `4001160`, com golden + Send/Sync gates).
  NÃO refazer. T1.1-T1.3/T1.5/T1.7 + audit Blocos 0/1/2 também fechados.
- Norte: [ADR-0075](architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)
  — monorepo Rust + ECS-decoupling. Velocidade: DIRETRIZ §6.6. Cap: **≤3 agentes**.

OBJETIVO DESTA RETOMADA: fechar o **H5/M3** (consolidação foundational) → depois disso
o Vector não toca mais nenhum crate compartilhado e sai da lista de fontes de conflito.

───────────────────────────────────────────────────────────────────
SANITY CHECK
───────────────────────────────────────────────────────────────────
  bash scripts/slot-seed.sh impl-vector     # clone CoW warm → imprime CARGO_TARGET_DIR=<path>
  # prefixe CADA cargo com esse path (Bash-tool não persiste env):
  #   CARGO_TARGET_DIR=<path> cargo check -p ph2d-vector-doc
  git log --oneline | grep -i vector | head   # confirma 4001160 (T1.4) na história
  CARGO_TARGET_DIR=<path> cargo nextest run -E 'rdeps(ph2d-vector-doc)' --cargo-profile ci-test  # baseline

───────────────────────────────────────────────────────────────────
SUA PASTA (vector isolado) + 1 touchpoint foundational AUTORIZADO
───────────────────────────────────────────────────────────────────
  crates/ph2d-vector-doc/ · ph2d-vector-traits/ · ph2d-brush-traits/ · ph2d-tool-vector-pen/
  shells/desktop/src/render_loop/vector_pen_bridge.rs   (seu tool-bridge)
  shells/desktop/src/input_dispatch/vector_pen_input.rs
  ⚠️ **crates/ph2d-render/src/camera.rs** — foundational, **AUTORIZADO p/ o H5/M3** (Sprite
     liberou; nenhum outro agente em ph2d-render agora). É a ÚNICA edição foundational desta
     retomada. Qualquer OUTRO arquivo de ph2d-render / editor-core / shell plumbing → PARE e
     reporte ao Coord.

───────────────────────────────────────────────────────────────────
TASK 1 (PRIORIDADE — foundational) — H5/M3: Camera2d como fonte única da projeção
───────────────────────────────────────────────────────────────────
Problema: o shell reimplementa a projeção câmera→tela à mão em
`vector_pen_bridge.rs:196` (`world_to_screen_affine`), que PODE divergir do
`Camera2d::screen_to_world`. A matemática já existe em `Camera2d` — falta consolidar.

  1. Em `crates/ph2d-render/src/camera.rs`: adicione
       `pub fn world_to_screen_affine(&self, window: WindowSize) -> vello::kurbo::Affine`
     (ph2d-render JÁ depende de `vello = "0.8"` → `Affine` é OK; sem dep nova).
     **Derive da MESMA base** que `world_to_screen`/`screen_to_world` (k = window.h /
     height_world.max(1e-6); translate(center_screen) · scale_non_uniform(k, -k) ·
     translate(-center)). Ideal: componha de primitivas já existentes p/ NÃO poder divergir.
  2. Teste no mesmo módulo (espelhe `world_to_screen_round_trips_screen_to_world`):
     transforme alguns pontos-mundo pela Affine e compare com `world_to_screen` (mesma
     saída ± epsilon). Determinismo HR-5: se usar trig, via libm (não deve precisar — é
     escala/translação linear).
  3. Em `vector_pen_bridge.rs`: **delete** o `fn world_to_screen_affine` local (linha ~196)
     e chame `camera.world_to_screen_affine(window_size)` no lugar (linha ~64).
  4. Verifique: `cargo check -p ph2d-render -p ph2d-tool-vector-pen` + o teste novo verde.

  DoD: zero duplicação da projeção; shell não pode mais divergir do Camera2d. Reporte a
  assinatura final ao Coord (é foundational — eu confiro no ship).

───────────────────────────────────────────────────────────────────
TASK 2 — fechamento W1 (depois do H5/M3)
───────────────────────────────────────────────────────────────────
  - LOW carry-overs §3.4 do audit (tolerância dedup 12px acoplada ao close-path · cursor
    crosshair no Pen ativo · atalho de teclado P · cap interativo conta só vertices) — o
    Coord prioriza quais entram em W1 vs W2; pergunte antes de pegar.
  - Mini-round de re-audit (1 round, lentes que pegaram alvo grande — NÃO round do zero;
    a auditoria 6-lente já cobre o grosso).
  - SMOKE (Enio, fim de W1): Pen pill → 3 cliques = triângulo; 4º perto do 1º = fecha;
    Esc cancela/limpa; sem `.ph2d-vector` no root; click rejeitado → toast.

DEFERIDO p/ W2 (NÃO nesta retomada): **T1.6 CRDT undo** (Ctrl+Z) — o plano coloca undo
via CRDT no W2 Day-14 (começar LWW per ADR-0057, migrar se necessário). Não abra agora.

───────────────────────────────────────────────────────────────────
LOOP (DIRETRIZ §6.6) + GIT + ANTI-COLISÃO
───────────────────────────────────────────────────────────────────
  - Inner loop = SÓ `cargo check -p` (prefixado com o CARGO_TARGET_DIR do slot) ou
    `scripts/cargo-check-narrow.sh`. ZERO test/clippy/auditor POR TASK.
  - Gate 1× no fechamento: `scripts/nextest-impacted.sh` (já força o golden de determinismo
    via `binary(transform_determinism)`) + clippy --all-targets + ≥2 lentes sobre o diff.
  - git: `git add -- <seus paths>` (nunca -A / stash); `git commit --no-verify -m "msg" -- <paths>`;
    race-guard (`git diff --cached --name-only` + `--diff-filter=U`) antes de comitar.
  - NÃO pusha (Coord faz ship). NÃO use o `target/` default.
  - Precisou de algo fora da sua pasta (além do camera.rs autorizado)? PARE e reporte ao Coord.
  - UI strings em INGLÊS.

REPORTE ao fechar: "Vector H5/M3 pronto, commit <sha>, Camera2d::world_to_screen_affine +
teste de consistência verdes, shell consolidado. [W1 status]."
═══════════════════════════════════════════════════════════════════
