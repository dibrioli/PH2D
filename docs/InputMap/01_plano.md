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
