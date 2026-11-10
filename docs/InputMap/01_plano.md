# Input Map — plano técnico da linha `line/InputMap` (10/11)

> Leitor: o agente que implementa uma onda. A ordem do dono e as 4 decisões estão no
> [prompt de missão](handoffs/PROMPT_MISSAO_InputMap_2026-11-10.md) §2; o alvo visual, nas
> [maquetes](maquetes/) (abra os PNG como IMAGEM) e no [texto](maquetes/critica_inputmap_texto.txt).
> Aqui fica o DESENHO e os CONTRATOS entre ondas (nomes de tipos, funções, chaves, ficheiros).
> Mudou um contrato? Escreva-o aqui no mesmo commit.

## 0 · Factos medidos no levantamento (10/11)

- O resolvedor JÁ consulta o comando: `ph2d-input/src/resolve.rs:78-91` (`PadButton` → `gamepad.held`,
  `PadAxis` → metade do eixo). A conta: `cru = max(ligações)`; `strength = cru < dead ? 0 :
  ((cru − dead)/(1 − dead)).clamp(0,1)`; `pressed = cru >= press_point` (`:104-119`).
- O gamepad já entra no `InputState` do shell a cada quadro (`app_state_devices.rs:53-80`,
  `gilrs_adapter.rs`), e o Play resolve com `resolve_player_input(&mut self.input_actions,
  hero_screen.input_map, &self.input)` (`render_loop/fase_pointer_subjects.rs:~49-58`).
- A captura do comando JÁ existe, presa à janela: `shells/desktop/src/input_map_drag.rs:55-127`
  (`poll_input_map_pad_binding`, filtra `listen_replaces`). A captura de tecla é partilhada (Trigger
  também usa): `input_dispatch/keyboard.rs:83` → `keyboard_bind_capture.rs` → `input_map_ops.rs:76-184`.
- O mapa vive em `hero.input_map` (estado do editor), é gravado em `shells/desktop/src/project.rs:152`
  e está FORA do undo (`action_bus.rs:385-386`).
- `Key(u32)` normalizado (`ph2d-input/src/keyboard.rs:21`), `Key::label()` = «Left Arrow».
- A aba central recebe teclas no vocabulário `doc_surface::Key` (sem Shift/Ctrl/F físicas).
- **Não existe jogo exportado** (nenhuma crate/binário/menu de export de jogo). A tela de controles
  entra no runtime do Play; a exportação, quando existir, herda-o. O relatório ao dono diz isto.
- Não existe id de projecto (só o contador de `StableId`). Preferências do utilizador:
  `shells/desktop/src/prefs.rs` (`~/.ph2d/prefs.txt`, `chave=valor`).
- `W` no editor abre o painel de Física (`input_handlers.rs:172-184`); no Play, tecla nua é do jogo
  (`posse_do_jogo.rs`), e com a aba focada a aba é dona do teclado — não briga.
- Teclas do Play: F5 transporte, acordes ao editor, tecla nua ao jogo, `tecla_do_jogo` antes de todo
  atalho (`keyboard.rs:106-112`).

## 1 · Crates e quem é dono de quê

| Crate | Papel | Depende de |
|---|---|---|
| `ph2d-input` (existe) | modelo puro: mapa, receitas, leitores, teclas do jogador, controlos | serde |
| `ph2d-input-map-editor` (NOVA, folha de pintura, molde `ph2d-anim-editor`) | estado da aba (`InputMapDoc`), layout, pintura das 3 vistas, censo | editor-core, card-stack, i18n, tokens, text, vector, ph2d-input, ph2d-player-controls |
| `ph2d-player-controls` (NOVA, folha) | a tela «Controls» do jogador no Play: estado, pintura, teclado da tela, ficheiro do jogador | editor-core (paint), tokens, i18n, text, ph2d-input |
| `ph2d-app-input-map` (NOVA, família) | `DocumentSurface`, `install`, `frame`, aplicar pedidos, factos dos leitores, ponte das prefs | editor-core, app-host, input-map-editor, player-controls, ph2d-input, ph2d-ecs |

A shell só COMPÕE (~3 linhas por ponto). `the_shell_only_shrinks`: a janela que sai paga as linhas novas.

## 2 · Modelo (`ph2d-input`)

- `controls.rs`: `pub struct PlayerControls { pub players_can_change_keys: bool, pub hidden_in_game:
  BTreeSet<ActionId>, pub open_key: Key }`. Default: `false`, vazio, `F1`.
  ⚠️ **Guarda-se o ESCONDIDO, não o mostrado** (desvio consciente do «shown_in_game» do prompt): uma
  acção nova entra visível na tela do jogador sem segunda escrita; a caixa «Shown in the game» é
  `!hidden`. `InputMap` ganha o campo `controls: PlayerControls` (formato muda ⇒ schema +1, §6).
- `recipes.rs`: `pub enum Recipe { Platformer, TopDown, Shooter, Blank }`, `Recipe::ALL`,
  `label_key()/blurb_key()` (chaves i18n), `actions()`. `InputMap::start_from(&mut self, Recipe)`
  troca as acções, mantém `next_id` a subir (id nunca reutilizado) e mantém `controls`.
  Conteúdo: Platformer = move_left · move_right · jump · down · dash · grab; Top-down = move_left ·
  move_right · move_up · move_down; Shooter = move_left · move_right · jump · fire · reload; Blank = nada.
  Teclas: as de fábrica + o comando no idioma do ecossistema (manche esquerda + cruz para andar, `South`
  salta, `West` dash/fire, `East` grab, `North` reload, `RightTrigger` fire), zona morta 0.2 só nas
  acções com manche (Godot e GDevelop usam 0.2). Teste: os nomes batem com `PLATFORM_PLAYER_READS` /
  `TOP_DOWN_PLAYER_READS`.
- `readers.rs`: `pub enum Reader { PlatformPlayer, TopDownPlayer, Trigger { object: String },
  Script { name: String } }`; `pub const PLATFORM_PLAYER_READS: [&str; 6]`,
  `pub const TOP_DOWN_PLAYER_READS: [&str; 4]` (a fonte dos grupos); `pub struct ReaderFacts {
  pub triggers: Vec<(String, String)>, pub scripts: Vec<(String, String)> }` (objecto/acção ·
  nome do script/fonte); `pub fn readers(&ReaderFacts) -> BTreeMap<String, Vec<Reader>>` (os leitores
  estáticos entram sempre, são contrato do código); `pub fn action_names_in_script(src) ->
  BTreeSet<String>` (procura `ph2d.action_down|action_pressed|action_released|action_strength("x")`,
  aspas simples ou duplas); `pub struct SharedBinding { binding, actions: Vec<ActionId>, on_purpose:
  bool }` e `pub fn shared_bindings(&InputMap, &BTreeMap<..>) -> Vec<SharedBinding>`.
  **Lei do «de propósito»:** para todo par do grupo, os dois têm leitores, só de mover
  (Platform/TopDown), e os conjuntos são DISJUNTOS (um objecto tem um mover). Senão é engano e avisa-se.
  Teste em `ph2d-physics-ecs`: cada nome das duas tabelas muda o `PlayerInput`, e nenhum outro muda.
- `player_keys.rs`: `pub struct PlayerKeys { by_action: BTreeMap<String, Vec<Binding>> }` (chave =
  NOME, o contrato de quem lê); `set`, `clear`, `apply_over(&InputMap) -> InputMap` (só troca as
  ligações de acções que existem), `changed(&InputMap) -> usize`, `to_text()/from_text()` (linha
  `jump = key:up, key:z, pad:south, axis:left_x-`; linha inválida salta-se), teste de ida e volta.
- `resolve.rs` expõe a lei: `pub fn strength_of(raw, dead) -> f32` e `pub fn pressed_of(raw, press) ->
  bool`, que o `resolve_action` chama (a UI usa ESTAS, nunca uma cópia). `InputAction::is_analog()` =
  tem `PadAxis` ou gatilho — é a lei «Dead/Press só com manche».
- Corrigir o comentário velho de `map.rs` («as SETAS, e não o WASD» — o código põe W/S; o W do editor
  abre a Física, mas com a aba ou o jogo com o teclado não briga).

## 3 · A aba (`ph2d-input-map-editor` + `ph2d-app-input-map`)

- `View { List, Keyboard, TryIt }` (`pref_key()`: `list|keyboard|try`). 1.ª vez: `Keyboard`. A última
  escolha vai para `Prefs.input_map_view` (`prefs.rs`, chave `input_map_view=`).
- **Topo:** seletor segmentado no estilo do cartão (`property_row/escolha.rs::paint_choice_row`,
  o mesmo do `Widget::Choice`), rótulos `List · Keyboard · Try it`, compacto à esquerda.
- **Um modelo, três vistas:** o `hero.input_map`. A aba nunca guarda cópia editável; pinta de um
  `Snapshot` publicado no `frame` e empurra `InputMapEdit` para `pending`.
- `enum InputMapEdit { Add(String), Remove(ActionId), AddBinding(ActionId, Binding),
  RemoveBinding(ActionId, usize), SetZone(ActionId, f32, f32), StartFrom(Recipe),
  SetPlayersCanChange(bool), SetShown(ActionId, bool), SetOpenKey(Key), ForgetPlayerKeys }`.
  A família aplica-os em `hero.input_map` no `frame`.
- **Undo:** o `InputMap` entra no `ProjectState` do undo (`shells/desktop/src/undo.rs`). Porquê: é
  documento (vai no `.ph2dproj`); o undo global é por DIFF (`post_frame_undo`), então toda edição do
  mapa — receita, remover, trocar, zona — vira passo sem instrumentar gesto; o «Create this action» do
  Trigger passa a desfazer-se também. Arrasto de zona marca gesto em curso (um passo por arrasto).
  «Start from…» com acções existentes pede confirmação (diálogo da aba) e é UM passo.
- **Escuta:** a partilhada (`listen_for_binding`/`capture_if_listening`), que o shell corre antes de
  tudo. A captura do comando (`poll_input_map_pad_binding`) sai do `input_map_drag.rs` e passa a servir a
  aba. Texto condicional: com comando ligado «Press a key or a gamepad button. Esc cancels.»; sem,
  «Press a key. Esc cancels.» O `Snapshot` leva `gamepad_connected`.
- **Teclas físicas na aba:** método novo, append-only, em `DocumentSurface`:
  `fn raw_key(&mut self, _id: u64, _key: ph2d_input::Key, _pressed: bool, _writes_text: bool) -> bool
  { false }`. O shell chama-o para a superfície activa logo depois da escuta e dos modais (antes de
  `tecla_do_jogo` e dos atalhos). `true` = consumida. Usos: busca por tecla (A) e Try it (C).
- **Busca (A):** campo «Find an action, or press a key». Tecla que não escreve (setas, Space, F, Shift…)
  com o campo focado ⇒ filtro POR TECLA (o campo mostra o chip da tecla, × limpa). Letra ⇒ texto; uma
  letra só filtra também as acções ligadas a essa tecla. Pé: «Up Arrow is in 2 of 11 actions. jump is
  read by Platform Player, move_up by Top-Down Player.»
- **Cor por acção:** paleta categórica de tokens `Categorical01..12` (`ph2d-tokens`), `cor =
  PALETA[id.0 % 12]` — por `ActionId`, nunca pela posição. A ordem da paleta casa a maquete (os ids de
  fábrica 0..7 nascem na ordem de `with_player_defaults`).
- **Leitores:** a família junta os `ReaderFacts` (Triggers da cena com o nome da acção; scripts Luau do
  projecto) e publica `readers` no `Snapshot`; B agrupa (PLATFORM PLAYER · TOP-DOWN PLAYER · YOURS),
  «Read by …», aviso âmbar «Nobody reads reload yet. A Trigger can turn it into a signal: type reload
  in its Action.», e a frase da tecla repetida (de propósito explica; engano avisa).
- **Dead/Press:** só quando a acção tem `PadAxis` ou gatilho (`LeftTrigger`/`RightTrigger`). Rótulos
  inteiros «Dead zone (0 to 1)», «Press at (0 to 1)». B tem a barra do curso (0 · Dead · Press · 1, zona
  morta hachurada); C tem o círculo (disco interno = Dead, anel tracejado = Press, ponto ao vivo,
  «Pushed 0.62 · Strength 0.55 · Pressed»). A conta vem de `resolve.rs` (função pública, não cópia).
- **Try it (C):** com o jogo parado, `raw_key` alimenta um `InputState` de BANCADA da família (nunca o
  do jogo); o comando lê-se do `InputState` do shell. Barras de força 0.00–1.00 por acção vindas de um
  `ActionState` de bancada a correr `tick` sobre o mapa. Nunca escreve no projecto. Com o Play a correr:
  «The game is running» e lê o estado do jogo, sem consumir teclas.
- **Censo (prova 22→0 e 38→0):** a pintura regista cada texto (rect, se foi elidido) e cada campo
  numérico (id da acção); teste pinta o mapa de 11 acções nas 3 vistas a 1600×1000 e 1280×720 e afirma
  0 elididos e 0 campos numéricos em acção sem manche/gatilho.

## 4 · Tela do jogador no Play (`ph2d-player-controls`)

- Abre no Play com `controls.open_key` (F1 por omissão; F1 não colide: F5 é transporte, acordes são do
  editor, o mapa de fábrica não usa F) — só se `players_can_change_keys`. A porta corre ANTES de
  `tecla_do_jogo`. Aberta: pausa o relógio do jogo e retoma ao fechar; teclado e rato são dela
  (↑/↓ escolhe, Enter ou clique escuta, a tecla nova troca a ligação de teclado escolhida, Esc cancela
  ou fecha, «Reset to defaults»). Lista as acções não escondidas, com nome legível («Move left»).
- Ficheiro do jogador: `~/.ph2d/players/<game_id>.txt` (pasta das prefs), `PlayerKeys::to_text`.
  Carrega-se ao entrar no Play e aplica-se por cima (`apply_over`); o MESMO mapa efectivo serve
  `resolve_player_input`, `trigger_bridge`, `personagem`, `morph_machine_drive` e o Luau. Nunca toca o
  `.ph2dproj`.
- `game_id: u64` novo no ficheiro do projecto (não no undo), sorteado ao criar o projecto.
- Na aba (C·4): prévia pintada pela MESMA função da tela, a caixa «Players can change keys», «Opens with
  F1» (escuta) e «Shown in the game» (uma caixa por acção); se este computador tem teclas do jogador
  para este jogo: «On this computer, a player changed 2 keys. Forget them».

## 5 · Luau

`ph2d.action_down(name)`, `ph2d.action_pressed(name)`, `ph2d.action_released(name)`,
`ph2d.action_strength(name) -> number` — o idioma de `key_down/key_pressed/key_released`. Nome que o
mapa não tem = erro que lista as acções (o «nome errado é silêncio» da crítica acaba aqui). Lê o MESMO
`ActionState` que o Play resolve. `runtime/luau/ph2d.d.luau`, `api_table.rs`, `api_help.rs`,
`ph2d-code-help`, `docs/CodeEditor/05_api_que_funciona.md`. O jeito pela tecla fica.

## 6 · Projecto e schema

`PROJECT_SCHEMA` 231 → 232 (UM degrau pela linha): `InputMap.controls` + `game_id` + o mapa no
`ProjectState`. Sem migração nem leitor velho (CLAUDE.md §0.11); recontar com
`python3 scripts/schema-recount.py`.

## 7 · i18n e ids (um ficheiro por dono, para agentes em paralelo não se pisarem)

`ph2d-i18n/src/`: `input_map_tab.rs` (topo, comum, diálogo), `input_map_list.rs` (A),
`input_map_keyboard.rs` (B + comando + barra), `input_map_try.rs` (C), `player_controls.rs` (tela do
jogador). Só inglês, tooltips em Plain English. Ids de controlo no `src/ids.rs` da crate que os lê.

## 8 · Smoke e fotos

Cenas em `PH2D_GAME_SMOKE` a partir da **6** (contado em `ph2d-app-game/src/smoke.rs:23`, `CENAS = 5`),
com o mapa de 11 acções (fábrica + fire `J`, reload sem tecla, laser `H`): A1 fábrica · A2 escutando
(reload) · A3 busca Up Arrow · A4 manche no move_left · B1 jump aberto · B2 ponteiro no ↑ · B3 comando
(move_left) · B4 reload sem tecla · C1 receitas · C2 ao vivo (↑ apertada) · C3 manche 0.62 · C4 tela de
controles · P1 Play com a tela aberta. Foto: `bash docs/Components/ferramentas/fotografa_cena.sh
PH2D_GAME_SMOKE=<n> 1600 1000 <saida.png>` e a 1280×720. Montagens finais em `docs/InputMap/fotos/`.

## 9 · Ondas e rodadas

| Rodada | Em paralelo (crates disjuntas) |
|---|---|
| R1 | modelo (`ph2d-input`) ‖ esqueleto (2 crates novas, aba, seletor, `raw_key`, menu, prefs, i18n) ‖ paleta (`ph2d-tokens`) ‖ Luau |
| R2 | A · List ‖ a janela flutuante sai + escuta do comando realojada ‖ undo + schema + `game_id` |
| R3 | B · Keyboard + comando + barra do curso ‖ tela do jogador (`ph2d-player-controls` + Play) |
| R4 | C · Try it + receitas + círculo + C·4 ‖ cenas de smoke |
| R5 | fotos + revisor lado a lado + correções (repetir até a lista vir vazia) |
| Fecho | fmt/clippy, mutação (4 leis), auditoria ≥2 lentes, rebase, censos, handoff, binário |

## 10 · Contratos como ficaram depois da R1 (fonte: o código; isto é o mapa)

- **ph2d-input:** `Recipe::actions() -> Vec<RecipeAction { name, bindings, dead_zone }>`; `RECIPE_FIRE`,
  `RECIPE_RELOAD` (Shooter: fire = J, reload = R); chaves `input_map_try.recipe.<r>` e `.blurb`;
  `Reader::label_key() -> Option<&str>` (`input_map_tab.reader.{platform_player,top_down_player}`),
  `Reader::is_mover()`; `PlayerKeys::{new,get,set,clear(action),clear_all,is_empty,apply_over,changed,
  to_text,from_text}`, `binding_text(Binding)`, `PLAYER_KEYS_HEADER`; eixo em texto com o nome real
  (`axis:left_stick_x-`); tecla sem nome de script = `key:0x<hex>`; `PlayerControls::{shown_in_game,
  set_shown_in_game}`; `shared_bindings` devolve por ordem de tecla; `is_analog` cobre gatilho como
  botão e como eixo.
- **Tokens:** `Categorical01..12` (`tokens.json`), `ph2d_tokens::{CATEGORICAL, categorical(i)}`.
- **Luau:** `ph2d.action_{down,pressed,released,strength}` (`ph2d-script/src/action_api.rs`), feed
  `Devices::feed_actions(Option<(&InputMap, &ActionState)>)`. ⚠️ O mapa do Play é lido em QUATRO sítios
  com `input_actions`: `fase_pointer_subjects.rs:53` (resolve), `app_state_devices.rs:116` (Luau),
  `fase_vector_view_and_drives.rs:90`, `fase_signal_outbox.rs:30`. O mapa efectivo (§4) entra por UM
  acessor do shell usado pelos quatro.
- **Aba:** `ph2d_app_input_map::{FAMILY ("input-map"), install(hero, View), frame(world, hero,
  gamepad_connected), view(&hero), surface_raw_key(hero, code, pressed, text), InputMapTab, TAB_ID}`;
  `ph2d_input_map_editor::{View, InputMapDoc{snap, pending, press, motion, hover, segment_rect},
  Snapshot{map, gamepad_connected, play_running}, layout::lay_out, paint::paint_doc,
  view_*::paint, ids::VIEW_*}`; menu → `hero.store.request_input_map_tab()`; prefs
  `input_map_view` gravado pelo detector de `forwarding_persist.rs`; o seletor pinta com
  `Widget::Choice` via `CardCtx::flutuante`. Arnês: `shells/game-scenes/tests/it/input_map/mod.rs`
  (`Mesa`).

## 11 · Contratos depois da R2

- **A janela saiu** (−1 211 editor-core, −188 shell, −1 023 testes, −48 i18n). `chrome/input_map.rs` ficou só
  com o abridor do menu, os braços partilhados da escuta (`INPUT_MAP_BIND_CAPTURED`,
  `INPUT_MAP_LISTEN_CANCELLED`) e `pedidos.rs` (Trigger). Scrollbar 842 reformada, nunca reutilizada.
- **Escuta:** o slot partilhado é `Option<Binding>` (`capture_binding`/`take_captured_binding`); o comando
  entra por `ph2d_app_input_map::listen::{pad_binding, deliver_pad}` (`AXIS_PUSHED = 0.5`), chamado em
  `render_loop/fase_pointer_subjects.rs:50-52`, antes do resolve. A aba pede a escuta por
  `InputMapDoc::take_listen_request`; a família chama `listen_for_binding`/`stop_listening`.
- **Undo:** `ProjectState.input_map` (`undo.rs:100`), `ProjectState::{do_hero, ao_hero}` (`undo_app.rs:578/585`,
  o restauro corta a escuta); `ProjectFile.input_map` saiu (uma fonte só). Gesto em curso:
  `ph2d_app_input_map::gesture_in_progress(&hero)` em `undo_app_macros.rs::motivo_da_supressao`.
- **game_id:** `ProjectFile.game_id: u64` (último campo), recurso do mundo `project::GameId`,
  `App::game_id()` (`project.rs:212`), sorteado na 1.ª chamada (não há comando «New project»). Schema 232
  (escada + const em `project_schema.rs`, tripla `(232, 13, 29)` em `project_schema_tests.rs`; conferido
  por `python3 scripts/recontar.py --apenas schema`).
- **A · List:** `Snapshot.{readers, listening}`; `InputMapDoc.{list, census}`; ids `NEW_NAME, ADD, SEARCH,
  SEARCH_CLEAR`, `ListHit::{RemoveBinding, Listen, Trash, Dead, Press, Row}`; `view_list::{shown, tip_of,
  binding_label}`, `palette::of_action`; leitores = `SignalOnAction` (nome da entidade + acção) e
  `ProjectScripts` do mundo (`readers::{facts, of_world}`). Comando com `IconId::Player`, tecla `IconId::Kbd`.
- **Aberto da A** (vai para a rodada de acabamento): nome da acção em peso normal (maquete: negrito); chip
  da manche sem seta de direcção; sem barra de rolagem desenhada; sem menu de botão direito; a frase do
  pé do estado 4 («Dead zone and Press at only show on rows with a stick…») não aparece.

## 12 · Contratos depois da R3

- **B · Keyboard:** `crates/ph2d-input-map-editor/src/view_keyboard/` (9 ficheiros; geometria em
  `geometry.rs`), `KbState{open, sub, hover, adding, select_name, bars}`, `KbHit`, `Sub`, `InputMapDoc.kb`,
  `view_keyboard::{paint, Group, group_of, open, tip_of, tip_here}`. Manche: quatro setas-alvo à volta de cada
  stick (uma por metade). Tecla física sem escuta liga à acção aberta só se ela não tem ligação, nenhum campo
  focado e o Play parado («or press one»). Linha do hover sempre reservada (idle: «Point at a key…»). Plural =
  chaves `_one`/`_many`. ABERTO: marcador ao vivo da manche na barra do curso (precisa do `GamepadState`).
- **Tela do jogador:** crate `ph2d-player-controls` (`paint`, `rows`, `ControlsScreen`, `PlaySession`,
  `player_file(base, game_id)`, `home_base()`, `file::{load, save}`, `SLOTS = 2`). Estado em
  `ComponentsShell.jogador`; `jogador_entra/jogador_sai` em `undo_jogo.rs`; acessor
  `input_dispatch::controles_do_jogador::play_map(&PlaySession, &HeroScreen)` nos quatro leitores; porta da
  tecla `keyboard.rs:108` (antes de `tecla_do_jogo` :124), do clique `input_dispatch.rs:414`; pausa por
  `playhead.pause()/play()`. Com «Players can change keys» desligado, o ficheiro do jogador é ignorado no Play.
  ABERTO: `ph2d_app_physics::player_input::resolve_player_input` ficou sem chamador; a tela não rola; tecla
  repetida sem aviso; edição do autor durante o Play só entra na próxima troca do jogador. Shell: 35 linhas de folga.

## 13 · Onda 2 — contratos (teclado de notebook + polimento pro)

Ordem do dono e foto: [prompt da onda 2](handoffs/PROMPT_MISSAO_InputMap_ONDA2_2026-11-10.md) §2 ·
[foto](maquetes/06_teclado_notebook_dono.png).

### 13.1 · W1 — códigos de tecla (`ph2d-input/src/keyboard.rs`)

Tecla FÍSICA (o `KeyCode` do winit, nunca a lógica): o numérico liga igual com o Num Lock aceso ou apagado.

| Grupo | Códigos | Rótulo (`label`) | Nome de script |
|---|---|---|---|
| Pontuação (ASCII) | `0x60` `` ` `` · `0x2D` `-` · `0x3D` `=` · `0x5C` `\` · `0x3B` `;` · `0x27` `'` · `0x2F` `/` | o próprio carácter | o próprio carácter |
| Navegação (faixa NSEvent) | `0xF727` Insert · `0xF72B` End · `0xF72C` Page Up · `0xF72D` Page Down | `Insert` `End` `Page Up` `Page Down` | `insert` `end` `pageup` `pagedown` |
| Travas | `0xF716` Caps Lock · `0xF717` Num Lock | `Caps Lock` `Num Lock` | `capslock` `numlock` |
| Numérico | `0xF740..=0xF749` kp0–kp9 · `0xF74A` . · `0xF74B` / · `0xF74C` * · `0xF74D` - · `0xF74E` + · `0xF74F` Enter | `Numpad 0`…`Numpad 9`, `Numpad .`, `Numpad /`, `Numpad *`, `Numpad -`, `Numpad +`, `Numpad Enter` | `kp0`…`kp9`, `kp.`, `kp/`, `kp*`, `kp-`, `kp+`, `kpenter` |
| Do SISTEMA (desenhadas, nunca ligam) | `0xF718` Left Windows · `0xF719` Right Windows · `0xF71A` Fn · `0xF72E` Print Screen | `Windows key`, `Fn`, `Print Screen` | **nenhum** (um script nunca as vê) |

- `NumpadEnter` deixa de cair no Enter (`0x0D`): Numpad Enter ≠ Enter, Numpad 7 ≠ 7 ≠ Home.
- **Medido nesta máquina (10/11):** o teclado do dono (`POP Icon Keys`) NÃO declara `KEY_FN` nas capacidades
  evdev (`/sys/class/input/event*/device/capabilities/key`, bit 464) — Fn resolve-se dentro do teclado. O KDE
  prende `Print` ao Spectacle (`org.kde.spectacle.desktop: X-KDE-Shortcuts=Print`) e `Meta` ao lançador
  (`kglobalshortcutsrc: activate application launcher=Meta`). `KEY_NUMLOCK`, `KEY_CAPSLOCK`, `KEY_INSERT` e o
  numérico inteiro chegam. O winit 0.30.13 não dá o ESTADO do Num Lock (`ModifiersState` = Shift/Ctrl/Alt/Super)
  ⇒ **a luz do Num Lock não se desenha** (luz falsa, não).
- `Key::cap()` (a legenda da foto: `Backspace`, `CapsLk`, `Num Lock`, `PrtSc`, `Insert`, `Delete`, `PgUp`…) e
  `Key::cap_short()` (a legenda curta dos teclados compactos: `Bksp`, `Caps`, `Num`, `Ins`, `Del`…) moram ao lado
  da `NAMED`; o pintor escolhe a que cabe, medida, por tecla. `Key::system_owned() -> Option<&'static str>`
  devolve a chave i18n do porquê (`input_map_keyboard.system.{fn,windows,print_screen}`).
- A tradução winit → `Key` SAI da shell (paga as linhas novas: catraca `the_shell_only_shrinks`).

### 13.2 · W2 — o desenho do teclado de notebook (`view_keyboard/geometry.rs` + `keys*.rs`)

- **Tabela de dados em unidades de tecla** (`KeyCap { key, col, row, span, rows }`; `rows = 2` no `+` e no Enter do
  numérico). Bloco principal ANSI de 15 u: R0 fila de função (Esc F1–F12 Insert PrtSc Delete = 16 teclas sobre 15 u)
  · R1 `` ` ``1–0 `-` `=` Backspace 2 u · R2 Tab 1,5 Q–P `[` `]` `\` 1,5 · R3 CapsLk 1,75 A–L `;` `'` Enter 2,25 · R4
  Shift 2,25 Z–M `,` `.` `/` Shift 2,75 · R5 Ctrl 1,25 Fn Win Alt 1,25 Space 5 Alt 1,25 Ctrl 1,25 e ↑ na col. 13 ·
  R6 ← ↓ → nas col. 12–14 (o T invertido sai por baixo, como na foto). **Bloco do numérico** (4 colunas de 1 u, a
  0,25 u do principal): R0 Home End PgUp PgDn (a foto alinha-as às colunas do numérico) · R1 Num Lock `/` `*` `-` ·
  R2 7 8 9 `+`(2 filas) · R3 4 5 6 · R4 1 2 3 Enter(2 filas) · R5 0 (2 u) `.`. Desvio consciente: a foto tem o
  numérico a ~0,82 u por coluna; aqui 1 u (o mesmo desenho, legendas legíveis).
- **Legenda por tecla, medida:** a da foto (`Key::cap`) → a curta (`cap_short`) → o símbolo ISO/IEC 9995-7 (⇱ Home,
  ⇲ End, ⇞ PgUp, ⇟ PgDn, ↵ Enter, ⌫ Backspace, ⇥ Tab, ⇪ Caps Lock, ⇧ Shift, ⎙ Print Screen) pintado como ícone
  vectorial (a fonte não tem os glifos). Glifo único a `VALUE_PX`, palavra a `LABEL_PX` (o menor texto do cartão,
  legível no modo de fábrica `CrispHeavyPlus`); ⛔ nada menor, ⛔ nunca elidir.
- **Um arranjo só, passo mínimo medido (dono 10/11):** o numérico fica SEMPRE ao lado do principal (a foto) —
  nunca desce. O passo da tecla não passa abaixo de `PITCH_FLOOR` = 31,5 px, medido por `measured_floor` nos dois
  modos de texto (todo grupo com legenda que cabe e ≥ 2 px de barra à vista de cada lado da plaquinha; a tecla com
  `KEY_MIN_H` e a proporção dela). Acima, o teclado enche a caixa; abaixo, fica nesse passo, recortado ao campo, e
  rola na HORIZONTAL (polegar por baixo, Shift+roda, trackpad de lado, o Tab traz a tecla à vista), na B e na C.
  Nunca esconder tecla. A escolha é uma função pura da caixa e do texto (`teclado_piso.rs`).
- **Teclas do sistema** (`Key::system_owned`): apagadas, sem alvo de clique que ligue, o hover diz o porquê (a
  frase da B) e o cursor fica a seta. Num Lock desenha-se sem luz.
- B e C usam o MESMO pintor. «Also:» só para o que não está no desenho.

### 13.3 · W3–W5 — o polimento pro (a infraestrutura primeiro, as vistas depois)

- **Um registo de controlos por pintura** (`ph2d-input-map-editor/src/interact.rs`): `doc.control(Control { id,
  rect, role, label, look, disabled_why, focusable })` regista o alvo de clique E o controlo — pintou ⟹ clicável,
  focável, com nó de acessibilidade e cursor. `ControlRole` → o papel accesskit. Daí saem `is_hovered`,
  `is_pressed`, `is_focused`, `look_at(x,y)`, a ordem do Tab e `a11y_nodes`.
- **Premido nativo:** o Down captura; o botão/tecla/linha/chip dispara no Up DENTRO; arrastar para fora apaga o
  premido e cancela. Gestos de arrasto (alças Dead/Press, polegares, scrub) continuam a começar no Down.
- **Foco:** Tab / Shift+Tab percorrem os controlos focáveis pela ordem da pintura; Enter/Space activam; Esc larga.
  Um só anel: `ColorToken::FocusRing` (helper único).
- **Hover = D32 do dono (02/11): o fundo acende e apaga NO QUADRO, sem fade** (`docs/CartoesUI/01_decisoes_do_dono.md`;
  o «fade do hover» do prompt da onda 2 é a coisa que o dono recusou ao vê-la). Só o polegar da barra desvanece
  (`Role::Fade`, como `screens/hero/live.rs::tick_hover`). A aba passa a alimentar o `hover_live` do seu
  `WidgetStore` (sem isso, um widget do cartão sem registo lê `SETTLED = 1.0` e pinta-se SEMPRE em hover).
- **Movimento só pelo `UiMotion` do `hero`** (a família anima no `frame`, a pintura lê os valores; `wants_frame`
  enquanto há voo): o pulso da escuta tracejada é `Role::Decoration` (ausente em Discreto e no reduced motion, como
  a lei manda); o caret pisca pelo relógio do substrato (`UiMotion::elapsed`) e fica fixo com reduced motion. Os
  medidores do Try it são `Role::Number` (um valor lido não balança).

