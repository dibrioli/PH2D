# HANDOFF → novo Coordenador — KTX2 ⊕ Image IO em paralelo (2026-05-31)

**De:** Coord saindo (fechou a fundação multi-agente + shipou 16 commits).
**Para:** novo Coordenador único (modelo 1 Coord + **≤3 implementadores**).
**Plano:** levar **KTX2** e **Image IO (AVIF)** em paralelo.

---

## §0 — LEIA ISTO PRIMEIRO (1 tela)

- **Modelo:** 1 Coordenador + **≤3 agentes** (RAM 8 GiB — teto aceito). Norte:
  [ADR-0075](architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)
  — monorepo Rust + ECS-decoupling, **sem plugins runtime/WASM**.
- **Leitura mínima (não leia docs inteiros):** [`CLAUDE.md`](../CLAUDE.md) §0-§2 (roteador) +
  [`DIRETRIZ.md §6.6`](IntegracaoMultiAgente/DIRETRIZ.md) (stack de velocidade). Posse viva:
  [`SESSION_ACTIVE.md`](SESSION_ACTIVE.md).
- **🎉 A FUNDAÇÃO ESTÁ FINALIZADA.** Os 5 módulos (Sprite, Vector, Painter, KTX2, AVIF) são
  **drop-crates isolados** — zero touchpoint foundational restante. **KTX2 e AVIF em paralelo
  não colidem por construção** (crates disjuntos). Seu trabalho de Coord é escalonar slots +
  ship + o bundle de CI — NÃO arbitrar colisões de superfície compartilhada (não há mais).

---

## §1 — Estado AGORA (verifique no início)

```bash
git log --oneline -3            # HEAD = 9491f9f (pushado)
git rev-list --count origin/main..HEAD   # esperado ~0 (acabei de pushar tudo)
git status --short              # WIP alheio não-commitado: .vscode, docs untracked, test_strip — NÃO são commits
```

**⏳ CI EM CURSO — sua primeira tarefa = babysit:**
- Run **26721192390**: https://github.com/dibrioli/PH2D/actions/runs/26721192390 (16 commits, headSha `9491f9f`).
- Status ao escrever: `in_progress` (~30min: matrix linux/macOS/win + replay-hash + bench).
- **Babysit (DIRETRIZ §8.4):** `gh run watch 26721192390 --exit-status`. Verde → reporta link ao Enio.
  Vermelho → `gh run view --log-failed`, diagnostica, fix local, re-push, re-watch. Escalona após
  **3 falhas do mesmo job**.
- **Contexto do push:** rodei só gates baratos zero-build (fmt — corrigi drift em transform.rs +
  color_tint.rs; deny ✓; typos ✓). **Pulei clippy --all-targets + nextest --workspace locais**
  (build cold ci-test = swap-fest no 8 GiB) → **o CI é o gate de verdade**. Se ele pegar clippy/teste,
  é esperado: corrija + re-push. (σ-4 já estava em origin via `fb50589`; SIGBUS do asset-cooker
  coberto por `.config/nextest.toml` serialize + retries=6.)

---

## §2 — Stack de velocidade (faça os ≤3 agentes voarem) — detalhe em DIRETRIZ §6.6

- **Slot warm:** cada agente roda `bash scripts/slot-seed.sh <slot>` (clone CoW da `target-slots/base`,
  ~1s) e **prefixa cada cargo** com o `CARGO_TARGET_DIR` impresso. **Nunca o `target/` default.**
  Rebuild da base (você, SOZINHO) só se `Cargo.lock`/toolchain mudar.
- **Inner loop = SÓ `cargo check -p`** (ou `scripts/cargo-check-narrow.sh` p/ cortar tokens). ZERO
  test/clippy/auditor POR TASK.
- **Gate 1× no fechamento do módulo:** `scripts/nextest-impacted.sh` + clippy `--all-targets` + ≥2
  lentes adversariais sobre o diff acumulado. Ship final = `./scripts/ship.sh` (paridade-CI, ci-test).
- **≤3 cargos simultâneos.** Você escalona via SESSION_ACTIVE.
- **NÃO:** LSP type-oracle / rust-analyzer-as-oracle (medido RAM-blocked no 8 GiB), Cranelift, mold.

---

## §3 — O PLANO: KTX2 ⊕ Image IO em paralelo (2 dos ≤3 slots)

| Slot | Módulo | Pasta exclusiva | Próxima task |
|---|---|---|---|
| `impl-ktx2` | **KTX2 Fase 2** | `crates/ph2d-asset-ktx2/` · `tools/asset-cooker/` (+ `crates/ph2d-asset/` p/ ESCRITA) | **W1.T8.1** patcher PH2D_PREMUL |
| `impl-avif` | **Image IO / AVIF** | `crates/ph2d-imageio-avif/` | **Path C** decode+encode+HDR (Task 0 = re-verification) |

**Disjunção (parallel-safe):** KTX2 mexe em asset-ktx2/asset-cooker/asset; AVIF só em imageio-avif.
**Único ponto a vigiar:** `crates/ph2d-asset/` — **escrita é do KTX2**; o AVIF só **lê** o bridge
`loader.rs::decode_via_imageio_registry` (read-only, lente de regressão). Se o W1.T8.1 do KTX2
acabar tocando `ph2d-asset`, avise o AVIF (que não escreve lá). Fora isso, zero overlap.

### KTX2 — impl handoff PRONTO: [`HANDOFF_ktx2_w1_impl.md`](HANDOFF_ktx2_w1_impl.md)
- W1.T15 audit APPROVE fechado. **W1.T8.1** = patcher post-hoc que insere a key `PH2D_PREMUL` nos
  bytes do KTX2 cookado (header offset rewrites + alignment, ~200-400 LOC), isolado em asset-ktx2/
  asset-cooker. Baixo risco.
- **`cargo test -p ph2d-asset-cooker` SEMPRE com `RUST_TEST_THREADS=1`** (ISPC) — ou via nextest
  (que já serializa o grupo + retry=6).
- **CI do KTX2 (W1.T10/T12/T13 + canonical runner + LFS + retry-SIGBUS) = BUNDLE DO COORD**, NÃO do
  impl. Toca `.github/workflows/` → provável `spike-texture-cook.yml` separado. Enio já OK'd a estratégia.

### AVIF — impl handoff PRONTO: [`HANDOFF_imageio_avif_impl.md`](HANDOFF_imageio_avif_impl.md)
- Escopo GO'd pelo Enio ("melhor possível sem custo"): **Path C (libavif-sys), decode E encode, HDR
  real** (NÃO o Path A decode-only). **Task 0 obrigatória:** refazer a verification protocol p/ Path C
  (cargo audit/tree/deny + HR-1 FFI 6-critério) ANTES de codar — reportar números ao Coord.
- libavif vendora libdav1d+aom → provável **sem** install de system-lib no CI (vantagem). Se precisar
  workflow → seu OK (Coord).

---

## §4 — O que está FECHADO (não refazer)

- **Fundação multi-agente:** Sprite (ph2d-render), Vector H5/M3 (Camera2d single-source, `172eff2`),
  Painter T2.5 shell-wire (Cmd/Ctrl+Enter commit, `d24bbd3`). Todos isolados.
- **Velocidade:** CoW seeding, check-only loop, ci-test gate, nextest serialize+retry, doc canônica
  agent-readable (CLAUDE router + DIRETRIZ §6.6 + ADR-0075). LSP-oracle descartado (RAM).
- **σ-4** (asset-cooker prefab hash) re-pinado em origin.

## §5 — O que o Coord PERGUNTA ao Enio
- Smoke do Cmd+Enter (Painter T2.5) — runtime não verificado, só compile.
- Estratégia final do `spike-texture-cook.yml` (confirmar antes de mexer em CI).
- 3º slot: alocar a quê (Painter W2.T2.3+ / Vector W1 closure / Sprite W4+), ou segurar.

## §6 — Memórias-âncora
[[feedback-perfection-no-deferrals]] (padrão-ouro sem custo) · [[feedback-audit-lens-diversity]] ·
[[feedback-scoped-commit-shared-index]] · [[feedback-ci-handling]] (link da run, não polling loop) ·
[[feedback-git-stash-multiagent-danger]] · [[feedback-app-ui-english-only]].

---

**Resumo:** CI verde pendente (babysit a run 26721192390). Fundação finalizada → KTX2 (`impl-ktx2`,
W1.T8.1) e AVIF (`impl-avif`, Path C) rodam em paralelo SEM colisão. Impl handoffs prontos pra colar.
CI do KTX2 é teu bundle. ≤3 slots, agentes voando com a stack §6.6.
