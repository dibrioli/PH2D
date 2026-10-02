# HANDOFF DE INTEGRAÇÃO — `line/components`: NAVEGAÇÃO W0–W4 (+ a VIDA W6/W7 ainda por fundir) — 2026-10-01

> ⛔ **Este documento SUPERSEDE o [`…_VIDA_W6_W7_2026-09-30.md`](HANDOFF_INTEGRACAO_line_components_VIDA_W6_W7_2026-09-30.md)
> como documento de integração da linha.** Aquele nunca chegou a ser integrado: os **10** commits dele
> continuam por fundir e vão **neste** bloco. O que ele diz sobre a VIDA (o §2 *o que os commits
> trazem*, o §3 *o que um merge textual pode partir*, o §4 *o que fica aberto*) **continua válido e
> não se repete aqui** — este documento acrescenta os **4** commits da navegação (mais este fecho)
> e refaz a prova sobre o diff acumulado INTEIRO.

## §0 — O `--ff-only` deve passar limpo (medido)

- O `main` (`912a9652e`) **é ancestral** do HEAD da linha (`git merge-base --is-ancestor main HEAD`
  ⇒ verdade; `git rebase main` ⇒ *«atualizado»*): **zero** commits do `main` por trazer.
- A árvore do **primário** estava **limpa** no fecho (`git status --short` vazio).
- A linha traz **quatro** ficheiros de `project-memory/` (todos da VIDA, §1 do handoff de 30/09). Se
  o primário entretanto gravar memória, a receita é a do §0 de 25/09: comitar primeiro a memória do
  primário, rebasear a linha, unir (memória é append; o `MEMORY.md` sob o tecto do gate
  `o_indice_cabe_no_orcamento_do_carregador`).

---

## §1 — Identidade e superfície de colisão, MEDIDA

- ramo `line/components` · **15 commits** sobre `912a9652e` (os 14 colados abaixo + este fecho) ·
  `178` ficheiros · `+16 600` / `−197` (medido antes do commit do fecho)

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, colado:

```
  merge-base 912a9652e   ·   14 commit(s)   ·   178 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        178   (base: 176)
  ⚠   └ tripla do gate               (178, 13, 22)   (base: (176, 13, 22))
    VEC_SCENE_SCHEMA 22 · FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23   (intactos)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109   (intactos)
▸ CONTRATO CONGELADO (§6)       node.rs intocado · tool.rs intocado
▸ ADR                           esta linha não cria ADR
▸ Cargo.lock                    2 '+name' novos: "ph2d-nav", "ph2d-navmesh"  (crates NOSSAS)
▸ MARCADORES DE CONFLITO        nenhum
▸ TETOS DE LOC                  nenhum arquivo da linha passa do teto
```

| grandeza | aqui | `main` | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `178` | `176` | **+2** | `+1` VIDA W6 (campos do `Damage`/`Health`) · `+1` NAV W4 (`NavRegion`+`NavAgent` passam a registados) |
| registos `ph2d-ecs` / `-render` / `-script` | `108`/`109`/`109` | igual | **0** | |
| ⚠️ registo da FÍSICA (`register_physics_components`) | `42` | `40` | **+2** | NAV W4: `NavRegion` · `NavAgent`. ⛔ o `collision-surface.sh` **não** o mostra — conferido no gate `registers_every_physics_component` |
| `LIVE_SECTIONS` | `45` | `43` | **+2** | NAV REGION · NAV AGENT |
| `ComponentEdit` | `+1` | | **+1** | `Nav(NavFieldEdit)`, **apendado** |
| `ProbeKind` | `+2` | | **+2** | `Path` (W3) · `NavEdge` (W4), **apendados** |
| catracas do Inspector (comandos · cortes · letras) | `85`/`89`/`85` | `81`/`87`/`83` | `+4`/`+2`/`+2` | **só a VIDA W6** — a navegação não mexe nelas |
| pacotes **externos** novos | `0` | | **0** | o `clipper2-rust` e o `spade` já estavam no `Cargo.lock` |
| crates novas | `ph2d-nav` · `ph2d-navmesh` | | **+2** | folhas: `ph2d-nav` com zero dependências; `ph2d-navmesh` sobre `clipper2-rust` + `spade` |

⚠️⚠️ **CONTE O DELTA, nunca o literal.** Os degraus `176 → 177 → 178` moram na escada do
[`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) e a tripla no
[`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs) — **três** sítios,
nunca um. Outra linha que registe um componente de física ou acrescente uma secção viva escreve os
mesmos literais (`42`, `45`): recontar é somar os dois deltas ao valor do `main` do dia.

### §1.1 — Os ficheiros da navegação FORA da família

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-physics-ecs/src/bridge/{controllers,tape,rewind,signals,dispatch,birth,player_channel,player_view}.rs` + `bridge.rs` | a ponte da navegação entra na **porta única** dos controladores (os dois laços — frente e replay), a memória dos agentes no anel (`ControllerMemory`), a limpeza no `rebuild_from_rest`, os factos pela porta `signal_events` (⛔ `SignalOrigin` **+0**), `ProbeKind::Path`/`NavEdge` apendados | sim — campos novos em structs da ponte; um `match` exaustivo sobre `ProbeKind` noutra linha deixa de compilar e o compilador diz onde (o único hoje é `measure_player_probes.rs`) |
| `crates/ph2d-physics-ecs/src/components/{nav,topdown}.rs` + `components.rs` + `lib.rs` | `NavRegion`/`NavAgent`/`NavTarget`/`NavNow`; `TopDownPlayer::conduzido_pela_navegacao`; o registo `40 → 42` | sim — `NavNow` é **derivado e NÃO registado** (o molde do `HealthNow`) |
| `crates/ph2d-physics-ecs/tests/it/no_std_transcendental_on_the_hash_path.rs` | o gate passa a cobrir `ph2d-nav` e `ph2d-navmesh` (apanhou três `.hypot(` na ponte da W3) | sim — população maior |
| `crates/ph2d-editor-core/src/{nav_edits,action_bus_component,lib}.rs` + `ids/{inspector_camera,live_sections}.rs` | o vocabulário das duas secções; `ComponentEdit::Nav`; quatro ids | sim — variante e ids **apendados** |
| `crates/ph2d-component-desc/src/catalog/physics.rs` | duas portas do catálogo; o agente **requer** o `TopDownPlayer` | sim |
| `crates/ph2d-app-physics/src/{physics_seed,overlay/probes}.rs` | a semente `seed_nav_agent`; o pintor não põe tiques num `NavEdge` | sim |
| `crates/ph2d-i18n/src/{component_catalog,inspector_nav,lib}.rs` | os nomes no catálogo e a tabela `panel.inspector.nav.*` (ficheiro próprio, como os irmãos `inspector_vida`/`inspector_game`) | sim |
| `shells/desktop/src/render_loop/{fase_bus_drain_out,fase_bus_inspector,fase_hero_commits,fase_inspector_commits,fase_inspector_commits_top20,snapshots_inspector,snapshots_inspector_physics,fase_physics_overlay,fase_app_scene_smokes}.rs` | a cadeia barramento → fila → aplicar da secção nova; o retrato do Inspector; o contorno no overlay | ⚠️ **ver §3** — campos apendados em structs partilhadas por todas as secções |
| `shells/desktop/src/{components_scenes_suplentes,app_state_components_smokes,component_seed_seam_tests,project_schema*}.rs` | o prólogo da cena `PH2D_NAV_SMOKE=1`; o gate da semente; o degrau `178` | sim |
| ⭐ `shells/desktop/src/render_loop/{state_machine_tick,inspector_statemachine}{,_tests}.rs` → `crates/ph2d-app-components/src/{state_machine_tick,statemachine_inspector}{,_tests}.rs` | **mudança de casa** (catraca `the_shell_only_shrinks`, §2) | ⚠️ **ver §3** |
| `.typos.toml` | o dossiê da pesquisa (verbatim, em inglês) isento por **ficheiro** | sim — entrada apendada |
| `Cargo.lock` | as duas crates nossas; zero pacote externo | sim |

---

## §2 — O que os commits da navegação trazem

Pesquisa: [`29_pesquisa_navegacao.md`](../29_pesquisa_navegacao.md) · plano e resultados:
[`30_plano_navegacao.md`](../30_plano_navegacao.md) (§12 é a W4).

| commit | o que traz |
|---|---|
| `7b33f7c3e` | **pesquisa + plano**: as três camadas da indústria (malha → caminho → desvio), 13 queixas medidas como gates, a triagem (a geometria custa **zero** pacotes novos), a sonda do Godot 4.7.2 MIT corrida sem interface |
| `0601e49f9` | **W0–W2:** `ph2d-nav` (malha, Polyanya — ótimo em qualquer ângulo, IJCAI 2017, lei do artigo — e o oráculo EXACTO do grafo de visibilidade) e `ph2d-navmesh` (recuo REDONDO só com `sqrt`, a esquadria do Godot para paridade, CDT, fusão Hertel-Mehlhorn em `i128`). Polyanya exacto a `1e-9` em 24 cenas; a mesma malha ao bit noutro PROCESSO; paridade de área com o Godot desvio `0`. Medido: construção `0,4` / `4,6` / `64 ms` a 10/100/1000 obstáculos, procura `1,8` / `10` / `658 µs`. ⛔ Recusa medida: virar num canto gerando tudo o que se vê dele custava `19 ms` a 1000 obstáculos (`29×` a sombra do artigo) |
| `6d94dfcc7` | **W3:** a condução (`ph2d_nav::agent`, lei pura), a ponte dentro da porta dos controladores (o agente PEDE, o `TopDownPlayer` ANDA — a lei de deslize do #13 não é duplicada), a malha DERIVADA dos corpos estáticos por `(região, raio)`, a memória no anel, o caminho desenhado, e a cena `=1` (o labirinto com o CONTROLO de perseguição em linha recta). Mutação **12 / 12** |
| `da7a09c57` | **W4:** as secções NAV REGION / NAV AGENT com a leitura viva e as queixas; o registo; a semente (`EightWay` de fábrica → `Free`, com o preço **medido**: `+3,3..5,6 %` de caminho); a ÁREA ANDÁVEL no canvas (um contorno por raio); o censo novo da cadeia da shell; a máquina de estados mudada para a família. Mutação **27 / 27** |
| (este) | o fecho: este handoff e o rename `nin → normal_dentro` que o `typos` project-wide acusou na `ph2d-navmesh` |

---

## §3 — ⚠️ O que um merge textual pode partir

1. **A cadeia da shell das edições do Inspector** — `DrainOut` (`nav_edits`), o `match` do
   `fase_bus_inspector` (`ComponentEdit::Nav`), o `take` do `fase_hero_commits`, as intenções do
   `fase_inspector_commits` e o parâmetro `nav` da `aplicar` do TOP-20. São listas que **toda** secção
   nova estende no fim: outra linha com uma secção nova conflitua **trivialmente** (cura: as duas
   linhas). ⭐ E o censo novo `toda_fila_do_inspector_chega_ao_seu_apply` (shell, `tests/it/`) reprova
   se a fusão perder um dos elos — ele é **derivado** do fonte, com piso de população (40 filas · 19
   parâmetros).
2. **A máquina de estados MUDOU DE CASA.** Uma linha que tenha editado
   `shells/desktop/src/render_loop/state_machine_tick.rs` ou `inspector_statemachine.rs` vê um
   conflito *modify/delete*: a edição tem de ir para `crates/ph2d-app-components/src/state_machine_tick.rs`
   / `statemachine_inspector.rs` (os `pub(crate)` viraram `pub`; o `fase_signal_outbox` e o
   `fase_statemachine_commits` chamam pela crate). ⚠️ A isenção `State {n}` do HR-15 **viajou** da
   lista da shell para a da família — as duas metades a acusar na mesma corrida são a assinatura da
   mudança de endereço, nunca texto novo.
3. **`snapshots_inspector.rs` está em `586` de `600`** e a `late()` em `<200` depois de a VIDA e a NAV
   irem para o irmão `snapshots_inspector_physics.rs::vida_e_nav` — outra secção que a outra linha
   escreva ali pode passar o tecto da função **por acumulação** (cura: corte por responsabilidade).
4. **A fixtura `o_inspector_armado.rs`** arma a navegação (região + agente no estado que pinta mais
   linhas) — uma linha que acrescente uma porta ao Inspector acrescenta outro bloco no mesmo sítio.
5. **`PROJECT_SCHEMA`, o registo da física e `LIVE_SECTIONS`** — SOMAM (§1).

---

## §4 — ⏳ O que fica ABERTO

- **A cena `=1` está APERTADA** (smoke do dono, 01/10: *«não sei se intencionalmente»*). É metade
  desenho e metade restrição: a porta de baixo tem de deixar passar o pequeno (`r = 0,3`) e barrar o
  grande (`r = 0,65`), e o recinto tem de caber na faixa que sobra por cima da timeline (medida na
  W3). Alargá-la é trabalho de cena, sem mexer em lei.
- **A divergência do plano**: sem interruptor próprio para a área andável (*Show Navigation*) — o
  contorno vive no overlay de física (tecla **B**), ao lado dos caminhos da W3. Se o dono o quiser à
  parte, é um interruptor de VISTA (no `WidgetStore`, nunca no componente).
- **O defeito G** (um `ProjectileMotion` com colisor SENSOR não se move de todo; o mesmo sem sensor
  voa) — medido numa sonda na W3; é do #14 e continua aberto (também no §4 de 30/09).
- **Flake de carga a promover:** `text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout`
  (shell) — gate de RAZÃO de relógio, reprovou **uma** vez numa varredura de `18 735` com outra linha
  na placa, **3 / 3 verde sozinho a `load 7,3`**, zero linhas desta linha naquele módulo; passou na
  varredura do fecho. Para a lista do §5.0 do `CLAUDE.md` (o integrador escreve).
- As waves seguintes do plano 30: **W5** o desvio entre agentes (`ph2d-orca`) · **W6** o mundo que
  muda, as portas, `Start/Stop Navigation`, a patrulha · **W7** custo por área e atalhos · **W8** a
  arena e o tutorial `03_navegacao.pdf`.
- Os abertos da VIDA: §4 do handoff de 30/09 (a lava renovar a queimadura é decisão do dono).

---

## §5 — A prova de fecho

Tudo corrido **1× sobre o diff acumulado** (VIDA + navegação), régua no merge-base
(`BASE=912a9652e`), dentro da fatia da linha (`ph2d-run.sh`):

| portão | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **`18 756 / 18 756`** verdes (`11 655` saltados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde, **zero** avisos |
| `bash scripts/check-standalone-optional.sh` · `bash scripts/check-workflow-packages.sh` | verdes (10 crates com dependência opcional · 32 nomes de workflow contra **396** membros — eram 394: as duas crates novas) |
| `bash scripts/censos-da-arvore-combinada.sh` (§1.5.9 5-bis; a árvore combinada **é** esta, o `main` é ancestral) | **`127 / 127`**, controlo do filtro `12 de 12` |
| `cargo clippy --all-targets --all-features -D warnings` nas 12 crates que a linha toca (`ph2d-nav` · `ph2d-navmesh` · `ph2d-health` · `ph2d-physics-ecs` · `ph2d-editor-core` · `ph2d-app-components` · `ph2d-app-physics` · `ph2d-component-desc` · `ph2d-i18n` · `ph2d-panel-inspector` · `ph2d-panel-registry-init` · `ph2d-host-desktop`) | **zero** |
| `cargo fmt --all --check` | limpo |
| `cargo machete` (⚠️ obrigatório: a linha MOVEU código) | **zero** dependências por usar |
| leitores por caminho dos ficheiros movidos (`git grep` em `crates tools scripts docs/Components/ferramentas`) | **nenhum** — o único (o arnês de mutação de 15/09) foi reapontado e as cinco âncoras dele casam `1×` nos caminhos novos |
| prova de que nada evaporou na mudança | `11` testes nos ficheiros movidos antes · `11` depois |
| `bash scripts/doc-index.sh --check` | `20` índices em dia |
| `typos` **project-wide** (como o `ship.sh`) | limpo — ⚠️ depois do rename `nin → normal_dentro` na `ph2d-navmesh` (uma variável da W1 que só a varredura inteira apanhou) |
| `#[cfg(target_os` escrito/movido | **nenhum** em código (as duas ocorrências do diff são prosa nos docs) ⇒ o cruzamento para macOS não se aplica |

**Mutação** (navegação): W0–W2 **4 / 4** no Polyanya · W3 **12 / 12**
([`mutacao_navegacao_w3_2026-10-01.py`](../ferramentas/mutacao_navegacao_w3_2026-10-01.py)) · W4 **27 / 27**
([`mutacao_navegacao_w4_2026-10-01.py`](../ferramentas/mutacao_navegacao_w4_2026-10-01.py), com os quatro
controlos — grupos limpos verdes e com população, pré-voo das âncoras, «não compila» acusado como defeito
do arnês, zero testes aborta — e por grupos `MUTA_G`, porque a fatia tem prazo e a shell recompila). A VIDA:
§5 do handoff de 30/09.

**Fotos** pelo [`fotografa_cena.sh`](../ferramentas/fotografa_cena.sh) (ecrã virtual): a `=1` com os dois
contornos (o do grande fecha a porta de baixo) e a secção NAV AGENT **aberta** a dizer *«Can't reach it ·
going to the nearest point»* e *«Radius 0.65 m, from the collider»* — a 1.ª foto mostrou-a dobrada e a cena
passou a abri-la.

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote externo
novo).

---

## §6 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

O labirinto: setas para andar com o **amarelo**; o **vermelho** dá a volta às paredes para o apanhar; o
**roxo** (grande, já escolhido) não cabe na porta de baixo e espera; o **cinzento** (o CONTROLO) persegue em
linha recta e bate nas paredes. A linha azul é o caminho planeado e o contorno claro é a área andável de cada
tamanho (tecla **B**); a secção **Nav Agent** do Inspector diz o que o roxo faz e porquê. ✅ **Smoke do dono
aprovado (01/10)**, com a nota da cena apertada (§4). As cenas da VIDA (`PH2D_VIDA_SMOKE=3|4`): §6 de 30/09.

**O binário do smoke está COMPILADO na worktree** (DIRETRIZ §1.5.9 item 9) — a 2.ª corrida, colada no fim
deste documento.

**Linhas propostas para o `CLAUDE.md §5`** (Componentes — quem as aplica é o integrador; a primeira é a de
30/09, que nunca entrou):

> ⭐⭐⭐ **E A VIDA E DANO FECHOU (30/09, plano 28 W6–W7 — [handoff](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_VIDA_W6_W7_2026-09-30.md)):** TIPOS de dano com resistências (imune · fraco · absorve, contra o addon MIT do Godot) e o dano que DURA; a arena `=4` junta tudo num jogo pequeno e o tutorial [`02_vida_e_dano.pdf`](docs/Components/tutoriais/02_vida_e_dano.pdf) ensina-o; ⚠️ **conte o DELTA:** `PROJECT_SCHEMA` **+1**, registos **0**, catracas do Inspector `+4/+2/+2`; ⏳ a lava renovar a queimadura a cada tique é decisão do dono. **Smokes:** `PH2D_VIDA_SMOKE=3|4`.

> ⭐⭐⭐ **E A NAVEGAÇÃO EXISTE (01/10, plano 30 W0–W4 — [handoff](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_NAVEGACAO_2026-10-01.md)):** um inimigo acha o caminho sozinho — malha andável derivada dos colisores com o raio do CORPO (`ph2d-navmesh`), caminho mais curto exacto (Polyanya, `ph2d-nav`), o agente PEDE e o `TopDownPlayer` ANDA; *Add Component → Nav Agent* basta, e o Inspector diz o que ele faz e porquê; ⚠️ **conte o DELTA:** `PROJECT_SCHEMA` **+1**, registo da FÍSICA **+2** (o `collision-surface.sh` não o mostra), `LIVE_SECTIONS` **+2**; ⛔ a máquina de estados **mudou-se** da shell para a `ph2d-app-components`. **Smoke:** `PH2D_NAV_SMOKE=1`.
