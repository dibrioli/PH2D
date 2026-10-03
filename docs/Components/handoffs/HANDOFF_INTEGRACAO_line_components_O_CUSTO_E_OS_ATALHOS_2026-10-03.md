# HANDOFF DE INTEGRAÇÃO — `line/components`: O CUSTO E OS ATALHOS (navegação W7) — 2026-10-03

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §15 (a medição que derrubou o A*+funil,
> as decisões, as recusas medidas, o que fica). Este documento é o que o integrador precisa.
>
> ⚠️⚠️ **A linha leva TRÊS waves por integrar**: a W5 (o desvio — handoff
> [`DESVIO_2026-10-02`](HANDOFF_INTEGRACAO_line_components_DESVIO_2026-10-02.md), smoke APROVADO), a W6 (handoff
> [`O_MUNDO_QUE_MUDA_2026-10-02`](HANDOFF_INTEGRACAO_line_components_O_MUNDO_QUE_MUDA_2026-10-02.md), smoke
> APROVADO) e esta W7 (**15 commits** `60a68c1e9..7e0253a2b` por cima de `c14e0a78c`; **38** sobre o `main`). Integram-se
> juntas: a W7 não reverte nada das anteriores.

## §0 — O `--ff-only` deve passar limpo (medido)

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · HEAD `7e0253a2b` ·
  `git merge-base --is-ancestor main HEAD` ⇒ verdade no fecho.
- ⚠️ A linha muda o `CLAUDE.md` da worktree (a entrada dos Componentes no §5.1: o smoke `=1..4` e o link deste
  handoff) — se o primário mexer no §5 entretanto, a fusão é de UMA linha.
- ⚠️ Os 15 commits da W7: `60a68c1e9` procura com custo · `90d908a70` split do polyanya · `3d5a2c953` mosaicos com
  áreas · `f1684eb17` atalhos na lei · `3eb1a902b` ponte · `473af7c63` plano §15 · `abdfe2cc1` réguas+arnês ·
  `3a4b4e4f7` clippy · `d771cb849` Inspector · `feb6b0a2b` cena `=4` · `b30d6b933` espigão do Clipper ·
  `3e0706c9d` chegada uma vez · `ec340bd7c` arnês do Inspector · `4cf6e0e93` fmt · `7e0253a2b` sai o dobrar redundante.

---

## §1 — Superfície de colisão, MEDIDA

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, colado no fecho:

```
SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 1ad60a1ce   ·   38 commit(s)   ·   128 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        180   (base: 178)
  ⚠   └ tripla do gate               (180, 13, 22)   (base: (178, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              108   (base: 108)
    ph2d-render (espelho)                 109   (base: 109)
    ph2d-script (espelho)                 109   (base: 109)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0175   próximo livre: 0176
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-orca"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

(`ph2d-orca` é da W5; a W7 não traz pacote externo nem crate nova.)

| grandeza | aqui | `main` | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `180` | `178` | **+2** (`178→179` é da **W5**; `179→180` é da **W7**) | UM degrau na W7: `NavAgent.avoid_harm` apendado + `NavCostArea` + `NavLink` registados. Tripla `(180, 13, 22)`. `python3 scripts/schema-recount.py` só funciona durante um rebase (é ferramenta de integração) |
| registo de física (`ph2d-physics-ecs`) | `44` | `42` | **+2** | `NavCostArea`, `NavLink` |
| registos `ph2d-ecs` / `-render` / `-script` | `108` / `109` / `109` | iguais | **0** | espelhos não mexem |
| paleta de física | `8` | `6` | +2 | `NavCostArea`, `NavLink` autorados, **sem companheiros** no catálogo |
| `LIVE_SECTIONS` | `47` | `45` | **+2** | `INSP_LIVE_NAV_COST_AREA_SECTION`/`_GRIP`, `INSP_LIVE_NAV_LINK_SECTION`/`_GRIP` (pares apendados) |
| `NavFieldEdit` | | | **+8** apendados depois de `AlvoTag` | `AvoidHarm(bool)` · `CostAreaCost(f32)` · `CostAreaForbidden(bool)` · `LinkTo(String)` · `LinkTwoWay(bool)` · `LinkTeleport(bool)` · `LinkCost(f32)` · `LinkOnCrossed(String)` |
| ids novos do painel | | | **+8** | `INSP_NAV_AVOID_HARM` · `INSP_NAV_AREA_COST` · `INSP_NAV_AREA_FORBIDDEN` · `INSP_NAV_LINK_TO` · `INSP_NAV_LINK_TWO_WAY` · `INSP_NAV_LINK_TELEPORT` · `INSP_NAV_LINK_COST` · `INSP_NAV_LINK_ON_CROSSED` |
| `NAV_AREA_COST_MIN` | `pub const … f32 = 0.01` | | novo (editor-core) | chão MATEMÁTICO: o custo tem de ser `> 0` para ser uma distância |
| `populate_nav::NUMEROS` | `10` | `8` | +2 | + o array novo `TAXAS` |
| `InspectorNavAgent` | | | +2 campos | `avoid_harm`, `has_health` |
| `InspectorNavInfo` | | | +2 campos | `cost_area`, `link` |
| espelhos novos | | | | `InspectorNavCostArea{cost, forbidden, has_shape, body_moves}` + `CostAreaQueixa{SemForma, CorpoQueAnda}` · `InspectorNavLink{to_nome, to_perdido, two_way, teleport, cost, on_crossed}` + `LinkQueixa{SemSaida, SaidaPerdida}` |
| i18n | | | **+23** chaves | `component.nav_cost_area.name` «Nav Cost Area» · `component.nav_link.name` «Nav Link» · +21 `panel.inspector.nav.*` |
| `ph2d_nav::Event` | | | +1 variante | `Crossed(u32)` apendada |
| `ph2d_nav::Path` | | | +1 campo | `cost` (construído só dentro da `ph2d-nav`) |
| `ph2d_nav::Steer` | | | +2 campos | `teleport: Option<V2>`, `crossed: Option<u32>` |
| `AgentRuntime` | | | +2 campos | `hops`, `arrival_told` (struct de campos privados: sem literais fora) |
| `NavMesh` | | | fns novas | `area_id`, `has_areas`, `from_polygons_with_areas` · `MeshError::AreaCount` apendado |
| `ph2d-navmesh` | | | API nova | `Area`, `build_with_areas`, `TiledMesh::update_with_areas` (`update` = com `&[]`), `triangulate_pieces`, `merge_convex_labeled` · `TriError` **igual** (um `Unowned` nasceu e saiu dentro da wave) |
| `Damage::magoa` | | | fn nova | `ph2d-physics-ecs` `components/health.rs` |
| chave da malha na ponte | `ChaveMalha = (Entity, u32, u64)` | `(Entity, u32)` | tipo mudou | `nav_meshes()` — a assinatura pública **igual** |
| `nav_smoke::CENAS` | `4` | `3` (W6) | +1 | a `=4`; `nav_smoke::parede`/`perseguidor` passam a `pub(crate)` |
| crates novas · pacotes externos | `0` · `0` | | | |
| `shells/desktop/src` (só a W7) | `+14 / −2` | | | só `project_schema.rs` + `project_schema_tests.rs` (o degrau); desde o `main`: `+35 / −12` |

### §1.1 — Foundational tocado, e porque é aditivo

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-nav/src/mesh.rs` | a malha com áreas (`area_id`, `has_areas`, `from_polygons_with_areas`) | apendado; a malha sem áreas é a de sempre ao bit (gate `areas::sem_areas_a_construcao_e_a_de_sempre_ao_bit`) |
| `crates/ph2d-nav/src/{cost.rs, link.rs}` **NOVOS** · `polyanya.rs` + filho **NOVO** `polyanya_custo.rs` | a procura ponderada (atravessar + deslizar + polimento de Snell, raízes preguiçosas, atalho exacto), os custos, os atalhos | módulos novos; `polyanya.rs` volta abaixo do tecto movendo a porta ponderada, os predicados de custo e o `reconstruct` para o filho |
| `crates/ph2d-nav/src/agent.rs` | `step_with`, `hops`, teleporte, `crossed`, chegada UMA vez | ⚠️ `Event::Crossed` apendado: todo `match` exaustivo sobre `Event` fora da crate tem de o cobrir (só `signals.rs` no repo) |
| `crates/ph2d-nav/src/oracle.rs` | `WeightedOracle` | não substitui nada público do oráculo uniforme |
| `crates/ph2d-navmesh/src/triangulate.rs` | `triangulate_pieces` (paridade generalizada, plano B exacto de ponto-na-peça, reparação de junções-T a ≤ 2 unidades da grelha) e `limpa_anel` | ⚠️ **reescrita interna**: a `limpa_anel` corre em **TODA** construção, também sem áreas — só remove espigões de área zero (cura de um vazamento da W6, ver §2) |
| `crates/ph2d-navmesh/src/{lib.rs, tiles.rs}` | `poligonos` é a porta partilhada; áreas por mosaico | apendado |
| `crates/ph2d-physics-ecs/src/components/{nav.rs, health.rs}` · `lib.rs` | `NavCostArea`, `NavLink`, `NavAgent.avoid_harm`, `Damage::magoa`, o registo | apendado (o degrau do schema) |
| `crates/ph2d-physics-ecs/src/bridge/{nav.rs, nav_malha.rs, nav_desvio.rs, signals.rs, tape.rs}` + `nav_custo.rs` **NOVO** | o teleporte por `set_body_pose` DENTRO de `drive_nav_agents`; custos, atalhos e zona evitada derivados por tique; `AgentRuntime` clonado inteiro para o anel | ⚠️ a chave da malha mudou de tipo (ver §3) |
| `crates/ph2d-editor-core/src/nav_edits.rs` + espelhos do Inspector | os 8 `NavFieldEdit`, `NAV_AREA_COST_MIN`, os espelhos | apendado; ⚠️ campos novos em `InspectorNavAgent`/`InspectorNavInfo` |
| `crates/ph2d-panel-inspector/src/{ids/inspector_nav.rs, populate_nav.rs, event_nav.rs, sections/nav_custo.rs (NOVO), sections/nav.rs, sections/mod.rs}` | as secções Nav Cost Area e Nav Link, o «Avoid Harm» | `nav.rs` **583 → 564** LOC; o gate `architecture_every_live_section_is_in_the_table` passou a ler as entradas **por parêntesis** (o rustfmt parte um par longo em 3 linhas) — **não afrouxado** |
| `crates/ph2d-panel-registry-init` (fixtura de teste) · `crates/ph2d-i18n` · `crates/ph2d-component-desc` | o registo das secções, as chaves, o catálogo | sim |
| `crates/ph2d-app-components/src/{nav_inspector.rs, nav_smoke.rs, nav_smoke_lava.rs (NOVO)}` + testes | a ponte do Inspector e a cena `=4` | sim |
| `shells/desktop/src/{project_schema.rs, project_schema_tests.rs}` | o degrau `179 → 180` | sim |

### §1.2 — Contratos congelados

**Nenhum** — `node.rs` e `tool.rs` intocados (a superfície acima o diz); nenhum ADR.

---

## §2 — O que a wave traz

O plano §15 tem tudo. Em um parágrafo: a **medição derrubou o A\*+funil planeado** (o idioma da indústria fica
27–43 % acima do óptimo no p95; ignorar o custo chega a 7,9×), por isso o produto é o **Polyanya que refracta**
(atravessar + deslizar + polimento de Snell, raízes preguiçosas, atalho exacto) com `STEINER_M` `0,25 m` (média
`1,0000`, máximo `~1 %`). Zona **proibida** = buraco; **lava** (um `Damage` em repouso que `Damage::magoa` diz que
fere ESTE agente) = buraco na malha DESSE agente (o `Avoid Harm`, decisão do dono de 02/10: o inimigo evita
sozinho, com caixa para desligar); **lama** = área de custo finito; **Nav Link** = portal de teleporte ou porta de
um sentido, com `on_crossed` vindo do atalho; cena `=4`.

**Dois defeitos da W6 achados e curados na W7 porque a foto da cena os mostrou** (nenhum gate os via):

1. o espigão do Clipper sobre a costura de um mosaico deixava **chão DENTRO de um furo** (precisa de um furo a
   cortar a quina de quatro mosaicos com paredes a tocar a região; reproduzido no HEAD da W6 `c14e0a78c`) ⇒
   `limpa_anel` + os gates `mosaicos::o_furo_na_quina_de_quatro_mosaicos_nao_vaza` e
   `em_regioes_negativas_com_paredes_no_bordo…`;
2. o «chegou» **re-anunciava a cada empurrão** entre dois agentes no mesmo alvo (6 sinais em 15 s) ⇒ a chegada
   anuncia-se UMA vez por aproximação (rearma além de `arrive`+`repath`); gates em `atalhos` e `nav_smoke_lava`.

---

## §3 — ⚠️ O que um merge textual pode partir

1. **`ph2d_nav::Event::Crossed(u32)`** apendada: todo `match` exaustivo sobre `Event` fora da `ph2d-nav` precisa
   da variante (no repo só o `signals.rs`; uma linha que tenha escrito outro `match` não compila).
2. **`InspectorNavAgent { … }` e `InspectorNavInfo { … }`**: literais noutra linha não compilam enquanto não
   levarem os campos novos (`avoid_harm`, `has_health` · `cost_area`, `link`) — `grep -rln 'InspectorNavAgent {'`.
3. **Os degraus do schema são DOIS** (W5 e W7), em ficheiros irmãos: contar (`180` contra o `main` do dia), nunca
   escolher.
4. **`LIVE_SECTIONS` (`+2` pares)** e **`NavFieldEdit` (`+8` variantes)**: se outra linha também apendou, o
   merge guarda as duas — contar, e conferir a posição das variantes.
5. **A chave da malha da ponte** passou a `ChaveMalha = (Entity, u32, u64)`: uma linha que toque `nav_malha.rs`
   / `nav.rs` pelas chaves conflitua; a assinatura pública de `nav_meshes()` não muda.
6. **`triangulate.rs` reescrito por dentro** (`limpa_anel` + peças): conflitos ali resolvem-se pelos gates
   `areas::*`, `mosaicos::*`, `contra_o_exacto` e o `determinismo` (entre processos).
7. **`SignalVerb` tags `12`/`13`** (da W6) continuam POSICIONAIS no ficheiro — ver o handoff da W6 §3.1.
8. **`nav_smoke::parede`/`perseguidor`** passaram a `pub(crate)`; a cena `=4` depende deles.

---

## §4 — ⏳ O que fica ABERTO

- **A procura ponderada numa cena cheia de lama custa `~11×` a uniforme** (100 lamas + 1 000 obstáculos:
  `64 500` nós contra `5 650`; mediana `~4,9 ms` sob carga). Próxima alavanca: **dominância entre frentes
  paralelas**. Só se paga com áreas de custo FINITO que o caminho uniforme toca (a lava é buraco ⇒ procura
  uniforme, sem este custo).
- **A construção inteira com áreas é lenta em escala** (`1,2 s` a 100 lamas / 1 000 obstáculos). A ponte usa
  mosaicos: `~71 ms` a frio; uma lama a mover-se `~11 ms` — a montagem O(malha) da W6.
- Os abertos da W6/W5 continuam: a montagem da malha O(malha) · todos os agentes da malha que mudou recalculam
  no mesmo tique · a porta que anda é um círculo para o desvio · o Inspector diz *«Switched off»* de um agente
  desligado que um `Start` pôs a andar · a leitura viva não diz «a dar passagem» · a cena `=1` apertada ·
  o defeito G · o flake `text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout`.
- **Smoke do dono: PENDENTE** (ainda não corrido — ver §6).
- Waves seguintes: **W8** (a arena `PH2D_VIDA_SMOKE=4` com morcegos que evitam paredes · o tutorial
  `03_navegacao.pdf`).

---

## §5 — A prova de fecho

Tudo corrido **1× sobre o diff acumulado**, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **`19 498 / 19 498`** verdes (`11 485` saltados, `134,8 s`) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `cargo clippy --all-targets --all-features -D warnings` nas 13 crates (`ph2d-nav` · `ph2d-navmesh` · `ph2d-orca` · `ph2d-physics` · `ph2d-physics-ecs` · `ph2d-ecs` · `ph2d-i18n` · `ph2d-editor-core` · `ph2d-panel-inspector` · `ph2d-panel-registry-init` · `ph2d-app-components` · `ph2d-component-desc` · `ph2d-host-desktop`) | **zero** |
| `cargo fmt --all --check` | verde |
| `typos` project-wide · `cargo machete` | zero · zero |
| `check-standalone-optional.sh` (10) · `check-workflow-packages.sh` (32/398) | verdes |
| `censos-da-arvore-combinada.sh` (a árvore combinada É esta: o `main` é ancestral) | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | `20` índices em dia |
| `#[cfg(target_os` escrito/movido | **nenhum** |
| a máquina no fim | nada desta linha a correr; `/dev/dri` só Xwayland/plasmashell/code |

**Mutação 40 / 40** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w7_2026-10-03.py`](../ferramentas/mutacao_navegacao_w7_2026-10-03.py), quatro controlos;
grupos NAVMESH 22 testes · ECS 5 · APPC 13 · PANEL 18 · EDCORE 8). A 1.ª corrida deu `40/41`: a **M8 «o canto de
custo não dobra» SOBREVIVEU e era EQUIVALENTE** (sem ela a sonda dá os mesmos custos ao dígito e `26 %` menos
nós) ⇒ a lei **saiu do código**. Antes da corrida escreveram-se **4 réguas** para leis que não tinham (o leque que
pára na fronteira: `9 972` contra `15 309 326` nós · a meio da porta não se replaneia · quem se cura com o fogo
atravessa · onde duas áreas se sobrepõem manda a mais cara).

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **determinismo / replay** | um scrub depois de um teleporte devolve a mesma corrida | o teleporte por `set_body_pose` dentro de `drive_nav_agents` (`bridge/nav.rs`, antes dos movers, nos dois laços) · `AgentRuntime` (`hops`, `arrival_told`) clonado INTEIRO para o anel (`bridge/tape.rs` `ControllerMemory`) · custos, atalhos e chaves da zona evitada derivados por tique em ordem de entidade (`BTreeMap`) · ordem do heap `ord_key`+`seq` · poda da raiz de fronteira em `BTreeMap` · `libm::atan2f` na rotação mantida | `nav_custo::um_scrub_depois_do_salto_devolve_a_mesma_corrida` · `atalhos::*` · `areas::sem_areas_a_construcao_e_a_de_sempre_ao_bit` · `determinismo` (entre processos) |
| **costura do painel** | Avoid Harm / Forbidden / Teleport / Both Ways / os campos chegam aos componentes | `ids/inspector_nav.rs` → `populate_nav.rs` → `sections/nav_custo.rs` (hit-index) → `event_nav.rs` (`interruptor_w7`) → `EditorAction::InspectorComponentEdit(ComponentEdit::Nav)` → `nav_inspector::apply_nav_edit` | panel-inspector `a_seccao_nav_custo_esta_viva` (cliques REAIS do ponteiro) · `nav_inspector_tests` (ida e volta) · mutações M33–M41 |
| **não-checado pela compilação** | a cena `=4` ensina o que acontece | `nav_smoke_lava::a_cena_contem_o_fenomeno` (o vermelho fica a `≥ 1,94 m` da lava, salta `9,57 m`, chega com os `100` HP) · `cada_inimigo_diz_que_apanhou_uma_vez` · as 4 fotos (ecrã virtual) **que ACHARAM os dois defeitos da W6** (§2) | o gate da cena (o cinzento atravessa e cai a `~80,8` HP) |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote externo novo).

---

## §6 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

O que o Enio vê: um **rio de lava laranja** com um vão estreito em cima, **dois portais azuis** em baixo e o
**herói amarelo** à direita (setas para o mover). O inimigo **VERMELHO** (Avoid Harm ligado; está seleccionado
no Inspector) vai ao portal da esquerda, salta para o da direita e apanha o herói **sem tocar na lava**. O
**CINZENTO** (o controlo: o mesmo inimigo com Avoid Harm desligado) atravessa a lava a direito e a barra de vida
dele desce. Desligar o Avoid Harm no vermelho faz com que ele também atravesse; tirar o Nav Link do Portal A faz
com que dê a volta pelo vão de cima. Errado = o vermelho pisar a lava com o Avoid Harm ligado, ou o cinzento
não perder vida. As cenas `=1`..`=3` continuam iguais.

⏳ **Smoke do dono: PENDENTE** (ainda não corrido).

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.09/passo   alvo: >= 1,5  (7% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                210   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check               1693 : 544   alvo: <= 1,0  razao 3.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1832 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         570 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

### O smoke compilado (a 2.ª corrida, colada)

<!-- colar aqui -->
