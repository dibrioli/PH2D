# HANDOFF — linha `line/Aceleracao` (04/11/2026)

> Leitor: o integrador e a próxima LLM de CI/suíte. Mecanismo e tabelas completas:
> [PESQUISA_CI_PROJETOS_GRANDES_2026-11-04.md](PESQUISA_CI_PROJETOS_GRANDES_2026-11-04.md) §Resultado · §Perfil · §Noite.
> Base: `main` `aa1b8df487`. Sem push, sem integração (a linha PARA aqui).

Referência da suíte de 04/11 (antes): 31 678 testes, 19 472 s somados, 1 703 s de parede, 68 testes ≥ 60 s = 52,8 %; 7 suítes na
integração, 3 (87 min) por lint achado depois.

## Superfície de colisão (por pasta, `git diff --name-only main...HEAD`)

`.github/workflows` (2: `spike.yml`, `noite.yml`) · `.github/actions/cache-estavel` · `.config` (`nextest.toml`) · `Cargo.toml` /
`Cargo.lock` (perfil `ci-test`, `[profile.noite]` fica no nextest) · `scripts` (5; prova `scripts/processo.prova.d/P.sh`) ·
`crates/ph2d-tool-painter` (wetpaint 8, macro_track 2, preset 1, testes 1) · `ph2d-panel-painter-layers` · `ph2d-node-registry-init`
(testes) · `ph2d-ui-testkit` · `ph2d-i18n` · `ph2d-app-motion` + `ph2d-node-rig-skin-deformer` + `ph2d-node-motion-expression`
(`par_build`). Docs: `docs/DevOps`, `docs/Motion Nodes` (doc 98 §4.0), `docs/Painter/BUGS_painter.md`, `DIRETRIZ.md`,
`.claude/commands/pd-integracao.md` e `pd-ship.md`, `project-memory/`. Contadores: nenhum degrau de schema, registo ou ADR novo.

## Feito

| Item | O quê | SHA | Antes → depois |
|---|---|---|---|
| A1 | pré-voo `ship.sh --so-estatico` dentro de `suite-completa.sh` (exit 3 = nada corre; `PH2D_PREVOO=0` salta) | `53c625a8d7` | evita as 3 suítes / 87 min; custa 8 min 16 s frio, 1 min quente |
| A2 | B2 de uma linha: `integrar-rodada.sh linha` → `suite` → `falhas` → curas num bloco → `b2` → `--ff-only` | `53c625a8d7` | 4 reds × ~28 min → B2 |
| A3 | `priority = -100` no gate da memória da placa | `03dab3d587` | mínimo 285,7 → 251,7 s (−12 %) |
| B | cache estável (`cache-estavel`), cron `0 5 * * *`, lint ao lado, C9 no `test`, bindgen fora da matriz, `ci-test` em host/fixtures/C9, `--timings` | `53c625a8d7` | CI 60–101 → 27–30 min com cache |
| C | `noite.yml` (cron `30 6 * * *`, 4 fatias `hash:i/4`, `[profile.noite]`) | `53c625a8d7`, `6c5aac0a15` | compilar+arquivar 41 min |
| D | testes lentos na causa: rayon por fio · `TextSystem` por fio · `tr` por fio · blocos de 30 quadros · `instantaneo()` · `opt-level` 3 | `ea067cf82d` `e5b980f5d2` `008e5433ce` `79b508363f` `2b6e89d67b` `10769cd33c` `c831ccd793` `3258fd4c0e` `a6804ee3fd` | `o_estado_inicial_leva_cada_controlo` 105,1 → 24,5 s; `a_mao_torta` 192,3 → 56,3 s; 14 testes dos laços CPU 1 533 → 968 s |
| E | `par_build` em `rig.skin_deformer` e `motion.expression` | `21fd7b0818` `42faa789dc` | 9,5× e 11,5× a N = 32 768; nenhuma cena de fábrica acelera |
| — | `le(Cor)` nas ferramentas da água | `9d2ca2aeab` | buraco antigo curado |

Nenhum teste rebaixado nem renomeado; nenhum prazo mudou; mutações vistas vermelhas em cada corte.

### Medido e recusado

- Teto de CPU A×B (ABCBCA, 354 testes): `PH2D_CPU_PCT=50` 448,3 · 448,3 s; `=100` 295,0 · 296,3 s (−34 %). NÃO adotado (ordem do dono).
- `RAYON_NUM_THREADS=4`: 595,9 · 595,5 s (+33 %) e 1 timeout. Hakari: 0 · 0 · 0 unidades. sccache: só bucket fora do GitHub.
- Hipóteses refutadas: `assenta_e_mede`/`com_passo_fixo` 0,09 % do CPU; `sleep` de `medida_undo.rs` é `#[cfg(test)]` + `#[ignore]`; macro por `(inicial, cargas)` só repete 7,7–16,5 %.

## O que um merge pode partir

- `spike.yml`/`noite.yml`/`cache-estavel`: outra linha que mexa nos workflows ou nos pacotes citados por eles (o `ship.sh --so-estatico` confere os pacotes).
- `nextest.toml`: o gate da memória da placa tem `threads-required` + `priority`; a faixa isolada e o perfil `noite` vivem no mesmo ficheiro.
- `ph2d-i18n`: memória por fio do `tr` (`008e5433ce`; `79b508363f` corrige o `tr` depois de a memória por fio morrer); é produto, o app também usa.
- `Cargo.toml`: `opt-level` 3 do `ci-test` só em 5 crates da pintura; alarmes de pincel por nome (`a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`) verdes.

## Prova de fecho

- Perfil: hash em `ci-test` == hash do `--release` antigo nos 3 SO (ECS `8f505bd9…`, physics `f72f20f9…`, physics-ECS `20e3e7a8…`; runs 40945593989 × 41800231668).
- CI com cache: 41808153285 (30 min 02 s) e 41808157477 (27 min 23 s), verdes. Sem cache: 41800231668 (Ubuntu 43,2 · macOS 48,3 · Windows 58,1 min).
- Guardas novos: `o_tempo_em_blocos_e_o_por_quadro` (2,1 s), `o_assentar_em_blocos_e_o_por_quadro` (3,0 s).
- Prova de processo: `bash scripts/processo.prova.sh` (inclui `P.sh`).
- `PENDENTE-SUITE` — suíte final da linha.
- `PENDENTE-UBUNTU` — remedição do Ubuntu com cache.
- `PENDENTE-NOITE` — primeira corrida de testes da noite.

## Smokes / comandos

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Aceleracao && bash scripts/processo.prova.sh
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Aceleracao && python3 scripts/ci-passos.py   # régua de passos do CI (A:id B:id)
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Aceleracao && python3 scripts/junit-ocupacao.py <junit.xml>   # ocupação por janela
```

Depois do ship: `gh workflow run spike.yml --ref main` (semente; a 1.ª corrida no `main` é FRIA e não salva).

## ABERTO

1. **Dono:** teto de CPU 50 % → 100 % só na suíte da integração sem smoke dele (−34 %). `PH2D_CPU_PCT` reescreve a unidade da fatia da linha: não se mede com outro agente na mesma worktree.
2. **Produto:** [Bug #57](../Painter/BUGS_painter.md) — `wet_water` + `PAINTER_WETPAINT_FASTDRY`, de raiz × `instantaneo()` difere em 58 texels (Δmax 3); causa não isolada (candidato `planos.copia_sem_campo()`). Curado, a `bancada_da_agua` ganha −30 % de CPU.
3. Os 7 membros da faixa isolada com motivo «prazo» (`FLAKES_DE_CARGA.md:80-97`, 218 s dos 288 s) não foram medidos fora da faixa; os prazos por nome no `nextest.toml` podem baixar quando a suíte confirmar.
4. Os `diag_*` / `oleo_diluido` por quadro.
5. sccache externo: só com bucket fora do GitHub.
6. [PLANO](PLANO_ACELERACAO_2026-11.md) C1, C3, C4; cortar o nível lento do laço de integração só depois de a noite ter histórico.
7. Ubuntu com cache e testes da noite: ver `PENDENTE-*` acima.
