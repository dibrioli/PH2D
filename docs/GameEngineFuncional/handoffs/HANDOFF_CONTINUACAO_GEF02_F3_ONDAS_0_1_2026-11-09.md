# HANDOFF DE CONTINUAÇÃO — GEF02 · F3 (corpo físico e gatilho), ondas 0–1 — 09/11/2026

Linha `line/GameEngineFuncional02` (worktree `Worktrees/line-GameEngineFuncional02`). Não é handoff de
integração: a linha NÃO fechou (faltam as ondas 2–4). Plano: [f3_corpo_e_gatilho_plano.md](../componentes/f3_corpo_e_gatilho_plano.md).
Origem: crítica externa (`GameEngineSamples/critica_ph2d.html`) + estudo; respostas do dono 09/11:
[memória](../../../project-memory/project_physics_critique_answers_2026_11_09.md) (Sensor fica · etiqueta de sinal ·
Width/Height · «More» fechado · tecla no cartão do Trigger).

## 1. Estado (commits desde `c1cb56940a`)

| Onda | Commits | Resumo |
|---|---|---|
| 0 | `9029969369`, `6cfd80dddf` | 53 cenas de smoke + `SceneCtx`/`SceneSetup` saem para a folha `ph2d-physics-smoke-scenes` |
| 1 | `007650926b`, `d00414503a`, `0f66f436f8`, `6d37f1b32a`, `a6afdcd621` | a caixa de nome de sinal diz quem está na outra ponta |
| docs | `d185772e7d`, `108ade3247` | plano F3 e ponteiro da memória |

Smoke do dono OK em 09/11 (ver §5). Sem suíte completa (é do integrador).

## 2. Já existia — NÃO se reconstrói

Tipo de corpo na 1.ª linha e campos de Dynamic escondidos por tipo (`physics_cartao.rs`); colisor + gatilho no
mesmo cartão (`Touch: Solid | Sensor`); aviso de corpo sem forma. O campo **Layer já mostra o NOME da camada**
(foto `fotos_fisica/sinais/02_coin_b.png`: «Layer: Default ▾»; teste `a_camada_pelo_nome_fora_do_cartao.rs`).
O item «nome da camada» da onda 2 deve ser CONFERIDO no código antes de construir — pode restar só o Ray Sensor
(a crítica diz que ali a layer é número de propósito; ver comentário em `a_seccao_ray_sensor_esta_viva.rs:58`).

## 3. Superfície de colisão e contadores (DELTA)

- **Onda 0.** `ph2d-app-physics` 55 515 → 38 085 linhas de tecto (medido 37 339 + 746). `ph2d-physics-smoke-scenes`
  nova, 18 056 linhas movidas por `git mv`, tecto 18 572 em `TETOS_DAS_CRATES_DA_FAMILIA`
  (`ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`). Re-export em `ph2d-app-physics`: nenhum
  chamador muda. `scene_sources()` dos 2 gates da shell (`handle_scenes_start_paused`,
  `scenes_that_ask_for_the_ruler_open_the_timeline`) varre a pasta nova: 82 ficheiros antes e depois, mesmo sha.
  `6cfd80dddf`: o teste da forma neta (`ph2d-panel-hierarchy`) pára no pedido Reparent — painel não depende de
  família (`architecture_no_dependency_climbs_a_layer` estava vermelho desde `de8e2cec50`, desta linha).
- **Onda 1.** Crates novas: `ph2d-signal-ends` (Role/Ends/Hint/SignalCensus) e `ph2d-signal-census`
  (`lib.rs`, `fora.rs`, `luau.rs`). Tabela `CAMPOS` = 37 linhas = 31 caixas do Inspector + HUD, partículas,
  `AnimWhen::Signal`. Luau `ph2d.emit`/`ph2d.connect` literais contam; não-literal/`on_signal` ⇒ «a script may …».
  Tabela FORA (documentos fora do World): `pulse.signal` do Motion (E), `value.input` do Motion (L),
  `Marker.signal` da timeline (E), StateSets «On signal» da UI (L).
- **Sufixo pintado** por fornecedor por fio: `ph2d-editor-core/src/widget/field_suffix.rs`, perguntado pelo pintor
  da caixa (`cards_linhas::text_reservando`, `property_row::paint_text_row`), instalado por
  `ph2d-panel-inspector/src/signal_hint.rs` (CAIXAS: 31 ids → papel). Nenhuma secção tocada.
  Textos: `1 listens` / `{n} listen` / `nobody listens` (Warn) / `1 sends` / `{n} send` / `nobody sends` /
  `a script may …`.
- **Custo.** Censo 60 µs/200 objectos ⇒ cache por impressão digital FNV (23,5 µs/quadro), publicada em
  `fase_snapshots_publish.rs` (`Fora::new(...).when_changed`).
- ⛔ **Shell EXACTAMENTE no tecto 189 041/189 041**: a próxima linha de shell exige mover código antes.
  `populate_physics` partida em botões e números (204 > 200 em `architecture_panel_loc_cap`).
- **Defeito antigo achado no smoke** (passo 5): a secção Signal Actions não semeava On/Target/Arg desde
  `9b0714dfa6` (`last_action_row` sem leitor). Cura em `sync_action.rs`, teste
  `os_campos_da_accao_aberta_mostram_os_valores_dela`.

## 4. O que um merge pode partir

- `the_shell_only_shrinks` (tecto da shell no limite; qualquer linha de shell de outra linha reprova) e a lista
  `TETOS_DAS_CRATES_DA_FAMILIA` (conflito textual provável).
- Quem acrescentar cena de smoke de física em `ph2d-app-physics` (agora lá é a folha nova) ou ler a pasta antiga.
- `project_schema.rs` NÃO foi tocado nesta linha; a onda 2 pode tocar — contar com `python3 scripts/schema-recount.py`.
- Linhas vivas a evitar: GameEngineFuncional (Animated Sprite, Align), PixelArt (`ph2d-app-components`,
  `project_schema.rs`, `cards_sprite_folhas`), PainterWatercolor (`cards_anim`, `cards_sprite_folhas`),
  TimelinePrincipal (`ph2d-panel-timeline`), ScriptEditor (Bancada).

## 5. Prova de fecho de onda

`fmt --all`; `clippy -D warnings` nas crates tocadas (smoke-scenes, app-physics, signal-census, signal-ends,
panel-inspector, editor-core, panel-hierarchy, physics-outline-vec, render, host-desktop) verde. Testes por nome
verdes: `o_sufixo_do_sinal` 2 · `os_campos_da_accao` 1 · `os_quadros_da_colisao` 11 · `a_camada_pelo_nome` 3 ·
`a_cena_ensina` 1 · `a_forma_neta` 1 · `architecture_panel_loc_cap` 4 · `no_dependency_climbs` 3 ·
`the_shell_only_shrinks` 2 · `handle_scenes_start_paused` 3 · `scenes_that_ask_for_the_ruler` 2 ·
os 9 do `ph2d-signal-census` · `a_outra_ponta` 1. Mutações: 6 na onda 1 + 6 na tabela FORA, todas sangraram.

Smoke (o dono já fez, OK):
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=121 cargo run --profile smoke -p ph2d-host-desktop`
2. Coin A (`On Hit = coin_taken`) mostra «1 listens»; Coin B (`coin_takn`) mostra «nobody listens» em aviso;
   Score, Signal Actions, On `coin_taken` mostra «1 sends»; corrigir Coin B ⇒ «2 send».
3. Errado = sufixo ausente, ou «nobody» com ouvinte existente. Fotos: `fotos_fisica/sinais/01_coin_a.png`,
   `02_coin_b.png`, `03_score.png` (cartão da Signal Action fechado na foto).

## 6. ABERTO

- **Onda 1:** `from_control` (clique em forma interativa na preview; nome = Name da entidade via `VecEntityMap`,
  só na shell) e botões/escolhas do painel autorado (`authored_intents.rs:45-51`) não entram no censo ⇒ uma caixa
  que OUVE pode dizer «nobody sends» quando um botão manda; a que EMITE nunca mente. Cura exige linhas de shell
  (tecto). Sinais fixos de sistema do `ph2d-runtime` estão cobertos pelos componentes da tabela.
  `authored_signals` (`ph2d-anim-signals`) não deriva do censo (`ph2d-app-sprite-anim` é de outra linha viva).
- **Herdadas da onda do Outline (não desta missão):** macro que muda o nº de pontos sem modal; guarda de clip
  aninhado com offset; painel antigo `PH2D_CARDS=0` sem esmaecer campos travados.
- **Onda 2:** Half Size → Width e Height na forma. Decidir no padrão-ouro: guardar a medida inteira e o motor
  recebe metade; se mudar o tipo, subir `PROJECT_SCHEMA` (sem migração, CLAUDE.md §0.11). Conferir «nome da camada»
  (§2). **Onda 3:** «More» fechado, recibo «Removed — Undo», aviso de corpo escalado. **Onda 4:** tecla no
  cartão do Trigger (Input Map continua a fonte).
- Antes de qualquer linha de shell: mover código para fora (tecto exacto).
- Integração só por ordem do Enio; nenhum push feito.
