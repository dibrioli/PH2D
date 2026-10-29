# HANDOFF — janela «Velocidade 2» (29/10/2026)

> Leitor: a próxima janela (a do «atualizar TUDO», adendo do dono 30/10) e o integrador. Detalhe e números:
> [`COMISSAO_VELOCIDADE_2026-10-29.md`](COMISSAO_VELOCIDADE_2026-10-29.md) §Implementado. Tudo no `main`, sem push.

## Feito (commits no main)

| Item | O quê | SHA | Antes → depois medido |
|---|---|---|---|
| K | `cargo-test-narrow`/`nextest-impacted`/`suite-completa` gravam compilar × correr × load (`python3 scripts/tempos-teste.py`) | `39c4570912` | 5 corridas por nome: 97 % compilar |
| B1 | `scripts/suite-completa.sh` regista a árvore verde; `ship.sh --enviar` salta pela árvore do HEAD | `b92bccfe62` `1e1a487362` | +25–35 min → 0,07 s |
| E | suíte e impactado fora do fecho da LINHA (comandos, verificador, DIRETRIZ §6) | `a2dd377341` | 1 544 s → 0 no fecho |
| D | `scripts/recontar.py` (ADR, registos, cenas, schema) no ship e no integrador | `b877f0d7d1` | achou CODE_SMOKE 1..9 ≠ 10 |
| G | nível lento medido (`.config/testes-lentos.tsv`, 55 testes, 9 335 s); `PH2D_NIVEL` | `8e47af360e` | impactado −57–61 % s somados |
| F | hook `commit-barato` (tectos, fmt, HR-15, nextest) | `6a9583fef1` | 48–67 ms/commit |
| I | hook `prazo-do-agente` (45 min imposto) + `scripts/fila-pesada.sh` | `a0fcfc1f19` | visto a cortar H e A no prazo |
| A+B2 | `scripts/integrar-rodada.sh` cadeia/suite/falhas/b2/ff; `/pd-integracao` e §1.5.3 | `958c906677` | ensaio: vermelho atribuído em ~2 min |
| M (1) | catraca «só encolhe» em toda `ph2d-app-*` + `PLANO_PARTIR_APP_MOTION.md` | `99f00989c3` | — |
| docs | comissão, CLAUDE.md §2, memória do selo pelo HEAD | `483cc0a2db` | CLAUDE.md 31,6 KB |

Todas as regras novas têm prova em `bash scripts/processo.prova.sh` (K B1 B1b D E F G I A; < 5 s), com mutação
vista vermelha, e o `ship.sh` corre-a.

## Em curso nesta janela no fecho (ver o último commit «L»)

- **L / Rust 1.99.1**: pins mudados (`rust-toolchain.toml`, `Cargo.toml`, `spike.yml` ×4, `miri.yml`,
  STACK_VERSOES). Medido ABBA: rebuild da `ph2d-app-motion` 7,4 → 7,0 s (−5 %; load a cair 27 → 14). Avisos novos
  do 1.99 (`std::f32::MIN` deprecado no `deep_filter` vendored; clippy `approx_constant` GOLDEN_RATIO na
  `ph2d-node-force-buoyancy`, …) a curar SEM `allow`. A suíte completa do 1.99 NÃO correu: entra na suíte única
  do «atualizar tudo» (§0.10).
- `scripts/suite-completa.sh`: compilação falhada agora diz «NÃO COMPILOU» (antes dizia «filtro casa zero»).

## Aberto

1. **«Atualizar TUDO»** (adendo do dono 30/10, substitui o L): vello 0.11 + wgpu 30 + naga 30 + wgpu-profiler
   0.28 + o que `stack-audit.sh --tetos` deixar; UMA suíte + placa + fotos uma a uma. O prompt está no fim.
2. **H**: janela fixa recusada por medição (patch em `patches/`); restam «raios > 300 só no gerador/nível lento»
   e «janela de 2–3 diâmetros nos misturadores», ambos com o Blender Soft (r > 723) visto vermelho.
3. **M ondas 0–4** (1–3 dias): onda 0 corta o ciclo `motion_state` ↔ `motion_bridge` — ver o plano.
4. **A real**: `suite`/`falhas --bisecar`/`b2` no worktree `Worktrees/integ-ensaio` (comandos no commit
   `958c906677`); depois remover o worktree e os ramos `integ/ensaio-2026-10-29`, `ensaio/*`.
5. **I**: medir um dia inteiro contra 28/10 com `python3 scripts/agent-time-split.py`.
6. **K**: deixar o log correr uma semana e reler as 71 h com `tempos-teste.py`.
7. Achado do ensaio A: `cargo check --all-targets` não avalia `const { assert! }` dentro de fn — a cadeia
   poderia usar `cargo test --no-run` no lugar do `check`.
8. Pronto para enviar (`/pd-ship`) só por ordem do dono; o ship agora corre `processo.prova.sh`,
   `recontar.py` e «nenhum teste lento sem nível».
