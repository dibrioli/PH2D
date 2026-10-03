# HANDOFF DE INTEGRAÇÃO — `line/components`: O MUNDO QUE MUDA (navegação W6) — 2026-10-02

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §14 (as medições, as decisões, o
> que a medição derrubou, as recusas medidas, o que fica). Este documento é o que o integrador precisa.
>
> ⚠️⚠️ **A linha leva DUAS waves por integrar**: a W5 (o desvio — handoff
> [`DESVIO_2026-10-02`](HANDOFF_INTEGRACAO_line_components_DESVIO_2026-10-02.md), 6 commits, smoke APROVADO)
> e esta W6 (11 commits por cima de `36116c3f5`). Integram-se juntas: a W6 não reverte nada da W5.

## §0 — O `--ff-only` deve passar limpo (medido)

- base `1ad60a1ce` (o `main` de 02/10, que não andou durante a linha) · HEAD no fim deste documento ·
  `git merge-base --is-ancestor main HEAD` ⇒ verdade no fecho.
- ⚠️ A linha muda o `CLAUDE.md` da worktree (a entrada dos Componentes no §5.1: o smoke `=1|2|3` e o link
  deste handoff) — se o primário mexer no §5 entretanto, a fusão é de UMA linha.

---

## §1 — Superfície de colisão, MEDIDA

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, colado no fecho:

```

SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 1ad60a1ce   ·   18 commit(s)   ·   95 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        179   (base: 178)
  ⚠   └ tripla do gate               (179, 13, 22)   (base: (178, 13, 22))
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

| grandeza | aqui | `main` | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `179` | `178` | **+1** (é da **W5**) | a W6 **não** sobe o schema: `NavTarget` ganha 2 variantes e `SignalVerb` 2 — APENDADAS (o postcard lê as antigas igual); nenhum componente registado novo |
| registos `ph2d-ecs` / `-render` / `-script` / física | iguais | | **0** | o `NavRoute` é DERIVADO e não registado (a lei do `NavNow`) |
| `SignalVerb::ALL` | `14` | `12` | **+2** | `StartNavigation` = tag `12`, `StopNavigation` = tag `13` ⚠️ **POSICIONAL no ficheiro** |
| `INSP_ACTION_VERB` | `14` | `12` | **+2** | `insp_action_verb_start_navigation` / `_stop_navigation` — PARALELO ao `ALL` (gate `the_verb_labels_come_from_the_engines_own_list`) |
| `ArgKind` / `ActionArgHint` | `+ObjectName` | | +1 cada | apendado |
| `NavAlvoModo::ALL` · `INSP_NAV_TARGET_MODE` | `5` | `3` | **+2** | `Tag`, `Patrulha` (`insp_nav_target_tag`, `insp_nav_target_patrol`) |
| ids novos do painel | | | **+65** | `INSP_NAV_TAG_PICK` (`insp_nav_tag_pick`) + `INSP_NAV_TAG_OPT[64]` (`insp_nav_tag_opt_00..63`) |
| `NavFieldEdit` · `AgentQueixa` | | | +1 · +1 | `AlvoTag(u64)` · `SemForma` (apendados) |
| slots de popover do Inspector | | | +1 | `PENDING_NAV_TAG_DD` (o 4.º de tag) |
| i18n | | | **+10** chaves | `ecs.signal_verb.{start,stop}_navigation` · `panel.inspector.actions.object_name_empty_own` · `panel.inspector.nav.{target_tag,target_patrol,shape,shape_name_u,pick_a_tag_u,that_tag_was_deleted,shape_lost}` |
| `nav_smoke::CENAS` | `3` | `1` | +2 (a `=2` é da W5) | a `=3` |
| `ControllerMemory` | | | +2 campos | `nav_ordens`, `nav_rondas` (o tipo obriga `record`/`seed`) |
| crates novas · pacotes externos | `0` · `0` | | | (a `ph2d-orca` é da W5) |
| linhas da `shells/desktop/src` (só a W6) | `+7 / −7` (**0**) desde `36116c3f5` | | | `fase_tabela_de_accoes` −1 (a entrega vira uma porta da família) · `components_scenes_suplentes` 0 (`montar` → `montar_cena`) · a árvore das tags vai à ponte (ver §3) |

### §1.1 — Foundational tocado, e porque é aditivo

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-navmesh/src/tiles.rs` (+`tiles_tests.rs`) · `lib.rs` | `TiledMesh` + `TILE_M`, `TileStats` | módulo NOVO; a `build` inteira fica igual |
| `crates/ph2d-nav/src/mesh.rs` | `from_polygons`: vizinhança pelos polígonos de cada vértice (sem `BTreeMap`), listas contíguas para `vert_polys` e a grelha, anéis movidos sem cópia | ⚠️ **reescrita interna, a MESMA malha** (`5,9 → 2,6 ms` a 11 017 polígonos); a API pública não muda; 18/18 + o gate novo da malha sobreposta |
| `crates/ph2d-orca/src/walls.rs` | `from_walkable_walls`: vectores indexados pelo vértice em vez de dois `BTreeMap` | a mesma resposta (fica a de menor índice — gate novo); `3,0 → 0,12 ms` |
| `crates/ph2d-physics/src/world/player.rs` | `PhysicsWorld::body_angvel` | fn nova |
| `crates/ph2d-physics-ecs/src/bridge/nav.rs` + filhos NOVOS `nav_malha.rs`, `nav_ordens.rs`, `nav_alvo.rs` (+`nav_alvo_tests.rs`) · `tape.rs` · `rewind.rs` · `controllers.rs` · `components/nav.rs` · `lib.rs` | a malha por mosaicos e o obstáculo cinemático parado; as ordens; os alvos; `NavRoute`; os campos do anel | `nav.rs` **688 → 549** LOC (a malha e a forma saíram para o filho); ⚠️ o tique da navegação passou a duas passagens — ver §3 |
| `crates/ph2d-ecs/src/signal_actions.rs` (+ testes) | os 2 verbos, `ArgKind::ObjectName` | apendados |
| `crates/ph2d-ecs/src/rewind_runtime.rs` (+ `tests/it/rewind_runtime.rs`) | ⚠️ **muda o comportamento de TODO cérebro no rebobinar**: quem já está no estado inicial e já o anunciou NÃO renasce (a porta corre a cada quadro parado, e cada quadro anunciava a entrada outra vez — o report do dono de 02/10, «milhões de mensagens»); o recomeço não muda | ⚠️ uma linha que dependa de um cérebro re-anunciar o inicial a cada quadro parado deixa de o ter (nenhum gate do repo dependia: `ph2d-ecs` 539/539, `ph2d-app-components` 820/820) |
| `crates/ph2d-editor-core/src/{nav_edits.rs, screens/hero/inspector_model_action.rs}` | `NavAlvoModo` +2, `alvo_tag`, `SemForma`, `AlvoTag`; `ActionArgHint::ObjectName` | apendados; ⚠️ `InspectorNavAgent` ganhou o campo `alvo_tag` (todo literal do struct precisa dele — 3 fixturas no repo, conferidas por `grep -rln 'InspectorNavAgent {'`) |
| `crates/ph2d-panel-inspector/src/{ids/inspector_nav.rs, ids/inspector_action.rs, populate_nav.rs, event_nav.rs, sections/nav.rs, sections/nav_tag_row.rs (NOVO), sections/mod.rs, state_popovers.rs, popovers_tags.rs, sections/actions_editor.rs}` | os modos, o chip da tag, o 14.º verbo | linhas a mais |
| `crates/ph2d-i18n/src/{ecs_scene,inspector,inspector_nav}.rs` | as 10 chaves | sim |
| `crates/ph2d-app-components/src/{nav_rota.rs (NOVO), nav_smoke_guarda.rs (NOVO), nav_inspector.rs, nav_smoke.rs, path_follow_bridge.rs, signal_actions_bridge.rs, lib.rs}` | a rota, a cena `=3`, os modos no Inspector, a entrega à ponte | sim |
| `shells/desktop/src/{render_loop/fase_tabela_de_accoes.rs, render_loop/fase_physics_step.rs, components_scenes_suplentes.rs}` · `shells/desktop/tests/it/os_pedidos_de_vida_chegam_a_ponte.rs` | a entrega por `entrega_a_fisica`; a árvore das tags à ponte; `montar_cena` | o gate de texto segue a porta nova |

### §1.2 — Contratos congelados

**Nenhum** — `node.rs` e `tool.rs` intocados (a superfície acima o diz); nenhum ADR.

---

## §2 — O que a wave traz

O plano §14 tem tudo (o que se consegue fazer, as medições, as decisões). Em uma linha: mosaicos de `15 m`
(uma porta a 1 000 obstáculos `44,1 → 7,3 ms` na ponte), o cinemático PARADO é parede, os verbos
`Start/Stop Navigation` (gravados por tique, no anel), os alvos `Tag` e `Patrol` com o Inspector, e a cena `=3`.

---

## §3 — ⚠️ O que um merge textual pode partir

1. **`SignalVerb::ALL` é POSICIONAL no ficheiro**: outra linha que tenha APENDADO um verbo escreveu as
   MESMAS tags `12`/`13`. ⇒ a ordem das duas linhas no `main` decide as tags; a segunda tem de mudar as
   tags dela (o gate `signal_actions_cerca_tests::GRAVADAS` reprova se não) e o `INSP_ACTION_VERB` tem de
   seguir o `ALL` posição a posição.
2. **O tique da navegação é de DUAS passagens** (`bridge/nav.rs::drive_nav_agents`: quem conduz e que malha
   pede · `malhas_em_dia` · a condução). Uma linha que tenha editado o laço antigo (W3–W5) conflitua aqui.
3. **`ph2d_nav::NavMesh::from_polygons`** foi reescrito por dentro (a mesma saída). Uma linha que lhe tenha
   mexido conflitua no ficheiro inteiro — conferir pelos gates da `ph2d-nav`, `ph2d-navmesh` (o exacto) e
   da navegação na ponte.
4. **`InspectorNavAgent { … }`** ganhou `alvo_tag`: um literal novo noutra linha não compila (o compilador
   diz onde).
5. **A entrega dos anúncios da tabela** passou a `signal_actions::entrega_a_fisica(vida, navegacao,
   physics)`: uma linha que tenha acrescentado um terceiro anúncio ao laço antigo da shell tem de o pôr
   nesta porta.
6. `PROJECT_SCHEMA` — o `+1` é da W5 (contar com `python3 scripts/schema-recount.py`).
7. **`rewind_runtime_state`** (o bloco dos cérebros) ganhou a idempotência no rebobinar: uma linha que
   tenha acrescentado outro estado vivo a esse bloco conflitua ali.

---

## §4 — ⏳ O que fica ABERTO

- A montagem da malha é O(malha) (`~5 ms` a 1 000 obstáculos; `~2,6` são a `NavMesh`): uma `NavMesh` por
  mosaicos é a próxima alavanca.
- Todos os agentes da malha que mudou recalculam no mesmo tique (`~2 ms` cada a 1 000 obstáculos).
- Enquanto anda, uma porta é para o desvio um círculo (o raio que a envolve).
- O Inspector diz *«Switched off»* de um agente autorado desligado que um `Start` pôs a andar.
- Os abertos da W5 e de 01/10 continuam (a leitura viva não diz «a dar passagem»; a cena `=1` apertada; o
  defeito G; o flake `text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout`).
- As waves seguintes: **W7** (custo por área e atalhos — **decisão do dono 02/10: o inimigo evita sozinho
  as zonas que ferem, com caixa para desligar**) · **W8** (a arena, o tutorial).
- **Decisões do dono tomadas nesta linha (02/10):** o desvio nasce LIGADO (já estava) · o inimigo evita a
  lava na W7.

---

## §5 — A prova de fecho

Tudo corrido **1× sobre o diff acumulado**, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **`19 443 / 19 443`** verdes (`11 503` saltados) — a 3.ª corrida, depois das curas do Rewind (report do dono); a 2.ª deu `19 442 / 19 442`; a 1.ª deu `19 440 / 19 441` (o `architecture_panel_loc_cap`: `paint_nav_agent_section` `221 > 200` ⇒ a linha do alvo saiu para `linha_do_alvo`) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `cargo clippy --all-targets --all-features -D warnings` nas 12 crates tocadas (`ph2d-navmesh` · `ph2d-nav` · `ph2d-orca` · `ph2d-physics` · `ph2d-physics-ecs` · `ph2d-ecs` · `ph2d-i18n` · `ph2d-editor-core` · `ph2d-panel-inspector` · `ph2d-panel-registry-init` · `ph2d-app-components` · `ph2d-host-desktop`) | **zero** — depois de 5 curas (`is_multiple_of`/`contains` nos mosaicos, a `entry` do `por_tag`, um `drop` de um `Mut`) |
| `file_loc_caps` (shell, 4/4) · `arch_safe_clamp_only` · `the_shell_only_shrinks` · `architecture_workspace_file_loc_cap` | verdes |
| `cargo fmt --all --check` | verde (depois de 2 ficheiros) |
| `typos` project-wide · `cargo machete` | zero · zero |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `censos-da-arvore-combinada.sh` (a árvore combinada É esta: o `main` é ancestral) | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | `20` índices em dia |
| `#[cfg(target_os` escrito/movido | **nenhum** |
| a máquina no fim | nada desta linha a correr; `/dev/dri` só Xwayland/plasmashell/code |

**Mutação 45 / 45** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w6_2026-10-02.py`](../ferramentas/mutacao_navegacao_w6_2026-10-02.py), os quatro
controlos, 8 grupos). A 1.ª corrida deu 37/45 e achou sete leis sem régua (uma equivalente, que saiu do
código) — o plano §14.5 tem a lista; cada régua nova foi medida com a mutação ao lado.

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **determinismo / replay** | um scrub devolve a mesma corrida com portas, ordens, rondas e a tag mais perto | a velocidade do solver (o «parado») vai no checkpoint · `ControllerMemory::{nav_ordens, nav_rondas}` (`bridge/tape.rs`) · a fita das ordens (`bridge/nav_ordens.rs::aplica_ordens_de_navegacao`, gravada no tique vivo, lida no replay) · a árvore das tags entregue ANTES do tique (`fase_physics_step.rs`) · tudo em `BTreeMap`/`BTreeSet` e a ordem das entidades | `nav_ordens::{um_scrub_refaz_as_ordens_ao_bit, a_fita_refaz_a_ordem_entre_o_checkpoint_e_o_alvo, o_recomeco_do_zero_esquece_as_ordens}` · `nav_alvos::um_scrub_devolve_a_ronda_ao_bit` · `mosaicos::incremental_e_a_frio_dao_o_mesmo` · `a_arvore_das_tags_vai_a_ponte_antes_do_tique` |
| **achado da lente** | ⛔ a árvore das tags ia à ponte na fase dos SINAIS (depois do tique): o 1.º tique depois de editar as tags não achava ninguém e o replay dele achava | `fase_signal_outbox.rs` → movida para `fase_physics_step.rs` antes do `dispatch` | o gate de texto acima (novo) |
| **costura do painel** | escolher uma tag / um modo / escrever o nome da forma chega ao componente | `ids/inspector_nav.rs` → `populate_nav.rs` (o chip `Dropdown` + 64 botões) → `sections/nav_tag_row.rs` (hit-index + slot do popover) → `popovers_tags.rs` → `event_nav.rs` (`AlvoTag`) → `EditorAction::InspectorComponentEdit` → `nav_inspector::apply_nav_edit` (o nome escreve no alvo que o agente JÁ tem) | `a_seccao_nav_esta_viva::{escolher_uma_tag_com_o_ponteiro_pede_essa_tag, os_modos_da_w6_pintam_a_sua_linha}` (ponteiro REAL) · `nav_inspector_tests::os_modos_da_w6_vao_e_voltam` · mutações M41–M43, M34–M36 |
| **não-checado pela compilação** | a cena `=3` ensina o que acontece | a cena jogada inteira sem ecrã, pela ordem do quadro da shell (e o cérebro ouve o outro no quadro SEGUINTE) + as duas fotos | `nav_smoke_guarda_tests::o_guarda_patrulha_ve_persegue_e_a_porta_fecha_se_lhe_na_cara` · M37–M38 |

LOC lidas na auditoria: `~1 900` (a ponte da navegação e os filhos, o anel, o recomeço, as duas fases da
shell, as pontas do painel, a ponte do Inspector).

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote externo).

---

## §6 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
```

O guarda VERMELHO faz a ronda pelo rectângulo desenhado de cima, o CINZENTO (o controlo, sem cérebro) pelo de
baixo. Setas: entrar pela porta e pisar a zona AMARELA — o vermelho persegue; voltar à zona VERDE — nasce a
porta atrás do herói (uma fábrica no vão), o vermelho desiste e volta à ronda; o Rewind abre-a (o que nasceu
na corrida é varrido) sem enxurrada de mensagens. ✅ **Smoke do dono APROVADO (02/10)**, depois das curas do
Rewind. Tecla `B`: o contorno claro da área andável fecha o
vão quando a porta pára. O guarda vem escolhido (Nav Agent: *Patrol* · *Patrol Route*). Fotografada aos `6 s`
e `9 s` (ecrã virtual). As cenas `=1` e `=2` continuam iguais.

**O binário do smoke está COMPILADO na worktree** — a 2.ª corrida, colada no fim deste documento.

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.06/passo   alvo: >= 1,5  (5% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                342   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check              6502 : 2183   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  21%   alvo: >= 80%  (7471 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         707 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.
  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.
```

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental`, a 2.ª corrida de `bash scripts/ph2d-run.sh cargo build -p
ph2d-host-desktop --profile smoke` (a árvore das curas do Rewind) — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```
