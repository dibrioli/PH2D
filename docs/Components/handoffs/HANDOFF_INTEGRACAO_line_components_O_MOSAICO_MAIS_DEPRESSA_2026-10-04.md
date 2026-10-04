# HANDOFF DE INTEGRAÇÃO — `line/components`: O MOSAICO REFEITO MAIS DEPRESSA (navegação W12) — 2026-10-04

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §20. Este documento é o que o
> integrador precisa.
>
> ⚠️⚠️ **A linha leva OITO waves por integrar**, juntas e sem reverter nada umas das outras: W5–W10 (os
> handoffs listados no da [W11](HANDOFF_INTEGRACAO_line_components_AS_PAREDES_POR_MOSAICOS_2026-10-04.md)), a
> W11 ([`AS_PAREDES_POR_MOSAICOS_2026-10-04`](HANDOFF_INTEGRACAO_line_components_AS_PAREDES_POR_MOSAICOS_2026-10-04.md))
> e esta W12. Smoke do dono APROVADO na W5–W10; **o da W11 e o da W12 estão por fazer** (o mesmo, §5).

## §0 — O `--ff-only` deve passar limpo

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · `git merge-base --is-ancestor main HEAD`.
- Os commits da W12, por cima de `cde668a8a` (o handoff da W11): `768ea0e29` o código, os oráculos e o arnês ·
  e o deste handoff (o plano §20, o roteador).

## §1 — Superfície de colisão da W12

**Só a `ph2d-navmesh`.** Zero número que soma entre linhas, zero crate nova, zero pacote externo, zero
`Cargo.toml`, **zero linha em `shells/desktop`**, zero i18n, **nenhuma API pública mudou**.

| ficheiro | o quê |
|---|---|
| `src/triangulate.rs` | `merge_convex_labeled` (semi-arestas ordenadas; um anel só ganha memória quando funde) · `try_merge` (decide antes de copiar) · `triangulate_pieces` (numeração e restrições por ordenação) · `repair_t_junctions` (grelha contígua) — **a mesma saída, ao bit** |
| `src/triangulate_oraculo.rs` **NOVO** (`#[cfg(test)]`) | as quatro funções de ANTES, verbatim (só o nome), e o gate que as compara |
| `src/lib.rs` | `build_with_areas` e `poligonos` partidas em `chao_e_areas` e `pedacos` (`pub(crate)`) — a mesma ordem de chamadas |
| `src/tiles.rs` | `caixa` sem lista; `Fnv::u64` por palavra com o finalizador do `splitmix64` (as assinaturas são internas a uma `TiledMesh` — nada as guarda) |
| `src/tiles_tests.rs` | +1 gate (`a_assinatura_distingue_os_sinais_trocados`) |
| docs | plano 30 §20; o arnês `ferramentas/mutacao_navegacao_w12_2026-10-04.py` |

### ⚠️ O que um merge pode partir

1. Uma linha que mexa em `triangulate.rs` (a fusão, as junções em T): as funções de antes vivem no oráculo e o
   gate compara-as — uma mudança de comportamento numa delas tem de ir também à cópia, ou o gate acusa.
2. Tectos: `triangulate.rs` `~550 / 700`, `triangulate_oraculo.rs` `~400`, `tiles.rs` `~560`, `tiles_tests.rs` `~425`.

## §2 — O que a wave traz

A malha é a MESMA, ao bit (a impressão digital da sonda `71c5f70e3d312ee3` antes e depois: 3 cenas × 4
combinações de raio, lamas e fusão × 4 mudanças); muda QUEM a calcula. Cena `100 × 100 m`, `1 000` obstáculos
(`--release`, o melhor de 8 alternado):

| | antes (W11) | W12 |
|---|---|---|
| o mosaico refeito (sem · com 100 lamas) | `0,217 · 0,503 ms` | **`0,190 · 0,446 ms`** |
| uma porta (sem · com lamas) | `0,547 · 0,929 ms` | **`0,486 · 0,824 ms`** (`−11 %`) |
| `medir_custo` `SO_GRANDE=1`, uma porta (load `22`) | `1,16–1,26 · 1,85–1,96` | `1,06–1,12 · 1,71–1,78` |

⚠️ **Premissas do briefing que a medição derrubou:** (1) o «`0,70 ms` do mosaico refeito» era a load `1,7–9`; a
frio de carga é `0,23` — e a montagem (`0,24`) já pesa o mesmo; (2) o kill-criterion (`≤ 0,17 · ≤ 0,40`) não foi
atingido: o que resta é o piso do `spade` e do Clipper (`0,15 ms` sem lamas), que não se trocam sem mudar a malha.

## §3 — ⏳ O que fica ABERTO

- **A montagem** (`0,24 ms`) é agora o maior pedaço de uma porta; o que resta O(malha) nela só desce com ids
  fixos (recusados, plano §18.6).
- O piso das bibliotecas no mosaico; as lamas (`0,16 ms` de Clipper por mosaico com áreas).
- Os abertos da W5–W11.

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha (load `20–41`, outras
sessões):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 538 / 19 538`**, `127 s` |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (orca, nav, navmesh, physics-ecs, app-components, host-desktop) | zero |
| `cargo fmt --all --check` · `typos` (o diff da W12) · `cargo machete` | verde · zero · zero |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `censos-da-arvore-combinada.sh` · `doc-index.sh --check` | `127 / 127` · `20` em dia |

⛔ **A 1.ª corrida do fecho reprovou três** (curados, e a mutação re-corrida sobre o código final):
`no_std_transcendental_reaches_the_deterministic_hash` — a fixtura do oráculo rodava caixas com `cos`/`sin` do
`std`, e a varredura do caminho do hash lê o FONTE da `ph2d-navmesh` inteira, testes incluídos (a rotação passou a
uma direcção normalizada por `sqrt`; as populações foram re-medidas) · um `unused_mut` · um `match_ref_pats`.

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote novo).

**Mutação 10 / 10** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w12_2026-10-04.py`](../ferramentas/mutacao_navegacao_w12_2026-10-04.py), quatro controlos; observadores:
os 3 gates da `--lib` e os 12 de `tests/it` das áreas e dos mosaicos). Duas EQUIVALENTES saíram com o porquê
(F1: o `try_merge` recusa o par errado; T2: o `spade` 2.15 não depende da ordem das restrições — a ordenação
fica, pela ordem de antes). O CONTROLO de população do oráculo: `30` cenas com vários pedaços · `10` pontos das
junções em T · `2 069` arestas com dois donos · `10 259` fusões.

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **correcção** | a mesma malha, ao bit | `TiledMesh::constroi` → `poligonos` → `pedacos` → `triangulate_pieces` (numeração, restrições, junções em T) → `merge_convex_labeled` | `a_triangulacao_e_a_fusao_de_agora_sao_as_de_antes_ao_bit` (as funções de antes) · os 12 de `tests/it` · a impressão digital · F2–F6, T1, T3, J1 |
| **frescura** | um obstáculo mudado refaz o mosaico | `update_with_areas` → `assinatura` (por palavra, misturada) → a assinatura do mosaico | `a_assinatura_distingue_os_sinais_trocados` · S1, S2 |
| **não-checado pela compilação** | os números de tempo | cronómetros provisórios (tirados linha a linha, com contagem) e `medir_custo`, alternados | — |

## §5 — O smoke

O mesmo da W11 (o mapa é o mesmo ao bit — o smoke é ver que nada partiu):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

Errado = na `=3` o guarda PASSA a porta depois de ela fechar, ou não volta à ronda; na `=4` um inimigo entra na
lava ou não usa o portal; na arena um morcego preso no muro.

Fotografadas no ecrã virtual antes de ir ao dono (`target/prova/w12/ph2dnavsmoke3.png`, `ph2dnavsmoke4.png`,
`ph2dvidasmoke4.png`, 59 fps): na `=3` o guarda patrulha (*«Moving · 0.80 m to go»*, frame 268); na `=4` o
vermelho deu a volta à lava e *«Arrived»* ao herói; na arena os morcegos e o herói andam (*«Signal: morcego»*).

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental` (`7,0 G` do `debug` + `735 M` do `smoke`), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                202   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                495 : 260   alvo: <= 1,0  razao 1.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (1583 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         492 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
