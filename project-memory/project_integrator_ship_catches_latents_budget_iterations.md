---
name: project-integrator-ship-catches-latents-budget-iterations
description: Modo L — o gate per-linha e o foundational-integrate NÃO rodam fmt/clippy-all-targets/machete/deny; o integrador orça 2-4 iterações de ship.sh pra drenar latentes das linhas
metadata:
  type: project
---

**Contexto:** integração de 5 linhas (MotionNodes/Painter/anim/Vector/audio),
2026-07-11, todas fechadas "verde" pelo próprio gate batched. O integrador
ainda drenou **~8 falhas latentes** antes do ship 100% verde.

## Por que verde-de-linha ≠ verde-de-ship

Duas camadas de gate rodam ANTES do ship, e nenhuma cobre tudo:

1. **Gate batched da linha** (`nextest-impacted` + clippy `-p` + fmt manual): a
   linha escolhe o que roda. Erra por omissão — clippy `-p <crate>` não pega um
   lint num crate-irmão; fmt "à mão" não roda o pin; a suíte completa do shell
   (`file_loc_caps`, `no_tofu_glyphs`) não roda se a linha não a invocou.
2. **`foundational-integrate.sh`** (rebase → sync → `cargo check --workspace` →
   `nextest-impacted`): pega LOC/tofu/count-mirrors **se** o gate vive num teste
   dentro do impacted-set (pegou o HR-18 do Painter, o LOC do vec-edit, os 2
   count-mirrors do Vector). Mas **NÃO roda fmt, clippy `--all-targets`, machete,
   deny, typos** — esses só existem no `ship.sh`.

⇒ Tudo que é **fmt-skew** (a linha não rodou `cargo fmt` no pin) e **lint de
clippy** (a linha não rodou `--all-targets` no crate certo) **atravessa as duas
camadas** e só vermelha no ship final. Nesta rodada: watercolor fmt-skew (4
arquivos), `HashMap` banido (`clippy.toml` ADR-0022) em ph2d-audio-encode,
`then_some` (unnecessary_lazy_evaluations) em audio_overlay.

## A pegadinha nova: o fmt do integrador quebra o LOC cap

`watercolor_render.rs` estava a **699** (não-canônico — a linha Painter não rodou
fmt, handoff §5). O `cargo fmt --all` do integrador canonicalizou e **re-expandiu
pra 701** (multi-arg fold), estourando o cap de 700 → nextest do ship vermelho.
É o [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]] no contexto de
integração: **rode fmt ANTES de medir LOC, e depois de fmt re-cheque o LOC gate.**
Fix = split (mover `watercolor_render_active` pro sibling), nunca allowlist.

## How to apply (integrador)

1. **Orce 2-4 iterações de `ship.sh`.** Cada uma descasca a próxima camada
   (fmt → LOC-pós-fmt → clippy-disallowed-type → clippy-lazy-eval). Não é
   retrabalho; é o desenho (as linhas delegam isso, §5 dos handoffs).
2. **Atalho pra clippy:** quando o ship vermelha em 1 lint, rode
   `rustup run <pin> cargo clippy --workspace --all-targets --features
   ph2d-spike/bevy_ecs -- -D warnings` DIRETO — pega TODOS os lints de uma vez,
   em vez de um-por-ship (cada ship é ~10min).
3. **Todo fix de latente é commit no main** (a linha já integrou) com msg
   `fix(ship): …` nomeando qual gate pegou.
4. Ver [[feedback_ship_parity_gaps_ci_only]] (ship↔CI), [[feedback_ci_direct_lint_gates_and_fmt_skew]],
   [[project_integration_prefork_lines_ship_drift]], [[feedback_ship_prep_no_fail_fast]].

## Infra: `/dev/shm/ph2d-target` some entre sessões

O `target/` do primário é symlink pro tmpfs (`workstation`). O
`/dev/shm/ph2d-target` **evapora no reboot** (2ª vez que aconteceu) → `cargo`
do primário falha com "failed to create directory target" / "Not a directory".
As integrações não pegam isso (cada worktree tem `target/` real próprio); só o
`ship.sh` (roda no primário) trava. Fix: `mkdir -p /dev/shm/ph2d-target`.
