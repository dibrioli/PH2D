# Handoff de continuação — GEF 17b: a câmera no canvas, curada (28/10)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · merge-base com o main `c3662cd8e4`
(o main não andou). Auditoria e plano: [`17b_auditoria_camera_e_plano.md`](../17b_auditoria_camera_e_plano.md).
Antecessor: [`HANDOFF_CONTINUACAO_2026-10-27_17_CAMERA.md`](HANDOFF_CONTINUACAO_2026-10-27_17_CAMERA.md).

## Estado

**Smoke do dono (17b): POR FAZER.** Plano §«Plano da cura» passos 1–9 feitos; o 10 é a onda 18 (já decidido).
Commits: ver `git log c3662cd8e4..` (o vermelho, a cura, o fecho e este handoff).

## Vermelhos vistos ANTES da cura (cena 4 real, `camera_2d::update` a correr, sem bool à mão)

`crates/ph2d-app-game/src/moldura_ponteiro_tests.rs` — corridos sobre o código da onda 17 (só com `edicoes` extraída
da shell, mesma semântica), 5/5 vermelhos:

| Gate | Achado | O que o vermelho leu |
|---|---|---|
| `o_miolo_agarra_com_o_ceu_por_baixo` | A1 | `alca_em(.., outro_por_baixo)` → `None` com o céu por baixo |
| `com_follow_arrastar_o_miolo_leva_a_moldura` | A2 | moldura em (−5,89; 3,00) antes e depois; a lei diz (−2; 3) |
| `as_alcas_dos_limites_chegam_inteiras_ao_mundo` | A3 | `LimiteCanto(0)`: mundo `([-13,-1],[17,9])`, arrasto `([-14.5,-1.75],..)` |
| `o_canto_com_limites_cai_no_ponteiro` | A7 | canto em x 7,22 com o ponteiro em 4,22 |
| `os_dois_offset_tem_nomes_distintos` | A6 | `"Offset" == "Offset"` |

⚠️ **«Pelo ponteiro (`App::pointer_button`)» num teste não existe:** o `App` headless tem `gfx = None` e toda porta da
câmera devolve cedo. A prova pelo ponteiro é a do **app real** (abaixo): as fotos-gesto entram por
`smoke_pointer_down/move/up` → `on_mouse_input` → `pointer_button`, com o céu e o `follow` vivos.

## O que entrou

- **Lei (`ph2d-app-game::moldura`)**: `alca_em` sem `outro_por_baixo`; o miolo é **clique pendente** — `Arrasto` nasce
  por armar (`Dentro`), `arma(ecra)` com `DRAG_THRESHOLD_PX` (4, o do editor), `era_um_clique()` no `Up`. Com `follow`
  o miolo escreve `Escrita::FollowOffset`; sem ele, `Pose`. `edicoes(e) -> Option<CameraFieldEdit>`: **uma** edição
  (`Limits(min,max)` novo — A3). O canto (`altura_do_canto`) resolve por eixo a menor altura cujo canto chega ao
  ponteiro com o centro que a CERCA dá a essa altura (posição do canto não-decrescente em h: livre / preso do lado /
  preso do outro lado / meio da caixa) — bisseção de 40 passos; o eixo que manda ganha. `Moldura` ganhou `mira`
  (antes da cerca), `offset` (da vista) e `segue: Option<Segue>`; as não-activas saem da MESMA lei
  (`centro_da_lei`); câmeras com `Visibility.hidden` não entram. `CameraNoCanvas` ganhou `em_edicao` e `vista_do_down`.
- **Porta única da cerca** `ph2d_app_components::camera_2d::enquadra` (o passe e a moldura); `mira_parada` e
  `alvo_do_follow` (o mesmo `follow_target` do passe).
- **Tinta (`moldura_tinta`)**: `Pintura { mostrar, unidade, ppm, icone_px, em_edicao, canvas, ponteiro }`; ícone com o
  lado do projecto (44 por omissão = 2 × 22); rótulo empurrado para dentro do `viewport` (medido com
  `prefix_width`); alça sob o ponteiro maior e cheia; com `follow`: fio tracejado ícone→alvo, «follows <alvo>» e a
  cruz da mira (alvo + offset do `follow`); fora de edição: moldura ténue, sem rótulo nem alças. `icones_em` = o dedo
  (a mesma conta da tinta), por cima de tudo no `pick_objects_at` como a lâmpada.
- **A câmera deixou de ser «vazio»** (`publishes_its_own_handles` + `GameCamera`): sem anel de 24 px de arte, sem caixa
  do gizmo genérico; o skip da shell em `empty_object_overlay` saiu.
- **Shell (`camera_no_canvas.rs`)**: `Down` recusa fora de edição e com modificador sobre o miolo; guarda
  `vista_do_down` (A5: o `Move` converte pela vista do `Down`); o `Up` de um miolo por armar escolhe o de cima sob o
  ponto do `Down` (sem a câmera; `descendants_first`) por `mode_drive::chosen` + `replace_selection`; sem nenhum, a
  câmera fica. `camera_frame_cursor` (setas nos cantos/bordas, `Move` no miolo) na cadeia do cursor.
  `em_edicao = !playhead.is_playing() && playhead.time() <= 0` em `fase_game_camera` (o invariante do rebobinar).
- **Projecto**: `ProjectSettings::camera_icon_px` (faixa 16–128, `CAMERA_ICON_*` em `game_format.rs`); blob `jogo`
  **v2** `(v, preset, custom, moldura, icone_px)`, lê a v1 com o ícone de fábrica; o clamp passa por
  `aplica_formato(IconSize)`. Sem degrau de `PROJECT_SCHEMA` (o blob tem versão própria).
- **Inspector**: fileira «Icon Size» (px, passo 4) depois de «Frame»; `CameraFieldEdit::IconSize`; `icon_px` no
  `InspectorCameraInfo` e `build_camera_info`. «Offset» → **«View Offset»** (câmera) e **«Follow Offset»**.
- **Hierarchy**: `hier_rows::type_icon(kind, categorias)` — tipo do objecto; num vazio a Câmera, depois a Luz; vazio
  sem nada = `Cube`. Posto em `snapshots_hierarchy` com `component_attach::kind_of`.
- **Fotos-gesto** (`smoke_mao.rs`, `PH2D_GAME_SMOKE_FOTO=camera_arrasta|camera_clique|camera_borda|camera_canto|
  camera_olhar_arrasta`): a mão desce no quadro 40, anda 6 quadros, sobe; converte pela `vista_do_down` (a mão anda
  na TELA); o `Up` imprime `[camera-mao]`.

## Decidido e recusado

- **Selo «segue <alvo>» na Hierarchy: recusado.** O selo é um código de 3 letras que é também a chave da cor
  (`badge_tone`, memória do selo-texto); um nome não cabe. A relação vê-se no canvas (fio + nome).
- **Arrastar com `follow` move o offset do `follow`**, não o da vista: com a cerca a morder a moldura fica na cerca e
  só a cruz da mira segue o ponteiro — é a verdade do jogo, e a cruz é o que o diz.

## Provas pelo app real (`PH2D_GAME_SMOKE=4`, 1930 × 1040, `fotos_17b/`)

| Foto | `[camera-mao]` | Lê-se na foto |
|---|---|---|
| `nivel_4_camera` | — | ícone 44 px, ícone de câmera na linha «Camera», cruz da mira, «follows Player», rótulo dentro |
| `nivel_4_camera_arrasta` | `follow_offset=[6.0, 1.0]`, seleção Camera | moldura em x −2, cruz levada |
| `nivel_4_camera_olhar_arrasta` | `follow_offset=[6.0, 1.0]` com «Look through» | (A5) a mesma escrita |
| `nivel_4_camera_clique` | `selecao=Sky` | o clique sem mexer foi para o céu |
| `nivel_4_camera_borda` | `limites=([-16,-1],[17,9])` | A3: só a borda esquerda andou |
| `nivel_4_camera_canto` | `altura=9.688` | A7: borda direita no ponteiro (x 4,22), presa à esquerda |
| `nivel_4_camera_jogo` (`DRIVE=editor`) | — | a correr no editor: sem rótulo nem alças |

## Gates e fecho

Novos: `moldura_ponteiro_tests` (7), `moldura_tests` (refeito: o canto livre, o canto com cerca, o miolo só depois do
limiar), `smoke_mao::a_mao_desce_anda_e_sobe`, `formato_do_jogo` (+IconSize), `hier_rows::o_icone_da_linha_e_o_do_tipo`,
`the_camera_section_is_alive::o_icone_e_os_dois_offset_chegam_ao_barramento`, `um_jogo_v1_abre_com_o_icone_de_fabrica`,
`a_joint_publishes_no_box…` (+câmera).

**Fecho (por nome, agente `verificador`)**: verdes — `moldura` 13, `formato_do_jogo`, `smoke_mao`,
`a_moldura_e_o_rectangulo`, `camera` (components) 45, `o_icone_da_linha_e_o_do_tipo`, `game_format` 2,
`architecture_workspace_file_loc_cap` 2, `arch_safe_clamp_only` 2, `the_shell_only_shrinks`, `hr15_no_hardcoded_ui_strings`,
`no_magic_numeric` 3, `todo_aponte_passa_pela_janela_da_cena` 6, `the_camera_section_is_alive` 8,
`every_id_the_inspector_paints_can_actually_be_clicked`, `two_sections_never_stack` 2, `a_faixa_media_do_dock`,
`os_grupos_dos_cartoes` 3, `o_icone_da_hierarquia_traz_o_cartao` 3, `o_inspector_armado` 3, `a_tabela_inglesa_fala_ingles` 4,
shell `jogo` 12, `every_setting_survives`, `the_load_installs_the_project_settings`, `file_loc_caps` 4, `fn_loc_caps` 2,
`the_app_only_sheds_fields` 3, `every_door_is_whole_and_every_ramo_is_called`, `the_mouse_and_the_pen_end_in_the_same_pointer_doors`,
`o_hud_conduz_a_raiz` 3, `luz` 3, `a_schema_bump_anywhere`. **Clippy** `-D warnings` limpo nas 6 crates
(`PH2D_SUITE=fecho`); **fmt** limpo. `every_editable_field_is_a_channel`: as fileiras do formato/moldura/ícone (17 e 17b)
classificadas `NaoAnimavel` (definição do projecto) + o ponto do decorator no seletor do formato; a isenção órfã
`anim_card.rs::character_rows` saiu (a função mudou para `anim_rules.rs` com chaves literais). **Fica 1 vermelho
herdado de onda anterior desta linha**: `todo_campo_sem_coluna_propria_ganha_a_do_losango` —
`sections/anim_rules.rs · fn rule_rows` pinta `paint_dropdown_chip` sem a coluna (a lista de regras não tem coluna de
decorator; o gate não tem tabela de isenção para isto).

**Mutação (agente `mutacao`, 16)**: 15 sangraram — Dentro recusado · Follow→Pose · `Limits`→`LimitMax` · canto sem
cerca · miolo nasce armado · sem filtro de `Visibility` · ícone de lado dobrado · câmera volta a «vazio» · `type_icon`
sempre Cube · sem a leitura da v1 · sem o braço `IconSize` · «Follow Offset»=«View Offset» · `Limits` só escreve o min
· `IconSize` sem clamp · e (app real) o `Move` pela vista viva com «Look through» ⇒ `follow_offset=[7.89, 1.0]` contra
`[6.0, 1.0]`. A 15.ª (`enquadra` sem cerca) sangrou em `com_follow…` e no passe (`na_borda_do_nivel_a_vista_ainda_treme`,
`the_limits_hold_the_window…`), e passou em `o_canto_com_limites_cai_no_ponteiro` — **por desenho**: a moldura e a
altura do canto leem a MESMA porta, logo sem cerca continuam coerentes; a cerca no canto é provada por
`o_canto_com_cerca_cai_no_ponteiro` (lei própria no teste). Binário smoke compilado 2× no fim.

## ABERTO

- Smoke do dono da 17b.
- O ícone (44 px) cobre o Player na cena 4 (câmera em (−8; 2), Player em (−8; 1,5)) e ganha o clique, como a
  lâmpada: o Player escolhe-se pela Hierarchy ou com o ícone menor («Icon Size»).
- O arrasto não grava chave com Auto Key; mudar o formato/ícone não tem desfazer (`ProjectSettings` fora do
  `ProjectState`) — herdados da 17.
- Câmera filha de outro objecto: o miolo soma o delta à translação LOCAL (pai rodado/escalado erra) — herdado.
- Onda 18: zona morta pintada, miniatura, escurecer fora; «a vista do editor segue a câmera do jogo» como opção.
- Os 13 vermelhos herdados da suíte `it` da shell (lista no handoff 17).
