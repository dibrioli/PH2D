//! **O VOCABULÁRIO DO PAINEL INSPECTOR** — as chaves `panel.inspector.<secção>.…`.
//!
//! ⚠️ **Irmão por ASSUNTO**, como o `painter_layers.rs`: um painel, uma tabela, consultada na cadeia
//! do [`crate::tr`]. Os braços entre os marcadores `ph2d-migrar-texto` são ESCRITOS pelo
//! `scripts/migrar-texto-pintado.py aplicar` a partir do plano; os de FORA deles — as frases com
//! peças do código, lidas por [`crate::tr_with`] com marcadores nomeados — são escritos à mão, e o
//! script não lhes toca.
//!
//! ⭐ A chave nomeia a SECÇÃO que o artista vê, nunca o ficheiro que a pinta
//! (`docs/UI_New_and_Simple/ferramentas/seccoes_inspector.tsv`).
//!
//! ⚠️ **A §14 Platform Player mora no irmão `inspector_player.rs`** — cortada pelo tecto de 700
//! linhas da workspace (esta tabela nasceu com 861). Uma chave `panel.inspector.player.…` escreve-se
//! LÁ (uma passagem do `aplicar` sobre a §14 aponta o `--tabela` para ele), e o gate do painel
//! reprova uma chave que more nas duas ou na metade errada.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── À MÃO (fora dos marcadores) ─────────────────────────────────────────────────────────
        // ⭐⭐ **O NOME DO PAINEL, lido por DUAS superfícies e declarado UMA vez** (2026-09-17): o
        // `Panel::TITLE` é um `TextKey` desde a migração das abas, logo a aba do encaixe e o
        // cabeçalho do próprio painel resolvem esta mesma chave. ⛔ Antes eram dois sítios — um
        // literal no `lib.rs` do painel e um `tr` no pintor — e cinco painéis divergiam.
        "panel.inspector.title" => "Inspector",
        // Os eixos da junta Custom: símbolos, não palavras — entram para a tabela `AXIS_ROWS` ser de
        // um tipo só (`TextKey`), com a `Rotation` ao lado.
        "panel.inspector.joint.axis_x" => "X",
        "panel.inspector.joint.axis_y" => "Y",
        // A camada de ordenação `UI`: sigla de duas letras que a régua não conta, na mesma tabela
        // `LAYER_LABELS` que as quatro palavras ao lado — uma tabela, um tipo.
        "panel.inspector.ordering.ui" => "UI",
        // ── As FRASES com peças do código (`tr_with`, marcadores nomeados) ───────────────────────
        // ⚠️ A frase inteira mora aqui; o código entrega os valores já formatados quando a precisão
        //    é da UI (`{:.1}`), porque o marcador só conhece `Display`.
        "panel.inspector.properties.title_of" => "Properties of \u{201c}{n}\u{201d}",
        // ⭐⭐⭐ **A FRASE DA SELECÇÃO — UMA, do PAINEL.**
        //
        // ⛔⛔⛔ Ela dizia o mesmo em **VINTE E UMA** secções, por **QUATRO** pintores diferentes
        // (`rows::aviso`, um `nota`, o `paint_text` cru e uma cópia própria na `tags`), com
        // quatro redacções. Medido em 2026-09-22 pela porta do produto: **cinco** cópias idênticas
        // no MESMO quadro, mais uma sexta a dizer o mesmo por outras palavras.
        //
        // ⚠️ **Ela leva o NÚMERO porque agora pode**: com vinte e uma cópias ninguém lhe punha o
        // `{n}` (cada secção teria de o formatar), e *«estás a editar só uma»* sem dizer de
        // quantas obriga o artista a contar a selecção.
        "panel.inspector.selection.primary_only" => {
            "{n} selected \u{b7} edits apply to the active object only."
        }
        "panel.inspector.properties.more_not_shown" => "{n} more not shown",
        "panel.inspector.actions.never_fires" => "never fires \u{b7} {verbo} \u{b7} {alvo}",
        "panel.inspector.actions.title_count" => "Signal Actions  ({n})",
        "panel.inspector.anchors.mount_missing" => "{name}  (missing)",
        "panel.inspector.anchors.off_anchor_by" => "Off anchor by {ox}, {oy} px",
        "panel.inspector.anchors.kind_riding" => "{kind} \u{b7} {n} riding",
        "panel.inspector.anchors.title_count" => "Sockets / Anchors  ({n})",
        "panel.inspector.animation.frame_of" => "Frame {n} / {span}",
        "panel.inspector.animation.title_count" => "Animation  ({n})",
        "panel.inspector.audio.listeners_this_one" => {
            "The scene has {n} listeners \u{b7} this is the one in use."
        }
        "panel.inspector.audio.listeners_another_one" => {
            "The scene has {n} listeners \u{b7} another one is in use, not this."
        }
        "panel.inspector.instance.deeper" => "+{n} deeper",
        "panel.inspector.instance.added_more" => {
            "+{left_out} more \u{2014} use Apply to Master on the row"
        }
        "panel.inspector.instance.orphans_more" => {
            "+{left_out} without a button \u{2014} Clear removes those too"
        }
        "panel.inspector.instance.clear_orphans" => "Clear {n} unused override(s)",
        "panel.inspector.instance.removed_more" => {
            "+{left_out} more \u{2014} Revert on the copy puts every piece back"
        }
        "panel.inspector.joint.paste_to" => "Paste to {targets} Joints",
        "panel.inspector.joint.min_unit" => "Min",
        "panel.inspector.joint.max_unit" => "Max",
        "panel.inspector.joint.add_wheel" => "Add Wheel ({n} on this rope)",
        "panel.inspector.joint.target_unit" => "Target",
        "panel.inspector.joint.speed_unit" => "Speed",
        "panel.inspector.joint.axis_min_unit" => "  Min",
        "panel.inspector.joint.axis_max_unit" => "  Max",
        "panel.inspector.physics.bake_range" => "Bake {start}-{end}s to Timeline",
        "panel.inspector.physics.bake_to" => "Bake {end}s to Timeline",
        "panel.inspector.physics.more_shapes" => "+ {n} more shapes from children",
        "panel.inspector.physics.add_shape_to" => "Add Shape to {owner}",
        "panel.inspector.physics.chain_bodies" => "Chain {join_count} Selected Bodies",
        "panel.inspector.physics.rig_parts" => "Rig {rig_parts} Parts from Hierarchy",
        "panel.inspector.physics.shape_of" => "Shape of {owner} \u{00b7} simulated as part of it",
        "panel.inspector.render_source.atlas_key" => "Atlas \u{00b7} key {key}",
        "panel.inspector.render_source.individual_texture" => {
            "Individual \u{00b7} texture {texture_id}"
        }
        "panel.inspector.render_source.hand_packed_label" => "Hand-packed \u{00b7} {label}",
        "panel.inspector.render_source.hand_packed_sheet_region" => {
            "Hand-packed \u{00b7} sheet {sheet} \u{00b7} region {region}"
        }
        "panel.inspector.render_source.size_px" => "{pw} \u{00d7} {ph} px",
        "panel.inspector.timers.summary_mute" => "{s}s \u{b7} {repeat} \u{b7} mute",
        // ── Leituras em MINÚSCULAS que a régua lexical não conta (uma palavra só, sem maiúscula) ───
        // ⚠️ Pintadas no ecrã como VALOR de uma linha — a régua não as vê por desenho (seriam
        //    indistinguíveis de identificadores), e foram achadas a ler o pintor, não o censo.
        "panel.inspector.timers.repeats" => "repeats",
        "panel.inspector.timers.once" => "once",
        "panel.inspector.timers.title_count" => "Timers  ({n})",
        // ── LETRAS SOLTAS, que a régua lexical não conta por CONSTRUÇÃO ────────────────────────
        // ⚠️⚠️ **O `is_language` exige DUAS letras SEGUIDAS** (senão acusaria todo identificador),
        //    logo uma letra sozinha é invisível para ele — e estas estavam pintadas no ecrã com o
        //    censo desta crate VERDE. Quem as achou foi o censo de PORTA (o que segue quem chega a
        //    um pintor), mais a leitura do painel do 9-slice.
        //
        // ⭐⭐ **As cinco do 9-slice são INICIAIS de palavras, e a legenda delas já vivia AQUI:**
        //    `panel.inspector.slice.corners_f_fixed_on_off` explica *«S stretch, R repeat, M
        //    mirror»*. Traduzida a legenda e não as letras, ela passava a explicar letras que a
        //    grelha não mostra — *uma legenda e o que ela explica têm de viajar juntas*.
        // ⚠️ Em ASCII de propósito no inglês (a grelha é pequena e um glifo largo não cabe), e o
        //    `-` entra pela mesma porta: ele é a marca do estado APAGADO, e uma língua pode
        //    escolher outra.
        // ── O que a régua lexical NÃO conta, achado pelo `censo --cegos` (2026-09-18) ──────────
        // O TÍTULO da secção do 9-slice: recusado por começar com DÍGITO (não é Capitalizado
        // nem GRITADO), logo a régua lia-o como identificador.
        "panel.inspector.slice.section_title" => "9-Slice",
        // A direcção «ping-pong» da animação: uma sigla de duas letras ENTRE quatro irmãs que
        // já vinham da tabela.
        "panel.inspector.animation.pp" => "PP",
        // Os dois formatos de precisão, pintados como chips — a mesma lei do
        // `panel.inspector.ordering.ui`: uma tabela, um tipo.
        "panel.inspector.render_source.rgba8" => "RGBA8",
        "panel.inspector.render_source.rgba16" => "RGBA16",
        "panel.inspector.slice.letter_stretch" => "S",
        "panel.inspector.slice.letter_repeat" => "R",
        "panel.inspector.slice.letter_mirror" => "M",
        "panel.inspector.slice.letter_blank" => "-",
        "panel.inspector.slice.letter_fixed" => "F",
        // ⚠️ A célula que não afirma modo nenhum (selecção MISTA) — uma marca, e as línguas não
        //    escolhem todas a mesma.
        "panel.inspector.slice.letter_mixed" => "?",
        // As quatro células da REGIÃO de uma sprite (`Region` do Render Source).
        "panel.inspector.region.x" => "X",
        "panel.inspector.region.y" => "Y",
        "panel.inspector.region.w" => "W",
        "panel.inspector.region.h" => "H",
        // ⭐⭐ **As duas EXPLICAÇÕES que saíram dos rótulos das caixas de âncora** (ordem do dono,
        //    2026-09-19). Elas ficam FORA dos marcadores porque **nunca foram literais pintados**:
        //    nasceram de um rótulo migrado que o dono mandou encurtar, e o
        //    `migrar-texto-pintado.py` só escreve o que encontrou no código.
        //
        // ⛔ Quem as regista é o `populate_anchor`, e há gate a exigir que as duas caixas tenham
        //    balão: *cumprir só a metade que REMOVE apaga a explicação em silêncio.*
        "panel.inspector.anchors.bounds_explica" => "Makes this anchor a Slice.",
        "panel.inspector.anchors.center_explica" => "Makes this anchor a 9-slice Region.",
        // ph2d-migrar-texto:begin
        "panel.inspector.panel.no_properties_yet_for_the" => {
            "No properties yet for the selected entity."
        }
        "panel.inspector.panel.select_an_entity_in_the" => {
            "Select an entity in the Hierarchy to inspect its properties."
        }
        "panel.inspector.panel.inspector" => "Inspector",
        "panel.inspector.panel.add_component" => "Add Component",
        "panel.inspector.sample.front" => "Front",
        "panel.inspector.sample.side" => "Side",
        "panel.inspector.sample.top" => "Top",
        "panel.inspector.sample.view" => "View",
        "panel.inspector.ordering.default" => "Default",
        "panel.inspector.anchors.parked_this_app_has_no" => {
            "Parked: this app has no game runtime yet, so there is no runtime for anchors to show in. \
         The setting is kept in the file so saved projects still load."
        }
        "panel.inspector.actions.this_object" => "(this object)",
        "panel.inspector.actions.plus_add_action" => "+ Add Action",
        "panel.inspector.actions.x_remove_action" => "x Remove Action",
        "panel.inspector.actions.on_signal" => "on signal\u{2026}",
        "panel.inspector.actions.target_empty_this_object" => "target (empty = this object)",
        "panel.inspector.actions.timer_name_empty_all" => "timer name (empty = all)",
        "panel.inspector.actions.count_empty_one" => "amount to add (empty = 1)",
        "panel.inspector.actions.amount_of_life" => "amount (e.g. 10)",
        "panel.inspector.actions.this_action_never_runs_it" => {
            "This action never runs: it has no signal name."
        }
        "panel.inspector.actions.signal_actions" => "Signal Actions",
        "panel.inspector.actions.no_actions_yet" => "No actions yet.",
        "panel.inspector.anchors.rides_parent_anchor" => "Rides Parent Anchor",
        "panel.inspector.anchors.the_parent_has_no_anchor" => {
            "The parent has no anchor with that name."
        }
        "panel.inspector.anchors.reset_to_anchor" => "Reset to Anchor",
        // ⛔⛔ **ENCURTADO por ordem do dono (2026-09-19)**, com a medição à frente dele: o
        //    parêntesis punha o rótulo em `~190 px` numa coluna de `174`, e ele saía
        //    `Show anchors at runtime (n…`. A razão de a caixa estar PARADA **não se perdeu** —
        //    ela já vivia no balão (`parked_this_app_has_no`, registado no `populate_anchor`).
        //    ⚠️ Isto INVERTE o que o doc do `RUNTIME_BOX_LABEL` argumentava (*«a razão vai no
        //    RÓTULO e não só na dica»*); aquele doc diz agora que a decisão foi revista.
        "panel.inspector.anchors.show_anchors_at_runtime_no" => "Show anchors at runtime",
        "panel.inspector.anchors.always_show_anchors" => "Always show anchors",
        "panel.inspector.anchors.anchor_name" => "anchor_name\u{2026}",
        "panel.inspector.anchors.rotation_deg" => "Rotation",
        // ⛔⛔ **As duas ENCURTARAM por ordem do dono (2026-09-19)** — o `Center` media `~190 px`
        //    numa coluna de `174` e saía `Center (makes it a 9-slice R…`. ⭐ A consequência de
        //    cada uma mudou-se para o BALÃO (`..._explica`, registados no `populate_anchor`), e o
        //    `Bounds` acompanha o irmão mesmo sem estar a cortar: *duas caixas irmãs com feitios
        //    diferentes leem-se como duas coisas diferentes.*
        //    ⚠️ A CHAVE guarda a redacção antiga (`..._makes_it_a_9`) de propósito: ela é escrita
        //    pelo `migrar-texto-pintado.py` a partir do literal original, e renomeá-la à mão
        //    descolaria a tabela do plano que a gerou.
        "panel.inspector.anchors.bounds_makes_it_a_slice" => "Bounds",
        "panel.inspector.anchors.center_makes_it_a_9" => "Center",
        "panel.inspector.anchors.x_remove_anchor" => "x Remove Anchor",
        "panel.inspector.anchors.this_object_s_anchors" => "This object's anchors",
        "panel.inspector.anchors.no_anchors_on_this_sprite" => "No anchors on this sprite.",
        "panel.inspector.anchors.plus_add_anchor" => "+ Add Anchor",
        "panel.inspector.animation.playing" => "Playing",
        "panel.inspector.animation.autoplay" => "Autoplay",
        "panel.inspector.animation.speed_x" => "Speed (x)",
        "panel.inspector.animation.direction_override" => "Direction override",
        "panel.inspector.animation.inherit" => "Inherit",
        "panel.inspector.animation.fwd" => "Fwd",
        "panel.inspector.animation.rev" => "Rev",
        "panel.inspector.animation.pp_rev" => "PP Rev",
        "panel.inspector.animation.loop_override" => "Loop override",
        "panel.inspector.animation.on" => "On",
        "panel.inspector.animation.off" => "Off",
        "panel.inspector.animation.this_frame_ms_0_use" => "This frame",
        "panel.inspector.animation.this_frame_hint" => "0 = use the Frame value.",
        "panel.inspector.animation.repeat_hint" => "0 = repeat forever.",
        "panel.inspector.animation.this_sprite_has_no_animation" => {
            "This sprite has no animation with that name, or the grid shrank under it."
        }
        "panel.inspector.animation.rewind" => "Rewind",
        "panel.inspector.animation.animation" => "Animation",
        "panel.inspector.animation.this_sprite_does_not_play" => {
            "This sprite does not play animations yet."
        }
        "panel.inspector.animation.plus_add_animator" => "+ Add Animator",
        "panel.inspector.animation.this_sprite_s_animations" => "This sprite's animations",
        "panel.inspector.animation.no_animations_yet" => "No animations yet.",
        "panel.inspector.animation.out_of_grid" => "out of grid",
        "panel.inspector.animation.plus_add_animation" => "+ Add Animation",
        "panel.inspector.animation.x_remove_animation" => "x Remove Animation",
        "panel.inspector.animation.reverse" => "Reverse",
        "panel.inspector.animation.ping_pong" => "Ping-Pong",
        "panel.inspector.animation.ping_pong_rev" => "Ping-Pong Rev",
        "panel.inspector.animation.forward" => "Forward",
        "panel.inspector.animation.animation_name" => "animation_name\u{2026}",
        "panel.inspector.animation.this_animation_has_per_frame" => {
            "\u{2022} this animation has per-frame timing (imported)"
        }
        "panel.inspector.animation.direction" => "Direction",
        "panel.inspector.animation.signals_empty_silent" => "Signals (empty = silent)",
        "panel.inspector.animation.on_finish" => "on finish\u{2026}",
        "panel.inspector.animation.on_loop" => "on loop\u{2026}",
        "panel.inspector.audio.browse" => "Browse\u{2026}",
        "panel.inspector.audio.preview" => "Preview",
        "panel.inspector.audio.stop" => "Stop",
        "panel.inspector.audio.sound_file" => "sound file\u{2026}",
        "panel.inspector.audio.no_sound_file_yet_use" => {
            "No sound file yet \u{2014} use Browse to pick one."
        }
        "panel.inspector.audio.that_file_is_gone_pick" => {
            "That file is gone \u{2014} pick it again."
        }
        "panel.inspector.audio.nothing_plays_this_autoplay_is" => {
            "Nothing plays this: Autoplay is off and no Signal Action targets it."
        }
        "panel.inspector.audio.no_audio_listener_2d_in" => {
            "No Audio Listener 2D in the scene \u{2014} sound plays with no position."
        }
        "panel.inspector.audio.volume_db" => "Volume (dB)",
        "panel.inspector.audio.pitch" => "Pitch",
        "panel.inspector.audio.max_distance_m" => "Max Distance",
        "panel.inspector.audio.attenuation" => "Attenuation",
        "panel.inspector.audio.non_spatialized_radius_m" => "Non-Spatialized Radius",
        "panel.inspector.audio.panning_strength" => "Panning Strength",
        "panel.inspector.audio.max_polyphony" => "Max Polyphony",
        "panel.inspector.audio.loop" => "Loop",
        "panel.inspector.audio.autoplay" => "Autoplay",
        "panel.inspector.audio.audio" => "Audio",
        "panel.inspector.audio.these_are_the_scene_s" => {
            "These are the scene's ears \u{2014} sound is heard from here."
        }
        "panel.inspector.camera.active_is_off_this_camera" => {
            "Active is off \u{2014} this camera never takes the view."
        }
        "panel.inspector.camera.another_camera_commands_raise_priority" => {
            "Another camera commands \u{2014} raise Priority to take the view."
        }
        "panel.inspector.camera.height_m" => "Height",
        "panel.inspector.camera.offset_m" => "Offset",
        "panel.inspector.camera.priority" => "Priority",
        "panel.inspector.camera.dolly" => "Dolly",
        "panel.inspector.camera.active" => "Active",
        "panel.inspector.camera.look_through" => "Look Through",
        "panel.inspector.camera.object_name" => "object name\u{2026}",
        "panel.inspector.camera.nothing_in_the_scene_has" => {
            "Nothing in the scene has that name \u{2014} the camera stays put."
        }
        "panel.inspector.camera.dead_zone" => "Dead Zone",
        "panel.inspector.camera.lookahead_s" => "Lookahead",
        "panel.inspector.camera.follow_offset_m" => "Follow Offset",
        "panel.inspector.camera.limits_are_smaller_than_the" => {
            "Limits are smaller than the view \u{2014} the camera pins to their centre."
        }
        "panel.inspector.camera.min_m" => "Min",
        "panel.inspector.camera.max_m" => "Max",
        "panel.inspector.camera.cull_mask" => "Cull Mask",
        "panel.inspector.camera.camera" => "Camera",
        "panel.inspector.color_tint.color_and_tint" => "Color & Tint",
        "panel.inspector.color_tint.tint" => "Tint",
        "panel.inspector.color_tint.self_tint" => "Self Tint",
        "panel.inspector.color_tint.opacity" => "Opacity",
        "panel.inspector.color_tint.tint_fill" => "Tint Fill",
        // ⭐ **Encurtado por ordem do dono (2026-09-21)** — ele escreveu-o assim no desenho dele.
        // ⛔⛔ E aqui isso NÃO é cosmética: medido, o nome comprido é mais largo do que METADE do
        // painel, logo ele come a coluna do controlo e as quatro amostras do per-corner saíam a
        // `35 px`. *A explicação vive no balão do controlo; o nome vive na coluna.*
        "panel.inspector.color_tint.per_corner_tint_vertex_gradient" => "Per-corner Tint",
        "panel.inspector.color_tint.per_corner_tint_hint" => {
            "Each corner of the quad gets its own tint; the renderer blends between them."
        }
        "panel.inspector.color_tint.top_left_corner_tint" => "Top-left corner tint",
        "panel.inspector.color_tint.top_right_corner_tint" => "Top-right corner tint",
        "panel.inspector.color_tint.bottom_left_corner_tint" => "Bottom-left corner tint",
        "panel.inspector.color_tint.bottom_right_corner_tint" => "Bottom-right corner tint",
        "panel.inspector.color_tint.equalize_corners" => "Equalize Corners",
        "panel.inspector.render_source.emissive" => "Emissive",
        "panel.inspector.identity.name" => "Name\u{2026}",
        "panel.inspector.identity.visible" => "Visible",
        "panel.inspector.instance.edit_prefab" => "Edit Prefab",
        "panel.inspector.joint.pin" => "Pin",
        "panel.inspector.joint.spring" => "Spring",
        "panel.inspector.joint.rope" => "Rope",
        "panel.inspector.joint.weld" => "Weld",
        "panel.inspector.joint.slider" => "Slider",
        "panel.inspector.joint.rod" => "Rod",
        "panel.inspector.joint.wheel" => "Wheel",
        "panel.inspector.joint.pulley" => "Pulley",
        "panel.inspector.joint.custom" => "Custom",
        "panel.inspector.joint.off" => "Off",
        "panel.inspector.joint.on" => "On",
        "panel.inspector.joint.paste_properties" => "Paste Properties",
        "panel.inspector.joint.travel" => "Travel",
        "panel.inspector.joint.limits" => "Limits",
        "panel.inspector.joint.rigid" => "Rigid",
        "panel.inspector.joint.soft" => "Soft",
        "panel.inspector.joint.velocity" => "Velocity",
        "panel.inspector.joint.position" => "Position",
        "panel.inspector.joint.physics_joint" => "Physics Joint",
        "panel.inspector.joint.active" => "Active",
        "panel.inspector.joint.kind" => "Kind",
        "panel.inspector.joint.copy_properties" => "Copy Properties",
        "panel.inspector.joint.delete_joint" => "Delete Joint",
        "panel.inspector.joint.rest_length_m" => "Rest Length",
        "panel.inspector.joint.stiffness" => "Stiffness",
        "panel.inspector.joint.damping" => "Damping",
        "panel.inspector.joint.length_m" => "Length",
        "panel.inspector.joint.rope_length_m" => "Rope Length",
        "panel.inspector.joint.max_length_m" => "Max Length",
        "panel.inspector.joint.motor" => "Motor",
        "panel.inspector.joint.mode" => "Mode",
        "panel.inspector.joint.max_force" => "Max Force",
        "panel.inspector.joint.breakable" => "Breakable",
        "panel.inspector.joint.break_force_n" => "Break Force",
        "panel.inspector.joint.break_torque_n_m" => "Break Torque",
        "panel.inspector.joint.rotation" => "Rotation",
        "panel.inspector.joint.free" => "Free",
        "panel.inspector.joint.limited" => "Limited",
        "panel.inspector.joint.locked" => "Locked",
        "panel.inspector.joint.motor_axis" => "Motor Axis",
        "panel.inspector.joint.anchor_b" => "Anchor B",
        "panel.inspector.joint.collide" => "Collide",
        "panel.inspector.joint.object" => "Object",
        "panel.inspector.joint.world" => "World",
        "panel.inspector.joint.body_a" => "Body A",
        "panel.inspector.joint.body_b" => "Body B",
        "panel.inspector.joint.missing" => "(missing)",
        "panel.inspector.material.mix" => "Mix",
        "panel.inspector.material.add" => "Add",
        "panel.inspector.material.subtract" => "Subtract",
        "panel.inspector.material.multiply" => "Multiply",
        "panel.inspector.material.screen" => "Screen",
        "panel.inspector.material.premult" => "Premult",
        "panel.inspector.material.material_and_blend" => "Material & Blend",
        "panel.inspector.material.blend_mode" => "Blend Mode",
        "panel.inspector.material.material_default_shader_runtime_pending" => {
            "Material: Default \u{00b7} shader runtime pending"
        }
        "panel.inspector.render_source.storage" => "Storage",
        "panel.inspector.render_source.source" => "Source",
        "panel.inspector.ordering.background" => "Background",
        "panel.inspector.ordering.midground" => "Midground",
        "panel.inspector.ordering.foreground" => "Foreground",
        "panel.inspector.ordering.sorting_layer" => "Sorting Layer",
        "panel.inspector.ordering.center" => "Center",
        "panel.inspector.ordering.pivot" => "Pivot",
        "panel.inspector.ordering.custom" => "Custom",
        "panel.inspector.ordering.sort_point" => "Sort Point",
        "panel.inspector.ordering.axis_x" => "Axis X",
        "panel.inspector.ordering.axis_y" => "Axis Y",
        "panel.inspector.ordering.ordering" => "Ordering",
        "panel.inspector.physics.ball" => "Ball",
        "panel.inspector.physics.box" => "Box",
        "panel.inspector.physics.capsule" => "Capsule",
        "panel.inspector.physics.physics_body" => "Physics Body",
        "panel.inspector.physics.radius_m" => "Radius",
        "panel.inspector.physics.half_height_m" => "Half Height",
        "panel.inspector.physics.half_width_m" => "Half Width",
        "panel.inspector.physics.dynamic" => "Dynamic",
        "panel.inspector.physics.static" => "Static",
        "panel.inspector.physics.kinematic" => "Kinematic",
        "panel.inspector.physics.discrete" => "Discrete",
        "panel.inspector.physics.continuous" => "Continuous",
        "panel.inspector.physics.free" => "Free",
        "panel.inspector.physics.locked" => "Locked",
        "panel.inspector.physics.all" => "All",
        "panel.inspector.physics.position" => "Position",
        "panel.inspector.physics.rotation" => "Rotation",
        "panel.inspector.physics.body" => "Body",
        "panel.inspector.physics.collider" => "Collider",
        "panel.inspector.physics.plus_1_more_shape_from" => "+ 1 more shape from a child",
        "panel.inspector.physics.offset_x_m" => "Offset X",
        "panel.inspector.physics.offset_y_m" => "Offset Y",
        "panel.inspector.physics.gravity_scale" => "Gravity Scale",
        "panel.inspector.physics.dominance" => "Dominance",
        "panel.inspector.physics.collision" => "Collision",
        "panel.inspector.physics.freeze_x" => "Freeze X",
        "panel.inspector.physics.freeze_y" => "Freeze Y",
        "panel.inspector.physics.bake" => "Bake",
        "panel.inspector.physics.remove_physics_body" => "Remove Physics Body",
        "panel.inspector.physics.not_simulated_add_a_body" => {
            "Not simulated \u{00b7} add a body to make it fall and collide"
        }
        "panel.inspector.physics.add_physics_body" => "Add Physics Body",
        "panel.inspector.physics.make_independent_body" => "Make Independent Body",
        "panel.inspector.physics.remove_shape" => "Remove Shape",
        "panel.inspector.physics.pin" => "Pin",
        "panel.inspector.physics.spring" => "Spring",
        "panel.inspector.physics.rope" => "Rope",
        "panel.inspector.physics.weld" => "Weld",
        "panel.inspector.physics.slider" => "Slider",
        "panel.inspector.physics.rod" => "Rod",
        "panel.inspector.physics.wheel" => "Wheel",
        "panel.inspector.physics.pulley" => "Pulley",
        "panel.inspector.physics.custom" => "Custom",
        "panel.inspector.physics.join_as" => "Join As",
        "panel.inspector.physics.cancel_joint_drawing" => "Cancel Joint Drawing",
        "panel.inspector.physics.draw_joint_on_canvas" => "Draw Joint on Canvas",
        "panel.inspector.physics.join_selected_bodies" => "Join Selected Bodies",
        "panel.inspector.physics.shape_with_no_body_above" => {
            "Shape with no body above it \u{00b7} not simulated"
        }
        "panel.inspector.physics.density" => "Density",
        "panel.inspector.physics.auto" => "Auto",
        "panel.inspector.physics.manual" => "Manual",
        "panel.inspector.physics.average" => "Average",
        "panel.inspector.physics.min" => "Min",
        "panel.inspector.physics.multiply" => "Multiply",
        "panel.inspector.physics.max" => "Max",
        "panel.inspector.physics.combine" => "Combine",
        "panel.inspector.physics.replace" => "Replace",
        "panel.inspector.physics.linear_damping" => "Linear Damping",
        "panel.inspector.physics.angular_damping" => "Angular Damping",
        "panel.inspector.physics.damp_mode" => "Damp Mode",
        "panel.inspector.physics.bounce" => "Bounce",
        "panel.inspector.physics.friction" => "Friction",
        "panel.inspector.physics.bounce_combine" => "Bounce Combine",
        "panel.inspector.physics.friction_combine" => "Friction Combine",
        "panel.inspector.physics.mass" => "Mass",
        "panel.inspector.physics.mass_kg" => "Mass (kg)",
        "panel.inspector.physics.solid" => "Solid",
        "panel.inspector.physics.sensor" => "Sensor",
        "panel.inspector.physics.off" => "Off",
        "panel.inspector.physics.on" => "On",
        "panel.inspector.physics.zone" => "Zone",
        "panel.inspector.physics.world" => "World",
        "panel.inspector.physics.layer" => "Layer",
        "panel.inspector.physics.trigger" => "Trigger",
        "panel.inspector.physics.one_way" => "One-Way",
        "panel.inspector.physics.wall_cling" => "Wall Cling",
        "panel.inspector.physics.grip" => "Grip",
        "panel.inspector.physics.signal_on_hit" => "Signal on hit\u{2026}",
        "panel.inspector.physics.signal_on_leave" => "Signal on leave\u{2026}",
        "panel.inspector.physics.force_x_n" => "Force X",
        "panel.inspector.physics.force_y_n" => "Force Y",
        "panel.inspector.physics.force_axes" => "Force Axes",
        "panel.inspector.physics.torque_n_m" => "Torque (N·m)",
        "panel.inspector.physics.falloff" => "Falloff",
        "panel.inspector.physics.drag" => "Drag",
        "panel.inspector.physics.fluid_density" => "Fluid Density",
        "panel.inspector.physics.shape_drag" => "Shape Drag",
        "panel.inspector.properties.variant" => "Variant",
        "panel.inspector.properties.properties" => "Properties",
        "panel.inspector.render_source.render_source" => "Render Source",
        "panel.inspector.render_source.strategy" => "Strategy",
        "panel.inspector.render_source.from_the_asset_pipeline_read" => {
            "From the asset pipeline \u{00b7} read-only"
        }
        "panel.inspector.render_source.atlas" => "Atlas",
        "panel.inspector.render_source.individual" => "Individual",
        "panel.inspector.render_source.hand_packed" => "Hand-packed",
        "panel.inspector.render_source.cooked_texture" => "Cooked texture",
        "panel.inspector.render_source.region" => "Region",
        "panel.inspector.render_source.filter_clip" => "Filter Clip",
        "panel.inspector.render_source.format" => "Format",
        "panel.inspector.render_source.rgba16_doubles_memory_forces_individual" => {
            "RGBA16 doubles memory, forces Individual, and leaves the sheet"
        }
        "panel.inspector.render_source.rgba16_doubles_memory_and_forces" => {
            "RGBA16 doubles memory and forces Individual"
        }
        "panel.inspector.sampling.inherit" => "Inherit",
        "panel.inspector.sampling.nearest" => "Nearest",
        "panel.inspector.sampling.linear" => "Linear",
        "panel.inspector.sampling.near_plus_mip" => "Near+Mip",
        "panel.inspector.sampling.lin_plus_mip" => "Lin+Mip",
        "panel.inspector.sampling.lin_plus_aniso" => "Lin+Aniso",
        "panel.inspector.sampling.clamp" => "Clamp",
        "panel.inspector.sampling.repeat" => "Repeat",
        "panel.inspector.sampling.mirror" => "Mirror",
        "panel.inspector.sampling.sampling" => "Sampling",
        "panel.inspector.sampling.texture_filter" => "Texture Filter",
        "panel.inspector.sampling.texture_repeat" => "Texture Repeat",
        "panel.inspector.sampling.uv_scale" => "UV Scale",
        "panel.inspector.sampling.uv_offset" => "UV Offset",
        "panel.inspector.slice.per_region_tiling" => "Per-region tiling",
        "panel.inspector.slice.tile_all" => "Tile all",
        "panel.inspector.slice.stretch_all" => "Stretch all",
        "panel.inspector.slice.continuous" => "Continuous",
        "panel.inspector.slice.whole" => "Whole",
        "panel.inspector.slice.whole_entire_tiles_so_the" => {
            "Whole = entire tiles, so the last one meets the border. \
                          Mirror always uses whole tiles."
        }
        "panel.inspector.slice.tile_mode" => "Tile Mode",
        "panel.inspector.slice.enable_9_slice" => "Enable 9-slice",
        "panel.inspector.slice.fill_center" => "Fill Center",
        "panel.inspector.sprite_sheet.sprite_sheet" => "Sprite Sheet",
        "panel.inspector.sprite_sheet.centered" => "Centered",
        "panel.inspector.sprite_sheet.offset_x" => "Offset X",
        "panel.inspector.sprite_sheet.offset_y" => "Offset Y",
        "panel.inspector.sprite_sheet.flip_h" => "Flip H",
        "panel.inspector.sprite_sheet.flip_v" => "Flip V",
        "panel.inspector.sprite_sheet.h_frames" => "H Frames",
        "panel.inspector.sprite_sheet.v_frames" => "V Frames",
        "panel.inspector.sprite_sheet.frame" => "Frame",
        "panel.inspector.sprite_sheet.show_sheet_on_canvas" => "Show sheet on canvas",
        "panel.inspector.timers.plus_add_timer" => "+ Add Timer",
        "panel.inspector.timers.x_remove_timer" => "x Remove Timer",
        "panel.inspector.timers.timer_name" => "timer_name\u{2026}",
        "panel.inspector.timers.duration_seconds" => "Duration",
        "panel.inspector.timers.repeat" => "Repeat",
        "panel.inspector.timers.autostart" => "Autostart",
        "panel.inspector.timers.signal_name_empty_mute" => "signal_name (empty = mute)",
        "panel.inspector.timers.this_timer_never_fires_duration" => {
            "This timer never fires: duration is zero."
        }
        "panel.inspector.timers.this_timer_never_starts_autostart" => {
            "This timer never starts: Autostart is off."
        }
        "panel.inspector.timers.this_timer_runs_but_says" => {
            "This timer runs but says nothing: the signal name is empty."
        }
        "panel.inspector.timers.timers" => "Timers",
        "panel.inspector.timers.no_timers_yet" => "No timers yet.",
        "panel.inspector.transform.position_m" => "Position X / Y",
        "panel.inspector.transform.position_px" => "Position X / Y",
        "panel.inspector.transform.scale" => "Scale X / Y",
        "panel.inspector.transform.transform" => "Transform",
        "panel.inspector.transform.rotation" => "Rotation",
        "panel.inspector.transform.skew" => "Skew X / Y",
        "panel.inspector.visibility.visibility_layer" => "Visibility Layer",
        "panel.inspector.visibility.clip_children" => "Clip Children",
        "panel.inspector.visibility.disabled" => "Disabled",
        "panel.inspector.visibility.clip" => "Clip",
        "panel.inspector.visibility.clip_plus_draw" => "Clip+Draw",
        "panel.inspector.visibility.mask_interaction" => "Mask Interaction",
        "panel.inspector.visibility.none" => "None",
        "panel.inspector.visibility.inside" => "Inside",
        "panel.inspector.visibility.outside" => "Outside",
        "panel.inspector.visibility.mask_alpha_cutoff" => "Mask Alpha Cutoff",
        "panel.inspector.visibility.mask_source_mask2d" => "Mask Source (Mask2D)",
        "panel.inspector.visibility.on_screen_enabler" => "On-Screen Enabler",
        "panel.inspector.visibility.enabler_rect" => "Enabler Rect",
        "panel.inspector.wheel.auto" => "Auto",
        "panel.inspector.wheel.over" => "Over",
        "panel.inspector.wheel.under" => "Under",
        "panel.inspector.wheel.drum" => "Drum",
        "panel.inspector.wheel.weston" => "Weston",
        "panel.inspector.wheel.pulley_wheel" => "Pulley Wheel",
        "panel.inspector.wheel.radius_m" => "Radius",
        "panel.inspector.wheel.out_radius_m" => "Out Radius",
        "panel.inspector.wheel.differential" => "Differential",
        "panel.inspector.wheel.order" => "Order",
        "panel.inspector.wheel.wrap" => "Wrap",
        "panel.inspector.wheel.axle_breaks" => "Axle Breaks",
        "panel.inspector.wheel.off" => "Off",
        "panel.inspector.wheel.on" => "On",
        "panel.inspector.wheel.break_force_n" => "Break Force",
        "panel.inspector.wheel.mounted_on" => "Mounted On",
        "panel.inspector.wheel.scenery" => "(scenery)",
        "panel.inspector.wheel.gear" => "Gear",
        "panel.inspector.wheel.rope" => "Rope",
        "panel.inspector.wheel.no_rope" => "(no rope)",
        "panel.inspector.anchors.position_x_y_px" => "Position X / Y",
        "panel.inspector.anchors.bounds_x_y_w_h" => "Bounds X / Y / W / H",
        "panel.inspector.anchors.center_x_y_w_h" => "Center X / Y / W / H",
        "panel.inspector.anchors.sockets_anchors" => "Sockets / Anchors",
        "panel.inspector.animation.from_cell" => "From (cell)",
        "panel.inspector.animation.to_cell" => "To (cell)",
        "panel.inspector.animation.frame" => "Frame",
        // ⭐ O nome ficou CURTO e a regra foi para o BALÃO (ordem do dono, 2026-09-21:
        //    *«os nomes grandes precisam reduzir, as dicas devem ir para o mouse Hover»*).
        //    ⚠️ A chave mantém o nome antigo de propósito: renomeá-la custa a prosa que a cita.
        "panel.inspector.animation.repeat_forever" => "Repeat",
        "panel.inspector.animation.hold" => "Hold",
        "panel.inspector.animation.repeat_delay" => "Repeat delay",
        "panel.inspector.camera.damping_1_s" => "Damping",
        "panel.inspector.joint.swap_a_b" => "Swap A / B",
        "panel.inspector.physics.init_vel_x_m_s" => "Init Vel X",
        "panel.inspector.physics.init_vel_y_m_s" => "Init Vel Y",
        "panel.inspector.physics.init_spin_deg_s" => "Init Spin",
        "panel.inspector.physics.belt_m_s" => "Belt",
        "panel.inspector.render_source.reimport_at_current_px_m" => "Reimport at current px/m",
        "panel.inspector.slice.corners_f_fixed_on_off" => {
            "Corners F fixed (on/off). Edges + centre: S stretch, R repeat, \
                             M mirror, - blank."
        }
        "panel.inspector.slice.borders_l_t_px" => "Borders L / T",
        "panel.inspector.slice.borders_r_b_px" => "Borders R / B",
        "panel.inspector.slice.size_x_y_m_0" => "Size X / Y",
        "panel.inspector.slice.size_hint" => "0 = use the sprite size.",
        "panel.inspector.wheel.motor_s" => "Motor",
        // ⭐⭐⭐ **OS NOMES das linhas de TEXTO** — report do dono, 2026-09-22: *«campos de texto
        //    difíceis de saber para que servem»*. A chave irmã sem `_label` é o EXEMPLO que a caixa
        //    mostra enquanto está VAZIA; esta é o NOME, que fica na coluna da esquerda para sempre.
        "panel.inspector.actions.on_label" => "On",
        "panel.inspector.actions.arg_label" => "Argument",
        "panel.inspector.actions.target_label" => "Target",
        "panel.inspector.animation.name_label" => "Name",
        "panel.inspector.animation.on_finish_label" => "On Finish",
        "panel.inspector.animation.on_loop_label" => "On Loop",
        "panel.inspector.audio.sound_label" => "Sound",
        "panel.inspector.camera.target_label" => "Target",
        "panel.inspector.timers.name_label" => "Name",
        "panel.inspector.timers.signal_label" => "Signal",
        "panel.inspector.anchors.name_label" => "Name",
        "panel.inspector.physics.on_hit_label" => "On Hit",
        "panel.inspector.physics.on_leave_label" => "On Leave",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
