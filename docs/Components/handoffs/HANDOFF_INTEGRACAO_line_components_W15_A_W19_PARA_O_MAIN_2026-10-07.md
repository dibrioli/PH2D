# HANDOFF DE INTEGRAÇÃO — `line/components` PARA O MAIN: navegação W15–W19 — 2026-10-07

> A porta ÚNICA do integrador para a linha inteira. O detalhe de cada onda (o porquê, as medições, as recusas)
> vive nos handoffs dela, que continuam válidos:
> W15 [`O_TIQUE_DEPOIS_DA_PORTA`](HANDOFF_INTEGRACAO_line_components_O_TIQUE_DEPOIS_DA_PORTA_2026-10-05.md) ·
> W16 [`O_DESENHO_E_O_CORPO`](HANDOFF_INTEGRACAO_line_components_O_DESENHO_E_O_CORPO_2026-10-05.md) ·
> W17 [`OS_TRES_ABERTOS_MEDIDOS`](HANDOFF_INTEGRACAO_line_components_OS_TRES_ABERTOS_MEDIDOS_2026-10-06.md) ·
> W18+W19 [`A_LAMA`](HANDOFF_INTEGRACAO_line_components_A_LAMA_2026-10-06.md).
> O plano: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §23–§28.

## §0 — O estado para o merge

- Worktree `Worktrees/line-components`, branch `line/components`: o código e as provas fecham em **`34f950397`**;
  por cima, só os commits deste handoff (docs).
- Merge-base **`a46c4c200`** = o `main` de hoje (07/10). Se o `main` não andou, o `git merge --ff-only
  line/components` passa limpo. Se andou: `git rebase main` na linha primeiro, e a recontagem do §2 contra o
  `main` DO DIA.
- **48 commits e 108 ficheiros até `34f950397`** (mais os deste handoff), NADA integrado desde `5d596eaaf` (o fecho da W14).
- Smoke do dono: **todas as ondas aprovadas** — W15 (05/10, cenas `NAV=3,4`, `VIDA=4`, depois de dois reports
  curados), W16 e W17 (06/10), W18 (cena `NAV=5`, 06/10, depois da cura do «R2 na quina»; cena `NAV=6`, 07/10) e
  W19 (cena `NAV=7`, 07/10).
- ⛔ Integrar e enviar só por ordem do dono (CLAUDE.md §0.7); este documento não é essa ordem.

### Os commits, por onda

| onda | commits | o quê |
|---|---|---|
| **W15** o tique depois da porta | `c3395b0a7` … `5ab72773f` (14) | a procura em fatias (pára entre dois pops, refaz-se ao bit), toda procura paga do orçamento do tique, as procuras a meio em PARALELO (ADR-0180); dois reports do dono curados (o atalho alcançado dentro do corpo; a fita da física grava todo tique devido — a 2.ª vida não repetia a 1.ª; ninguém sai do portal dentro de outro corpo; o corpo do morcego é o desenhado) |
| **W16** o desenho e o corpo | `baf14765b` … `640499189` (6) | o desenho cabe no corpo em toda cena de smoke (o disco reservado no atlas); o caminho contorna um corpo que anda; o alvo à vista sem procura |
| **W17** os três abertos medidos | `1aa46cb94` … `da900fb02` (5) | ZERO código de produto: as alavancas medidas não cumpriram e saíram; o `wasm32` medido (adenda 2 do ADR-0180); a decisão do dono sobre o navegador |
| **W18** a lama | `359dc42a3` … `a9fbdac97` (13) | a área de custo ao vivo (o 5.º motivo de replaneio), o canto alcançado (`ALCANCE_DO_CANTO`), as cenas `=5` e `=6`, a área BARATA recua para DENTRO (`Area::dentro`) |
| **W19** a cota | `085d9d196` … `34f950397` (10) | a cota pela distância às áreas baratas (`ph2d_nav::cota`: heurístico, saída cedo, atalhos, «à vista» — `4,8×` com uma estrada barata no mapa), a queixa «mais estreita que o corpo» (`NavCostAreaNow`), a cena `=7`, os textos que mentiam |

## §1 — Superfície de colisão (medida: `scripts/collision-surface.sh` no worktree, 07/10)

**Fora da família da navegação** (`ph2d-app-components`, `ph2d-nav`, `ph2d-navmesh`, `ph2d-physics-ecs/src/bridge/nav*`,
`docs/Components`) a linha toca:

| ficheiro | o quê | onda |
|---|---|---|
| `shells/desktop/src/init_subsystems.rs` | 1 linha trocada: `insert_white_tile` → `insert_reserved_tiles` | W16 |
| `shells/desktop/src/components_scenes_suplentes.rs` | 1 linha trocada: `montada.secao_do_roteiro()` | W18 |
| `crates/ph2d-render/src/atlas/{mod,tests}.rs`, `lib.rs` | as peças reservadas do atlas (`insert_reserved_tiles`, `DISC_TILE_*`; `insert_white_tile` deixa de ser público) | W16 |
| `crates/ph2d-app-physics/src/{bridge/dispatch.rs, lib.rs, tape_second_life_tests.rs}`, `ph2d-physics-ecs/src/bridge/tape.rs` | a fita grava TODO tique devido (report do dono) | W15 |
| `crates/ph2d-physics-ecs/Cargo.toml` + `Cargo.lock` | `rayon = "1"` na ponte (as procuras a meio em paralelo); o pacote JÁ estava no lock (é o da `ph2d-navmesh`) — nenhum pacote externo novo | W15 |
| `crates/ph2d-physics-ecs/src/{components.rs, components/nav.rs, lib.rs}` | `NavCostAreaNow` (DERIVADO, NÃO registado) e o doc-comment da `NavCostArea` | W19 |
| `crates/ph2d-editor-core/src/nav_edits{,_tests}.rs` · `ph2d-panel-inspector/src/sections/nav_custo.rs` · `ph2d-i18n/src/inspector_nav.rs` | a queixa nova e a frase dela (UMA chave i18n) | W19 |
| testes: `ph2d-panel-inspector/tests/it/a_seccao_nav_custo_esta_viva.rs`, `ph2d-panel-registry-init/tests/it/o_inspector_armado.rs` (um campo), `ph2d-physics-ecs/tests/it/main.rs` | os gates | W15–W19 |
| `docs/architecture/decisions/0180-…md` + `README.md` | o ADR-0180 (PROVISÓRIO no número) | W15–W17 |
| `docs/Physics/BUGS_physics.md` | o #10 (a fita) | W15 |
| `CLAUDE.md` | a linha do §5 dos Componentes (aponta para ESTE handoff, `NAV_SMOKE=1..7`) | W15–W19 |

A shell **não cresce** (duas linhas trocadas, zero a mais). Nenhum contrato congelado (§6) tocado.

## §2 — O que se RECONTA contra o `main` do dia

| número | na linha | a regra |
|---|---|---|
| **ADR** | cria o **`0180`** (*«As procuras de caminho a meio avançam em paralelo»*, com duas adendas) | se outra linha integrar um `0180` antes, este renumera para o próximo livre — o ficheiro, o `README` dos ADR e as citações em `30_plano_navegacao.md` §23–§28 e nos handoffs |
| `PROJECT_SCHEMA` (e a tripla) | `183`, `(183, 13, 22)` — igual à base | intocado: o `NavCostAreaNow` e o `NavNow` NÃO são registados |
| registo de componentes | `106 · 107 · 107` — igual à base | intocado |
| tectos de LOC | nenhum ficheiro da linha passa do tecto; `ph2d-physics-ecs/src/bridge/nav.rs` em **`685 / 700`** | soma entre linhas: se outra linha o fez crescer, quem passa MOVE (não sobe o tecto) |
| `nav_smoke::CENAS` | `7`, contado do roteador por gate | conta-se, não se escolhe |
| i18n | uma chave nova (`panel.inspector.nav.area_narrower_than_body`) | os censos correm no gate da árvore combinada |

⚠️ O `collision-surface.sh` imprime `✗ SONDA CEGA em: FIELD_DOC_VERSION` — é um defeito do SCRIPT (a const
mudou de ficheiro noutro sítio e o caminho dele ficou velho), não desta linha: a linha não toca nenhum
`FIELD_DOC_VERSION`.

## §3 — A prova (sobre o merge-base, W15–W19 juntas)

- **Gate batched** (07/10, o diff final): `BASE=a46c4c200 nextest-impacted` **`16 672 / 16 672`** (`90,9 s`) ·
  clippy `--workspace --all-targets -D warnings` **limpo** · fmt limpo nos ficheiros da linha —
  `target/prova/w19/gate_{nextest,clippy}_2.txt` no worktree.
- **Mutação** por onda: W15 `18/19` e W16 `18/18` (handoffs delas) · W18 `20/20` · W19 **`15/15`**
  (`docs/Components/ferramentas/mutacao_navegacao_w{18,19}_*.py`).
- **O binário do smoke** já compilado no worktree (perfil `smoke`; a 2.ª corrida `Finished … in 0.21s`).
- ⚠️ Os testes de `ph2d-panel-registry-init` REPROVAM numa corrida estreita (`-p ph2d-panel-registry-init`):
  painéis que só entram com as features de um build de WORKSPACE (a própria mensagem do gate o diz). Na corrida do
  workspace passam todos.

## §4 — Depois do merge (o integrador)

1. `cargo check --workspace` na árvore combinada (prova que nenhum consumidor partiu: a linha mudou a API do atlas
   e acrescentou um campo a `InspectorNavCostArea`).
2. O gate da árvore combinada (`/pd-integracao`): `nextest-impacted` + clippy, os censos de i18n e do tutorial.
3. O ADR (§2) e o §5 do `CLAUDE.md`: a linha dos Componentes já aponta para ESTE handoff.

## §5 — ⏳ O que fica aberto (não bloqueia o merge)

- **Quem persegue sem o alvo à vista**, numa zona cheia de áreas de custo e com `200` agentes: até `30` tiques
  para replanear. Medido; só outra procura o baixaria.
- **O crítico do tique** em cenas enormes (`orçamento · 2^k`): o preço da vivacidade numa malha que nunca pára;
  não se vê no relógio. Fechado como escolha medida.
- **A cota com milhares de polígonos baratos** varreria muitas caixas por raiz — por medir quando existir tal cena.
- **A versão no navegador:** aprovada pelo dono e ESTACIONADA («não agora»).
