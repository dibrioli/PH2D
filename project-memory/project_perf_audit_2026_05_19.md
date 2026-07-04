---
name: project-perf-audit-2026-05-19
description: Perf audit cross-cutting que cortou pre-commit T2 de 40min → 5min e workspace nextest de 17min → 1.5min. Descobertas e mudanças.
metadata: 
  node_type: memory
  type: project
  originSessionId: fe59209c-4f42-43aa-a540-0a60c10ff373
---

**Sessão 2026-05-19 noite** — pós Wave 9 + DIRETRIZ v6.0. Implementador delegado fechou 7 commits sobre o sha `10ef2b6`.

## Diagnóstico inicial (medição cache-quente)

- `cargo test --doc --workspace` = 3min pra zero retorno (maioria das crates tem 0 doctests)
- `cargo clippy --workspace --all-targets` = 1s quente / ~10min frio (compila benches+examples desnecessariamente)
- **`cargo nextest run --workspace` = 14min** quente. 1347 tests, 105 SLOW. **Vilão real.**
- Pre-commit T2 medido em 40min cache-frio (workspace) num commit docs+2-comments.

## Cortes A+B no pre-commit (`10ef2b6`)

- Drop `cargo test --doc --workspace` (deixa pro CI)
- Drop `--all-targets` do clippy (benches/examples só CI)
- Escopar `nextest -p <crates-tocados>` quando trigger é multi-crate **sem** foundational/Cargo.toml/shells
- T2 estimado: 40min → 2-5min cache-frio, segundos cache-quente

## Slow tests audit (`436626e..cb13efe`) — 105 SLOW → 0 SLOW

**Root cause:** `TextSystem::new()` enumerava fontes do sistema via CoreText (25-77s neste Mac M-series). Multiplicado por ~48 sites de teste em ph2d-editor-core = catastrófico.

**Fix:** Nova API `TextSystem::without_system_fonts()` em `ph2d-text` que passa `fontique::CollectionOptions { system_fonts: false }` + força `family_name = "InterVariable"` (bundled). **Production `new()` intocado** — só test code migrado.

**Outros fixes:**
- `ph2d-asset` png_bomb test: alloc reduzido de `(16384, 16384)` (1 GiB) pra `(8193, 1)` (32 KiB), ainda exercita o reject path de `image::Limits` que é checado contra IHDR antes da alloc.
- `ph2d-render` `try_headless_gpu()` agora cacheia `GpuContext` via `OnceLock<Option<GpuContext>>` (Clone via Arc internals). Primeiro test por binário paga ~14s; demais zero-init.

**Resultado:** nextest workspace 1033s → 99s = **-90.4%** (1:39 wall time, alvo DIRETRIZ ≤5min batido com folga).

## Slow remanescentes (aceitos)

- `ph2d-asset watcher_*` × 2 (~32s cada) — FSEvents 5s deadline + 250ms poll cycles. **Inerente** ao subsystema, security-critical.
- `ph2d-render` GPU init × 4-5 binários (14-35s cada) — Metal driver cold load per binary. **ROI baixo** pra compartilhar cross-process.

## Descoberta importante — Inter family override

A nova `without_system_fonts()` força `family_name = "InterVariable"` via `FontInfoOverride`. **Production `TextSystem::new()` sem override pode estar usando system sans-serif** (não Inter) há tempos — descoberto pelo Implementador durante debug. Não foi alterado.

**How to apply:** Se algum smoke visual sobre Inter pixel-metric quebrar no futuro, investigar essa hipótese. Pode ser bug real de UI que ninguém notou.

## Tarefa D — lld linker

**Achado:** `~/.cargo/config.toml` já tinha `lld` configurado desde 2026-05-13. Coord não sabia.

**Speedup real medido:** **1.5-3% no macOS 16** (Darwin 25.4.0 vem com `ld-prime` desde Xcode 15, que já é 2-5× mais rápido que o `ld` legado — gap pra lld é pequeno). Não os 30-50% previstos com base em data antiga.

**How to apply:** Manter como está, é grátis. Não escalar pra opções mais avançadas (mold, sccache) sem benefício mensurável neste hardware.

## Refs

- Commit base: `10ef2b6` (pre-commit T2 cuts A+B)
- Commit perf range: `436626e..cb13efe` (6 commits)
- CI run perf audit: https://github.com/dibrioli/PH2D/actions/runs/26134715986
- Relacionado: [[project-multi-agent-v6-2026-05-19]] (modelo operacional vigente — esta foi a primeira execução de Implementador delegado pós-v6.0)
- Relacionado: [[feedback-codificacao-rapida]] (atualizado: T2 caps agora baixos, `cargo test -p` continua ideal para iteração)
