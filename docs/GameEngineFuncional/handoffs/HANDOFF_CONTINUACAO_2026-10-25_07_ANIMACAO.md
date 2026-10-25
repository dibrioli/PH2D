# Handoff de continuação — `line/GameEngineFuncional`, plano 07 (a animação num cartão e o editor) (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`; HEAD antes deste doc
`419cfedd49`. **Não integrada, não empurrada.** Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-25_N1_ARTE.md`](HANDOFF_CONTINUACAO_2026-10-25_N1_ARTE.md). Plano:
[`07_animacao_num_cartao_plano.md`](../07_animacao_num_cartao_plano.md). Outros: [`06_arte_vetorial_plano.md`](../06_arte_vetorial_plano.md).

## Estado

Plano 07 feito e fotografado no app real (Xvfb + xdotool + `import`, método
`docs/CodeEditor/ferramentas/olha_cena.sh`; fotos `10_*.png` no scratchpad, não versionadas). **Smoke do dono
ainda por fazer** (passos abaixo).

- **Cartão «Animation» único** (`crates/ph2d-panel-inspector/src/sections/anim_card.rs`): preview; lista com
  miniatura/quadros/▶; Add/Remove; «Edit Animations…»; quadros H/V + «Show Sheet»; «Who Picks: Me · Script ·
  The Character» (Me = `SpriteAnimator.autoplay` ligado; Script = desligado, a dica mostra
  `ph2d.set(self.id,"animator.animation","run")` + `ph2d.set(self.id,"animator.playing",true)`; The Character =
  `CharacterAnimations` anexado, só com `PlatformPlayer`); quatro menus de estado a partir da lista de tags
  (renomear acompanha); Run From; Flip; dobra «More» fechada com os controlos antigos do tocador e os campos
  por animação. **Sumiram** a secção «Sprite Sheet» e o bloco «Auto Animate» do Platform Player (centro/offset/
  flip foram para o fim de Render Source).
- **Editor de animações** = `DocumentSurface` em `crates/ph2d-app-sprite-anim/src/editor/` (uma aba por sprite,
  «Animations — <nome>»): lista à esquerda (escolher/adicionar/remover/renomear); grelha da folha ao meio
  (clicar/arrastar células acrescenta ao Frames); faixa de quadros em baixo (arrastar reordena, ×/Delete,
  ms por quadro); barra de cima ▶ · fps · Repeat/Once/Ping-pong · cols · rows; faixa de proposta de grelha via
  `guess_grid`. Largar imagem num sprite, ou criar um a partir de imagem, abre o editor com a proposta
  (`shells/desktop/src/asset_drop_apply.rs` → `editor::propose_after_drop`). Pixels da folha:
  `crates/ph2d-app-sprite-anim/src/sheet.rs` (`factory_pixels` / `SpritePixels` / `AssetDb`, LRU 16, palpite no
  decode). Layout em `editor/layout.rs`.
- **Dados:** componentes NOVOS `ph2d::ecs::SpriteAnimFrames` (por NOME de tag, a lista de células) e
  `ph2d::ecs::SpriteAnimCursor` (o passo) em `crates/ph2d-ecs/src/sprite_anim_frames.rs` — IRMÃOS, não campos
  (postcard posicional) ⇒ **sem bump de `PROJECT_SCHEMA`**; ficheiros antigos abrem idênticos. O motor anda em
  PASSOS (`advance_span`, `advance_frames`, `entry_cell`, `tag_cells`, `rename_tag`, `fps_of_ms`/`ms_of_fps`).
  Modo do ciclo: Repeat = `repeat None`+`Forward` · Once = `Some(1)`+`Forward` · Ping-pong = `PingPong`.
- **Grelha proposta:** `crates/ph2d-sprite-sheet/src/grid_guess.rs` (corridas de ocupação + o MENOR divisor
  ≥ corridas que não parte figura). Gate `a_folha_do_heroi_propoe_a_sua_grelha` = 8×4.
- **Duas premissas do plano REFUTADAS por medição:** «o dado não muda» (tags são intervalos ⇒ lista irmã) e
  «a folha do herói tem calhas» (não tem; o plano diz «maior divisor», o código é o menor ≥ corridas).

## Commits da onda (base `6bae4ec8ad`, 15 + este handoff)

| sha | O quê |
|---|---|
| `c72c841344` | `SpriteAnimFrames`/`SpriteAnimCursor` + `grid_guess` + gate do herói + plano |
| `ddf10f175a` | a animação de sprite sai da shell → crate `ph2d-app-sprite-anim` |
| `52db3f3f17` | contrato do cartão: `AnimCardInfo`/`AnimChooser`/`AnimCycleMode`, edições novas |
| `73ceee3a5a` | `paint_sheet_cell` (caixa `StableImage`) + pedido `OpenEditor` |
| `6958ca8227` | barra da §11 anda em passos (`scrub_edit` = `SetStep`) |
| `ff878cb2ad` | dois gates da shell seguem a animação para a crate nova |
| `b84e60b050` | o editor (DocumentSurface) |
| `87ba4b74f4` | a shell instala o editor; `OpenEditor` abre a aba; largar folha propõe grelha |
| `f734c2abbc` | F1: o cartão Animation (sem Sprite Sheet/Auto Animate) |
| `3204ec400d` `2e435f1caf` | fmt do teste do modelo · portões do painel |
| `d2c1b73484` | F1: braços do cartão saem de `apply_anim_event` (tecto 200); i18n/lib.rs volta a zero |
| `6a10939765` | fecho da onda: fmt, clippy, tectos de linhas, pontas da junção |
| `6d2f93162e` | dica do modo Script diz as duas chamadas |
| `b298611fca` | smoke: barra/faixa medem o texto, «+» põe a nova no ecrã, Hierarquia não conta recursos |
| `419cfedd49` | smoke: cartão não corta nome da grelha/«Who Picks» e segue a animação do editor |

## Superfície que um merge pode partir (colisão)

- **Crate NOVA** `ph2d-app-sprite-anim` + `Cargo.lock`; dep em `shells/desktop/Cargo.toml` (conflito provável).
- **Contadores de registo (DELTA +2; o integrador CONTA, não escolhe):** ECS 108→110, render 109→111,
  script 109→111 (`crates/ph2d-render/src/registry.rs`, `crates/ph2d-script/src/registry.rs`, testes do
  registo em `ph2d-ecs`). Sem bump de schema. Novo `SimWorld::component_count` (`ph2d-ecs/src/sim.rs`): no
  bevy 0.19 recursos são entidades com 2 componentes; o cabeçalho da Hierarquia conta só componentes de jogo.
- **Código MOVIDO da shell:** `render_loop/inspector_anim.rs` (+testes +transport) → `…/src/inspector.rs`;
  `sprite_anim_tick.rs` (+signal tests) → `…/src/tick.rs`. A shell mantém
  `pub(crate) use ph2d_app_sprite_anim::tick as sprite_anim_tick`. Outra linha que edite esses ficheiros conflita.
  Gates que leem caminhos já atualizados: `the_draw_pass_asks_the_door_that_has_the_gates` (caminho do tick),
  `every_physics_component_is_authorable` (o escritor agora é `card.rs`).
- **REMOVIDOS:** `PlayerFieldEdit::Anim*`, `InspectorCharacterAnimInfo`, `sections/player_anim.rs`,
  `ids/inspector_player_anim.rs`, `ph2d-app-physics` `inspector/player_anim.rs`, a secção Sprite Sheet. Outra
  linha a editar `player.rs` ou `sprite_sheet.rs` conflita. `INSP_LIVE_SHEET_SECTION` ficou em `LIVE_SECTIONS`
  (estabilidade posicional das notas) mas nunca é pintada; `reveal.rs` ainda a mapeia a `SpriteGrid` (inerte).
- **NOVAS edições** `AnimFieldEdit::{Frames, CycleMode, Chooser, CharacterState, CharacterRunFrom,
  CharacterFlip, SetStep, Grid, OpenEditor}`. Modelo: `AnimCardInfo`/`AnimChooser`/`AnimCycleMode`/
  `SheetImage(StableImage)`/`row_cells` (submódulo de `inspector_model_anim.rs`); `widget::sheet_cell::paint_sheet_cell`.
- **i18n:** `ph2d-i18n/src/lib.rs` (+1 linha `mod`, +1 `or_else`; no tecto de 700) — `anim_editor.rs` é submódulo
  de `anim_card.rs`. `hero.rs` no tecto de 700. `mode_drive_tests.rs` partido (→ `mode_drive_join_tests.rs`,
  760→572; excesso pré-existente da W1).

## Prova de fecho (DELTA)

Testes por nome, verdes: `ph2d-ecs` sprite_anim 29 · registry 6; `ph2d-sprite-sheet` grid_guess 4;
`ph2d-render` a_folha_do_heroi_propoe 1; `ph2d-app-sprite-anim` crate inteira (nova) 54+ (card_tests 7,
editor 15+); `ph2d-panel-inspector` seam_anim 22 (seam_anim_card 11) · seam_player 22; editor-core
inspector_model_anim 12 · mode_drive 19 · architecture_workspace_file_loc_cap · the_shell_only_shrinks ·
architecture_panel_wiring_parity · architecture_every_editable_field_is_a_channel · architecture_panel_loc_cap;
`ph2d-panel-registry-init` o_inspector_armado; `ph2d-i18n` a_tabela_inglesa_fala_ingles; `app-physics` player;
`app-game` personagem, o_nivel_2; desktop every_physics_component_is_authorable, a_schema_bump_anywhere,
the_animation_tick_asks, the_panel_names_the_directions, preview_drive 12. `clippy -D warnings` limpo em todas
as crates tocadas; `fmt --all`. **`nextest-impacted` NÃO foi corrido** (regra do dono: suíte completa só antes
da integração) — **o integrador TEM de a correr**. Smoke build, 2.ª corrida: `Finished smoke in 0.23s`, zero
`Compiling`.

## Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=2 cargo run --profile smoke -p ph2d-host-desktop
```

Para o dono (nomes em inglês, como na tela):
1. Corra o comando acima; selecione **Player** na Hierarquia.
2. No Inspector, ache o cartão **Animation**; clique ▶ em **run** (o boneco corre no preview).
3. Clique **Edit Animations…** (abre a aba «Animations — Player»); **+ Add**; clique 3 células da folha; mude
   o fps; escolha **Once**; ▶. Tem de tocar só 1 vez e parar.
4. Volte ao cartão: **Who Picks ▸ The Character**; no menu de **Run** escolha **anim_0**; **Play in a Window**;
   ande. Errado se: o boneco não usa a anim_0 ao correr, ou a aba do editor não abre.

## ABERTO

- Tooltip de «Edit Animations…» cortado na borda da janela (pintor de tooltips, pré-existente).
- Rótulo «Show sheet on canvas» encurtado para «Show Sheet» (+ tooltip) — decisão de vocabulário do dono.
- Controlos do editor não estão no `WidgetStore` (testados pelas portas do `DocumentSurface`, não por
  `ph2d-ui-testkit`).
- A câmara do nível 2 não abriu sobre o herói num Xvfb 1600×900 (herói em x −800) — conferir.
- `AnimFieldEdit::Autoplay` ficou sem gesto no painel (o `Chooser` escreve o autoplay).
- Da linha: R1 vista dupla ao vivo · View ▸ Focus mode morto · sem `ph2d.disconnect` · resto da arte (plano 06) ·
  fecho da linha (DIRETRIZ §1.5.9) e a bateria do integrador.

## Rodada 2 — report do dono (25/10)

Pedido: pausa no cartão; faixa do editor maior que as células e retorno visual do arrasto; arrumação do
«More». Commits: `010be94189` (editor: miniatura = 1,25 × a célula pintada, limitada a metade da altura
partilhada; `drop_gap(x)` é a lei única do marcador e da largada; fantasma + origem escurecida; gates
`a_miniatura_da_faixa_e_maior_que_a_celula_da_folha`, `o_marcador_pintado_e_onde_a_largada_poe_o_quadro`,
`o_fantasma_segue_a_mao_e_some_ao_largar`), `cf51799653` (cartão: ⏸ na linha que toca; grupos
Reprodução sempre visível · a animação escolhida (From/To só sem lista; Frames/s; Repeat/Once/Ping-pong)
· Advanced fechado; 7 seams novos, 30 no total), e o «tempo próprio» = valor ≠ `frame_ms` (gate
`um_tempo_igual_ao_ritmo_nao_e_tempo_proprio`) + `smoke_hover` nomeado no censo de NodeId. Fotos
`fotos/20_*`, `fotos/21_*` (scratchpad, não versionadas). Aberto: a faixa não rola sozinha ao arrastar
junto à borda (só a roda); o fantasma usa o encolher/opacidade partilhados do app.

## Rodada 3 — report do dono (25/10)

`10fe2efbaa`: um clique real treme 1–2 px entre premir e soltar e o despacho lia-o como arrasto de selecção de
texto (selecção encolhia; «7» sobre «250» dava 7250) — a cura é no despacho partilhado: premir num campo
numérico ou mantém a selecção do clique ou, passado o limiar, arrasta o valor; nunca selecciona texto por
arrasto (gate `a_trembling_click_still_selects_the_whole_number`). `713de64ba8`: «This frame» pinta-se SEMPRE
com animação a tocar; sem tempo próprio mostra o ritmo esbatido e continua editável (seam
`seam_anim_this_frame.rs`, sem salto de layout). Aberto: o 2.º clique num campo já em edição põe o cursor
(digitar insere) — comportamento antigo.

## PRÓXIMO — plano 08: a LISTA DE REGRAS (decisão do dono 25/10)

Os 4 estados fixos de «The Character» + o resto por script dividem o mesmo conceito. Aprovado: no cartão
Animation, uma lista de regras «Quando ___ → toca ___» (os dois lados em MENUS); escolher «The Character»
pré-preenche as 4 de hoje (parado→idle, a correr→run, a subir→jump, a cair→fall); «+ Regra» para o resto; a de
cima ganha, arrastar reordena; o script continua para o difícil. Condições da 1.ª versão: estados do personagem,
teclas (ex. ↓ premida), mensagens/sinais de outros objectos (ex. «levou dano»). «Abaixar» com corpo baixo é peça
à parte. Referência de memória: Construct 3 / GDevelop (regras com condições do comportamento). MEÇA primeiro:
o `CharacterAnimations` (ph2d-physics-ecs) e quem o lê (a ponte do jogo em ph2d-app-game `personagem.rs`), o
mapa de teclas (input_map), os sinais (`ph2d-runtime`) — e como um ficheiro antigo com `CharacterAnimations`
abre como as 4 regras (sem degrau de schema, se possível: componente irmão). Escrever
`docs/GameEngineFuncional/08_regras_de_animacao_plano.md` antes de codar.
