# HANDOFF DE INTEGRAÇÃO — `line/components`: OS TRÊS ABERTOS DA W16, MEDIDOS NUM CICLO (navegação W17) — 2026-10-06

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §26 (os kill-criteria §26.1–§26.4, as três
> rodadas §26.5–§26.6, os vereditos §26.7, as recusas §26.8, o aberto §26.9). A linha ainda leva a W15 e a W16
> inteiras, por integrar: os handoffs delas,
> [`O_TIQUE_DEPOIS_DA_PORTA`](HANDOFF_INTEGRACAO_line_components_O_TIQUE_DEPOIS_DA_PORTA_2026-10-05.md) (§1–§4b) e
> [`O_DESENHO_E_O_CORPO`](HANDOFF_INTEGRACAO_line_components_O_DESENHO_E_O_CORPO_2026-10-05.md) (§1–§2, o smoke §5),
> continuam válidos para a parte delas — este acrescenta a W17 por cima.

## §0 — O `--ff-only` deve passar limpo

- Base `main` `a46c4c200` (o `main` não andou desde a W16; o `git rebase main` do início foi vazio). NADA integrado
  desde `5d596eaaf`: a W15 (`c3395b0a7..5ab72773f`), a W16 (`baf14765b..640499189`) e a W17 por cima — `1aa46cb94`
  (o plano §26, antes do código) · `4ccf96f2c` (as alavancas medidas, para a medição se repetir) · `539ce6527` (as
  alavancas saem; o veredito; o ADR) — e o deste handoff.
- **O ADR-0180 continua livre** no `main` e nas outras sete worktrees (máximo `0179` em todas, recontado no fecho).

## §1 — Superfície de colisão (a W17)

| ficheiro | o quê |
|---|---|
| `ph2d-physics-ecs/examples/medir_replaneio.rs` | a sonda: a régua da 4.ª rodada (`Atraso`/`Seguidos`) e o diagnóstico da DOBRA (o maior `recomecos` de quem persegue · das outras) |
| `docs/Components/30_plano_navegacao.md` | §26 (nova) |
| `docs/architecture/decisions/0180-…md` | adenda 2 (o que compila para `wasm32`) |

**Zero código de produto:** `git diff 640499189 -- crates shells ':!crates/*/examples/*'` é vazio — o produto é o
da W16 ao bit. Zero crate, zero pacote no `Cargo.lock`, zero i18n, zero componente, `PROJECT_SCHEMA` intocado.

## §2 — O que a W17 traz

| item | veredito | medido |
|---|---|---|
| **A** quem persegue sem o alvo à vista | ✗ nenhum candidato cumpre (crítico `≤ 40 mil`, colunas `≤ +10 %`) | A1 (primeiro sem a dobra delas) crítico `160 mil`; A2 (o fim do caminho segue o alvo) máx `30`; **A4** (vagas a mais) máx `30 → 4` mas crítico `40 → 72 mil` e uma thread `+12 %`; A5 (A4 sem a dobra delas) = A4 |
| **B** a web numa thread | ✓ medido, escrito no ADR-0180 | `ph2d-nav` · `ph2d-navmesh` · `ph2d-orca` compilam para `wasm32-unknown-unknown`; a `ph2d-physics-ecs` pára SÓ no codec AVIF (`libavif-sys`, `libdav1d-sys`, `rav1e`), via `ph2d-ecs → ph2d-asset → registry-init → imageio-avif` |
| **C** a dominância da ponderada | ✗ recusa medida (metade do tempo) | o perfil novo: `domina` `37 %` (o `corte` `32 %`), o heap `~15 %`; C1 (corte fechado) `0,95`; C2 (heap de aridade 4) `1,01–1,02`; C3 (lamas do mesmo id num pedaço) `0,94–0,95`; C1 + C3 `0,89–0,90` — custo contra o oráculo igual ao dígito em todas |

**O achado** (§26.6): o crítico do PRODUTO não é dois orçamentos — a lei da dobra (`fatia_depois_de`) põe o tecto em
`orçamento · 2^k`, e o produto já paga `80 mil` a `10` e `50` agentes na cena de stress; o `40 mil` a `200` era a
corrida que calhou. E a dobra que sobe quando quem persegue toma vagas é a das OUTRAS (diagnóstico `1 · 3`).

## §3 — ⏳ O que fica ABERTO

- **Quem persegue sem o alvo à vista**, na lama cerrada: máx `30` tiques (média `2,2`) a `200` agentes. A cura passa
  por limitar a DOBRA das procuras adiantadas (vivacidade com outro tecto) — só então as vagas a mais (A4, que dá `4`)
  cabem no crítico.
- **A web numa thread**: a shell web não existe; o 1.º degrau é o codec AVIF fora do caminho de `ph2d-ecs`. Decisão de
  PRODUTO do dono (perguntada no fecho da W17).
- **A dominância da procura ponderada**: o custo por nó está esgotado (§26.8); falta fazer MENOS nós.

## §4 — A prova de fecho

- **As medições** (a régua do dono de 05/10: as versões no MESMO processo, 7 rodadas intercaladas com a ordem rodada,
  o mínimo, perfil `smoke`, loadavg anotado), `target/prova/w17/`: o perfil (`perfil_dominancia.txt`), o `wasm32`
  (`wasm32_check.txt`, `wasm32_physics_keep_going.txt`), e três rodadas — `medir_{custo,replaneio}_w17.txt`
  (load `1,9–3,5`), `…_rodada2.txt` (`5–10`), `medir_replaneio_w17_rodada3.txt` (`4,4–5,8`). Série declarada no plano:
  a 2.ª nasceu do diagnóstico da 1.ª (a dobra das outras) e de um defeito da SONDA (o C3 medido sobre a mesma malha:
  os pedaços eram por área); a 3.ª, de o A4 subir o crítico.
- **Gates novos: nenhum; mutação: nenhuma** — nenhuma lei nova entrou (o produto é o da W16 ao bit). A prova da W16
  (`18 / 18`) e a da W15 continuam a valer para o código que está.
- **Gate batched** (sobre o merge-base, a W15, a W16 e a W17 juntas): `nextest-impacted` **`15 779 / 15 779`**
  (`103,5 s`, `1` lento) · clippy `--workspace --all-targets -D warnings` limpo — `target/prova/w17/gate_{nextest,clippy}.txt`.
  Load `6,4` antes e `25–32` durante, puxado pela própria suíte; os gates de relógio e de alocação passaram.

## §5 — O smoke

Nenhuma cena nova nem tocada: o smoke da W16 continua a ser o da linha (os discos, o «Rumo», a barreira contornada,
aprovado pelo dono). O binário compilado (a 2.ª corrida, colada), depois de `rm -rf target/*/incremental`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.28s
```

(`rm -rf target/*/incremental`: `1,2 G` do `debug` e `1,3 G` do `smoke`. Zero linhas `Compiling` na 2.ª.)

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.11/passo   alvo: >= 1,5  (8% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                252   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                769 : 247   alvo: <= 1,0  razao 3.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1068 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         399 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
