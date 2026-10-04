# HANDOFF DE INTEGRAÇÃO — `line/components`: O COMPORTAMENTO QUE FALTAVA e A VELOCIDADE EM CENAS GRANDES (navegação W13 + W14) — 2026-10-04

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §21 (W13) e §22 (W14). Este documento é
> o que o integrador precisa.
>
> ⚠️⚠️ **A linha leva DEZ waves por integrar**, juntas e sem reverter nada umas das outras: W5–W10 (os handoffs
> listados no da [W11](HANDOFF_INTEGRACAO_line_components_AS_PAREDES_POR_MOSAICOS_2026-10-04.md)), a W11, a W12
> ([`O_MOSAICO_MAIS_DEPRESSA_2026-10-04`](HANDOFF_INTEGRACAO_line_components_O_MOSAICO_MAIS_DEPRESSA_2026-10-04.md))
> e estas duas (a W13 não teve handoff próprio — está aqui). Smoke do dono APROVADO na W5–W10; **o da W11–W14
> está por fazer** (§5). ⛔ Ordem do dono: a navegação fecha COMPLETAMENTE antes de integrar.

## §0 — O `--ff-only` deve passar limpo

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · `git merge-base --is-ancestor main HEAD`.
- Por cima de `ecc7c8c58` (o handoff da W12): a W13 `5e5eaef78` (o golpe) · `4196b7a6f` (o corpo composto) ·
  `4744e87df` (quem nasce junto) · `8027d10aa` `32bc93ad8` `8b104c633` (o arnês, o gate da fila cheia, clippy);
  a W14 `2e547489f` (o trabalho da procura) · `010f447e8` (os mosaicos em paralelo) · `227bffa4a` (os vizinhos
  por anéis) · `a12c7196c` (o corpo largo em polígono) — e os do fecho (gates da cápsula e do torniquete, a
  mutação, este handoff).

## §1 — Superfície de colisão

**`ph2d-nav`, `ph2d-navmesh`, `ph2d-orca`, `ph2d-physics-ecs` (a ponte da navegação).** Zero número que soma entre
linhas **salvo o ADR** (abaixo), zero crate nova, **zero pacote novo no `Cargo.lock`** (o `rayon` já estava
resolvido; a linha do `Cargo.lock` é a dependência da `ph2d-navmesh`), **zero linha em `shells/desktop`**, zero
i18n, zero componente registado, `PROJECT_SCHEMA` intocado. Prova por grep sobre a W13+W14 (`git diff --name-only ecc7c8c58 HEAD`): zero
ficheiro em `ph2d-nodegraph`, `ph2d-tool-*`, `ph2d-vector*`, `ph2d-imageio`, registos ou esquemas (os contratos congelados
do §6), e zero linha `PROJECT_SCHEMA` no diff (os degraus do esquema da linha são da W5–W8, nos handoffs delas).

| ficheiro | o quê |
|---|---|
| `ph2d-nav/src/polyanya.rs` · `polyanya_dominancia.rs` · **`polyanya_trabalho.rs` NOVO** | `Stats` ganha `pending` e `compared`; `Stats::work()` = o trabalho em nós uniformes (sem lama = `expanded` ao bit) |
| `ph2d-nav/src/agent.rs` · `refresh.rs` | **API pública renomeada:** `AgentRuntime::last_nodes → last_work`, `refresh::Owed::nodes → work` (só a ponte e os testes da linha os liam) |
| `ph2d-nav/src/polyanya_custo.rs` | uma tabela de custos toda a `1` é uniforme sem varrer a malha |
| `ph2d-navmesh/Cargo.toml` · `src/tiles.rs` | `rayon`: os mosaicos a refazer constroem-se em paralelo (`TiledMesh::update*` pede `S: Sync`) |
| `ph2d-orca/src/crowd.rs` · **`vizinhos.rs` NOVO** | a grelha fina dos vizinhos por anéis; **`Movel` NOVO** (polígonos com velocidade) e `Crowd::with_moving` |
| `ph2d-physics-ecs/src/bridge/nav.rs` · `nav_fila.rs` · `nav_desvio.rs` | a fila pelo trabalho (`ORCAMENTO_DE_TRABALHO_POR_TIQUE`, era `…_DE_NOS_…`); a W13 (golpe, peças, quem nasce, a ordem das entidades); `forma()` substitui `discos()` |
| testes | `ph2d-nav` `agent_tests.rs` · `ph2d-navmesh` `tiles_tests.rs`, `tests/it/dominancia.rs` · `ph2d-orca` `vizinhos_tests.rs` (`#[cfg(test)]`), `tests/it/oraculo_do_godot_largo.rs` + `tests/fixtures/godot/largo.txt` · `ph2d-physics-ecs` `tests/it/nav_desvio_corpos.rs`, `nav_nascer.rs`, **`nav_desvio_largo.rs` NOVO**, `nav_mundo.rs` |
| docs | plano 30 §21–§22; **ADR-0178** (o paralelo dos mosaicos); `ferramentas/godot_nav_oraculo/largo.gd`; os arneses `mutacao_navegacao_w13/w14_2026-10-04.py`; `examples/medir_replaneio.rs` (`LAMAS=`) |

### ⚠️ O que um merge pode partir

1. **O número do ADR SOMA entre linhas:** `0178` foi contado contra o máximo das worktrees do dia (`0177`) —
   reconte-o contra o `main` na integração (o nome do ficheiro, o título, e as citações no plano §22.2 e no
   `Cargo.toml` da `ph2d-navmesh`).
2. Uma linha que chame `TiledMesh::update`/`update_with_areas` com um tipo de forma que não seja `Sync`, ou que
   leia `last_nodes`/`Owed::nodes`, não compila — a cura é o nome novo.
3. Tectos: `ph2d-nav` `polyanya.rs` `~685 / 700` · `bridge/nav.rs` `~651` · `ph2d-orca` `crowd.rs` `~450` ·
   `nav_desvio.rs` `~290` · `tests/it/nav_mundo.rs` `~680`.

## §2 — O que as waves trazem

**W13 (plano §21) — o comportamento:** um corpo EMPURRADO por um golpe é visto a andar pelo desvio (a velocidade
é o comando mais o empurrão); um corpo COMPOSTO desvia-se pela forma inteira (as peças); quem NASCE junto procura
pela fila (a 1.ª procura de quem não tem caminho gasta do orçamento do tique; quem não cabe espera parado); a
condução corre pela ordem das ENTIDADES. Mutação **8 / 8**.

**W14 (plano §22) — a velocidade:**

| item | antes | depois |
|---|---|---|
| a fila do replaneio | contava NÓS (`85–321 ns` por nó conforme a lama) | conta o TRABALHO (`Stats::work`, pesos medidos: `85–123 ns` por unidade, `20 000 ≈ 2,3 ms`); sem lama, a mesma fila ao tique |
| uma procura sem lama | varria `~15 000` polígonos para saber que é uniforme (`5–7 µs`) | não varre |
| a malha a frio (`100 × 100 m`, `1 000` obstáculos, `100` lamas) | `55,3 ms` | **`7,5 ms`** calmo (a mesma malha ao bit) |
| o desvio a `1 000` agentes | `2,44 ms` (a vizinhança `~70 %`) | **`0,66 ms`** calmo, a vizinhança `0,48` (as mesmas listas) |
| um corpo LARGO que vem de frente | o agente nunca chegava (colado, ou empurrado `6 m`) | contorna-o (polígono com velocidade; o Godot: `458` quadros, nós `461`) |
| a procura ponderada com muita lama | `~4×` os nós da uniforme | ⛔ **recusa medida** (§22.4): o excesso é a região `f < C*` de uma procura exacta, não a grelha |

## §3 — ⏳ O que fica ABERTO

- **A barreira larga e LENTA** (`3 m` a `0,3 m/s`): contorna a `2,9×` o tempo do CONTROLO — o caminho puxa para o
  eixo contra o peso de lado (§22.6). A cura é do CAMINHO, não do desvio.
- A procura ponderada com muita lama (§22.4) — nenhuma alavanca medida chega a metade.
- **Com muita lama o tique depois de uma porta continua alto** (`150` lamas: `39 · 47 · 88 ms` a 10 · 50 · 200
  agentes, calmo, contra `3 · 4 · 7` sem lama) — o critério 3 do §22.1 NÃO se cumpriu: UMA procura passa o
  orçamento sozinha (§22.4), e os replaneios que não passam pela fila (alvo que se mexeu, corredor perdido,
  preso) fazem `1–7` procuras num tique mesmo sem porta (plano §22.8). Pô-los no orçamento é uma wave própria.
- O piso das bibliotecas no mosaico; a montagem O(malha) (`~2,6 ms` a frio, série).

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), 96 commits e 185 ficheiros, dentro da fatia
da linha (load `15–29`):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 553 / 19 553`**, `127 s` |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (nav, navmesh, orca, physics-ecs, app-components, host-desktop) | zero |
| `file_loc_caps` da shell `4/4` · `workspace_src_files_under_loc_cap` · `arch_safe_clamp_only` `2/2` · `architecture_*` da shell `3/3` e do editor-core `102` · `fn_loc_caps` `2/2` | verdes |
| `no_std_transcendental_reaches_the_deterministic_hash` · `deterministic::` `2/2` · `scrub::` `5/5` | verdes |
| `cargo fmt --all --check` · `typos` (o diff, com `--force-exclude`) · `cargo machete` | verde · zero · zero |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` · `censos-da-arvore-combinada.sh` · `doc-index.sh --check` | verdes · `12/12` censos · `20` índices em dia |
| o tecto do `CLAUDE.md` `3/3` · o do índice da memória `3/3` | verdes |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote novo).

**Mutação:** W13 **8 / 8** ([`mutacao_navegacao_w13_2026-10-04.py`](../ferramentas/mutacao_navegacao_w13_2026-10-04.py));
W14 **17 / 17**, zero defeitos de arnês ([`mutacao_navegacao_w14_2026-10-04.py`](../ferramentas/mutacao_navegacao_w14_2026-10-04.py),
seis conjuntos de observadores; a 1.ª corrida deu `16 / 17` — a L2, a velocidade ABSOLUTA do agente contra um corpo
que anda, sobrevivia aos desfechos; o gate da invariância de referencial nasceu disso). Os gates da cápsula e do
torniquete nasceram ANTES da prova (as duas leis não tinham régua).

**As medições** (a mesma malha ao bit: impressão digital `42f3d779b02e0329` sobre `180` malhas, antes == depois;
o oráculo do Godot do corpo largo: `458` quadros lá, `461` cá) — no plano §22.1–§22.8, com o load de cada uma.

**Auditoria (3 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **determinismo** | o paralelo não muda a malha; a fila não lê relógios | `update_with_areas` → `faltam` (série, ordem da chave) → `par_iter().map(constroi).collect()` (indexado) → `poe` (série) · `Stats::work` só contagens → `last_work` no anel | `os_mosaicos_feitos_em_paralelo_sao_os_de_uma_thread_ao_bit` (P1) · `deterministic::`, `scrub::`, os scrubs da fila e dos nascimentos |
| **correcção** | os vizinhos e as linhas são os de antes / os do referencial | `Crowd::neighbors` → `GrelhaFina::anel` (minorante com a folga da célula) → `select_nth` · `resolve` → `Movel::vel_em` → `wall_lines(rel)` + `u` | `os_vizinhos_por_aneis_sao_os_da_varrida` (V1–V4) · `um_corpo_que_anda_e_o_mesmo_parado_no_referencial_dele` (L1–L3) · o oráculo do Godot |
| **comportamento** | um corpo largo que anda é contornado sem roçar | `desvia` → `forma` (polígono; bola = disco) → `Movel` com a folga `|u|·τ` | `um_corpo_largo_que_vem_de_frente_e_contornado` (+ CONTROLO) · `uma_capsula_que_anda_e_um_torniquete_que_roda…` (L4–L6) · `um_corpo_composto_desvia_se_pela_forma_inteira` |
| **não-checado pela compilação** | os números de tempo | sondas provisórias (fora do repo no fecho) e as de sempre, alternadas antes/depois; calmo no fecho | — |

## §5 — O smoke

As mesmas cenas da W11/W12 (nenhuma cena nova: o que mudou é a velocidade e o desvio contra corpos que andam):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

Errado = na `=3` o guarda PASSA a porta depois de ela fechar, roça nela enquanto ela anda, ou não volta à ronda;
na `=4` um inimigo entra na lava ou não usa o portal; na arena um morcego preso no muro, ou um inimigo empurrado
por um golpe faz os outros desviarem à toa. ⚠️ **Só a bateria prova** (nenhuma cena o mostra): muitos que nascem
JUNTOS começam a andar aos poucos (numa cena pequena todos cabem no 1.º tique), o arranque de uma cena GRANDE, o
desvio a `1 000`, e o corpo largo que vem de frente (`nav_desvio_largo`).

Fotografadas no ecrã virtual antes de ir ao dono (`target/prova/w14/ph2dnavsmoke3.png`, `ph2dnavsmoke4.png`,
`ph2dvidasmoke4.png`, `60 fps`): na `=3` o guarda patrulha (*«Moving · 2.09 m to go»*); na `=4` o vermelho deu a
volta à lava e *«Arrived»* ao herói; na arena os morcegos andam (*«Signal: morcego»*).

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental` (`11 G` do `debug` + `756 M` do `smoke`), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.35s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                230   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                724 : 247   alvo: <= 1,0  razao 2.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1739 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         486 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
