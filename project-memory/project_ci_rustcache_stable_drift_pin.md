---
name: project-ci-rustcache-stable-drift-pin
description: "CI cold-build/timeout after a no-op-ish push = @stable rust-cache drift, not your diff; toolchain now pinned @1.95"
metadata: 
  node_type: memory
  type: project
  originSessionId: 51cf1fb1-5b82-4b86-b5ec-6d576c363793
---

CI (`spike.yml`) usava `dtolnay/rust-toolchain@stable`, então o `Swatinem/rust-cache`
derivava o `rustc-hash` da chave do toolchain default. Cada release de Rust stable (~6 sem)
rotacionava esse hash e **invalidava o cache de TODOS os OSes de uma vez** → cold build geral.
O compilador real sempre foi 1.95 (override do `rust-toolchain.toml`); só a chave do cache
balançava — desperdício puro. Linux/macOS absorvem o cold build sob o cap; o **Windows não**
(deps nativos dav1d/rav1e via meson+nasm+MSBuild passam de 45min) → cancelado no cap →
job cancelado **nunca salva cache** → próximo run frio de novo = **loop stuck-cold** (2026-06-30,
run 28473978977; Windows cold ≈ 48min).

**Fix (2026-06-30, commits `2d7abd72` + `eacdccaa`):** (1) test-job `timeout-minutes` 45→90 =
auto-cura (deixa 1 cold build completar e salvar cache). (2) Pin dos 4 `@stable`→`@1.95`
(lint/test/determinism/bench; MSRV fica `@1.92`) = mata a recorrência. **Coupling:** bumpar
os refs `@1.95` em lockstep ao mexer no channel do `rust-toolchain.toml`.

**Why:** atribuir lentidão/vermelho de CI ao SEU diff quando a causa é drift de toolchain
queima rounds (o Enio quase atribuiu ao revert do wet-paint).

**How to apply — forense de cache antes de culpar seu commit:** `git diff <last-green> HEAD --
Cargo.lock '**/Cargo.toml'` (vazio = você não mexeu em input de build) + `gh cache list`:
decodifique a chave `v0-rust-{shared-key}-{OS}-{arch}-{rustc-hash}-{lockfiles-hash}`.
**lockfiles-hash igual + rustc-hash mudou = drift de toolchain/runner, não seu código.**
Relacionado: [[feedback-measure-perf-symptom-scale]], [[project-imageio-avif-pathc-2026-05-28]].
