---
name: project_dev_speed_audit_2026_09_10_w0_applied
description: A auditoria de velocidade de desenvolvimento (2026-09-10) vive em docs/DevOps e a onda W0 (perfil `smoke` · `jobs = 32` · tecto global do nextest + lane dos gates de relógio · `incremental = false` no ci-test · RA por pacote) FOI aplicada e commitada no mesmo dia; as ondas W1 (um binário de teste por crate) e W2 (partir a shell) ficam por abrir
metadata:
  type: project
---
Estado em 2026-09-10, fim do dia:

- Relatório: `docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md` (medições,
  pesquisa com URL, plano W0–W4, recusas medidas no §9).
- **W0 aplicada** (commit local em `main`, sem push — §0.7): `[profile.smoke]` no `Cargo.toml`
  (a reconstrução da shell após uma linha passou de **161 s num núcleo para 2,7 s**, medido por
  mim e confirmado pelo Enio) · `[profile.ci-test] incremental = false` · `.config/nextest.toml`
  com `slow-timeout = { period = "60s", terminate-after = 3 }`, `fail-fast = false`, junit e a
  lane dos 23 gates de relógio/alocação (`threads-required = num-cpus` + `priority = -100`) ·
  `~/.cargo/config.toml` `jobs = 32` (era 6; 156 → 93 s no check frio do workspace) ·
  `rust-analyzer.check.workspace = false` no settings do VSCode · o molde do smoke no
  `CLAUDE.md §5.0`, a DIRETRIZ §1.5.9 item 9 e o `scripts/run-shell.sh` passaram a
  `--profile smoke` (o `--release` fica para smokes de PERFORMANCE).
- ⚠️ `~/.cargo/config.toml` e o settings do VSCode são POR MÁQUINA (fora do repo): o Mac e o
  Windows não foram tocados, e o `hw-profile.sh` continua a mandar lá.
- ✅ **W1 FEITA no mesmo dia, na `main` por ordem do Enio (sem linha):** 1 446 → 127 binários de
  teste de integração (`tests/it/main.rs` + um `mod` por ficheiro, 73 crates, script com prova: lista
  de 22 635 testes idêntica antes/depois; 19 ficheiros com `#[global_allocator]` ficam binário próprio).
  Regra nova: teste de integração novo = ficheiro em `tests/it/` + `mod` no `main.rs` (DIRETRIZ §6.3).
- ✅ **W0.1 medida (10/09):** `[profile.dev] debug = "line-tables-only"` aplicado (−6 % tempo, −25 % disco
  de deps); `build-override` **recusado** (gate 145 → 146 s — as duplicadas são feature/`check`≠`build`);
  o `ld.mold` no gate é 12 linkers/198 threads em 20 % do tempo, não é o gargalo.
- ✅ **W2.0 censo feito:** `App` tem ~401 campos; física toca 126 (22 `impl App`), sculpt 180, vec 89; 425
  ficheiros de smoke (95 k LOC), 22 com `impl App`. ⛔ Uma feature `smokes` por `#[cfg]` foi recusada.
- ⭐ **ORDEM DO ENIO (10/09, noite): «Amanhã partiremos o shell» — a W2 começa em 11/09.** ⭐⭐ **Os
  briefings das linhas estão em `docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md`:
  L0 `line/app-host` PRIMEIRO e sozinha (substrato `ph2d-app-host` + `ph2d-app-registry-init` + piloto
  `field3d` + o HOWTO); L1–L5 (motion, physics, sculpt3d, vec, flip) em paralelo SÓ depois de L0 integrar
  — as seis tocam as mesmas costuras (`main.rs`, `App`, `render_loop`, `input_dispatch`) e sem ponto de
  extensão colidem por mesmo-símbolo. A prova de cada linha usa `scripts/nextest-list-diff.py`.** Ponto de
  partida: o censo em `docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md` §4-C2
  (W2.0: 425 ficheiros de smoke / 95 k LOC, 22 com `impl App`; `App` com ~401 campos; física 126,
  sculpt 180, vec 89) e DIRETRIZ §6.7 item 5. Degrau 1 = cenas de smoke fora da shell atrás de uma
  interface (trait `SmokeHost` ou eventos ECS); apagar as cenas que nenhum doc cita; depois
  `ph2d-app-<módulo>` por família, motion primeiro. ⛔ Não é uma feature `smokes` por `#[cfg]`.
- **Por abrir, em ordem:** W2 (cenas de smoke para crates atrás de um trait `SmokeHost`; depois
  `ph2d-app-<módulo>` por família — dias por família, motion primeiro) · W1b (`ph2d-arch-gates`, valor baixo
  depois da W1) · W2.0 (censo de quanto de `App`
  cada família da shell toca) · W2 (cenas de smoke não citadas apagadas, as citadas para crates
  próprias atrás de uma feature; depois `ph2d-app-<módulo>` por família, Motion primeiro) ·
  W3 (CI: archive + partition, impactado no PR, GPU por software) · W4 (nightly só para medir
  `-Zthreads`; 1.99 quando sair).

**Why:** o Enio pediu a auditoria, testou o smoke (2,69 s) e disse «parece OK»; a onda de
configuração é reversível, estava decidida pela medição, e a integração que corria tinha
acabado de aterrar e ser pushada — foi o momento sem colisão.

**How to apply:** um agente que vá propor optimização de build/teste lê o §6 e o §9 do doc
antes; um agente que escreva um smoke usa `--profile smoke` (o molde do §5.0); uma linha que
feche deixa `target/smoke/` construído, não `target/release/`.
[[feedback_a_touch_does_not_measure_an_edit_and_timings_inflate_under_contention]]
