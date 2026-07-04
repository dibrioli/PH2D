# ADR-0104 — Speed strategy is hardware-tiered, not fixed to the 8 GiB Mac

**Status:** Accepted (Enio, 2026-07-04) · **Supersedes:** none · **Amends:** DIRETRIZ §6.6 (the
"agents flying" stack) · **Related:** ADR-0075 (monorepo Rust + ECS-decoupling + build-speed).

## Context

The entire iteration-speed strategy (DIRETRIZ §6.6, the `scripts/slot-*.sh` machinery, CLAUDE.md §2)
was calibrated for **one** machine: the 8 GiB Apple Silicon Mac. That calibration is written into the
code and docs as **absolute rules**, e.g.:

- **≤3 cargos simultâneos** — because a 4th caused swap thrashing (`slot-env.sh`: "this machine is 8 GiB").
- **rust-analyzer BLOCKED** — §6.6.B measured RA indexing at ~1.5–4 GB, "não cabe nem ×1" on 8 GiB.
- **CoW slots mandatory** — APFS clonefile + a Mac-specific external disk (`MAC_EXTERNO`).
- **`cargo-check-narrow` on-demand only** — continuous check (bacon-ls) "pode ser PIOR" under swap.

PH2D is now developed across machines of very different power: a **weak Mac**, a **weak Windows** box,
and a **128 GB / 32-thread Linux desktop** (RTX 5060 Ti). The fixed strategy **wastes the strong box by
an order of magnitude** and would **crash the weak one** if inverted. The strategy must be a **function
of the hardware**, not a constant.

The §6.6 deep-research already flagged this: "Linux-benchmarks não transferem direto pro Apple Silicon
8 GiB." The fix is to make that conditionality **first-class and detected**, not tribal knowledge.

## Decision

**Iteration-speed knobs are selected by a runtime-detected hardware TIER. No per-machine value is
hardcoded; each machine self-classifies.**

1. **Detector — `scripts/hw-profile.sh`** (committed). Reads OS + RAM + logical cores + filesystem
   reflink capability and prints a **tier** plus the knob values (`--env` emits sourceable `PH2D_*`).
   Three tiers:
   - **`constrained`** (≤12 GiB RAM): the original §6.6 baseline, verbatim.
   - **`standard`** (mid box, e.g. Windows): measured middle ground — knobs are **piloted**, not yet
     mandated (Linux benchmarks don't transfer).
   - **`workstation`** (≥48 GiB RAM **and** ≥12 cores, e.g. the Linux desktop): unlocked.

2. **`workstation` overrides** (on top of the §6.6 baseline):
   - **rust-analyzer as oracle** (was RAM-blocked) — the agent reads LSP diagnostics, not raw cargo output.
   - **High parallelism** — ~cores/6 full builds, ~cores/3 checks; the ≤3 ceiling is dropped.
   - **CoW slots optional** — a single `target/` suffices; reflink slots are available but rarely needed.
   - **`target/` on tmpfs** (`scripts/target-on-tmpfs.sh`) — RAM-disk for link/IO; incremental state in RAM.
   - **sccache** — transparent caching wrapper (global `~/.cargo/config.toml`). Deterministic output,
     so it does **not** diverge `ship.sh` from CI. Its disk cache survives reboot and repopulates the
     tmpfs `target/` after a wipe.
   - **`mold`** linker (Linux; ELF-only, incompatible with macOS).

3. **The core loop is tier-invariant.** §6.6.A (inner loop = `cargo check -p`, batched heavy validation
   at module close, "audit ≠ compilar") holds on **every** tier. Only the RAM/concurrency limits move.

4. **Explicitly rejected even on `workstation`:** `-Ctarget-cpu=native` as a default. It changes codegen
   → diverges from the CI cache and breaks `ship.sh` CI-parity, while giving **zero** inner-loop benefit
   (`cargo check` does not codegen; the app is GPU-bound). Available only as an opt-in for isolated run
   builds in a separate target dir.

## Consequences

- **The weak Mac/Windows are untouched** — they classify `constrained`/`standard` and keep the exact
  rules they have today. Nothing regresses for them.
- **The Linux desktop unlocks** — RA-as-oracle + high parallelism + tmpfs + sccache. Setup is captured
  in [`docs/DevOps/MULTI_MACHINE_SETUP.md`](../../DevOps/MULTI_MACHINE_SETUP.md) §3.2.
- **Machine-specific config never enters the repo.** Linker, sccache wrapper, tmpfs target-dir and RA
  settings live in **per-user** locations (`~/.cargo/config.toml`, VS Code User settings,
  `/etc/tmpfiles.d/`), honoring the Chesterton fence in the repo's `.cargo/config.toml` (a linker/
  target-dir pinned in-repo already broke the Mac/Windows CI once).
- **New machines self-configure.** `hw-profile.sh` reads the truth at runtime; adding a 4th machine
  needs no doc edit — it lands in the right tier automatically.

## Kill-criterion / revisit

If `standard`-tier (Windows) measurement shows the mid-box knobs are wrong (e.g. RA on-save is net-
negative there), re-tier from measurement — do not copy the workstation profile. Re-open this ADR when
a machine crosses a tier boundary (RAM upgrade) or when the CI build itself moves to a tiered runner.
