# HANDOFF de INTEGRAÇÃO — `line/render-loop` (A9 + O QUADRO)

> Modo L. Branch `line/render-loop`, worktree `Worktrees/line-render-loop`. **Nada integrado, nada enviado.**
> Leitor: o agente INTEGRADOR (e a próxima LLM que tocar no quadro).
> ⛔⛔ **Leia o §12 antes de tudo:** a CAUDA da obra (as três últimas fases) NÃO correu — o tecto da shell
> (`architecture_the_shell_only_shrinks`) reprovou nesta árvore, e a decisão é do dono.

## §1 · Identidade

| | |
|---|---|
| branch | `line/render-loop` |
| HEAD | `447b30838` (as duas grafias que o `typos` do fecho reprovou) — o handoff entra no commit seguinte; o último commit de CÓDIGO do quadro é `d573402d5` (a preparação da cauda) |
| merge-base com `main` | `e18e75307` |
| commits da linha | 136 (+ o do handoff) |

## §2 · O veredito, em números

| grandeza | antes | depois |
|---|---|---|
| campos da `App` (A9) | 245 | **187** (catraca `the_app_only_sheds_fields`, tecto 187 no nascimento) |
| `run_render_frame` (LOC medido pelo `fn_loc_caps`) | 13 566 | **3 255** (a cauda por fazer: o dreno do barramento ~2 207, o `snapshots::publish` ~194, as vistas do gizmo ~100) |
| `render_loop/mod.rs` (linhas) | 13 922 | **3 869** |
| fases do quadro (`render_loop/fase_*.rs`) | 0 | **122** (121 chamadas no `run_render_frame`; uma fase chama outra) |
| entradas numeradas novas no `FN_OVERAGE_OK` | — | **2** (`fase_audio_panels`, `fase_vector_bands`: cada uma um statement indivisível, com a razão escrita) |
| linhas `.rs` em `shells/desktop` (o tecto `the_shell_only_shrinks` = 190 629) | 187 356 | **192 199** ⛔ (ver §12) |

## §3 · A9 — os campos `vec_*` da `App` agrupados por ASSUNTO

Cura campo a campo (morto apagado; família + tipo de crate → `VecState`; tipo da shell de assunto da família →
o TIPO desce primeiro; composição → grupo da shell com nome de assunto). Nenhum `.clone()` para acalmar o
borrow checker. Catraca nova: `shells/desktop/tests/it/the_app_only_sheds_fields.rs` (as duas metades + a família
vetorial num campo só, prova de mutação).

| campo da `App` (base) | tipo | cura | destino |
|---|---|---|---|
| `vec_history` | `ph2d_vec_edit::History` | MORTO (escrito, nunca lido) | apagado, com o `History` e o `HISTORY_CAP` (commit `dcfa2ec4d`) |
| `vec_blend` | `Option<crate::vec_blend::BlendSession>` | tipo era FACHADA de módulo da crate | `app.vec.blend` |
| `vec_restack` | `Vec<Vec<ph2d_vec_scene::VecPathId>>` | família + tipo de crate | `app.vec.restack` |
| `vec_pen` | `ph2d_vec_edit::PenTool` | família + tipo de crate | `app.vec.pen` |
| `vec_draw_config` | `ph2d_tool_vector::VectorDrawConfig` | família + tipo de crate | `app.vec.draw_config` |
| `vec_pencil_hand` | `crate::vec_pencil_input::PencilHand` | tipo era FACHADA de módulo da crate | `app.vec.pencil_hand` |
| `vec_marquee` | `Option<crate::vec_marquee::VecMarquee>` | tipo era FACHADA de módulo da crate | `app.vec.marquee` |
| `vec_connect` | `Option<ph2d_app_vec::connector_drag::ConnectorDrag>` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::connector_drag::ConnectorDrag`) | `app.vec.connect` |
| `vec_conn_handle` | `Option<ph2d_app_vec::connector_drag::HandleDrag>` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::connector_drag::HandleDrag`) | `app.vec.conn_handle` |
| `vec_connect_pending` | `Option<(ph2d_vec_scene::VecPathId, ph2d_ecs::VecConnector)>` | família + tipo de crate | `app.vec.connect_pending` |
| `vec_connect_sides` | `crate::connector_live::SideCache` | tipo era FACHADA de módulo da crate | `app.vec.connect_sides` |
| `vec_blend_pending` | `Option<(ph2d_vec_scene::VecPathId, ph2d_ecs::VecBlend)>` | família + tipo de crate | `app.vec.blend_pending` |
| `vec_morph_pending` | `Option<(ph2d_vec_scene::VecPathId, ph2d_ecs::VecMorph)>` | família + tipo de crate | `app.vec.morph_pending` |
| `vec_morph_set_pending` | `Option<ph2d_vec_entities::morph_set::MorphSetPending>` | família + tipo de crate | `app.vec.morph_set_pending` |
| `vec_envelope_drag` | `Option<(u64, usize)>` | família + tipo de crate | `app.vec.envelope_drag` |
| `vec_patternpath_handle` | `Option<crate::pattern_live::PatternHandle>` | tipo era FACHADA de módulo da crate | `app.vec.patternpath_handle` |
| `vec_path_pick` | `Option<crate::vec_pick::PathPick>` | tipo era FACHADA de módulo da crate | `app.vec.path_pick` |
| `vec_widget_applied` | `crate::vec_widget_value::Applied` | tipo era FACHADA de módulo da crate | `app.vec.widget_applied` |
| `vec_morph_plans` | `crate::morph_live::MorphPlans` | tipo era FACHADA de módulo da crate | `app.vec.morph_plans` |
| `vec_blend_spines` | `crate::blend_live::BlendSpines` | tipo era FACHADA de módulo da crate | `app.vec.blend_spines` |
| `vec_label_pending` | `Option<ph2d_vec_scene::VecPathId>` | família + tipo de crate | `app.vec.label_pending` |
| `vec_label_poses` | `ph2d_app_vec::state::LabelPoses` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::state::LabelPoses`) | `app.vec.label_poses` |
| `vec_live_drawn` | `ph2d_vec_render::LiveGeometry` | família + tipo de crate | `app.vec.live_drawn` |
| `vec_view_derived` | `ph2d_vec_scene::VecViewState` | família + tipo de crate | `app.vec.view_derived` |
| `vec_expand_knobs` | `(u8, u8)` | família + tipo de crate (arranque não-padrão) | `app.vec.expand_knobs` (`ExpandKnobs`, nasce de `EXPAND_KNOBS_AT_BIRTH`) |
| `vec_offset_mirrored` | `Option<ph2d_vec_scene::VecPathId>` | família + tipo de crate | `app.vec.offset_mirrored` |
| `vec_width_grab` | `Option<ph2d_app_vec::width_grab::Grab>` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::width_grab::Grab`) | `app.vec.width_grab` |
| `vec_width_ref` | `Option<ph2d_vec_scene::VecPathId>` | família + tipo de crate | `app.vec.width_ref` |
| `vec_build` | `Option<ph2d_app_vec::shape_build::BuildSession>` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::shape_build::BuildSession`) | `app.vec.build` |
| `vec_snap` | `crate::vec_snap::VecSnapSettings` | tipo era FACHADA de módulo da crate | `app.vec.snap` |
| `vec_snap_targets` | `ph2d_vec_edit::SnapTargets` | família + tipo de crate | `app.vec.snap_targets` |
| `vec_snap_guides` | `Vec<ph2d_vec_render::Guide>` | família + tipo de crate | `app.vec.snap_guides` |
| `vec_text_edit` | `Option<ph2d_app_vec::text_edit::VecTextEdit>` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::text_edit::VecTextEdit`) | `app.vec.text_edit` |
| `vec_text_size` | `f64` | família + tipo de crate (arranque não-padrão) | `app.vec.text.size` (`TextKnobs`) |
| `vec_text_weight` | `f32` | família + tipo de crate (arranque não-padrão) | `app.vec.text.weight` (`TextKnobs`) |
| `vec_text_line_height` | `f64` | família + tipo de crate (arranque não-padrão) | `app.vec.text.line_height` (`TextKnobs`) |
| `vec_text_tracking` | `f64` | família + tipo de crate (arranque não-padrão) | `app.vec.text.tracking` (`TextKnobs`) |
| `vec_text_align` | `ph2d_vec_text::TextAlign` | família + tipo de crate (arranque não-padrão) | `app.vec.text.align` (`TextKnobs`) |
| `vec_text_extra_axes` | `Vec<(ph2d_vector_font::AxisTag, f32)>` | família + tipo de crate (arranque não-padrão) | `app.vec.text.extra_axes` (`TextKnobs`) |
| `vec_text_family` | `Option<String>` | família + tipo de crate (arranque não-padrão) | `app.vec.text.family` (`TextKnobs`) |
| `vec_last_canvas_click` | `Option<(std::time::Instant, (f32, f32))>` | família + tipo de crate | `app.vec.last_canvas_click` |
| `vec_text_last_target` | `Option<ph2d_vec_scene::VecPathId>` | família + tipo de crate | `app.vec.text_last_target` |
| `vec_shape_last_focus` | `Option<(Option<ph2d_vec_scene::VecPathId>, ph2d_vec_scene::ShapeKind)>` | família + tipo de crate | `app.vec.shape_last_focus` |
| `vec_shape_armed` | `bool` | família + tipo de crate | `app.vec.shape_armed` |
| `vec_trim_hit` | `Option<ph2d_app_vec::trim::TrimHit>` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::trim::TrimHit`) | `app.vec.trim_hit` |
| `vec_bucket_face` | `Option<crate::vec_bucket::BucketHit>` | tipo era FACHADA de módulo da crate | `app.vec.bucket_face` |
| `vec_bucket_cache` | `Option<crate::vec_bucket::BucketCache>` | tipo era FACHADA de módulo da crate | `app.vec.bucket_cache` |
| `vec_bucket_new` | `Vec<(u64, [f32; 2], Vec<ph2d_ecs::FillAnchor>)>` | família + tipo de crate | `app.vec.bucket_new` |
| `vec_shape_armed_target` | `Option<ph2d_vec_scene::VecPathId>` | família + tipo de crate | `app.vec.shape_armed_target` |
| `vec_entities` | `ph2d_vec_entities::entities::VecEntityMap` | família + tipo de crate | `app.vec.entities` |
| `vec_sel` | `ph2d_app_vec::selection_sync::VecSelSync` | tipo da shell ⇒ o TIPO desceu (`ph2d_app_vec::selection_sync::VecSelSync`) | `app.vec.sel` |
| `vec_state` | `ph2d_app_vec::state::VecState` | renomeado | `app.vec` |
| `vec_state.text_wrap` (já no VecState) | `Option<f64>` | assunto do texto | `app.vec.text.wrap` |

| campo da `App` (esqueleto, commit `045401a07`) | destino |
|---|---|
| `smart_pick` | `app.skeleton.smart_pick` (`ph2d_app_skeleton::state::SkeletonState`) |
| `osso_revelado` | `app.skeleton.osso_revelado` (`ph2d_app_skeleton::state::SkeletonState`) |
| `vec_bone_arm_pending` | `app.skeleton.bone_arm_pending` (`ph2d_app_skeleton::state::SkeletonState`) |
| `vec_bone_hover` | `app.skeleton.bone_hover` (`ph2d_app_skeleton::state::SkeletonState`) |
| `vec_bone_preview` | `app.skeleton.bone_preview` (`ph2d_app_skeleton::state::SkeletonState`) |
| `skin_image_cache` | `app.skeleton.skin_image_cache` (`ph2d_app_skeleton::state::SkeletonState`) |
| `vec_bone_drag` | `app.skeleton.bone_drag` (`ph2d_app_skeleton::state::SkeletonState`) |
| `vec_bone_pose` | `app.skeleton.bone_pose` (`ph2d_app_skeleton::state::SkeletonState`) |

## §4 · O QUADRO — como o `run_render_frame` virou índice

**A regra:** cada fase é um MÉTODO `impl crate::App { pub(super) fn fase_x(&mut self, …) }` num ficheiro irmão
`render_loop/fase_x.rs`, chamado pela MESMA ordem. ⛔ Nenhum gancho genérico (nada de `Vec<Box<dyn Phase>>`,
trait de fase ou registo de callbacks). O corpo MUDA-SE verbatim; a prova é `verbatim.py` (o trecho de HEAD
aparece UMA vez na fase — byte a byte nas fases de nível 1, e módulo espaço em branco + vírgula final nas de
dentro do bloco hero, que o rustfmt dedenta e pode re-juntar).

**As peças que atravessam fases:**
- **`FrameGfx::of(gfx)`** (`render_loop/frame_gfx.rs`): o destructure EXAUSTIVO do `AppGfx` numa porta só; cada
  fase pós-`gfx` re-empresta só os campos que usa (`let FrameGfx { a, b, .. } = FrameGfx::of(gfx);`).
- **Os locais que atravessam** entram por parâmetro (por valor) e saem por retorno (`Option<…>` destructurado
  no quadro com os MESMOS nomes, ou um contexto nomeado: `FrameClocks`, `ExtractInputs`, `TimelineView`). Um
  local não-`Copy` que o quadro volta a ler depois da chamada VOLTA no retorno (mover e devolver é a posse
  exacta; zero cópias).
- **Os pedidos do dreno do barramento** (≈230 `let mut pending_*`) viajam em structs de insumos POR FASE
  consumidora (`ImageEditIntents`, `InspectorIntents`, …), desfeitos no prelúdio com os mesmos nomes.
- **O `PaintCtx`** reconstrói-se em cada fase que pinta com os três campos do quadro (`theme` copiado,
  `viewport`, `text_system`): sem `Drop` nem estado, e nada escreve o `theme` nem um campo de `PaintCtx`
  (medido por grep em `crates/` e `shells/`).
- **`frame_text::render_frame()`** (`tests/it/frame_text.rs`): o corpo do `run_render_frame` com cada
  `self.fase_*(` EMENDADA pelo corpo da fase, recursivamente — a lente dos gates de ORDEM. Tem autoteste nas
  duas metades (fase chamada e não encontrada falha alto; `fn fase_*` órfã reprova).

**A direcção do corte mudou a meio, e a razão é o borrow checker:**
- **P1/P2 (de CIMA para baixo):** as fases antes do `let Some(gfx)` do quadro, e as pós-`gfx` chamadas ANTES
  dele (entre as duas posições só guardas e o destructure, sem efeitos).
- **P3 (de BAIXO para cima), desde o bloco `if let Some(hero)`:** uma chamada `self.fase_x()` no sítio exacto
  do corpo só compila se nada emprestado do `gfx` do quadro for usado DEPOIS dela no mesmo caminho (NLL).
  Extraindo do fim, tudo o que vem depois já é fase — a chamada nunca muda de posição, e não há argumento de
  comutação a fazer.

### §4.1 · Os statements indivisíveis

Um statement acima do tecto de 200 LOC não se parte por linhas: ou leva uma **entrada numerada** no `FN_OVERAGE_OK` com
a razão escrita, ou é cortado **UM nível abaixo**, quando o corpo do `if` é uma sequência de statements.
- **Entradas numeradas (2):** `fase_audio_panels` (UM `if` de ~373 linhas) e `fase_vector_bands` (~209).
- **Cortes aninhados** (a fase leva statements de DENTRO de um bloco; a cabeça e a cola ficam no quadro): P4k, P5b, os
  dois do esqueleto (P6i `[6, 13]` e P6j `[1, 5]` dentro do `if let Some(bits) = osso_selecionado`) e os dois dos
  filtros (P6m, P6n dentro de um bloco nu). ⚠️ Os dois do esqueleto ensinaram duas regras ao corte aninhado (§8): um
  `} else if … {` tem saldo zero de chavetas, e um nome ligado pela CABEÇA do bloco de fora não é visto como livre.
- **A cauda por fazer** são três statements que ficam inteiros por desenho (§9): o dreno do barramento (UM `match` de
  ~1 700 linhas dentro de um `for`, com os ~230 pedidos declarados antes dele), a chamada ao `snapshots::publish`
  (UMA chamada cuja lista de argumentos traz os blocos que os calculam) e as vistas do gizmo.

## §5 · As fases, pela ordem do quadro

Gerada do `git log` no fecho (`phase_table.py`: o commit que CRIOU cada ficheiro de fase; `↳` marca uma fase que outra fase chama). A ordem é a do quadro.

| # | fase | ficheiro | L | commit | assunto do commit |
|---|---|---|---|---|---|
| 1 | `fase_pointer_subjects` | `fase_pointer_subjects.rs` | 69 | `52d7aa294` | o que esta' sob o cursor sai do run_render_frame |
| 2 | `fase_audio_panels` | `fase_audio_panels.rs` | 388 | `0b4254e43` | arch(quadro P1b): os paineis de audio saem do run_render_frame |
| 3 | `fase_input_and_drops` | `fase_input_and_drops.rs` | 53 | `f48bc8e69` | carimbo coalescido, diagnostico, gamepad, soltos |
| 4 | `fase_app_scene_smokes` | `fase_app_scene_smokes.rs` | 56 | `ca28bbbdf` | arch(quadro P1d): as cenas de smoke que pedem a App inteira saem do run_render_frame (1.a metade) |
| 5 | `fase_sculpt3d_pre_frame` | `fase_sculpt3d_pre_frame.rs` | 44 | `a4168861f` | o pre-quadro do sculpt3d vira a fase `fase_sculpt3d_pre_frame` |
| 6 | `fase_app_scene_smokes_late` | `fase_app_scene_smokes_late.rs` | 53 | `b29934f96` | a 2.a metade das cenas de smoke que pedem a App vira `fase_app_scene_smokes_late` |
| 7 | `fase_session_upkeep` | `fase_session_upkeep.rs` | 36 | `592208e1b` | a manutencao de sessao vira `fase_session_upkeep`, a ultima antes do `gfx` |
| 8 | `fase_chrome_clock` | `fase_chrome_clock.rs` | 99 | `866a2e7a3` | o relógio do chrome vira `fase_chrome_clock`, a 1.ª fase com o `gfx` |
| 9 | `fase_atlas_scene_smokes` | `fase_atlas_scene_smokes.rs` | 114 | `fcc1bffc7` | as cenas de smoke que precisam do atlas (1.ª metade) viram `fase_atlas_scene_smokes` |
| 10 | `fase_sculpt3d_donation_smoke` | `fase_sculpt3d_donation_smoke.rs` | 65 | `b39ac5add` | a cena da doação do sculpt3d vira `fase_sculpt3d_donation_smoke` |
| 11 | `fase_sculpt3d_bake` | `fase_sculpt3d_bake.rs` | 179 | `c04a517a8` | o objeto misto do sculpt3d vira `fase_sculpt3d_bake` |
| 12 | `fase_relight_baked_forms` | `fase_relight_baked_forms.rs` | 37 | `9d237ad7e` | a re-acendida dos objetos assados vira `fase_relight_baked_forms`, FORA da feature |
| 13 | `fase_atlas_scene_smokes_late` | `fase_atlas_scene_smokes_late.rs` | 135 | `e10765f0c` | as cenas de smoke que precisam do atlas (2.ª metade) viram `fase_atlas_scene_smokes_late` |
| 14 | `fase_sprite_inspector_smokes` | `fase_sprite_inspector_smokes.rs` | 138 | `399ff3409` | as cenas do Sprite Inspector viram `fase_sprite_inspector_smokes` |
| 15 | `fase_sprite_pixel_smokes` | `fase_sprite_pixel_smokes.rs` | 113 | `b560cd825` | as cenas dos pixels da sprite viram `fase_sprite_pixel_smokes` |
| 16 | `fase_painter_brush_smokes` | `fase_painter_brush_smokes.rs` | 81 | `e934d7612` | as cenas do pincel do Painter viram `fase_painter_brush_smokes` |
| 17 | `fase_new_image_modal` | `fase_new_image_modal.rs` | 61 | `5a2e3c5b4` | o modal de imagem nova vira `fase_new_image_modal` |
| 18 | `fase_script_gc` | `fase_script_gc.rs` | 24 | `0b87e026f` | o passo do GC do Luau vira `fase_script_gc` |
| 19 | `fase_surface_resize` | `fase_surface_resize.rs` | 109 | `9003908ed` | o resize coalescido vira `fase_surface_resize` |
| 20 | `fase_fixed_step_clocks` | `fase_fixed_step_clocks.rs` | 145 | `bbc9bdf23` | os relógios do passo fixo viram `fase_fixed_step_clocks`, com o contexto `FrameClocks` |
| 21 | `fase_scene_audio` | `fase_scene_audio.rs` | 49 | `391a8bcd0` | o som de cena vira `fase_scene_audio` |
| 22 | `fase_game_camera` | `fase_game_camera.rs` | 91 | `42cc63474` | a câmera de jogo vira `fase_game_camera` |
| 23 | `fase_extract_inputs` | `fase_extract_inputs.rs` | 148 | `99e3e8618` | os insumos do extract viram `fase_extract_inputs`, com o contexto `ExtractInputs` |
| 24 | `fase_timeline_view` | `fase_timeline_view.rs` | 141 | `4a94a7387` | a vista da timeline vira `fase_timeline_view`, com o contexto `TimelineView` |
| 25 | `fase_timeline_containers` | `fase_timeline_containers.rs` | 202 | `7a035260e` | o relógio dos contêineres vira `fase_timeline_containers` |
| 26 | `fase_timeline_drain` | `fase_timeline_drain.rs` | 117 | `6dd040810` | o dreno da timeline vira `fase_timeline_drain` |
| 27 | `fase_physics_step` | `fase_physics_step.rs` | 89 | `4e1fab3b6` | o passo da física vira `fase_physics_step` |
| 28 | `fase_signal_outbox` | `fase_signal_outbox.rs` | 180 | `1823c3d2a` | o outbox de sinais vira `fase_signal_outbox` |
| 29 | `fase_open_recipe` | `fase_open_recipe.rs` | 68 | `db1357e44` | a receita aberta vira `fase_open_recipe` |
| 30 | `fase_sim_extract` | `fase_sim_extract.rs` | 56 | `842a015e2` | o extract vira `fase_sim_extract` |
| 31 | `fase_drain_leftovers` | `fase_drain_leftovers.rs` | 34 | `9733c23e0` | os restos do dreno vira `fase_drain_leftovers` |
| 32 | `fase_image_tool_activation` | `fase_image_tool_activation.rs` | 127 | `4a95a27c0` | a activacao da ferramenta de imagem vira `fase_image_tool_activation` |
| 33 | `fase_image_tools_mode_and_pills` | `fase_image_tools_mode_and_pills.rs` | 113 | `14bad4b14` | o modo Image Tools e as pills vira `fase_image_tools_mode_and_pills` |
| 34 | `fase_image_tool_bridges` | `fase_image_tool_bridges.rs` | 114 | `a5d3cee13` | as pontes das ferramentas de imagem vira `fase_image_tool_bridges` |
| 35 | `fase_painter_dispatch` | `fase_painter_dispatch.rs` | 160 | `783a7e60a` | o Painter: persistir, despachar e medir vira `fase_painter_dispatch` |
| 36 | `fase_vector_scale` | `fase_vector_scale.rs` | 31 | `bcc7a2a75` | a escala do desenho vectorial vira `fase_vector_scale` |
| 37 | `fase_blend_and_morph` | `fase_blend_and_morph.rs` | 157 | `57d406217` | o blend e o morph vira `fase_blend_and_morph` |
| 38 | `fase_text_on_path` | `fase_text_on_path.rs` | 75 | `09abc031b` | o texto no caminho vira `fase_text_on_path` |
| 39 | `fase_contour_verbs` | `fase_contour_verbs.rs` | 115 | `6d0affd7b` | os comandos e os knobs do contorno vira `fase_contour_verbs` |
| 40 | `fase_filter_commands` | `fase_filter_commands.rs` | 169 | `2a41b9382` | os comandos da pilha de filtros vira `fase_filter_commands` |
| 41 | `fase_filter_values_and_colour` | `fase_filter_values_and_colour.rs` | 102 | `50336fb9c` | o valor do filtro e a cor do picker vira `fase_filter_values_and_colour` |
| 42 | `fase_pattern_path_and_pickers` | `fase_pattern_path_and_pickers.rs` | 148 | `7c230cfca` | o padrao no caminho, o pincel e os pickers vira `fase_pattern_path_and_pickers` |
| 43 | `fase_skeleton_verbs` | `fase_skeleton_verbs.rs` | 131 | `5ba9e4024` | os verbos do esqueleto vira `fase_skeleton_verbs` |
| 44 | `fase_bone_ik_and_limits` | `fase_bone_ik_and_limits.rs` | 58 | `7c2238a23` | o IK e os limites do osso vira `fase_bone_ik_and_limits` |
| 45 | `fase_bone_smart_and_knobs` | `fase_bone_smart_and_knobs.rs` | 150 | `d5fc42dcf` | os smart bones e os numeros do osso vira `fase_bone_smart_and_knobs` |
| 46 | `fase_envelope` | `fase_envelope.rs` | 129 | `f7d0155e6` | o envelope vira `fase_envelope` |
| 47 | `fase_path_effects_spine_bool` | `fase_path_effects_spine_bool.rs` | 92 | `17437b693` | os efeitos, o spine, os passos e a booleana vira `fase_path_effects_spine_bool` |
| 48 | `fase_live_offset_and_width` | `fase_live_offset_and_width.rs` | 149 | `79d2c24dd` | o offset e a largura vivos vira `fase_live_offset_and_width` |
| 49 | `fase_authored_controls_and_ui_states` | `fase_authored_controls_and_ui_states.rs` | 111 | `4eca531ee` | os controlos autorados e os estados de UI vira `fase_authored_controls_and_ui_states` |
| 50 | `fase_ui_host_transition` | `fase_ui_host_transition.rs` | 113 | `9c45834f8` | a transicao do hospedeiro vira `fase_ui_host_transition` |
| 51 | `fase_ui_state_preview` | `fase_ui_state_preview.rs` | 140 | `06cba5535` | a previa dos estados e mover com todos os estados vira `fase_ui_state_preview` |
| 52 | `fase_component_verbs` | `fase_component_verbs.rs` | 96 | `118852208` | os verbos de componente vira `fase_component_verbs` |
| 53 | `fase_compound_snap_rulers` | `fase_compound_snap_rulers.rs` | 68 | `278e38358` | o composto, os encaixes e as reguas vira `fase_compound_snap_rulers` |
| 54 | `fase_node_and_arrange_verbs` | `fase_node_and_arrange_verbs.rs` | 168 | `cf133e2e5` | os verbos de no e de arranjo vira `fase_node_and_arrange_verbs` |
| 55 | `fase_transform_ops` | `fase_transform_ops.rs` | 205 | `c70c5305f` | as operacoes de transformacao vira `fase_transform_ops` |
| 56 | `fase_connector_and_shape_params` | `fase_connector_and_shape_params.rs` | 68 | `4e77d9992` | o conector e os parametros de forma vira `fase_connector_and_shape_params` |
| 57 | `fase_text_fields` | `fase_text_fields.rs` | 166 | `021ec3aab` | os campos de texto vira `fase_text_fields` |
| 58 | `fase_fonts` | `fase_fonts.rs` | 120 | `d392826e8` | as fontes vira `fase_fonts` |
| 59 | `fase_path_shape_and_paint` | `fase_path_shape_and_paint.rs` | 159 | `8186cf486` | a forma do caminho e as tintas vira `fase_path_shape_and_paint` |
| 60 | `fase_texpat_gradient_align` | `fase_texpat_gradient_align.rs` | 183 | `135c59bac` | o padrao de textura, os gradientes, o alinhamento e o pivo vira `fase_texpat_gradient_align` |
| 61 | `fase_vector_panel_dispatch` | `fase_vector_panel_dispatch.rs` | 116 | `20f72c42c` | o despacho do painel vectorial e a tinta do traco vira `fase_vector_panel_dispatch` |
| 62 | `fase_motion_bridge` | `fase_motion_bridge.rs` | 119 | `0a6a1bdd4` | o Motion: publicar, despachar e os sinais vira `fase_motion_bridge` |
| 63 | `fase_tool_mirrors` | `fase_tool_mirrors.rs` | 68 | `62612d119` | os espelhos da ferramenta vetorial, do Flip e da fisica vira `fase_tool_mirrors` |
| 64 | `fase_world_panel_bridges` | `fase_world_panel_bridges.rs` | 171 | `a970962a7` | as pontes do modelador 3D, dos tokens e da escultura vira `fase_world_panel_bridges` |
| 65 | `fase_flip_strip_and_cursor` | `fase_flip_strip_and_cursor.rs` | 61 | `fdccde186` | a tira e o cursor do Flip vira `fase_flip_strip_and_cursor` |
| 66 | `fase_physics_overlay` | `fase_physics_overlay.rs` | 154 | `c654dad84` | a sobreposicao da fisica vira `fase_physics_overlay` |
| 67 | `fase_canvas_overlays` | `fase_canvas_overlays.rs` | 98 | `fd0d666b4` | as sobreposicoes do canvas vira `fase_canvas_overlays` |
| 68 | `fase_selection_highlight` | `fase_selection_highlight.rs` | 192 | `c9164fe2e` | o realce da seleccao vira `fase_selection_highlight` |
| 69 | `fase_text_panel` | `fase_text_panel.rs` | 119 | `9a333f8e7` | o estilo do texto e o painel de texto vira `fase_text_panel` |
| 70 | `fase_entity_sync` | `fase_entity_sync.rs` | 52 | `33c61e4f5` | a sincronizacao das entidades e as formas vivas vira `fase_entity_sync` |
| 71 | `fase_convert_to_curves` | `fase_convert_to_curves.rs` | 41 | `53bea810e` | o converter em curvas vira `fase_convert_to_curves` |
| 72 | `fase_selection_mirror_convert_envelope` | `fase_selection_mirror_convert_envelope.rs` | 31 | `ea6c58f06` | o converter e o envelope no painel vira `fase_selection_mirror_convert_envelope` |
| 73 | `fase_selection_mirror_skin` | `fase_selection_mirror_skin.rs` | 63 | `1097607cf` | a pele e a ferramenta do osso no painel vira `fase_selection_mirror_skin` |
| 74 | `fase_selection_mirror_bone_focus` | `fase_selection_mirror_bone_focus.rs` | 180 | `f838ffa23` | o osso em foco no painel do esqueleto vira `fase_selection_mirror_bone_focus` |
| 75 | `fase_selection_mirror_path_links` | `fase_selection_mirror_path_links.rs` | 136 | `a6faa67d5` | o texto, o padrao e o contorno no painel vira `fase_selection_mirror_path_links` |
| 76 | `fase_selection_mirror_filters` | `fase_selection_mirror_filters.rs` | 100 | `ac97c9a5d` | a pilha de filtros no painel vira `fase_selection_mirror_filters` |
| 77 | `fase_selection_mirror_effects_envelope` | `fase_selection_mirror_effects_envelope.rs` | 47 | `e9d1c16a9` | os efeitos e os presets de envelope no painel vira `fase_selection_mirror_effects_envelope` |
| 78 | `fase_shape_fields` | `fase_shape_fields.rs` | 89 | `e5345175a` | o latch da forma armada e os campos de forma vira `fase_shape_fields` |
| 79 | `fase_vector_upkeeps` | `fase_vector_upkeeps.rs` | 67 | `701dc3384` | as manutencoes vivas do vector vira `fase_vector_upkeeps` |
| 80 | `fase_vector_tree_settle` | `fase_vector_tree_settle.rs` | 143 | `025aa437f` | o assentamento das origens e da arvore vira `fase_vector_tree_settle` |
| 81 | `fase_vector_view_and_drives` | `fase_vector_view_and_drives.rs` | 198 | `abfcbe951` | a vista vectorial, os estilos conduzidos e as recozeduras de forma vira `fase_vector_view_and_drives` |
| 82 | `fase_vector_live_recooks` | `fase_vector_live_recooks.rs` | 104 | `772cf1d49` | as etiquetas, a vista do pen e as recozeduras vivas vira `fase_vector_live_recooks` |
| 83 | `fase_vector_live_geometry` | `fase_vector_live_geometry.rs` | 155 | `d11a5ac38` | a simetria, o lapis e a geometria viva fundida vira `fase_vector_live_geometry` |
| 84 | `fase_vector_selection_frame_panel` | `fase_vector_selection_frame_panel.rs` | 152 | `2e368acba` | a moldura, o layout, o z e as ancoras da seleccao vira `fase_vector_selection_frame_panel` |
| 85 | `fase_vector_selection_states_panel` | `fase_vector_selection_states_panel.rs` | 121 | `db21ec3f9` | a pele, os estados, o z-index e o layout publicados vira `fase_vector_selection_states_panel` |
| 86 | `fase_vector_tokens_and_labels` | `fase_vector_tokens_and_labels.rs` | 61 | `977c3d782` | os tokens e as etiquetas das molduras vira `fase_vector_tokens_and_labels` |
| 87 | `fase_vector_bool_shape_row` | `fase_vector_bool_shape_row.rs` | 61 | `849268c42` | o grupo booleano e o verbo da forma vira `fase_vector_bool_shape_row` |
| 88 | `fase_vector_morph_verbs` | `fase_vector_morph_verbs.rs` | 191 | `43d435f0b` | os verbos do Morph vira `fase_vector_morph_verbs` |
| 89 | `fase_vector_bool_apply_and_morph_reconcile` | `fase_vector_bool_apply_and_morph_reconcile.rs` | 58 | `76799d7c9` | o Apply booleano e a reconciliacao dos conjuntos de Morph vira `fase_vector_bool_apply_and_morph_reconcile` |
| 90 | `fase_vector_layout_recook` | `fase_vector_layout_recook.rs` | 81 | `7eddc1ef5` | o layout vivo, o alinhamento e a silhueta vira `fase_vector_layout_recook` |
| 91 | `fase_vector_fx_recook` | `fase_vector_fx_recook.rs` | 204 | `5673554e1` | a recozedura de FX e de padroes vira `fase_vector_fx_recook` |
| 92 | `fase_vector_bands` | `fase_vector_bands.rs` | 233 | `6a5adeec8` | as faixas do documento vira `fase_vector_bands` |
| 93 | `fase_vector_overlays` | `fase_vector_overlays.rs` | 162 | `7a4e00f8f` | os overlays vectoriais do quadro vira `fase_vector_overlays` |
| 94 | `fase_vector_edit_overlay` | `fase_vector_edit_overlay.rs` | 173 | `a014b66bf` | o overlay de edicao vectorial e as imagens com pele vira `fase_vector_edit_overlay` |
| 95 | `fase_vector_bone_overlay` | `fase_vector_bone_overlay.rs` | 167 | `92b71fa0f` | o overlay dos ossos vira `fase_vector_bone_overlay` |
| 96 | `fase_vector_guides_and_build` | `fase_vector_guides_and_build.rs` | 198 | `67883ccd0` | as linhas de corte, as guias e a construcao de forma vira `fase_vector_guides_and_build` |
| 97 | `fase_vector_tool_handles` | `fase_vector_tool_handles.rs` | 161 | `bef94db7a` | as alcas da ferramenta vectorial vira `fase_vector_tool_handles` |
| 98 | `fase_gizmo_suppression_and_field3d_frame` | `fase_gizmo_suppression_and_field3d_frame.rs` | 178 | `fe1ed4fd1` | a supressao do gizmo e a moldura do modelador 3D vira `fase_gizmo_suppression_and_field3d_frame` |
| 99 | `fase_field3d_smoke_draw` | `fase_field3d_smoke_draw.rs` | 156 | `50e014edd` | o desenho do modelador 3D vira `fase_field3d_smoke_draw` |
| 100 | `fase_field3d_requests` | `fase_field3d_requests.rs` | 125 | `a62580c4a` | os pedidos do modelador 3D vira `fase_field3d_requests` |
| 101 | `fase_hero_paint` | `fase_hero_paint.rs` | 89 | `1f49ff694` | a pintura do ecra hero vira `fase_hero_paint` |
| 102 | `fase_hierarchy_select_lock` | `fase_hierarchy_select_lock.rs` | 59 | `4ce720abb` | a trava do Painter na seleccao da Hierarquia vira `fase_hierarchy_select_lock` |
| 103 | `fase_recipe_and_asset_verbs` | `fase_recipe_and_asset_verbs.rs` | 193 | `05f2aefce` | os verbos de receita e de assets vira `fase_recipe_and_asset_verbs` |
| 104 | `fase_hierarchy_dispatch` | `fase_hierarchy_dispatch.rs` | 182 | `74aa022ac` | o despacho da Hierarquia vira `fase_hierarchy_dispatch` |
| 105 | `fase_inspector_commits` | `fase_inspector_commits.rs` | 197 | `306dbb013` | os commits do Inspector vira `fase_inspector_commits` |
| 106 | `fase_source_strategy_and_joint_pivot` | `fase_source_strategy_and_joint_pivot.rs` | 87 | `416321e99` | a estrategia de origem e o re-assento do pivo vira `fase_source_strategy_and_joint_pivot` |
| 107 | `fase_component_palette` | `fase_component_palette.rs` | 67 | `9d930fa07` | a paleta de componentes vira `fase_component_palette` |
| 108 | `fase_sprite_precision_emissive` | `fase_sprite_precision_emissive.rs` | 140 | `c0dc8ad66` | a precisao, a emissao e o Remove from Sheet vira `fase_sprite_precision_emissive` |
| 109 | `fase_sheet_verbs` | `fase_sheet_verbs.rs` | 199 | `56018a0e0` | os verbos da folha de sprites vira `fase_sheet_verbs` |
| 110 | `fase_physics_edits` | `fase_physics_edits.rs` | 187 | `ff10d6a79` | as edicoes de joint, player e roldana vira `fase_physics_edits` |
| 111 | `fase_physics_join_rig_bake` | `fase_physics_join_rig_bake.rs` | 177 | `b8192b4b1` | ligar, rigar e assar a fisica vira `fase_physics_join_rig_bake` |
| 112 | `fase_autokey` | `fase_autokey.rs` | 52 | `8eed132bf` | o AutoKey vira `fase_autokey` |
| 113 | `fase_hierarchy_group_merge` | `fase_hierarchy_group_merge.rs` | 161 | `8aba9f569` | agrupar, recolher e fundir sprites vira `fase_hierarchy_group_merge` |
| 114 | `fase_use_as_brush` | `fase_use_as_brush.rs` | 142 | `50e3cf078` | o Use as Brush Shape / Grain da Hierarquia vira `fase_use_as_brush` |
| 115 | `fase_use_as_paper` | `fase_use_as_paper.rs` | 113 | `2c99f78c7` | o Use as Paper / Granulation da Hierarquia vira `fase_use_as_paper` |
| 116 | `fase_image_edit_apply` | `fase_image_edit_apply.rs` | 179 | `f6f4ecee0` | o dreno de edicao de imagem e os desmontes do Apply vira `fase_image_edit_apply` |
| 117 | `fase_hero_chrome_tail` | `fase_hero_chrome_tail.rs` | 55 | `16e22ec1e` | o fim do ramo hero vira `fase_hero_chrome_tail` |
| 118 | `fase_legacy_chrome` | `fase_legacy_chrome.rs` | 71 | `301058410` | o ramo sem `HeroScreen` vira `fase_legacy_chrome` |
| 119 | `fase_ui_burst_paint` | `fase_ui_burst_paint.rs` | 25 | `ac8a201e8` | a poeira de impacto vira `fase_ui_burst_paint` |
| 120 | `fase_frame_profile` | `fase_frame_profile.rs` | 38 | `ada7c0678` | o perfilador do quadro vira `fase_frame_profile` |
| 121 | ↳ `fase_frame_profile_report` | `fase_frame_profile_report.rs` | 206 | `f38cddfef` | o relatorio do perfilador vira `fase_frame_profile_report` |

121 fases.

## §6 · Gates re-apontados (todos com prova de mutação)

Gerada do `git log` no fecho (`gates_table.py`: os ficheiros de gate que cada commit mudou). Cada re-apontamento tem a prova de mutação na mensagem do commit dele.

| commit | fase | gates re-apontados (ficheiros que o commit mudou) |
|---|---|---|
| `dcfa2ec4d` | — | `morph_set_world_tests` · `vec_convert_tests` · `vec_entities_tests` · `vec_frame_resize_tests` · `the_bucket_tool_owns_its_gesture` · `the_node_ops_are_wired` · `the_pencil_owns_its_whole_gesture` · `the_trim_tool_owns_its_gesture` · `the_width_reaches_the_selection_when_it_is_authored` · `the_width_tool_owns_its_gesture` |
| `045401a07` | — | `the_bone_pickers_are_modal` · `the_skeleton_panel_only_opens_where_it_has_a_subject` |
| `94199167b` | — | `envelope_pins_tests` · `vec_frame_edit_tests` · `vec_text_tests` |
| `6c058a7d1` | — | `morph_arrow_seam_tests` · `joint_draw_gesture` · `the_armed_shape_latch_is_wired` · `the_bone_pickers_are_modal` · `the_boolean_cooks_before_the_alignment` · `the_bucket_tool_owns_its_gesture` · `the_curve_selector_reaches_the_document` · `the_gesture_reads_what_the_frame_drew` · `the_marquee_shape_comes_from_one_door` · `the_node_ops_are_wired` · `the_node_press_freezes_a_live_shape_recipe` · `the_node_selection_scale_is_wired` · `the_patternpath_handles_are_drawn_and_dragged` · `the_pencil_owns_its_whole_gesture` · `the_pick_reads_the_map_that_was_drawn` · `the_preview_owns_the_pointer_and_the_undo` · `the_shape_fields_are_seeded_by_the_pair` · `the_swap_pick_is_armed_and_resolved` · `the_symmetry_is_a_mode_not_a_selection` · `the_textpath_handle_is_drawn_and_dragged` · `the_trim_tool_owns_its_gesture` · `the_width_sliders_author_the_live_profile` · `the_width_tool_owns_its_gesture` |
| `320b91d94` | — | `the_app_only_sheds_fields` |
| `b292fe26f` | P0b | `rust_src` |
| `52d7aa294` | P1a | `the_bucket_tool_owns_its_gesture` · `the_highlight_has_one_source` · `the_players_finger_reaches_the_bridge` · `the_trim_tool_owns_its_gesture` |
| `0b4254e43` | P1b | `audio_pricing_is_export_work_not_edit_work` |
| `a4168861f` | P1e | `the_donated_form_crosses_the_shell` · `the_grab_is_stamped_once_per_frame` · `the_sculpt_document_is_wired` · `the_sculpt_gesture_is_wired` · `the_sculpt_pill_enters_and_leaves_the_mode` |
| `866a2e7a3` | P2b | `modal_tests` · `the_ui_burst_is_wired` |
| `c04a517a8` | P2e | `a_verb_that_costs_precision_says_so` · `architecture_no_downcast_to_concrete_tool_in_shell` · `the_alpha_image_comes_from_a_straight_sprite` |
| `9d237ad7e` | P2f | `a_baked_object_outlives_the_3d_module` |
| `bbc9bdf23` | P2n | `modal_tests` · `the_stamp_line_carries_its_divisor` |
| `4a94a7387` | P2r | `the_motion_path_anchor_is_drawn_and_dragged` |
| `7a035260e` | P2s | `the_signal_frame_has_one_order` · `the_two_halves_read_the_glyph_through_one_door` |
| `db1357e44` | P2w | `the_open_recipe_comes_to_the_artist` · `the_recipe_mark_reads_the_whole_selection` |
| `f38cddfef` | P3a | `the_dispatch_line_carries_its_divisor` · `the_frame_partition_measures_the_acquire` · `the_stamp_line_carries_its_divisor` |
| `ac8a201e8` | P3c | `the_ui_burst_is_wired` |
| `f467d61ae` | — | `architecture_no_per_tool_branch_in_render_loop` |
| `8699f6b14` | — | `joint_anchor_gizmo` · `joint_draw_gesture` · `pulley_wheel_handles` · `selection_gestures_are_not_fanned_out` · `the_bake_replays_the_recorded_run` · `the_joint_edit_loop_flushes_the_command_queue` |
| `2c99f78c7` | — | `architecture_no_downcast_to_concrete_tool_in_shell` |
| `50e3cf078` | P3h | `architecture_no_downcast_to_concrete_tool_in_shell` |
| `ff10d6a79` | P3l | `the_paste_is_the_one_joint_edit_that_fans_out` |
| `74aa022ac` | P3q | `architecture_no_downcast_to_concrete_tool_in_shell` |
| `05f2aefce` | P3r1 | `the_added_piece_gesture_reaches_the_verb` · `the_apply_ladder_has_one_door` · `the_axis_chip_goes_through_the_swap_door` |
| `1f49ff694` | P3s | `the_arrangement_is_read_at_boot_and_written_on_change` |
| `50e014edd` | P3u | `o_quadro_publica_o_que_o_pintor_mediu` · `the_node_ops_are_wired` |
| `fe1ed4fd1` | P3v | `architecture_no_downcast_to_concrete_tool_in_shell` |
| `bef94db7a` | P3w | `architecture_no_downcast_to_concrete_tool_in_shell` · `the_patternpath_handles_are_drawn_and_dragged` · `the_textpath_handle_is_drawn_and_dragged` |
| `67883ccd0` | P3x | `the_node_ops_are_wired` · `the_snap_label_says_how_far` · `the_symmetry_is_a_mode_not_a_selection` |
| `92b71fa0f` | P3y | `a_skinned_image_is_drawn_once` |
| `a014b66bf` | P3z | `the_marquee_shape_comes_from_one_door` · `the_node_overlay_is_suppressed_under_an_envelope` |
| `6a5adeec8` | P4b | `the_frame_draws_the_live_offset_geometry` · `the_glass_sits_between_the_world_and_the_recipe` · `the_pick_reads_the_map_that_was_drawn` · `the_picked_colour_reaches_the_shape_before_the_rows_are_read` · `the_width_sliders_author_the_live_profile` |
| `7eddc1ef5` | P4d | `the_boolean_cooks_before_the_alignment` · `the_gesture_reads_what_the_frame_drew` |
| `76799d7c9` | P4e | `morph_arrow_seam_tests` · `the_highlight_has_one_source` |
| `977c3d782` | P4h | `the_frame_labels_reach_the_canvas` · `the_token_reaches_the_drawing` |
| `db21ec3f9` | P4i | `the_arrange_buttons_write_the_z` · `the_resize_box_checkbox_reaches_the_world` · `the_vector_panel_crosses_the_display_frontier` |
| `2e368acba` | P4j | `the_show_as_panel_switch_drives_the_authored_panel` |
| `d11a5ac38` | P4k | `the_symmetry_is_a_mode_not_a_selection` · `the_width_sliders_author_the_live_profile` |
| `772cf1d49` | P4l | `the_frame_draws_the_live_offset_geometry` · `the_width_sliders_author_the_live_profile` |
| `abfcbe951` | P4m | `morph_arrow_seam_tests` · `the_draw_pass_publishes_what_the_rows_drive` · `the_driven_style_is_read_before_it_settles` · `the_tree_settles_before_the_capture` |
| `025aa437f` | P4n | `the_net_knows_every_derived_writer` · `the_node_ops_are_wired` · `the_z_projection_reads_the_tree_after_the_sync` |
| `e5345175a` | P5a | `the_armed_shape_latch_is_wired` · `the_shape_fields_are_seeded_by_the_pair` |
| `f838ffa23` | P5e | `the_skeleton_panel_only_opens_where_it_has_a_subject` |
| `c9164fe2e` | P5k | `an_empty_object_is_reachable` · `the_warp_gizmo_is_wired_to_the_pointer` |
| `fd0d666b4` | P5l | `the_brush_ring_marks_the_hit_the_dab_will_use` · `the_motion_path_anchor_is_drawn_and_dragged` |
| `c654dad84` | P5m | `the_grab_is_wired_to_the_pointer` · `the_overlay_reads_the_sensors_the_bridge_published` |
| `fdccde186` | P5n | `the_strip_drag_lands_before_the_snapshot` |
| `a970962a7` | P5o | `the_bake_button_is_wired` · `the_scale_is_published_before_the_paint` · `the_tokens_panel_is_reachable_and_persisted` |
| `0a6a1bdd4` | P5q | `the_graph_only_shouts_while_the_clock_plays_forward` |
| `20f72c42c` | P5r | `the_stroke_checkbox_is_wired` · `the_stroke_paint_row_is_wired` |
| `021ec3aab` | P5v | `the_width_chips_are_wired` |
| `cf133e2e5` | P5y | `the_node_selection_scale_is_wired` |
| `278e38358` | P5z | `the_node_ops_are_wired` |
| `118852208` | P6b | `a_placed_instance_lands_a_screen_step_from_its_main` · `the_swap_pick_is_armed_and_resolved` |
| `06cba5535` | P6c | `the_preview_owns_the_pointer_and_the_undo` · `the_ui_state_machines_run_and_undo_waits` |
| `9c45834f8` | P6d | `the_curve_selector_reaches_the_document` |
| `4eca531ee` | P6e | `the_signal_table_is_wired_into_the_frame` |
| `5ba9e4024` | P6k | `a_skinned_image_is_drawn_once` |
| `7c230cfca` | P6l | `the_shape_art_picker_is_wired` |
| `50336fb9c` | P6m | `the_picker_writes_the_ramp_end_it_was_opened_from` |
| `09abc031b` | P6p | `every_text_on_path_id_is_consumed_by_the_render_loop` |
| `783a7e60a` | P6s | `architecture_no_downcast_to_concrete_tool_in_shell` · `the_painter_asks_the_tool_whether_it_needs_a_document` · `the_panel_drag_drafts_the_shape` |
| `d573402d5` | — | `every_inspector_verb_declares_its_bulk_behaviour` · `the_card_opens_the_prefab_through_the_same_door` · `the_clear_run_verb_reaches_the_tape` · `the_hierarchy_opens_the_prefab_through_the_same_verb` · `the_skeleton_speaks_when_it_has_no_subject` · `the_unused_override_gestures_reach_the_verb` |

112 ficheiros de gate distintos em 64 commits.

⭐ **Dois gates ficaram MAIS FORTES do que eram, porque a mudança de casa os expunha:**
- `architecture_no_per_tool_branch_in_render_loop` contava os literais por-ferramenta SÓ no `mod.rs`; com o
  dreno a sair para uma fase ele leria ZERO e um ramo novo numa fase passaria verde. Passa a contar
  `mod.rs` + todo `fase_*.rs`, com piso de população, e o tecto desce de 16 para o medido (6). Medido antes:
  6 no P0 e 6 em HEAD — a cura chegou ANTES do furo (commit `f467d61ae`).
- `selection_gestures_are_not_fanned_out`: a janela do braço `InspectorPhysicsEdit` acabava no próximo
  `EditorAction::` achado por uma INDENTAÇÃO de 20 espaços (que a fase do dreno muda, e cuja falha alargava a
  janela até ao fim do ficheiro em silêncio); passa a ser o bloco pelas chavetas. ⚠️ A 1.ª redacção abria na
  chaveta do PADRÃO (`{ entity_bits, edit }`) e fechava uma dezena de caracteres adiante — quem a apanhou foi o
  CONTROLO da própria prova de mutação.

## §7 · Foundational / compartilhado tocado

| área (desde o merge-base `e18e75307`) | ficheiros | + / − | porquê |
|---|---|---|---|
| `shells/desktop/src` | 229 | +18 926 / −15 031 | a OBRA 2 (122 fases e o `run_render_frame` índice) e a A9 (os campos `vec_*` num `VecState`) |
| `shells/desktop/tests` | 107 | +2 510 / −637 | os gates re-apontados para o quadro emendado (com prova de mutação), a régua `frame_text`, a catraca `the_app_only_sheds_fields` |
| `crates/ph2d-app-vec` | 34 | +807 / −503 | A9: os oito tipos da família vec que a shell guardava descem para a crate (`ConnectorDrag`, `HandleDrag`, `LabelPoses`, `Grab`, `BuildSession`, `VecTextEdit`, `TrimHit`, `VecSelSync`) e o `VecState` ganha os campos |
| `crates/ph2d-app-skeleton` | 2 | +110 / −0 | A9: o `SkeletonState` (os oito campos do esqueleto que viviam na `App`) |
| `crates/ph2d-vec-edit` | 4 | +14 / −119 | A9: o `History`/`HISTORY_CAP` MORTO apagado (escrito, nunca lido) |
| `crates/ph2d-panel-vector` | 3 | +24 / −13 | A9: os leitores dos campos renomeados |
| `crates/ph2d-skeleton-live` · `crates/ph2d-app-motion` | 1 · 1 | +9/−2 · +6/−1 | A9: os leitores dos campos renomeados |
| `Cargo.lock` | 1 | +1 | uma aresta INTERNA: `ph2d-app-vec` passa a depender da `ph2d-vec-text` (o `TextKnobs` desceu com um `TextAlign`); nenhum pacote externo novo |

Nenhum contrato congelado (§6 do `CLAUDE.md`) foi tocado; nenhum ADR; nenhum schema (`PROJECT_SCHEMA`, `FLIP`,
`DOC_VERSION`) mexe; os ids do `ph2d-editor-core` (a cerca da `line/editor-core`) ficaram intocados.

- ⚠️ **Um comentário novo no `render_loop/mod.rs` FORA de uma fase** (P3q, `74aa022ac`): `// PRECISION-READONLY: …` ao
  lado do fecho de leitura da ponte do Painter (`painter_bridge::dispatch`). O gate
  `a_verb_that_costs_precision_says_so` prescreve o marcador com o motivo ao lado do código, e ele reprovou quando o
  censo de fecho de leitura mudou de casa com a fase; o motivo verificado: o fecho só entrega os pixels ao canvas de
  TRABALHO, e a escrita de volta na sprite é o Apply (`commit_edited_texture`). Só comentário; zero mudança de produto —
  e o marcador viaja com o statement quando ele sair para a sua fase.

## §8 · Construído, medido e REVERTIDO

- **P2d a 203 LOC** (acima do cap de 200): a extracção foi revertida e partida em duas (doação ANTES do bake,
  de cima para baixo — extrair o bake primeiro inverteria a ordem contra a cena da doação).
- **P3p (os commits do Inspector) passou do tecto por função** com a estimativa do gerador a dizer 194: o
  struct de insumos de 21 campos desdobra-se em 21 linhas no prelúdio. Revertida sem commit e partida por
  assunto em P3p1 (estratégia de origem + re-assento do pivô) e P3p2 (os commits) — nunca uma entrada nova.
- **A prova de movimento byte-a-byte** reprovou a 1.ª fase de dentro do bloco hero sobre um movimento correcto
  (o rustfmt tirou a vírgula final de um padrão de tuplo ao dedentá-lo); a régua passou a "módulo espaço em
  branco + vírgula final antes de um fecho". ⚠️ **E "é tudo o que o rustfmt muda" estava ERRADO:** na P3t ele
  tirou as CHAVETAS de um fecho de expressão única que passou a caber numa linha
  (`take_command_pick_if(|id| { … })` → `|id| …`), e a régua reprovou outra vez sobre um movimento correcto.
  Terceira regra, aplicada aos DOIS lados: um bloco de fecho sem `;` ao nível dele perde as chavetas; com `;`,
  fica. Com controlo das duas metades (`test_unbrace.py`: o que o rustfmt faz lê igual; um bloco com instruções
  não perde as chavetas; código diferente continua diferente; aninhados).
- **A saída do quadro do Power Stroke** (`return;` dentro de `if let Some(cmd) = pending_vec_expand`): movida
  verbatim para um método, ela sairia só da FASE e o quadro continuaria. A régua ganhou `body_subst` (a troca
  `return;` → `return None;` declarada pela linha, conferida no extractor e aplicada ao trecho de HEAD antes de
  comparar), a fase devolve `Option`, e a chamada é `if self.fase_x(..).is_none() { return; }` — o mesmo
  `return;` no mesmo sítio. Provado em seco sobre o statement antes do corte.
- **A janela estrutural da G2 abria na chaveta errada**: a 1.ª redacção do braço `InspectorPhysicsEdit` abria
  na chaveta do PADRÃO (`{ entity_bits, edit }`) e fechava uma dezena de caracteres adiante. Nada foi
  commitado: quem a apanhou foi o CONTROLO da prova de mutação S2 (vermelho sobre o quadro correcto).
- **O gerador de specs (scratchpad) mentiu de quatro formas, todas apanhadas ALTO antes de um commit:** o
  `||` do «ou» lido como a barra de um fecho (sumiam pedidos do dreno dos insumos); um tipo escrito em várias
  linhas (`Option<(` …) truncado pelo censo, que só lê a 1.ª linha (o rustfmt recusou o ficheiro); um caminho
  de tipo aparado pelo compilador (`ph2d_gpu::WindowSize` não existe — é `ph2d_host::WindowSize`); e um local
  homónimo do corpo lido como parâmetro (o `b` da cor) — o compilador acusa-o de não usado, e o driver tira-o
  da assinatura e da chamada com asserts de nome. ⚠️ **A mesma espécie voltou na P3u com outra cara:** o `r` do
  corpo nascia num padrão de fecho (`.map(|(i, r)| …)`) e mais abaixo num `let r = …`, e o gerador leu-o como o
  `r` do quadro — MUTADO (o `let r =` casava a régua de mutação) e VOLTANDO no retorno. O compilador não o
  acusou de não usado (o retorno usa-o): acusou `mut` desnecessário na assinatura e um `let Some(r)` morto na
  chamada. Revertida sem commit; o `binds_first` passou a reconhecer um nome dentro do padrão de um `let` e da
  lista de parâmetros de um fecho, e a meta ganhou `exclude_params` como sobreposição à mão.
- **Um `mut` a mais no prelúdio** (P3s): o `let mut paint_ctx` era emitido sempre, e um corpo que só lê
  `paint_ctx.text` reprova no clippy. O gerador decide o `mut` pelo uso.
- **O censo não via os nomes do CONTEXTO que uma fase devolve** (P3x, E0425 `tool_preview_bits`): o destructure
  `let Some(fase_fixed_step_clocks::FrameClocks { report, tool_preview_bits, … })` vem qualificado pelo MÓDULO da
  fase, e a régua do bloco exigia a maiúscula logo depois do `Some(` — o mesmo ponto cego que o `frontier.py`
  tinha (§10.6), noutra ferramenta. Revertida sem commit; a régua aceita o caminho do módulo e lê o TIPO de cada
  campo do struct na casa da fase (`[Option<u64>; 3]`, `ph2d_core::FixedStepReport`, …), e a v3 do censo leva a
  mesma cura. *Uma cegueira curada numa ferramenta não está curada na irmã que copiou a régua.*
- **A retoma de uma fase com entrada numerada reprovava sobre a entrada CERTA** (P4b, `fase_vector_bands`, 209 LOC):
  a 1.ª corrida inseriu a entrada no `FN_OVERAGE_OK` e parou na suíte; na retoma (`--no-extract`) o driver voltou a
  procurar a ACUSAÇÃO do gate para ler o número — e com a entrada no sítio o gate já não acusa. *Uma ferramenta que
  lê o sintoma para escrever a cura não sobrevive a correr duas vezes: a cura apaga o sintoma.* A inserção passou a
  ser idempotente (a entrada existente vale), para a entrada de função e para a de ficheiro.
- **A régua de mutação do gerador não via a escrita num CAMPO** (P4d, E0594 `vec_view.poses`): ela casava
  `nome = …`, `&mut nome` e `nome.push(…)`, e o corpo escrevia `vec_view.poses = …` — a fase nasceu com o
  parâmetro sem `mut`. Revertida sem commit; a régua aceita um caminho de campos entre o nome e a escrita (e o
  `=>` de um braço deixou de contar como atribuição).
- **Um censo de CONTAGEM POR FICHEIRO parte-se a cada fase, e a cura não é uma entrada por fase** (P4e,
  `the_highlight_has_one_source::only_the_listed_gestures_arm_a_sound`): a lista dizia «o `render_loop/mod.rs` arma
  5 sons», e o do Apply booleano mudou de casa com a fase. Acrescentar a fase à lista faria a lista medir
  ENDEREÇOS (e reescrevê-la a cada uma das quatro fases que ainda levam sons), sem que um gesto mudasse. ⇒ a
  unidade passou a ser o QUADRO (`render_loop/{mod.rs, fase_*.rs}`, com piso), que continua a armar 5, e os
  ficheiros do quadro saem da metade «nenhum outro ficheiro arma». É a mesma cura do censo dos ramos
  por-ferramenta (G1), noutro gate.
- **Um teste de UNIDADE em `src/` não alcança o `frame_text`** (que vive em `tests/it/`) — P4e,
  `morph_arrow_seam_tests`. Como as afirmações dele são de PRESENÇA e não de ordem, a lente dele passou a ser o
  DIRECTÓRIO `src/render_loop/` inteiro, com piso de população; uma lista de ficheiros envelheceria a cada fase.
- **O driver formatava a fase e o `mod.rs`, e não os gates re-apontados à mão** (P4e): uma linha longa escrita na
  cura de um gate reprovou o `fmt --check` da retoma. Os `--extra` passaram a ir ao `rustfmt` também.
- **Os tipos em falta passaram a ser procurados ANTES da cadeia, e não uma paragem de cada vez** (P4e e P4g, os dois
  sobre o mesmo `group`): com a cadeia parada e o `mod.rs` estável, os geradores correm em seco sobre as metas que
  faltam (specs `_dry`, nada é tocado) e imprimem os `tipos_em_falta` de todas. Na P4h–P4o deram zero.
- ⛔ **Numa fase ANINHADA, «mover e devolver» só vale para nomes do MESMO bloco** (P4h, E0382 `vec_xf`): a chamada
  mora DENTRO do statement de fora, e o `let Some((vec_xf, sel)) = self.fase_x(..)` liga um `vec_xf` NOVO ao escopo
  daquele bloco — o `vec_xf` do quadro, declarado fora, fica movido, e o quadro lê-o depois do `}`. O `sel`
  (declarado dentro do bloco) não tinha o problema, e foi por isso que a P4e e a P4g passaram. Revertida sem commit.
  ⛔ **A 1.ª cura — passá-lo por REFERÊNCIA — foi construída e REFUTADA pelo clippy** (`needless_borrow`): o corpo
  verbatim diz `&vec_xf`, e com o parâmetro já referência aquilo é um empréstimo que o compilador desfaz. Tirar o `&`
  mudaria o corpo, e um `allow` silenciaria um diagnóstico certo. ⇒ **a cura que fica:** o nome entra POR VALOR,
  volta no retorno como `<nome>_back`, e é REATRIBUÍDO ao do quadro a seguir à chamada
  (`let Some((vec_xf_back, sel)) = self.fase_x(.., vec_xf, sel) else { return; }; vec_xf = vec_xf_back;`), com a
  declaração do quadro em `let mut` (ela já o era). Zero clones, corpo igual, nenhum lint calado.
- **Os gates de ORDEM re-apontam-se pelo FICHEIRO INTEIRO, não pelo teste vermelho** (P4b, cinco gates): um ficheiro
  com `const SRC = include_str!("…/mod.rs")` mede várias ordens, e as que ainda estão verdes (o `sync` e o cozimento
  no `mod.rs`) partem-se na fase seguinte. Re-apontado inteiro ao texto emendado, o ficheiro fica verde nas duas
  casas, e a prova de mutação leva UMA mutação no `mod.rs` para provar que o gate re-apontado ainda vê o que ficou lá.
- **O censo colava o statement ATRIBUTADO ao anterior**: uma linha `#[cfg(feature = "panel-vector")]` a nível
  de statement não acaba em `;`/`}`, e o `if pending_vec_convert` de 13 linhas lia-se com 556. Os cortes feitos
  com o censo antigo são consistentes com ele (as mesmas fronteiras nos dois lados); a v3 do censo corrige-o
  para a região que ainda falta.
- ⛔ **Uma agulha de AUSÊNCIA sobre UM ficheiro enfraquece em SILÊNCIO quando o código muda de casa** (P4j,
  `the_show_as_panel_switch_drives_the_authored_panel`): o ficheiro lia o `mod.rs` por `include_str!` com dois testes
  de presença (um reprovou ALTO — a publicação do interruptor mudou-se para a fase) e um de ausência
  (`!s.contains("\"authored\"")`), que ficaria VERDE para sempre sem ver fase nenhuma. Re-apontar só o teste vermelho
  deixaria a agulha do literal a medir um ficheiro que perde código a cada fase. ⇒ as janelas passam ao texto
  emendado, o `include_str!` fica (falha a compilar se o caminho mentir) e a ausência varre o `mod.rs` E o quadro
  emendado; a mutação P4 escreve o literal NA FASE e o gate reprova — antes da cura teria passado.
- ⛔ **A cola das fases aninhadas NÃO entra numa fase de fora** (P4k, `fase_vector_live_geometry`): o corte ia até ao
  statement do bloco do painel vectorial, cujo corpo — depois da P4g–P4j — é SÓ chamadas às fases aninhadas, cada uma
  com `let … else { return; };` (um `return;` do QUADRO, escrito por esta linha). O gerador parou ALTO no `return`, e a
  1.ª cura foi a saída do quadro declarada (`frame_exits: 4` → `return None;`, a fase devolve `Option`).
  ⛔ **Construída e REFUTADA pelo clippy** (`question_mark`): numa fn que devolve `Option`,
  `let Some(x) = f() else { return None; };` é um `?` escrito à mão. Reescrevê-lo mudaria o texto que a prova de
  movimento lê, e um `allow` calaria um diagnóstico certo. ⇒ **o corte pára ANTES do bloco** (statements 479–489): um
  bloco que só chama fases já É índice e fica no orquestrador, onde o `return;` é do quadro e o clippy não tem nada a
  dizer. Revertida sem commit. *A cola de uma fase é do orquestrador, nunca de outra fase.*
- **O padrão da CHAMADA liga nomes NOVOS, e o gerador não perguntava se o quadro ainda os ESCREVE** (P4k, E0384
  `vec_xf`): com o corte a parar antes do bloco do painel, a chamada `let Some((vec_live, vec_view, vec_xf)) =
  self.fase_vector_live_geometry(..)` ficou ao MESMO nível da cola da P4h (`vec_xf = vec_xf_back;`, dentro do bloco a
  seguir) — e o `vec_xf` novo é imutável. Revertida sem commit. O padrão passou a levar `mut` exactamente nos nomes que
  o quadro escreve ENQUANTO são aquela ligação: o alcance pára na re-ligação ao mesmo nível (cujo lado direito ainda lê
  a antiga) e no fim do bloco, e uma sombra num bloco interior só vale até ao fim dele (`scope_mut.in_scope_text`, 9
  casos de controlo, incluindo chavetas numa string). Um `mut` a mais reprova no clippy, um a menos no compilador. E a
  heurística da LIGAÇÃO, que vivia COPIADA nos dois geradores com três remendos, passou a morar uma vez
  (`scope_mut.is_binding_at`): as specs em seco da P4k–P4o saem iguais antes e depois, fora o `mut` do padrão.
- **Uma agulha que carrega INDENTAÇÃO mede a CASA, não a chamada** (P4k,
  `the_width_sliders_author_the_live_profile`): o gate já lia o quadro emendado, e ainda assim reprovou sobre o quadro
  certo — a agulha era `"self.profile_live\n                .recook("`, e a fase põe a cadeia noutra coluna. Cura:
  `at_chain(head, tail)`, só espaço em branco entre as duas metades, com uma mutação (`. recook(`) a provar que a
  relaxada ainda exige a chamada. ⚠️ **A mesma espécie mordeu na fase SEGUINTE** (P4l, o irmão
  `the_frame_draws_the_live_offset_geometry`, prevista pelo censo abaixo antes de reprovar): em vez de uma segunda
  cópia do ajudante, a régua passou a morar UMA vez em `frame_text::find_chain` (com teste das duas metades — acha em
  qualquer coluna, NÃO acha a cabeça sem a cauda), e os dois gates chamam-na pelo seu `at_chain`. ⛔ **E à TERCEIRA
  (P4m, `morph_arrow_seam_tests`) ela trouxe a metade MUDA:** o teste de unidade em `src/` (que não alcança o
  `frame_text`) tinha uma agulha de presença com indentação, que reprovou alto, e a sua gémea de AUSÊNCIA
  (`!shell.contains("self.playhead.is_playing(),\n                self.fixed_step.fixed_dt(),")`) — que, com a chamada
  noutra coluna, deixa de casar QUALQUER texto e fica verde para sempre, inclusive sobre o regresso exacto que proíbe.
  Cura: o fonte lê-se com o espaço colapsado e as agulhas escrevem-se numa linha; a mutação A2 injecta a forma
  proibida na fase e o gate reprova — antes da cura teria passado.
- ⛔⛔ **O censo das PONTES da rede estava MUDO desde ANTES desta linha** (achado na P4n, ao re-apontar
  `the_net_knows_every_derived_writer`): `every_document_to_tree_bridge_is_in_the_net` casava `_entities::sync(`, e
  desde `3ba2cacb8` (*a PONTE documento⟺árvore sai da shell*) a chamada é `ph2d_vec_entities::entities::sync(` — e o
  Flip idem. No merge-base o `mod.rs` tinha **0** ocorrências da agulha: o censo achava zero pontes e ficava verde
  sobre nada, a espécie muda do §5.0 («um censo que varre por prefixo passa a varrer zero»). ⇒ a agulha passa a ser
  `entities::sync(` com o caminho inteiro, a lente o `mod.rs` + o quadro emendado, e um **PISO de 2** (vectorial e
  Flip); a mutação N4 (a rede esquece a ponte vectorial) reprova, e teria passado antes da cura. *Não é trabalho
  desta linha que partiu o gate — é trabalho desta linha que o apanhou, porque re-apontar obriga a medir a população.*
- ⛔ **O fim de uma chamada medido por INDENTAÇÃO, com um `map_or(len)` de recuo, alarga-se em SILÊNCIO** (P5m, dois
  gates da física): três testes percorriam os argumentos do `overlay::outline::draw(` até `"\n            );"` (doze
  espaços). A fase pôs a chamada noutra coluna — dois reprovaram alto, e o terceiro (`the_overlay_is_handed_the_tool_marks`)
  tinha `map_or(src.len(), …)`: sem achar o fecho, a janela ia até ao FIM do texto e passava a achar os argumentos em
  qualquer sítio a seguir. ⇒ uma porta só, `frame_text::call_end`, o `)` que FECHA por parêntesis equilibrados
  (saltando strings, literais de carácter e comentários de linha, com teste das duas metades — fecha certo com um `(`
  numa nota pelo meio; `None` a uma chamada que não fecha). ⚠️ E um desses gates tinha um controlo positivo cuja premissa
  ERA esta linha (*«o `render_loop/mod.rs` encolheu — se ele foi partido, os dois gates acima podem estar a ler o lado
  sem a chamada»*): o controlo não se apaga, muda de texto — as duas metades (a chamada existe; um piso de tamanho)
  passam a medir o quadro emendado.
- ⛔ **Uma agulha casada na PROSA da própria janela fica verde com o código trocado** (P5l,
  `the_brush_ring_marks_the_hit_the_dab_will_use`): o gate abre uma janela de 2000 bytes na âncora do bloco do anel (um
  comentário) e pergunta se ela contém `Affine::IDENTITY` — e o bloco CITA `Affine::IDENTITY` num comentário a
  explicar porquê. Trocar o argumento da chamada deixava o gate verde; o mesmo para o painel. Pré-existente, e
  apanhado ao escrever a prova de mutação do re-apontar (a mutação só-no-código não tinha como reprovar). Cura: a
  âncora continua a ser o comentário, e a janela que ela abre lê-se sem prosa (`//` e o resto da linha saem); a mutação
  R2 — só o `Affine:: IDENTITY,` do código, com o comentário intacto — reprova. *É a lei do censo textual que não separa
  prosa de código, dentro de uma janela em vez de um ficheiro.*
- **Uma mutação SOBREVIVEU sobre um gate certo, e o defeito era a PROVA** (P4n, C2): para tirar do quadro a agulha
  `settle_origins(` a mutação escrevia `transform:: settle_origins(` — o espaço depois do `::` deixa a agulha
  intacta. O driver parou ALTO (`SOBREVIVEU`), nada foi commitado, e a mutação passou a pôr o espaço antes do `(`.
  *O `assert` de contagem do `mutate` prova que a mutação APLICOU; não prova que ela tira o que o gate procura* —
  quem a escreve tem de ler a agulha do gate, não a do vizinho (N1 usa a outra forma porque a agulha do N1 é o
  caminho inteiro).
- **As metas da região [238, 439] foram RE-MEDIDAS antes da cadeia, e duas mentiam** (depois da P4o, sobre o
  instantâneo do plano): os cortes aninhados `p5b [42, 48]` e `p6m [4, 6]` apontavam UM statement além do fim (438 tem
  48 de dentro, 0..47; 272 tem 6, 0..5) — o gerador reprovaria alto num índice inexistente, mas só na hora. E os três
  cortes de FORA (`p5h`, `p6k`, `p6o`) engoliam o statement cujo corpo os aninhados cobrem INTEIRO — que, depois deles,
  é só chamadas + cola: a mesma parede da P4k, três vezes, apanhada ANTES de construir. ⇒ `p5h [437, 437]`,
  `p6k [285, 288]`, `p6o [271, 271]` (renomeados pelo que sobra), e os blocos 438, 289 e 272 ficam no orquestrador. A
  conferência da região inteira (índice, tamanho, 1.ª linha, instantâneo contra o `mod.rs` de agora) deu **dois**
  tamanhos diferentes e ZERO código diferente: o [439] perdeu o comentário «Conectores, 1ª metade» (que viajou com a
  P4o, cuja fase ele precede) e o [252] ganhou o comentário `PRECISION-READONLY` da P3q (ver §7).
- **A sonda por teste (`evasion_probe2.py`) acusou 2 e as 2 eram falsas** — uma lê o `ph2d-vec-entities`, a outra
  mede uma ausência DENTRO da lista de argumentos de uma chamada que ainda mora no `mod.rs`. Ela vê a asserção, não o
  texto que a asserção lê: *uma sonda que acusa imprime, e quem lê o teste decide*. ⭐ E ela passou a correr DENTRO da
  cadeia, depois de cada fase commitada: pára se o número de candidatos SUBIR acima da base aceite (2).
- **O ensaio em seco da região inteira (49 fases) ANTES da cadeia** deu zero fases acima do tecto (a maior, 182) e
  nove paragens, todas por coisa que o censo não lê sozinho — o TIPO de um local sem anotação cuja 1.ª linha é uma
  chamada (`env_container` ← `sole_container(..) -> Option<u64>`; `sel`; `(flip_active, flip_style)` ←
  `ph2d_app_flip::bridge::publish(..) -> (bool, Option<FlipStyleSnapshot>)`; `osso` ← `Entity::from_bits`) e os dois
  `return` da P6m, que moram DENTRO do fecho `|f|` do `fx_live::edit` e saem do fecho, nunca do quadro
  (`return_ack`). Cada valor foi lido da assinatura e escrito no `write_metas_region.py` com a fonte ao lado — a
  cadeia não pára uma vez por tipo.
- **«Usado depois» é da LIGAÇÃO, não do NOME** (P5c, clippy `unused variable: sel`): o gerador devolvia um parâmetro
  porque o nome `sel` reaparecia mais abaixo no quadro, em blocos que ligam o seu próprio `sel`, e a chamada ligava um
  `sel` que ninguém lia. Revertida sem commit. Cura: `used_after(nome)` lê só o alcance da ligação — do fim do trecho
  até ao fecho do bloco que a DECLAROU, cortado numa re-ligação à mesma profundidade (`scope_mut.in_scope_text` com
  `base` = profundidade da declaração − a da chamada), nos dois geradores e no `needs_assign_back`.
  ⛔⛔ **A 1.ª versão da cura partiu SETE retornos certos, e a A/B apanhou-os antes de uma fase:** `vector_active`,
  `painter_apply_committed` e as cinco saídas das pontes de imagem deixavam de voltar (a fase compilaria sem eles e o
  quadro reprovaria no uso). ⚠️ A 1.ª hipótese — um `|` de FECHO de fecho lido como abertura — foi **refutada por
  medição** (nenhum dos sete aparece depois de um `|..|`), e a sonda do alcance (`probe_scope.py`) mostrou o alcance a
  acabar 3 000 linhas antes do uso; a do descolamento (`probe_drift.py`) achou a causa na L3717: uma **string de
  VÁRIAS linhas**. O `code()` por linha não a fecha e conta as chavetas dela (é a régua do `dstart`, que dá a `base`);
  o `in_scope_text` re-saneava o texto JUNTO, emparelhava aspas de linhas diferentes, e as duas réguas descolavam de
  um nível. ⇒ o `_depths` conta chavetas sobre o texto TAL COMO VEM (já por linha) — *duas réguas da mesma grandeza
  têm de concordar, e quem define a base é a por linha*. A regra do `|` de abertura ficou mesmo assim (`(`, `,`, `=`,
  `{` ou `move` antes dele): o caso de controlo `run(|t| vec_xf = t)` reprovava com a antiga.
  **A A/B final** (geradores antigos isolados com o `scope_mut` deles, contra os novos, nas 47 metas que faltavam):
  QUATRO diferenças, todas retornos FALSOS que caem (`sel` na P5c e na P6m, `osso` na P6i, `live` na P6u), e zero
  retornos certos perdidos; controlo com 10 casos de escrita e 8 de uso.
- ⛔ **Um TIPO declarado DENTRO do `run_render_frame` não se nomeia na assinatura de outra fase** (achado ANTES de
  morder, ao planear a cauda): `enum IkKnob { Mix, Softness, Chain }` é um item LOCAL do quadro (statement [98] do
  bloco hero), e `let mut pending_ik_knob: Option<(IkKnob, f64)>` viaja nas intenções da P6i
  (`fase_bone_smart_and_knobs`) — o struct de intenções vive noutro ficheiro, onde o nome não existe, e o ensaio em
  seco não o apanha (o gerador escreve o tipo que o censo leu; é o compilador que o recusa). Cura: um commit de
  PREPARAÇÃO, sem produto, que muda o `enum` (com o `#[derive]` e o doc) para o nível do módulo `render_loop` — as
  fases já o vêem pelo `use super::*;`, e o corpo que o constrói (`IkKnob::Mix`) e o que o lê ficam verbatim. ⚠️ Tirar
  um statement ABAIXO da região desloca de −1 todos os índices dela: as metas que faltam são deslocadas por uma
  edição derivada com contagem (`shift_region_metas.py`) e conferidas pela 1.ª linha de cada statement.
  **Aterrou em `dd1785072`**, entre a P5e e a P5f. Conferência: as 44 metas que faltavam deslocadas pela edição
  derivada, e os 200 statements da região comparados um a um (tamanho e 1.ª linha, o [238..437] de antes contra o
  [237..436] de depois) — **zero** diferentes; o ensaio em seco das 44 limpo; suíte da shell verde (1 422 + 800),
  fmt e clippy limpos; `run_render_frame` 6 617 → 6 610. ⚠️ **A 1.ª redacção reprovou no tecto do FICHEIRO** (`mod.rs`
  7 139 contra 7 134): ela acrescentava ao item movido um bloco de doc de cinco linhas a explicar a mudança. Subir o
  tecto não é cura; o item muda-se com o doc de UMA linha que já tinha (com a razão na mesma linha), e a explicação
  mora na mensagem do commit e aqui — o `mod.rs` fica com as mesmas 7 134 linhas. ⚠️ **Censo:** 51 literais nos gates da shell carregam `\n` + indentação; a maioria
  delimita funções noutros ficheiros (`\n    pub(crate) fn `), e os que leem o QUADRO entram na triagem do
  re-apontar em lote.
- **Uma leitura SEM COMENTÁRIOS continua sem comentários quando muda de lente** (P5o,
  `the_bake_button_is_wired`): o gate lia o `mod.rs` pelo `sculpt_source::source`, que descasca `//` — e o
  `braced_block` que ele abre procura a próxima `{` depois da âncora. Re-apontado para o quadro CRU, uma `{` numa nota
  entre a âncora e o corpo seria o bloco lido. ⇒ o quadro emendado passa pelo MESMO descasque (`l.find("//")`), e o
  mesmo vale para os dois gates da tinta do traço (P5r, `frame_code()` com o filtro do `code()` deles). *Mudar o texto
  que um gate lê sem mudar o que ele tira do texto é mudar a régua pela metade.*
- ⛔ **A 1.ª extracção da P5v parou no CLIPPY, não num gate** (`type_complexity`: o retorno era o tuplo
  `Option<(bool, Option<(usize, f64)>, Vec<VecPathId>)>`). Revertida sem commit (`git checkout` do `mod.rs`, a fase e a
  mensagem gerada apagadas — a mensagem descrevia o tuplo); a meta ganhou `ret_struct` (`TextFieldsOut`), a mesma porta
  que o `DrainOut` da cauda já usava, e a fase aterrou com o struct. O ensaio em seco não o apanha: ele escreve o tipo e
  quem o recusa é o clippy. A P6t repetiu-o (cinco retornos das pontes das ferramentas de imagem) com a mesma cura
  (`ImageToolBridgesOut`).
- ⭐ **Os gates passaram a ser re-apontados ANTES da extracção que os parte** (depois da P5u). A cadeia parava uma vez
  por gate, e cada paragem custava três voltas. Duas observações o tornam seguro: *o quadro emendado CONTÉM o
  `mod.rs`*, logo um gate apontado a `frame_text::render_frame()` é verde antes e depois da fase; e a prova de mutação
  só pode correr DEPOIS da extracção, logo ela não pode ser dispensada. ⇒ uma PRÉ-VARREDURA (`prescan_gates.py`: os
  literais de cada gate que ainda lê só o `mod.rs`, contra o trecho de cada fase por fazer no instantâneo congelado)
  nomeia a fase que o partirá; o gate é re-apontado com a cadeia PARADA, e a fase leva três ficheiros que a cadeia lê
  pelo nome — `extras_<fase>.txt` (o gate entra no MESMO commit), `mut_<fase>.sh` (a prova) e `gates_<fase>.txt` (a
  mensagem diz quais). ⚠️ **Sem os três, re-apontar antes é pior do que não re-apontar**: a fase ficaria verde e
  commitaria SEM o gate e sem prova. Aplicado a P5v, P5y, P6b, P6c, P6d, P6i, P6l, P6p e P6s; a varredura tem falsos
  positivos (a P6g: a agulha `if let Some(steps)` do gate do sculpt lê a FAMÍLIA, não o quadro) e é por isso que ela
  acusa e quem lê o teste decide.
- **Um ficheiro de gate MEIO re-apontado parte pela metade que ficou** (P5z, `the_node_ops_are_wired`): três testes
  já liam o `LOOP_SRC` do quadro desde a P4n, e dois outros, mais abaixo, abriam o `mod.rs` por `read_to_string` — a
  varredura dos leitores via o ficheiro como «lente» e não o acusou. *A unidade da lente é o TESTE, não o ficheiro.* ⇒
  os dois mudaram juntos, incluindo o que ainda não partia (os braços dos interruptores, que saem com o dreno do
  barramento na cauda).
  ⚠️ E um gate re-apontado À MÃO também passa pelo `fmt --check` do driver: a P5z parou ali (dois `static … LazyLock`
  numa linha de 110 colunas), antes de qualquer teste. ⇒ `rustfmt` sobre os gates editados ANTES de relançar.
- ⛔⛔ **O INSTRUMENTO também tinha uma agulha de formatação** (P6a): a extracção correu verde e a suíte reprovou em
  DOIS gates com UMA causa — o `rustfmt` escreveu a chamada da fase como `if self\n    .fase_vec_expand(…)\n
  .is_none()`, e a EMENDA (`frame_text::splice`) procurava `self.fase_` CONTÍGUO. A fase ficou ÓRFÃ no texto do quadro:
  o gate das órfãs reprovou ALTO (é a metade que existe para isso), e o da largura viva por consequência. ⚠️ Nenhum dos
  gates de ordem teria dado pela falta se o gate das órfãs não existisse: *um texto emendado a que falta um pedaço lê-se
  como um quadro mais curto*. Cura no instrumento (`next_phase_ref`: espaço em branco entre o `self` e o `.`, e o `self`
  tem de ser palavra), com auto-teste das duas metades, e a chamada no `mod.rs` fica como o `rustfmt` a escreve.
- ⛔ **Uma prova de mutação SOBREVIVEU, e o defeito era do GATE — pré-existente** (P6b,
  `the_swap_pick_is_armed_and_resolved`): a agulha do «o Swap ARMA o pick» era o NOME `PathPick::InstanceMain(`, e o
  quadro tem esse nome também onde LÊ o pick armado (`Some(crate::vec_pick::PathPick::InstanceMain(_))`, hoje na
  `fase_vector_selection_frame_panel`). Tirar o ARM do quadro deixava o gate verde — e o `mod.rs` de antes da obra
  tinha os dois, logo o furo é anterior a esta linha; partir o quadro só o pôs em ficheiros diferentes, e a mutação
  escrita contra a fase é que o expôs. O driver parou (`SOBREVIVEU`), nada foi commitado, e a agulha passou a ser a
  ESCRITA (`path_pick = Some(crate::vec_pick::PathPick::InstanceMain(`). *Uma agulha que casa com a LEITURA de uma coisa
  aprova um produto que já não a escreve.*
- ⛔ **A PRÉ-VARREDURA tinha a mesma doença que ela caça** (P6c): a lente dela — *este gate já lê o quadro?* —
  perguntava se o ficheiro continha `render_frame`, e a palavra aparece DENTRO de `fn run_render_frame`, que o controlo
  positivo do `the_ui_state_machines_run_and_undo_waits` cita. O gate leu-se como re-apontado, a varredura calou-o, e a
  suíte parou a cadeia nele. *Uma régua de «isto já foi curado?» que casa com um nome parecido aprova o doente.* ⇒ a
  lente é o CAMINHO do instrumento (`frame_text::render_frame`), e a varredura corrigida, corrida outra vez sobre as
  fases por fazer, acusou mais DOIS que a primeira calava (P6e, a tabela sinal → papel; P6m, o readback do picker de
  filtro) — re-apontados antes de a cadeia chegar a eles.
- **A sonda do FURO parou a cadeia depois da P6d, e os QUATRO candidatos eram falsos** — lidos um a um: o `later` é
  uma string sintética do auto-teste do `call_end`; o censo dos sons conta a UNIDADE do quadro (`mod.rs` + `fase_*`),
  que é onde os armamentos moram; o release do osso lê o `input_dispatch.rs`, e o `gizmo.selection)` só coincide com
  texto das fases; o do token lê o `ph2d-vec-entities`. A contagem subiu porque as fases P6c/P6d levaram as últimas
  cópias dessas palavras para fora do `mod.rs` — a sonda vê a PALAVRA sair, não o texto que a asserção lê. ⚠️ **A base
  deixou de ser um NÚMERO e passou a ser a LISTA NOMEADA dos aceites** (`evasion_accepted.txt`): com um número, um falso
  positivo que desaparece e um furo real que aparece dão a mesma contagem, e a cadeia seguiria calada — a catraca sem
  censo de obsolescência do `CLAUDE.md` §5.0, dentro do instrumento. E a cadeia pára também se a sonda não correr.
  A lista nomeada pagou-se na fase seguinte: depois da P6f ela acusou UM candidato NOVO com os quatro aceites intactos
  (`the_pick_reads_the_map_that_was_drawn::every_pick_door_is_handed_the_drawn_map`, `offset_live`) — lido, é falso: a
  ausência lê os argumentos das portas de pick no `input_dispatch.rs`, e a P6f só levou a última cópia da palavra para
  fora do `mod.rs`. Com a base numérica (então `4`) ele teria parado a cadeia na mesma, mas pelo número; um que
  *substituísse* outro teria passado.
- ⛔⛔ **O CENSO ANINHADO atravessava um `} else if … {`** (P6i, `fase_bone_smart_and_knobs`, corte dentro do
  statement 288 — `if let Some(bits) = osso_selecionado { … } else if pending_bone_needs_focus { … }`): a meta pedia os
  statements `[6, 14]` do bloco de dentro, e o 14 era o `eprintln!` do RAMO `else`. O censo conta chavetas por LINHA, e
  uma linha que fecha um bloco e abre outro tem saldo ZERO — o ramo leu-se como mais statements do bloco. A extracção
  escreveu uma fase com um `}` a mais e o `rustfmt` recusou-a (*non-item in item list*); revertida sem commit. ⚠️ **O
  ensaio em seco tinha o defeito dentro** (`b_line` 4411, a linha do `eprintln!`) e não o via: ele GERA a spec, não a
  compila. ⇒ a meta passou a `[6, 13]` (o braço `else` fica na cola do quadro), e um PRÉ-VOO DE CHAVETAS
  (`brace_preflight.py`: o trecho de cada fase por fazer, sem comentários nem strings, nunca abaixo de zero e a acabar em
  zero) correu sobre as dezoito fases que faltavam — só a P6i velha reprovou, e é o controlo. ⚠️ E a consequência para
  os gates: o braço «nenhum osso em foco» NÃO sai com a P6i corrigida, logo o `the_skeleton_speaks_when_it_has_no_subject`
  (já re-apontado) é commitado e provado na P7a, que é quem leva as duas alimentações com o dreno.
- **O corte da P6j começava num `let` que usava um nome ligado pela CABEÇA do bloco de fora** (`let osso =
  Entity::from_bits(bits);`, com `bits` do `if let Some(bits) = osso_selecionado`): o censo de nomes livres não vê
  ligações de PADRÃO na cabeça do statement de fora, e o clippy recusou a fase (`E0425`). Revertida sem commit; a meta
  passou a `[1, 5]` e o `let osso` fica na cola (a P6i já o recebia por parâmetro). A sonda
  `header_bind_scan.py` correu sobre os cortes aninhados que faltavam (P6m e P6n, dentro de um bloco nu `{ … }`) e não
  achou outro — mas achou a vizinha: o bloco começa com `use crate::fx_live::FilterHit;`, a P6n leva o `use` para dentro
  do corpo dela, e a P6m (cortada ANTES) usava o nome sem o `use` ⇒ `uses_extra` na meta da P6m, antes de a cadeia lá
  chegar.
- **A prova de movimento reprovou a P6l com ZERO ocorrências, e o corpo estava certo** (P6l): ao dedentar, o braço
  `PatternPathCmd::Link => { … }` passou a caber na linha e o `rustfmt` tirou-lhe as chavetas e pôs a vírgula. É a
  QUARTA transformação do `rustfmt` que a régua (`verbatim.py`) aprende, depois do espaço em branco, da vírgula final e
  das chavetas de um fecho de expressão única (P3a, P3t) — cada uma MEDIDA numa fase que parou, nenhuma adivinhada.
  Perdoa-se aos DOIS lados e só à volta de UMA expressão, com auto-teste das duas metades (um token dentro dela reprova).
  *A barra honesta de «verbatim» num repo com `cargo fmt` obrigatório é «igual a menos do que o `rustfmt` faz ao
  dedentar», e a lista do que ele faz cresce por medição.*
- **A P6s levou o ÚLTIMO leitor do `paint_ctx` do quadro** e o clippy parou no `mod.rs` (`unused variable: paint_ctx`):
  cada fase de pintura reconstrói o dela a partir do `theme`/`viewport`/`text_system`, e o do quadro ficou sem ninguém.
  Ele saiu — uma construção de struct sem efeito, fora do trecho movido — e ⛔ não foi calado com `_paint_ctx`: um
  prelúdio partilhado que ninguém lê é código morto, e silenciar o aviso deixava-o a fingir que o quadro ainda pinta ali.

## §9 · Premissas refutadas

- ⛔ *«O dreno do barramento parte-se em fases de ≤ 200 linhas, verbatim»* — é UM `match` de ~1 700 linhas
  dentro de um `for`; braços não são statements. Partir por braço obrigaria cada corpo a escrever `*pedido = …`
  através de referências (os `pending_*` são locais do quadro), e isso já não é o corpo verbatim. ⇒ **decisão:** os
  pedidos declarados e o dreno saem JUNTOS numa fase só (`fase_bus_drain`, statements [9, 236] do bloco hero depois
  do `IkKnob`), que devolve os pedidos num struct (`DrainOut`) e leva entradas NUMERADAS nos tectos por função e por
  ficheiro — como a `fase_audio_panels` (373) e a `fase_vector_bands` (209). O mesmo vale para a chamada ao
  `snapshots::publish` (statement [0], 194 linhas): UM statement cuja lista de argumentos traz os blocos que os
  calculam, e parti-la em `let`s mudaria a ordem de avaliação.
- ⛔ *«As fases de dentro do bloco hero chamam-se no sítio, de cima para baixo»* — a chamada colide com os
  empréstimos do `gfx` que o resto do bloco ainda usa; só de baixo para cima a chamada fica no sítio.
- ⛔ *«O censo dos ramos por-ferramenta já foi furado pelas fases extraídas»* — medido: 6 no P0, 6 em HEAD,
  todos no `mod.rs`.

## §10 · Para o INTEGRADOR

1. ⚠️ **`render_loop/mod.rs` é o ficheiro mais quente desta rodada.** Uma linha que tenha editado o
   `run_render_frame` num sítio que aqui virou fase conflita TEXTUALMENTE e a cura NÃO é resolver o conflito no
   `mod.rs`: é levar a edição para o ficheiro da fase onde o código mora hoje (ache-o por
   `crate::frame_text::render_frame()` ou por `grep -rn "<linha>" shells/desktop/src/render_loop/fase_*.rs`).
2. **`shells/desktop/src/input_dispatch.rs` ficou FORA d'O QUADRO** (as entradas `on_mouse_input` 3 102 e
   `on_cursor_moved` 375 no `FN_OVERAGE_OK` dizem-no por escrito) — ⚠️ **mas NÃO está intocado: a A9 editou-o em
   CINCO commits** (`dcfa2ec4d` a `History` morta · `045401a07` o estado do esqueleto · `94199167b` os tipos da família
   · `6c058a7d1` o `app.vec` · `7b171a0a5` o `if` literal), `+292 / −465`, e o tecto dele desceu `7 290 → 7 117`. São
   remoções de parâmetro e renomeações de campo (`self.vec_x` → `self.vec.x`), nunca uma partição: uma linha que o
   tenha editado conflita nessas linhas, e a cura é re-escrever a edição dela com o nome novo.
3. **Erros PRÉ-EXISTENTES em `--no-default-features`** (31× `ph2d_panel_vector` não resolvido, 19 variáveis
   não usadas no `snapshots.rs`, …): o conjunto por ficheiro é IDÊNTICO antes e depois das fases desta linha.
4. **Entradas novas na allowlist de downcast** (`architecture_no_downcast_to_concrete_tool_in_shell`), cada
   uma com a justificação escrita e todas a MESMA excepção de classe da entrada do `mod.rs`, mudada de casa com
   o bloco verbatim (nenhum downcast novo): `fase_sculpt3d_bake.rs` · `fase_use_as_paper.rs` ·
   `fase_use_as_brush.rs` · `fase_hierarchy_dispatch.rs` · `fase_gizmo_suppression_and_field3d_frame.rs`.
   ⚠️ A entrada do `render_loop/mod.rs` FICA enquanto ele ainda tiver um downcast; o censo de obsolescência do
   próprio gate acusa-a no dia em que o último sair.
5. **Bug latente achado:** a âncora do gate `the_recipe_mark_reads_the_whole_selection` casava primeiro com o
   ajudante de nível de ficheiro `master_editing::mark` (FORA do `run_render_frame`) e só passava porque a janela
   se estendia até à marca verdadeira; hoje lê o texto emendado, cuja 1.ª marca é a certa.
6. **Ponto cego de ferramenta:** o `frontier.py` do scratchpad não via nomes ligados por destructure de struct;
   a partir da P3 o censo passou a ser `census_hero.py` (com a profundidade de uma linha de fecho contada pelo
   FIM da linha — a v1 colava o statement seguinte a um bloco).
7. **A superfície de colisão** (`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, corrida no fecho sobre `d573402d5`). ⚠️ Os «TETOS DE LOC» abaixo são o MAPA contra o tecto de 600 do workspace: os cinco ficheiros têm tecto NUMERADO próprio no `file_loc_caps` da shell, e o `render_loop/mod.rs` DESCEU de 13 922 para 3 869.

```

SUPERFÍCIE DE COLISÃO — line/render-loop contra main
  merge-base e18e75307   ·   135 commit(s)   ·   384 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗  1551 / 600   shells/desktop/src/app_state.rs
  ✗  7117 / 600   shells/desktop/src/input_dispatch.rs
  ✗   816 / 600   shells/desktop/src/input_dispatch/gizmo_drag.rs
  ✗  1289 / 600   shells/desktop/src/main.rs
  ✗  3869 / 600   shells/desktop/src/render_loop/mod.rs
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

## §11 · Smoke do dono

O quadro não mudou de produto, de pixel nem de ORDEM — o smoke não procura uma feature nova: é abrir o app e passar
**por cada assunto que saiu para uma fase**, porque um defeito de extracção lê-se como «isto deixou de reagir».
Comando (worktree desta linha, perfil `smoke`):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-render-loop && cargo run -p ph2d-host-desktop --profile smoke
```

1. **Arranque sem env** — o canvas pinta, os painéis abrem, a Hierarquia lista, o relógio do topo anda (prelúdio:
   ponteiro, áudio, cenas de smoke, relógio do chrome, atlas, modal de imagem nova, redimensionar a janela).
2. **Vector** — ferramenta Vector: desenhar com a caneta; na ferramenta Shape trocar de forma no catálogo e ver os
   campos mudarem logo (o latch, P5a); seleccionar e mexer com o gizmo; booleana viva e *Apply*; Offset/Width com os
   sliders e *Apply*; Envelope (criar, gesto, preset); padrão e texto no caminho; Blend e Morph (fazer o conjunto,
   pré-visualizar, setas); tokens (vincular um campo); moldura e auto layout (z, âncoras, *Resize Box*); filtros e
   efeitos (acrescentar, reordenar, esconder, slider); guias e régua; simetria ligada ao desenhar.
3. **Hierarquia** — arrastar para reordenar e **Ctrl+Z** (a fila de undo não pode parar); apagar, duplicar, cadeado,
   olho; *Make Component* / *Edit Prefab* (o vidro e o *Done*).
4. **Esqueleto** — ossos, *Bind*, *Add IK*, o limite, um smart bone (a secção do painel acende com o osso em foco).
5. **Imagem** — Painter (pintar e *Apply*), Remoção de fundo, Upscale, as pills do modo de edição de imagem.
6. **Flip** — tira de quadros, desenhar, o cursor, o gap.
7. **Física** — `env PH2D_PHYSICS_SMOKE=6` (junta + Play) e o painel de mundo (`W`).
8. **3D** — pill **MODEL** (acrescentar forma, gizmo, vistas) e `env PH2D_SCULPT3D_SMOKE=34`.
9. **Timeline / UI States** — Play, scrub, uma transição de estado de UI com *Preview*.

**Como saber que partiu:** um botão que acende e não faz nada, um painel que mostra o valor anterior por um quadro
(clicar duas vezes para «pegar»), um Ctrl+Z que desfaz só uma vez, ou um aviso no terminal que antes não aparecia.

## §12 · O que fica ABERTO

- ⛔⛔ **A CAUDA (P7a `fase_bus_drain` · P7b `fase_gizmo_views_and_prefab` · P7c `fase_snapshots_publish`) NÃO CORREU —
  o tecto da SHELL reprovou, e as duas regras do dono colidem.** Medido antes de a lançar, com o gate que o driver das
  fases nunca corre (`ph2d-editor-core::architecture_the_shell_only_shrinks`):

  | | linhas `.rs` em `shells/desktop` |
  |---|---|
  | tecto (`TETO_LOC`, já com a folga de composição) | **190 629** |
  | merge-base `e18e75307` (`src` 153 234 + `tests` 34 122) | **187 356** (verde) |
  | agora, depois da P6w | **192 199** (**+1 570** acima) |
  | a cauda, estimada sobre o `mod.rs` vivo (`DrainOut` de 230 campos: struct + retorno + destructure) | **~+750** |

  **De onde veio o crescimento** (`git diff --numstat` desde o merge-base): as 122 fases `+14 515`, o `mod.rs` `−10 169`,
  o resto de `src` `−1 376` ⇒ **`src` +2 970** (a prelude, a assinatura, o `FrameGfx::of`, as intenções de cada fase —
  ~24 linhas por fase); e **`tests` +1 857** (os gates re-apontados, as notas, a régua do quadro).

  **O que as curas permitidas alcançam, medido:** (a) mover código de família — `scripts/fecho-da-familia.py vec --extra
  'vec_*'` dá **7 ficheiros / 412 LOC movíveis e 38 / 11 624 presos** a 12 âncoras da shell (`app_state.rs`,
  `input_dispatch.rs`, `envelope_live.rs`, `layout_live.rs`, …) — desprendê-las é trabalho de substrato de uma linha de
  família, não um anexo desta; (b) consolidar o que se repete em cada fase (os dois comentários de prelúdio: 165 linhas;
  o `/// Ver o cabeçalho do módulo.`: 122) e condensar as notas dos gates: **~600–1 100**. Somadas, não pagam os `+1 570`
  de hoje, e muito menos os `~2 300` com a cauda.

  **O que o tecto protege, medido:** o `cargo check` da shell NÃO-incremental, depois de uma mudança de conteúdo, com a
  máquina calma (load < 2), leva **2,28 s** (duas corridas: 2,286 · 2,279) — o `ESTADO_W2` registava 5,9 s. ⚠️ Um
  `touch` e uma edição incremental dão ~1 s e não medem o tamanho da unidade.

  **As saídas, para o dono decidir** (a regra da linha proíbe a primeira; as outras mudam o âmbito):
  1. o integrador **reconta o tecto** com o custo da divisão do quadro dentro, por ordem expressa do dono (a divisão
     compra legibilidade e ordem gateada, e custa ~2,5 % da unidade e nenhum tempo mensurável de `check`);
  2. uma **linha de família** desprende as âncoras da `vec` (≥ 1 570 LOC a mover) antes de a cauda correr;
  3. a cauda fica por fazer e o **dreno do barramento continua no `run_render_frame`** (o índice fica com ~2 400 linhas
     de dreno), e a consolidação do que se repete devolve parte do crescimento de hoje.
  ⚠️ **Tudo o que a cauda precisa está pronto e nada é perdido:** as metas `meta_p7a/b/c.json` com o `ret_struct`, os
  gates que ela partiria JÁ re-apontados e commitados (o commit de preparação, provado contra o `mod.rs`), o `pre_p7a.sh`
  que troca a entrada do `mod.rs` pela da fase na allowlist de downcast (o `mod.rs` ficará sem downcast nenhum), e as
  provas de mutação dela (`mut_p7a.sh`, `mut_p7c.sh`), mais o driver inteiro (`run_phases2.sh` · `extract_phase.py` ·
  `verbatim.py --selftest` · `mutlib.sh` · `prescan_gates.py` · `brace_preflight.py` · `header_bind_scan.py` ·
  `evasion_probe2.py` + `evasion_accepted.txt`) e os logs do fecho (`fecho/`). ⚠️ **Vivem em
  `Worktrees/line-render-loop/.cauda-render-loop/`, NÃO versionados** (1 608 ficheiros, 51 MB, copiados do scratchpad
  da sessão — que mora num `tmpfs` e morreria no desligar). ⛔ **Antes de `git worktree remove` desta árvore, copie
  essa pasta para fora** — ela é a única cópia, e o `remove` só a apaga com `--force`.
- ⚠️ **A prova de fim de linha dá `ONLY-A = 8`, não `0`, e os 8 são DE PROPÓSITO** (`nextest list --workspace
  --cargo-profile ci-test` sobre o HEAD final contra o merge-base: 22 700 → 22 704 testes, 17 `MOVED`, 12 `ONLY-B`).
  São os testes que mediam SÓ a pilha `ph2d_vec_edit::History`, que morreu no `dcfa2ec4d` (A9: ~40 portas escreviam-na e
  nenhum Ctrl+Z a lia — o undo é a fila global `ProjectUndo`, por diff de estado): `ph2d-vec-edit`
  `history_undo_redo_cycle` · `commit_without_change_is_noop`; `ph2d-app-vec` `every_action_is_exactly_one_undo_step` ·
  `the_whole_gesture_is_one_undo_step`; shell `it` `each_node_op_opens_exactly_one_undo_step_and_only_when_it_changed_something`
  · `the_cut_is_one_undo_step_and_a_miss_cancels_it` · `the_pencil_release_commits_one_undo_step` ·
  `the_undo_session_still_asks_whether_a_drag_is_in_flight`. As asserções vizinhas que perguntavam «um passo» viraram
  a propriedade que o undo global vê (`scene != antes` / `scene == antes`). ⛔ **Um 9.º nome na coluna é defeito.**
- ⚠️ **Auditoria dos ~40 gates que leem o `render_loop/mod.rs` com asserções de AUSÊNCIA ou CONTAGEM**: uma
  ausência medida só no `mod.rs` passa a ser trivialmente verdadeira quando o código sai para uma fase. **Estado no
  fecho (HEAD `447b30838`):** a sonda por teste (`evasion_probe2.py`, com o controlo meio/todo a ver as duas metades)
  acusa **5 candidatos em 77 ficheiros que nomeiam o `mod.rs`, e os 5 são exactamente a lista nomeada dos aceites**
  (`evasion_accepted.txt`, cada um lido e explicado no §8: `later` · `pending_ui_sound = Some` · `gizmo.selection)` ·
  `vec_bindings::resolve` · `offset_live`) ⇒ **0 novos**. ⚠️ O que isto NÃO fecha: a sonda vê agulhas LITERAIS que
  saem do `mod.rs`; um gate que conte por regex, por posição ou por texto construído em runtime fica fora dela, e
  esses foram tratados um a um na cadeia (os `mut_<tag>.sh` de cada fase).
