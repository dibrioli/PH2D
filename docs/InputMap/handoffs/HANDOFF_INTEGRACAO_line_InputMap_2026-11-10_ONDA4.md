# HANDOFF de integração — `line/InputMap`, ONDA 4, 10/11 («vamos resolver todos» o aberto da onda 3)

Branch `line/InputMap`, código até `6dd3de020d`: a linha inteira, **244 commits** (com o do handoff), 470 files, `+34 221 −3 599`, sobre `bc4a6ce930` = merge-base (reconfira). A onda 4: **47 commits** (`git log --oneline 44af187eba..HEAD`), 147 files, `+4 056 −405`. Nada integrado nem pushado; **NÃO integrar até ordem do Enio**. Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram (integrador). Completa o da onda 3 (schema 232, `play_map`, F1): [onda 3](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA3.md) · [onda 2](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) · [onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano §14](../01_plano.md) · [prompt da onda 4](PROMPT_MISSAO_InputMap_ONDA4_2026-11-10.md).

## 1. Ordem do dono (10/11) — cumprida

«Vamos resolver todos» = o ABERTO do §10 da onda 3. O que sobra: §10.

## 2. O que entrou, por item

1. **Botão Controls na barra de TELA CHEIA** (`8623ef2e95`, `bfbba831ff`, `941143fd78`). `transport_bar::com_controls` é a regra única de `rects` e `rects_palco`; `paint_palco` usa a mesma `paint()` e a dica partilhada (recebe `text_system`). `caixa_do_controls` = realce do Pause (3 px). `chao_da_barra` partilhado com `menu_bar.rs`.
2. **Hover com a tela aberta** (`bfbba831ff`): `host::pointer_move_com_a_barra(s, hero, at)`; `transport_bar::sobre_o_controls`, `movimento_com_a_tela`, `solta_o_controls` (também no `CursorLeft`, `at=None`). Aceso + pairado = contorno/ícone `AccentHover`. Painel Controls do jogo: largura medida em repouso + `folga` para «Press a key» (`ph2d-player-controls/src/layout.rs`). Testes: `a_tela_cheia_tem_o_controls_com_separador_clique_e_dica_inteira`, `o_painel_nao_muda_de_largura_ao_escutar`.
3. **AltGr** (`943cdc4027`, `5b2ff74e8a`, `7dbf42b127`). Medido em winit 0.30.13: no Linux br/us(intl) o Alt direito = `ISO_Level3_Shift` em Mod5 (bandeira ALT apagada); `us` simples: `Alt_R` em Mod1. Lei nova `ph2d_input::editor_keys::chord_keys(held, alt)`: Alt direito só faz acorde com a bandeira ALT. `DocumentSurface::raw_key` ganhou `alt` (de `hero.store.alt_held()`, guardado em `InputMapDoc::alt`); `surface_raw_key` mudou para `ph2d-input-map-editor/src/surface_key.rs` (re-exportado). Keyboard e Try it usam `chord_keys`. macOS: Option direito = Alt (limite). Testes: `altgr_nao_faz_acorde_e_o_alt_esquerdo_faz`, `onda4_altgr::*`.
4. **Kanban sem swipe** (`bc639894a6`, `4f352456c5`). MEDIDO, código intacto: Calendário (Month/Week/Day/Agenda), Dashboard e gaveta da Timeline NÃO rolam na horizontal (cabem a 1600/1280/460); só a grelha da Timeline rola. Travado por `deslize::o_deslize_de_lado_so_mexe_onde_o_conteudo_rola_na_horizontal`.
5. **Planner roda ao contrário: CONFIRMADO** (−480 levou out → fev/2026). Uma conversão só: `ph2d_kanban_views::wheel_px` (app-kanban apagou a cópia e as constantes de `metrics.rs`). O Planner converte 1× à entrada (`surface.rs`): mês, Week, Agenda, «Move to». `Row/rows/first_pick` movidos para `ph2d-kanban-views/src/dest.rs`; `CalendarState::mode_rect` novo (`6d02d1db2e`). Testes: `swipe_lateral::a_roda_do_calendario_anda_para_o_mesmo_lado_que_no_kanban`, `deslize::a_roda_convertida_rola_a_agenda_para_baixo_e_a_crua_ao_contrario`.
6. **Token de cromo `key-face`** (`5bc6ea28c0`). `tokens.json` forge/sunstone/blueprint = valores de `kanban-card-raised`; `derive.rs` empresta como o card; `ColorToken::KeyFace` na lista da macro; `REST_FACE` aponta-o. Igual byte a byte nos 12 temas; píxeis fora do numérico = 0. `color.rs` 699→698.
7. **Segundas legendas do numérico** (`view_keyboard/sub_legend.rs`; `5bc6ea28c0..e0c68cb7a8`). Número 15 px sup.-esq.; legenda Regular 10 px (piso) à direita na faixa de baixo, `SUB_PAD` 2 px, fora das curvas dos cantos. Tudo-ou-nada POR CONJUNTO (palavras; setas); as barras vencem na tecla ligada; legenda ≤ 75 % da tinta do número. Contagens: 1600 = 9/10, 1280 = 3 (hoje 9, §2c), 1920 10. 
8. **Inspector na aba Input Map** (`26ea907258`, `5b2ff74e8a`, `a7e487c7a6`, `245b15f3f5`, `38397df513`, `c462b02c3a`). Tipo `InputMapPane` em `ph2d-input/src/inspected.rs` (layering); `ph2d-panel-inspector/src/cards_input_map.rs` (só com cartões); `inspect::set_snapshot` publica DEPOIS do snapshot. Cartão Action (Name, Group, Read by, Keys) + Stick (Dead zone / Press at, só leitura). Uma fonte: `view_keyboard::open(doc)` (a escolhida, senão a 1.ª) nas três vistas; «Pick an action in the list.» só com mapa vazio; `map_swapped` confere id+nome. Linha aberta da List: `row_face` (`CardFieldHover`), rótulos `Text2` ≥ 6,70:1. i18n novo `inspector_input_map.rs`. Testes: `o_inspector_da_aba_input_map` (5), `the_inspector_shows_the_new_snapshot_in_the_same_frame`, game-scenes `onda4_inspector`.
9. **Cena nova P5 = 23** (`27362e6bb3`): ponteiro no centro do Controls pela porta `pointer_moved` da shell; `janela_do_jogo.rs` líquido 0; `ph2d-host` dep de `ph2d-input-map-scene`. CENAS 22→23.
10. **Revisão das fotos + auditoria de 2 lentes:** 8 achados visuais + 9 de código, todos curados. Fixes: clippy (`af8ba09c94`), censo HR-15 «Ag» do `tooltip.rs` em `NOT_LANGUAGE_TEXT` (`79e5978c32`). Fotos `o4_*.png` (`1669011bbe`, `761c9f1589`, `a37a25a56e`, `cd8d758511`, `b496314f1d`).

## 2b. Pós-smoke do dono (11/11)

- **(a)** Alt ESQUERDO + F acendia o F na Try it (como AltGr+F). `view_try/mod.rs` guardava a tecla em baixo ANTES de consultar a lei (que só decidia o retorno). Cura `cfccd9c93e`: uma decisão governa retorno e estado (só tecla tomada ou modificador). Testes: `onda4_altgr::na_try_it_o_alt_esquerdo_mais_f_nao_acende_e_o_altgr_mais_f_acende`, `no_keyboard_o_alt_esquerdo_mais_f_nao_liga_e_o_altgr_mais_f_liga`.
- **(b)** Teste REAL no app (Xvfb, us/intl): AltGr+F acende/liga, AltL+F não. Fotos `o4_altgr_*` (`3bea4bfaf0`).
- **(c)** O teste real achou a tampa da tecla F vazia no cartão da acção: `view_keyboard/tiles.rs` media em Medium e pintava em Regular; a elisão cortava a letra. Mesmo par curado em `view_try/{cards,stick}.rs`, `travel.rs`, `view_list/rows.rs`. `17972271fb`; teste `teclado_tampas.rs` (vermelho→verde).
- **(d)** Pedido do dono (2.º smoke OK): Alt+F devia dizer porquê o F não acende. `TryState::kept` (a tecla não-modificador recusada, enquanto em baixo); `footer_parts` (agora `pub`): «…Left Alt + F: a shortcut of the editor…» (+ dica do AltGr). `e8a2ba2fdf`; teste em `onda4_altgr`; foto `o4_altgr_porque_alt_f.png`.

## 2c. Pedidos do dono pós-smoke (11/11): o aberto curado

- **Cores (`87072d4f54`):** AccentSoft dos Plumber escurecido até Text1 4,5:1 (PlumberRed 200,81,47→158,64,37; PlumberGreen 47,132,104→38,108,85; par novo no gate de contraste); numérico estreito com símbolos ISO 9995-7 (⇱⇲⇞⇟⎀⌦; 1280 = 9 legendas); tokens `key-ink`/`key-legend` (painel tokens 2104→2166 px). `921fb77d1e` (fecho): o critério normativo do par key-ink fica isento do censo (margem ×1,25 no `min_ratio`), `clone_on_copy`, ordem dos `mod`.
- **Hierarchy (`b31fcee90c`, `464efd009a`, `3c2302e97a`):** em repouso NENHUM ícone à direita da linha (interruptores no valor normal e ícones de componente); com hover/seleção/arrasto aparecem todos, em posições fixas; estado não-normal (escondido/travado) sempre. Balão do nome cortado (só com cartões); o hover da lista conta também em `PH2D_CARDS=0`. Testes `o_nome_tem_prioridade_sobre_os_interruptores` (4) + ajustados `os_cartoes`, `the_new_rows_of_the_hierarchy`, `the_clips_of_an_animation`.
- **Palco alarga (`81ca051861`):** Godot expand, sem tarjas. `crates/ph2d-camera-frame/src/palco.rs` (`area_do_palco`, `centro_no_palco`, `caixa_do_jogo` movida de ph2d-app-game, reexportada em `ph2d_app_game::encaixe`); shell `camera_no_canvas.rs::tarjas_do_palco` e `fase_game_camera.rs`. Formatos retro (Game Boy/NES/PICO-8) guardam as tarjas (keep), por decisão. Testes `jogo_palco_alarga` (3).
- **Defeito novo (`6dd3de020d`):** com uma aba de documento aberta (Input Map, Kanban, scripts…) o hover do canvas furava e acendia objectos na Hierarchy. `shells/desktop/src/hover_highlight.rs::pick_hovered_object` conta `hero.documents.surface_active().is_some()` como painel (responde a linha, nunca o canvas); shell sem crescer. Gate de fonte `uma_aba_de_documento_aberta_tapa_o_hover_do_canvas` em `shells/desktop/tests/it/o_palco_do_jogo_nao_tem_realce_de_hover.rs`; app real conferido.
- Fotos `docs/InputMap/fotos/`: `o5_numerico_1280(_zoom)`, `o5_tela_cheia_1600/1280`, `o5_hierarchy_repouso_e_hover`, `o5_aba_tapa_o_hover_do_canvas`, `o4_altgr_porque_alt_f` (.png).

## 3. Foundational/compartilhado tocado

Aditivo salvo o indicado: `ph2d-tokens` (`tokens.json` = formato gravado; `derive.rs` 700/700) · `ph2d-camera-frame` · `ph2d-panel-hierarchy` · `ph2d-editor-core` (`transport_bar`, `menu_bar`, `doc_surface` `raw_key(alt)`, `hero/paint`) · `ph2d-input` · `ph2d-panel-inspector` · `ph2d-panel-registry-init` · `ph2d-i18n` · `ph2d-kanban-views` · `ph2d-app-kanban`/`-planner` · `ph2d-host` · `shells/desktop` (`hover_highlight`, `camera_no_canvas`, `fase_game_camera`, `controles_do_jogador`).

## 4. Superfície de colisão (colada de `collision-surface.sh`, 10/11, linha inteira)

```

SUPERFÍCIE DE COLISÃO — line/InputMap contra main
  merge-base bc4a6ce930   ·   221 commit(s)   ·   431 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        232   (base: 231)
  ⚠   └ tripla do gate               (232, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs (escada e tripla em arquivos IRMÃOS)

▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              121   (base: 121)
  ✗ ph2d-render (espelho = ecs + K)       —   (base: —)
  ✗ ph2d-script (espelho = ecs + K)       —   (base: —)

▸ CONTRATO CONGELADO (§6): node.rs e tool.rs intocados · ADR: nenhum (próximo livre 0201)

▸ Cargo.lock
  ⚠ 5 pacote(s) '+name' novo(s): ph2d-app-input-map, ph2d-input-map-editor, ph2d-input-map-scene, ph2d-input-winit, ph2d-player-controls

▸ MARCADORES DE CONFLITO: nenhum

▸ TETOS DE LOC
  ✗  1023 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ✗ SONDA CEGA em ph2d-render/ph2d-script (const mudou de ficheiro; «—» dos dois lados)
```

Colada com 221 commits; os posteriores não tocam schema, registo, §6 nem `Cargo.lock`. **Os dois ✗ já existiam na base** (ondas 1–3): `main.rs` pré-existente; a SONDA CEGA é do script.

**Contadores como DELTA:** `PROJECT_SCHEMA` 0 nesta onda (231→232 é da onda 1; `python3 scripts/schema-recount.py`). Componentes 0. §6 intocado. ADR nenhum. `Cargo.lock`: só arestas internas e `bumpalo` dev-dep em game-scenes. `CENAS` **22 → 23**. Shell: −1 (`controles_do_jogador.rs`), `janela_do_jogo.rs` 0; re-meça contra o `main` do dia. Tokens do painel +92 (2074 → 2166).

## 5. Valores literais que colidem

Token `key-face`; smoke `PH2D_GAME_SMOKE` **6..23**.

## 7. O que um merge pode partir

1. **Tectos:** `ph2d-app-kanban` 32 175/32 175 · `ph2d-app-planner` 5 806/5 824 · `action_bus.rs` 700/700 · `ph2d-tokens/src/color.rs` 698/700 · `ph2d-app-input-map` 434/449. 
2. **Tecto do painel tokens 2166**; `ph2d-tokens/src/derive.rs` 700/700; `tokens.json` (formato gravado) com Plumber e tokens novos. `caixa_do_jogo` mudou de crate (`ph2d-camera-frame`, reexportada).
3. **`ColorToken::KeyFace` na lista da macro** (ordem): linha que acrescente token na mesma lista conflita.
4. **`DocumentSurface::raw_key` com `alt`:** implementações novas noutras linhas NÃO compilam. `surface_raw_key` mudou de crate (re-exportado).
5. **`ph2d_kanban_views::wheel_px`:** quem converter a roda à mão no kanban/planner converte DUAS vezes.
6. **`transport_bar`:** `rects_palco` com Controls, `paint_palco` com `text_system`. `menu_bar.rs` usa `chao_da_barra`.
7. **`CENAS` 23** (`ph2d-app-game/src/smoke.rs`) e `.config/testes-lentos.tsv`. Schema 232 (o da onda 1, não mexido).
8. **Atalhos com Alt:** linha que leia Alt do winit à mão deve passar por `chord_keys`.

## 8. Prova de fecho (feita na linha)

- `cargo fmt` ✓; `clippy --workspace --all-targets --keep-going -D warnings` ✓; machete ✓.
- **Fecho final (`921fb77d1e`):** clippy `--workspace --all-targets --keep-going` limpo, `fmt --check`, censos **133/133**, `architecture_workspace_file_loc_cap`, `architecture_the_shell_only_shrinks`, `architecture_no_dependency_climbs_a_layer`, testes AccentSoft fora de tokens (editor-core 4, ui-table 1) e os por nome. Depois de `6dd3de020d`: clippy `ph2d-panel-hierarchy` e `ph2d-host-desktop` limpos, `the_shell_only_shrinks` 2/2.
- Por nome (verdes, antes do §2b): editor-core `transport_bar` 16, `menu_bar` 19, `tooltip` 8, `dica` 21, `topbar` 12 e os gates `architecture`/`hr12`/`hr15`; player-controls `onda3` 16; game-scenes `input_map` 244, `cenas_do_smoke` 9, `mutacao` 18; kanban-views `deslize` 2; `swipe_lateral` 2+1; `ph2d-tokens` 196; `ph2d-tokens-dtcg` 26; i18n 4.
- **Mutação: 18 mutações, 17 sangraram na 1.ª.** Sobrevivente (tirar a `folga` do `layout.rs`) morto pelo teste reforçado em `3b902f4b62`.

## 9. Smoke (do dono) — **OK em 11/11**: Try it (AltGr/Alt+F com o porquê), numérico, Plumber, tela cheia, Hierarchy, hover tapado pela aba

Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=<n> PH2D_GAME_SMOKE_DRIVE=editor bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (cenas 6..23). 10 = teclado/Inspector; 9 = List com stick; 18 com `PH2D_GAME_SMOKE_DRIVE=fullscreen` = tela cheia com o botão Controls; 23 = hover (P5). Gamepad físico não smokado.

## 10. ABERTO e fora da linha

- CURADOS 11/11 (a)–(e): ver §2c (Plumber, símbolos ISO, `key-ink`/`key-legend`, Hierarchy, palco) + o hover tapado.
- (f) Num Lock é ícone e há fila de navegação sobre o numérico (onda 1, aprovado).
- (g) Sprite: o teste «nada muda» corre sem aba aberta (da onda 3, continua).
- (h) Gamepad físico não smokado (o dono: depois). (i) O token `selection` usa a mesma fórmula do AccentSoft e NÃO foi revisto.

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração; a linha NÃO a escreve)

Proposta (≤ 700 B):

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), teclado de notebook e comando, tela «Controls» no jogo (F1), ligação física, Luau pela ação (`ph2d-input*`, `ph2d-app-input-map`, `ph2d-player-controls`). Smoke `PH2D_GAME_SMOKE=6..23`. Último: [onda 4, 11/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA4.md) · [plano](docs/InputMap/01_plano.md)
