# HANDOFF de integração — `line/InputMap`, ONDA 4, 10/11 («vamos resolver todos» o aberto da onda 3)

Branch `line/InputMap`, código até `79e5978c32` (este handoff vem depois): a linha inteira, **225 commits**, 433 files, `+32 921 −3 479`, sobre `bc4a6ce930` = merge-base (a onda 3 o deu como ponta do `main`: reconfira no dia). A onda 4 são **28 commits** (`git log --oneline 44af187eba..HEAD`, base = `44af187eba`), 102 files, `+2 736 −265`. Nada integrado nem pushado. Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram (integrador). Completa o da onda 3 (schema 232, `play_map`, F1, tokens `danger-press`/`row-hover` valem): [onda 3](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA3.md) · [onda 2](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) · [onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano §14](../01_plano.md) · [prompt da onda 4](PROMPT_MISSAO_InputMap_ONDA4_2026-11-10.md).

## 1. Ordem do dono (10/11) — cumprida

«Vamos resolver todos» = o ABERTO do §10 da onda 3. O que sobra está no §10 e é de fora da linha.

## 2. O que entrou, por item

1. **Botão Controls na barra de TELA CHEIA** (`8623ef2e95`, `bfbba831ff`, `941143fd78`). `transport_bar::com_controls` é a regra única de `rects` e `rects_palco`; `paint_palco` usa a mesma `paint()` e pinta a dica partilhada (recebe `text_system`; `screens/hero/paint.rs:222` +1). `caixa_do_controls`: recuo do realce do Pause (`FIELD_BORDER`+`SEG_PAD`, 3 px), contorno recuado meia espessura (tolerância 0 no teste). Chão da barra `chao_da_barra` partilhado com `menu_bar.rs`. Separador = o filete da barra do editor (par (22,27,41)/(17,21,32), igual, aprovado na onda 3).
2. **Hover com a tela aberta** (`bfbba831ff`): `host::pointer_move_com_a_barra(s, hero, at)`; `transport_bar::sobre_o_controls`, `movimento_com_a_tela`, `solta_o_controls` (também no `CursorLeft`, `at=None`). Aceso + pairado = contorno/ícone `AccentHover`. Shell `controles_do_jogador.rs` (shell −1 líquido na onda). Painel Controls do jogo: largura medida com as caixas em repouso + `folga` reservada para «Press a key» (`ph2d-player-controls/src/layout.rs`); o título não desce ao escutar. Teste `r11a` antigo ajustado (espaço igual só em repouso). Testes: `a_tela_cheia_tem_o_controls_com_separador_clique_e_dica_inteira`, `com_a_tela_aberta_o_controls_acende_so_sob_o_ponteiro`, `sair_da_janela_pelo_botao_solta_o_hover_dele`, `o_painel_nao_muda_de_largura_ao_escutar` (`3b902f4b62`).
3. **AltGr** (`943cdc4027`, `5b2ff74e8a`, `7dbf42b127`). Medido em winit 0.30.13: `lalt/ralt_state` só macOS/Orbital; Windows apaga ALT+CTRL com AltGr; Linux br(abnt2)/us(intl): Alt direito = `ISO_Level3_Shift` em Mod5 (bandeira ALT apagada); `us` simples: `Alt_R` em Mod1 (Alt de verdade). Os atalhos do editor com Alt leem `ModifiersState::ALT` (app-code `keys.rs`, `keyboard_files.rs`, `board_keys.rs`, …). Lei nova `ph2d_input::editor_keys::chord_keys(held, alt)`: Alt direito só faz acorde com a bandeira ALT. `DocumentSurface::raw_key` ganhou `alt` (de `hero.store.alt_held()`, guardado em `InputMapDoc::alt`); `surface_raw_key` mudou para `ph2d-input-map-editor/src/surface_key.rs` (re-exportado). Vistas Keyboard (`view_keyboard/input.rs:213`) e Try it (`view_try/mod.rs:100`) usam `chord_keys`. macOS: Option direito = Alt (limite de desenho). Testes: `altgr_nao_faz_acorde_e_o_alt_esquerdo_faz`, `onda4_altgr::*`.
4. **Kanban sem swipe** (`bc639894a6`, `4f352456c5`). MEDIDO, nada mudou no código: Calendário Month/Week/Day/Agenda, Dashboard e gaveta da Timeline NÃO rolam na horizontal (conteúdo cabe a 1600/1280/460); só a grelha da Timeline rola. Travado por `deslize::o_deslize_de_lado_so_mexe_onde_o_conteudo_rola_na_horizontal`. Sonda `TableState::cell` movida para kanban-views; `probe_covers_pending` apagada.
5. **Planner roda ao contrário: CONFIRMADO** (−480 levou out → fev/2026). Uma conversão só: `ph2d_kanban_views::wheel_px` (app-kanban apagou a cópia e as constantes de `metrics.rs`, re-exporta em `pointer.rs`). O Planner converte 1× à entrada (`surface.rs`) e cobre mês, Week, Agenda e «Move to» (`dest_pop`). `Row/rows/first_pick` movidos para `ph2d-kanban-views/src/dest.rs`; `CalendarState::mode_rect` novo (`6d02d1db2e`). Testes: `swipe_lateral::a_roda_do_calendario_anda_para_o_mesmo_lado_que_no_kanban`, `deslize::a_roda_convertida_rola_a_agenda_para_baixo_e_a_crua_ao_contrario`.
6. **Token de cromo `key-face`** (`5bc6ea28c0`). `tokens.json` forge/sunstone/blueprint = valores de `kanban-card-raised`; `derive.rs` empresta como o card; `ColorToken::KeyFace` na lista da macro; `REST_FACE` aponta-o. Igual byte a byte nos 12 temas; compare de píxeis fora do numérico = 0. `color.rs` 699→698; tecto do painel tokens 2074→2104 (medido 30 px, não 31). Teste `a_face_da_tecla_e_o_token_de_cromo_com_o_tom_aprovado`.
7. **Segundas legendas do numérico** (`view_keyboard/sub_legend.rs`; `5bc6ea28c0..e0c68cb7a8`). Arranjo da foto do dono: número 15 px no canto sup.-esq.; legenda (nome da tabela de teclas; setas = ícones à altura de maiúscula, 7,14 px) Regular 10 px (piso), alinhada à direita na faixa de baixo, `SUB_PAD` 2 px (= `BAR_VISIBLE_MIN`), fora das curvas dos cantos (raio 4). Tudo-ou-nada POR CONJUNTO (palavras; setas); as barras vencem na tecla ligada; legenda ≤ 75 % da tinta do número (67 %), contraste menor nas teclas ligadas. Contagens: 1600 = 9/10, 1280 = 3 (só setas), estreita 3, 1920 10 — iguais em Default e CrispHeavyPlus. `LEGEND_PAD`/`LEGEND_MARGIN` pub. Testes `onda4_numerico` (5). 
8. **Inspector na aba Input Map** (`26ea907258`, `5b2ff74e8a`, `a7e487c7a6`, `245b15f3f5`, `38397df513`, `c462b02c3a`). Tipo `InputMapPane` em `ph2d-input/src/inspected.rs` (layering); `ph2d-panel-inspector/src/cards_input_map.rs` + `paint.rs` (só com cartões; `PH2D_CARDS=0` intocado; × do cabeçalho registado); `inspect::set_snapshot` publica DEPOIS do snapshot (mesmo quadro). Cartão Action (Name, Group, Read by, Keys) + Stick (Dead zone (0 to 1) / Press at (0 to 1), só leitura, chaves i18n da aba). Uma fonte: `view_keyboard::open(doc)` (a escolhida, senão a 1.ª) nas três vistas; a frase «Pick an action in the list.» só com mapa vazio; clique numa linha (ou nos botões dela) abre a acção; `map_swapped` confere id+nome. Linha aberta da List: `row_face` (5 faces distintas; aberta = `CardFieldHover`, nunca AccentSoft) e rótulos `Text2` ≥ 6,70:1 nos 12 temas. i18n novo `ph2d-i18n/src/inspector_input_map.rs`. `ph2d-app-input-map` 449→434. Testes: `o_inspector_da_aba_input_map` (5), `the_inspector_shows_the_new_snapshot_in_the_same_frame`, `a_swapped_map_with_the_same_ids_closes_the_open_action`, `the_open_row_labels_read_in_every_theme_and_its_face_is_its_own`, game-scenes `onda4_inspector`.
9. **Cena nova P5 = 23** (`27362e6bb3`): ponteiro no centro do Controls (rect do `hit_index` do quadro anterior) pela porta `pointer_moved` da shell; `janela_do_jogo.rs` líquido 0; `ph2d-host` dep de `ph2d-input-map-scene`, `bumpalo` dev-dep de game-scenes. CENAS 22→23. Teste `a_cena_p5_poe_o_ponteiro_no_controls_da_barra`.
10. **Revisão independente das fotos + auditoria de 2 lentes:** 8 achados visuais + 9 de código, todos curados ou aceites (aceites no §10). Fixes de fecho: clippy (`af8ba09c94`), censo HR-15 «Ag» do `tooltip.rs` da onda 3 em `NOT_LANGUAGE_TEXT`, com o mecanismo (`79e5978c32`). Fotos `docs/InputMap/fotos/o4_*.png` (`1669011bbe`, `761c9f1589`, `a37a25a56e`, `cd8d758511`, `b496314f1d`).

## 3. Foundational/compartilhado tocado

Aditivo salvo o indicado: `ph2d-tokens` · `ph2d-editor-core` (`transport_bar`, `menu_bar`, `doc_surface` `raw_key(alt)`, `hero/paint`) · `ph2d-input` · `ph2d-panel-inspector` · `ph2d-panel-registry-init` · `ph2d-i18n` · `ph2d-kanban-views` · `ph2d-app-kanban`/`-planner` · `ph2d-host` · `shells/desktop`.

## 4. Superfície de colisão (colada de `collision-surface.sh`, 10/11, linha inteira)

```

SUPERFÍCIE DE COLISÃO — line/InputMap contra main
  merge-base bc4a6ce930   ·   221 commit(s)   ·   431 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        232   (base: 231)
  ⚠   └ tripla do gate               (232, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs (escada e tripla em arquivos IRMÃOS)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              121   (base: 121)
  ✗ ph2d-render (espelho = ecs + K)       —   (base: —)
  ✗ ph2d-script (espelho = ecs + K)       —   (base: —)

▸ CONTRATO CONGELADO (§6): node.rs e tool.rs intocados · ADR: nenhum (próximo livre 0201)

▸ Cargo.lock
  ⚠ 5 pacote(s) '+name' novo(s):
      "ph2d-app-input-map"
      "ph2d-input-map-editor"
      "ph2d-input-map-scene"
      "ph2d-input-winit"
      "ph2d-player-controls"

▸ MARCADORES DE CONFLITO: nenhum

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗  1023 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ✗ SONDA CEGA em ph2d-render/ph2d-script (const mudou de ficheiro; «—» dos dois lados)
```

Colada com 221 commits; os 4 últimos (`a37a25a56e`..`79e5978c32`) vieram depois (docs, testes, censo), sem tocar schema, registo, §6 nem `Cargo.lock`. **Os dois ✗ já existiam na base `44af187eba`** (e nas ondas 1–3): `main.rs` 1 023/600 é pré-existente (a linha não o toca); a SONDA CEGA `ph2d-render`/`ph2d-script` é do script (a const mudou de ficheiro), a linha não toca o registo.

**Contadores como DELTA:** `PROJECT_SCHEMA` 0 nesta onda (231→232 é da onda 1; `python3 scripts/schema-recount.py`). Componentes 0. §6 intocado. ADR nenhum. `Cargo.lock`: só arestas internas (+`ph2d-input` no panel-inspector, `ph2d-host` em input-map-scene, `bumpalo` dev-dep em game-scenes); nenhum pacote externo novo. `CENAS` **22 → 23**. Shell: −1 (`controles_do_jogador.rs`), `janela_do_jogo.rs` 0; re-meça contra o `main` do dia. Tokens do painel +30 (2074 → 2104).

## 5. Valores literais que colidem

Token `key-face`; smoke `PH2D_GAME_SMOKE` **6..23**. Renumere pelo `main` do dia.

## 7. O que um merge pode partir

1. **Tectos:** `ph2d-app-kanban` 32 175/32 175 · `ph2d-app-planner` 5 806/5 824 · `action_bus.rs` 700/700 (não tocado) · `ph2d-tokens/src/color.rs` 698/700 · `ph2d-app-input-map` 434/449. 
2. **Tecto do painel tokens 2104** (medido 30 px por fileira de cromo).
3. **`ColorToken::KeyFace` na lista da macro** (ordem): linha que acrescente token na mesma lista conflita.
4. **`DocumentSurface::raw_key` com `alt`:** implementações novas noutras linhas NÃO compilam. `surface_raw_key` mudou de crate (`ph2d-input-map-editor/src/surface_key.rs`, re-exportado).
5. **`ph2d_kanban_views::wheel_px`:** quem converter a roda à mão no kanban/planner converte DUAS vezes. Constantes de `metrics.rs` do app-kanban apagadas.
6. **`transport_bar`:** `rects_palco` com Controls, `paint_palco` com `text_system`; chamadores novos noutras linhas não compilam. `menu_bar.rs` usa `chao_da_barra`.
7. **`CENAS` 23** (`ph2d-app-game/src/smoke.rs`) e `.config/testes-lentos.tsv` (gerado). Schema 232 (o da onda 1, não mexido). Itens do §7 da onda 3 valem.
8. **Atalhos com Alt:** linha que leia Alt do winit à mão deve passar por `chord_keys`.

## 8. Prova de fecho (feita na linha)

- `cargo fmt` ✓; `clippy --workspace --all-targets --keep-going -D warnings` ✓; machete ✓; `check-standalone-optional.sh` ✓ (11); `check-workflow-packages.sh` ✓ (32/519).
- Censos da árvore combinada **132/133**: o vermelho (HR-15 «Ag») curado em `79e5978c32`; teste re-corrido por nome: 5 verdes.
- Por nome (todos verdes): editor-core `transport_bar` 16, `menu_bar` 19, `tooltip` 8, `dica` 21, `topbar` 12, gates `architecture` 122, `hr12`/`hr15`/`no_literal_color`/`no_magic_numeric`/`file_loc_cap`/`the_shell_only_shrinks`; player-controls `onda3` 16, `r7c`/`r10a`/`r11a`/`r9d`/`r5c`/`f3_tests`/`play_tests`/`largura`; `ph2d-input` `editor_keys` 2; input-map-editor `doc::` 8, `inspect` 2; panel-inspector 5; game-scenes `input_map` 244, `cenas_do_smoke` 9, `mutacao` 18; kanban-views `deslize` 2; app-kanban `swipe_lateral` 2, `j1_` 4; planner `swipe_lateral` 1; `ph2d-tokens` 196 (inteira, contrato de token); `ph2d-tokens-dtcg` 26; panel-registry-init 18; i18n 4.
- **Mutação: 18 mutações, 17 sangraram na 1.ª.** Sobrevivente (tirar a `folga` do `layout.rs`) morta pelo teste reforçado em `3b902f4b62` (M1 sem `folga` → vermelho; M2 medida antiga → 242→283 vermelho).

## 9. Smoke (do dono) — **PENDENTE**

Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=<n> PH2D_GAME_SMOKE_DRIVE=editor bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (cenas 6..23). 10 = teclado/Inspector; 9 = List com stick; 18 com `PH2D_GAME_SMOKE_DRIVE=fullscreen` = tela cheia com o botão Controls; 23 = hover (P5). Gamepad físico não smokado.

## 10. ABERTO e fora da linha

- (a) `AccentSoft` (seleção da casa) dá Text1 3,18:1 no PlumberRed e 3,36:1 no PlumberGreen — app inteiro; a vista Keyboard mantém o azul aprovado; a cura é de token da casa.
- (b) Legendas de texto do numérico ausentes a 1280 (Home > tecla a 10 px; o piso de 10 px é da casa).
- (c) Na tecla não ligada a legenda tem a mesma cor do número (nenhum token de texto dá ≥3:1 e < número nos 12 temas).
- (d) Nomes cortados «Collision Shape (…» na Hierarchy em Play (outra família).
- (e) Faixa preta abaixo do chão na tela cheia (letterbox do jogo). (f) Num Lock é ícone e há fila de navegação sobre o numérico (onda 1, aprovado).
- (g) Sprite: o teste «nada muda» corre sem aba aberta (da onda 3, continua).
- (h) Gamepad físico não smokado. (i) Smoke do dono desta onda (§9).

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração; a linha NÃO a escreve)

Proposta (≤ 700 B), igual à da onda 3 com smoke e link novos:

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), teclado de notebook e comando, tela «Controls» no jogo (F1), ligação física, Luau pela ação (`ph2d-input`, `ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`, `ph2d-input-winit`). Smoke `PH2D_GAME_SMOKE=6..23`. Último: [onda 4, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA4.md) · [plano](docs/InputMap/01_plano.md)
