# Handoff de continuação — GEF 17: a câmera no canvas, o formato do jogo no projecto (27/10)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · base `8eb998edfe` (merge-base com
o main `c3662cd8e4`, o main não andou). Plano: [`17_camera_no_canvas_plano.md`](../17_camera_no_canvas_plano.md).
Antecessor: [`HANDOFF_CONTINUACAO_2026-10-27_16_REGION.md`](HANDOFF_CONTINUACAO_2026-10-27_16_REGION.md).

## Estado

**Smoke do dono (onda 17): POR FAZER.** Commits: `0463109083` (feat: A1–A4, D-F, D-V) · `3712b167a8` (fecho: os
tectos que a onda furou, o portão do load do formato, clippy, fmt) · este handoff. Onda 17 = A1–A4 + D-F (formato do
jogo no projecto) + D-V (moldura `Always` / `When Selected`). **A5–A7 = onda 18.** Fotos do app real em
[`fotos_17/`](fotos_17/).

## Medido antes de decidir (§5.0)

- **Onde gravar o formato.** As settings do projecto (`ProjectSettings`, editor-core) gravam-se em `SavedSettings`,
  que fica no MEIO do `ProjectFile` (postcard posicional) ⇒ mexer nele exigiria tipo congelado. Escolha: blob
  `jogo: Vec<u8>` com versão própria (`JOGO_VERSAO = 1`: versão, preset `u8`, custom `[u32; 2]`, moldura `u8`) no
  FIM do `ProjectFile`, precedente `marcas_do_artista`.
- `encaixe::aspecto` usava o aspecto do CANVAS (a moldura mentiria) e a janela do jogo abria 1280×720 fixa.
- O gate `o_hud_conduz_a_raiz…` exigia `aspect_of(surface.size())` em `fase_game_camera`, que já não existia no HEAD
  de partida (vermelho herdado, curado: agora exige `surface.size()` + `caixa_do_jogo()`).
- A shell estava a 187 293 linhas contra tecto 189 041.

## O que entrou

- **Schema:** `PROJECT_SCHEMA 197 → 198`, `PROJECT_SCHEMA_SEM_FORMATO_DO_JOGO = 197`. Braços de leitura: 198…193 num só
  (`falta` = `[]` ou `[0]`), 192/191 com `[0,0]`, 190/189 `read_before_scripts` com `[0,0]` (+ o `[0]` dele). Registos
  0 (a tripla não vê).
- **editor-core `game_format`:** `GamePreset` (15): 16:9 1920×1080, 16:10, 4:3 1440×1080, 3:2 1620×1080, 21:9
  2520×1080, 9:16, iPad 2048×1536 deitado / em pé, Game Boy 160×144, NES 256×240, SNES 256×224, GBA 240×160, Mega
  Drive 320×224, PICO-8 128×128, Custom (clamp 16..8192 = textura wgpu). `GameFormat::aspect` = conta única;
  `CameraFrameShow`.
- **ph2d-app-game:** `encaixe::caixa_do_jogo` (maior rect do aspecto do jogo dentro do canvas), usada em
  `fase_game_camera` (vista do jogo e rect do `DestroyOutside`); `encaixe::tamanho_da_janela` (caixa 1280×900; retro
  por inteiro: GB 960×864, PICO-8 896×896; 9:16 506×900); `formato_do_jogo::aplica_formato`.
- **`moldura.rs`:** `Moldura`; a activa = centro da vista do passe da câmera deste quadro, as outras = pose + offset.
  `Alca` = `Canto`/`Dentro`/`LimiteCanto`/`LimiteBorda`/`LimiteCentro`; `alca_em` só na câmera selecionada, `Dentro`
  só sem outro objecto por baixo. Arrastar: canto = só `height_world` a partir do centro, `max(2|dy|, 2|dx|/aspecto)`,
  clamp do motor; `Dentro` = translação local + delta; limites nunca invertem, vão ≥ `LIMITE_VAO_MIN` (1 cm);
  `AGARRE_PX` 11.
- **`moldura_tinta.rs`:** ícone `IconId::Camera` no lugar do anel do vazio (o anel deixa de pintar câmeras, o pick
  continua no anel); moldura forte selecionada, ténue 0,45 sem seleção, tracejada se não é a activa; rótulo «Nome · W
  × H unidade» (display unit); 4 alças; limites na cor `Warn` com 9 alças; tarjas opacas (`BgScrim` alpha 1) fora da
  caixa no palco.
- **Shell:** `camera_no_canvas.rs` (`camera_frame_down/move/up` ao lado do gizmo do colisor em
  `despacho_clique_flip`/`despacho_mover`; altura e limites pela porta do Inspector `EditorAction::InspectorCameraEdit`;
  pose escrita directa no `Transform`; `tarjas_do_palco`). Estado em `ProjectUndo::camera_no_canvas` (não no `App`:
  `the_app_only_sheds_fields`). `window_size` saiu de `inspector::publish/late`; a janela do jogo abre com o formato.
- **Inspector (secção Game Camera):** dropdown Game Format (15), Size W/H só no Custom, Frame `Always | When
  Selected`; `CameraFieldEdit::{Format(u8), CustomSize, Frame(bool)}`; `build_camera_info` recebe format + frame. i18n
  em `inspector_game.rs`.

## Defeitos que as fotos apanharam

1. O rótulo saía cortado à esquerda quando a moldura encosta à borda do canvas (a vista das fotos passou a (-6, 3)
   15 m; no app real o rótulo ainda pode sair pela esquerda — ABERTO).
2. Os limites saíam da vista (foto limites: (2, 4) 27 m).
3. Em ecrã cheio o formato Game Boy mostrava mundo a mais ⇒ tarjas (foto `play_retro`: 160:144 com faixas pretas).

Fotos (`PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=<nome>`; `camera_retro` também com `PH2D_GAME_SMOKE_DRIVE=fullscreen`,
espera 2): [`nivel_4_camera`](fotos_17/nivel_4_camera.png) · [`_camera_livre`](fotos_17/nivel_4_camera_livre.png) ·
[`_camera_zoom`](fotos_17/nivel_4_camera_zoom.png) · [`_camera_limites`](fotos_17/nivel_4_camera_limites.png) ·
[`_camera_retro`](fotos_17/nivel_4_camera_retro.png) · [`_camera_olhar`](fotos_17/nivel_4_camera_olhar.png) ·
[`nivel_4_play_retro`](fotos_17/nivel_4_play_retro.png).

## Gates (por nome)

Todos verdes.

- `ph2d-app-game`: `a_moldura_e_o_rectangulo_que_o_jogo_mostra` (16:9 e Game Boy, cantos da moldura nos cantos da
  caixa com o Look through) · `o_canto_so_muda_a_altura_e_mantem_o_centro` · `clicar_dentro_da_moldura_nao_muda_a_selecao`
  · `as_alcas_dos_limites_nunca_invertem` · `a_caixa_tem_a_proporcao_do_jogo_e_nao_a_do_canvas` (vista VERMELHA com o
  stub antes da cura) · `a_janela_do_jogo_abre_com_o_formato` (vermelha antes da cura) · `formato_do_jogo`.
- `ph2d-editor-core`: `game_format` (2); `architecture_workspace_file_loc_cap`, `arch_safe_clamp_only` (curado:
  `frame_ms_of_fps` sem `CLAMP-OK` vinha de `cf51799653`, desta linha), `the_shell_only_shrinks`,
  `todo_aponte_passa_pela_janela_da_cena`, `hr15_no_hardcoded_ui_strings`, `no_magic_numeric`.
- shell: `o_formato_do_jogo_atravessa_o_arquivo`, `um_v197_abre_com_o_formato_de_fabrica`,
  `every_setting_survives_the_round_trip`, `the_project_scale_and_unit_survive_the_project_file`,
  `the_load_installs_the_project_settings_from_the_file`, `a_schema_bump_anywhere_must_bump_the_project_schema`
  (198, 13, 23), marcas (2), v128 (4), v95, scripts (4), `o_hud_conduz_a_raiz_antes_de_as_ancoras_a_lerem` (3);
  `file_loc_caps`, `fn_loc_caps`, `the_app_only_sheds_fields` (os três furados pela onda e curados no fecho).
- `ph2d-panel-inspector`: `the_camera_section_is_alive` (7, nova `o_formato_e_a_moldura_chegam_ao_barramento`),
  `every_id_the_inspector_paints_can_actually_be_clicked`. `ph2d-app-components`: `camera_inspector` (7).
  registry-init `o_inspector_armado` (3). i18n `a_tabela_inglesa_fala_ingles`.

## Mutação, clippy, fmt e censos

**Mutação (agente `mutacao`): 13 mutações, 12 sangraram** — `caixa_do_jogo` identidade · activa por pose · aspecto
fixo · canto só `dy` · `Dentro` sem `outro_por_baixo` · sem guarda selecionada · borda sem vão · canto sem vão ·
janela retro sem `floor` · braço 197 sem `[0]` · `install_jogo` ignora bytes (×2 testes) · `Frame` sempre false. A
12.ª (apagar `install_jogo` no load) SOBREVIVEU (`hero_screen` é `None` sem janela) ⇒ arch-gate novo em
`the_load_installs…`, visto verde → vermelho sem a linha → verde. **Clippy** `-D warnings` limpo nas 6 crates
(`PH2D_SUITE=fecho`); **fmt** aplicado. Binário smoke compilado 2× (2.ª: `Finished 0.28s`, zero `Compiling`).

**Vermelhos HERDADOS da suíte `it` da shell** (16 antes; 3 eram da onda e foram curados; 13 herdados, não tocados):
`every_word_the_shell_shows` (`janela_do_jogo.rs:153` «Player», `PH2D_GAME_SMOKE_USOS`),
`the_tree_settles_before_the_capture`, `cada_motor_ouve_antes_de_andar`,
`the_reconstructed_dispatch_still_has_every_region`, `the_grab_is_wired_to_the_pointer`, `the_pose_is_wired_to_the_pointer`
×2, `the_drag_carries_the_jointed_rig` ×2, `the_marquee_shape_comes_from_one_door`, `the_node_miss_picks_another_kind`,
`the_highlight_has_one_source` ×2. **Não verificados contra `8eb998edfe`/main** (o verificador inferiu pelos símbolos
que o diff não toca).

## Smoke (para o Enio)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Na Hierarchy, clique em «Camera»: a moldura da câmera aparece forte no canvas, com o nome e o tamanho.
3. Arraste um canto da moldura: ela cresce ou encolhe (zoom) mantendo o centro. Arraste por dentro: a câmera anda.
4. No Inspector, secção Game Camera: troque Game Format (ex.: Game Boy) — a moldura muda de proporção; Frame =
   When Selected esconde-a quando outra coisa está selecionada.
5. Salve, feche e reabra o projecto: o formato continua. Dê Play: a janela abre na proporção do formato (Game Boy: faixas
   pretas nas bordas).
6. Deu errado se: a moldura não acompanha o formato, o clique dentro dela troca a seleção, o formato volta ao 16:9
   depois de reabrir, ou o rótulo some cortado na borda esquerda (conhecido).

## ABERTO

- Smoke do dono da onda 17.
- O rótulo da moldura pode sair pela esquerda do canvas.
- Arrastar o miolo de uma câmera com Follow move o `Transform`, mas a moldura fica no alvo (é a verdade; pode confundir).
- O arrasto da moldura não grava chave com Auto Key ligado: altura e limites passam pela porta do Inspector (conferir
  se essa porta grava chave); a pose é escrita directa.
- O «Look through» no editor mostra mundo fora da caixa (A7, escurecer = onda 18).
- Câmera filha de outro objecto: o miolo soma o delta à translação LOCAL (pai rodado/escalado erra).
- Os 13 vermelhos herdados acima.
- Os ABERTOS dos handoffs [15](HANDOFF_CONTINUACAO_2026-10-26_15_USO_A_VISTA.md) e
  [16](HANDOFF_CONTINUACAO_2026-10-27_16_REGION.md).

## PRÓXIMO

O smoke do dono decide. Depois a onda 18: A5 zona morta pintada, A6 miniatura, A7 escurecer fora.
