//! **AS STRINGS DO PAINEL TIMELINE** — `panel.timeline.*`.
//!
//! ⚠️ **Um corte por ASSUNTO, e por TECTO DE LOC** (integração de 2026-09-20): o `lib.rs` estava em
//! `696` de `700` no `main` e a `line/Vector` acrescentou-lhe as cinco chaves das alças do osso que
//! arqueia ⇒ `704`. ⛔ Nenhum dos dois lados o estoura sozinho — *um tecto por-ficheiro é a única
//! grandeza deste repo que SOMA entre linhas sem ninguém a contar* (`CLAUDE.md` §5.0) —, e a cura
//! de um tecto vermelho é **corte por responsabilidade**, nunca uma entrada nova no
//! `FILE_OVERAGE_OK`.
//!
//! ⭐ O corte é o que os irmãos já fizeram (`vector.rs`, `grid_snap.rs`, …): *ali
//! ficam as chaves do APP, aqui as de UM painel*. Elas saem VERBATIM — nenhuma palavra muda.

/// A tradução de uma chave do painel Timeline, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Timeline panel (W2.E9) — dope-sheet + graph editor + transport chrome.
        // English by canon (feedback_app_ui_english_only); routed through tr() so
        // the strings live in one table for the eventual Fluent migration.
        "panel.timeline.title" => "Timeline",
        "panel.timeline.dock" => "Snap back to its place",
        "panel.timeline.summary" => "Summary",
        // A linha das etapas da macro do clipe activo (ADR-0186, W5d).
        "panel.timeline.macro_steps" => "Macro steps",
        "panel.timeline.add_track" => "+ Track  \u{25be}",
        // Em cartões: a seta é o ícone da lista.
        "panel.timeline.add_track_lista" => "+ Track",
        "panel.timeline.add_lane" => "+ Lane",
        // The lane's eye: open = the lane plays, closed = it is left out of the animation.
        "panel.timeline.no_lugar.same_piece" => "Same piece: what you change here changes here too",
        "panel.timeline.lane_mute" => "Mute this lane (open eye: it plays)",
        "panel.timeline.lane_unmute" => "Unmute this lane (closed eye: it is left out)",
        "panel.timeline.add_container" => "+ Container",
        "panel.timeline.crumb_root" => "Scene",
        // The «Animation of» chip (ObjectsAndComponents 01 §6): whose animation the panel shows.
        "panel.timeline.animation_of" => "Animation: {name}",
        "panel.timeline.animation_add" => "Add animation to {name}",
        "panel.timeline.animation_remove" => "Remove animation from {name}",
        // **Where the open container's interior actually plays, in SCENE seconds.**
        // Inside a container the ruler counts the INTERIOR while the transport chip keeps
        // showing the scene's second, so two different numbers sit on screen at once. The
        // research found that no product labels its ruler at all — this readout is what
        // makes the two legible as one arithmetic instead of a contradiction.
        "panel.timeline.host_window" => "plays",
        // ...and when it does not play at the current second, which is WHY the playhead is
        // absent rather than broken.
        "panel.timeline.host_not_playing" => "not playing here",
        // ...and when it plays NOWHERE, which is a different fact: the container exists and
        // is authorable, but no strip instances it, so the ruler is counting its own seconds
        // rather than relating them to any. Printing "plays 0.00 - 3.00" here would label the
        // container's own axis with the scene's name.
        "panel.timeline.host_not_placed" => "not placed",
        // The Keys tab under a stack: its ruler moves the clip, and the scene (the Motion, the
        // physics) cannot follow — the clip has no single scene second (ADR-0115 R8 emenda 3).
        "panel.timeline.scene_held_not_arranged" => "scene stays: clip not in Arrange",
        "panel.timeline.scene_held_plays_twice" => "scene stays: clip plays 2x in Arrange",
        "panel.timeline.add_marker" => "+M",
        "panel.timeline.time_seconds" => "Time(s)",
        "panel.timeline.length" => "Dur(s)",
        "panel.timeline.frame" => "Frame",
        "panel.timeline.card.time" => "Time (s)",
        // A barra no molde do Rive (docs/Timeline/15, onda 1): os nomes dizem o que o controlo FAZ.
        "panel.timeline.card.clips" => "Clips",
        "panel.timeline.card.more" => "+{n} more",
        // Onda 3 (docs/Timeline/16): onde um container é usado, no chip e na lista.
        "panel.timeline.card.used_times" => "used {n}×",
        "panel.timeline.card.not_placed" => "not placed",
        "panel.timeline.card.containers" => "Containers",
        "panel.timeline.card.fps" => "{fps} fps · project",
        "panel.timeline.card.record" => "Record while playing",
        "panel.timeline.card.add_marker" => "Marker",
        // onda 1 — trecho (doc 17, item 2): o chip que toca só o pedaço entre dois marcadores.
        "panel.timeline.stretch.label" => "{from} → {to}",
        // A faixa própria dos marcadores no pé da lista (maquete B6).
        "panel.timeline.markers_row" => "Markers",
        // A leitura do marcador agarrado: o nome e o tempo juntos, no lugar do nome (dono 10/11).
        "panel.timeline.marker_readout" => "{name} {time}",
        "panel.timeline.tip.stretch" => {
            "Playing only this stretch, in a loop. Click to play the whole clip again"
        }
        "panel.timeline.card.length_long" => "Length (s)",
        "panel.timeline.card.end_stop" => "Stop",
        "panel.timeline.card.snap" => "Snap to frames",
        "panel.timeline.card.physics" => "Run physics during Play",
        "panel.timeline.card.motion_path" => "A position key is a point on the path",
        "panel.timeline.card.speed" => "Show speed in the curves",
        "panel.timeline.card.clips_as_list" => "Show clips as a list",
        "panel.timeline.card.onion" => "Ghosts",
        "panel.timeline.card.onion_keys" => "Only at keys",
        "panel.timeline.card.onion_settings" => "Ghost settings…",
        "panel.timeline.card.rename" => "Rename",
        "panel.timeline.card.duplicate" => "Duplicate",
        "panel.timeline.card.reverse" => "Reverse keys",
        "panel.timeline.card.starts_with" => "Starts with this clip",
        "panel.timeline.card.delete" => "Delete",
        "panel.timeline.card.legend_curve" => "Curve",
        "panel.timeline.card.legend_linear" => "Linear",
        "panel.timeline.card.legend_hold" => "Hold",
        "panel.timeline.tip.options" => "Timeline options",
        "panel.timeline.tip.time" => "Time, in seconds. Drag the icon to scrub",
        "panel.timeline.tip.frame" => "Frame number. Drag the icon to scrub",
        "panel.timeline.tip.clip_menu" => "Clip options",
        "panel.timeline.tip.end_dd" => "What the clip does when it ends. Click to choose",
        "panel.timeline.tip.end_stop" => "When the clip ends, stop on the last frame",
        "panel.timeline.tip.end_loop" => "When the clip ends, start again from the beginning",
        "panel.timeline.tip.end_pingpong" => {
            "When the clip ends, play it backwards, then forwards again"
        }
        "panel.timeline.tip.starts_with" => "The game starts by playing this clip",
        "panel.timeline.tip.go_start" => "Go to start",
        "panel.timeline.tip.stop" => "Stop and go back to the start",
        "panel.timeline.tip.autokey" => "Auto Key: record a key whenever you change something",
        "panel.timeline.tip.length" => "Clip length (seconds)",
        "panel.timeline.tip.prev_frame" => "Previous frame",
        "panel.timeline.tip.play" => "Play or pause",
        "panel.timeline.tip.next_frame" => "Next frame",
        "panel.timeline.tip.go_end" => "Go to end",
        "panel.timeline.tip.add_clip" => "New clip",
        "panel.timeline.tip.dup_clip" => "Duplicate this clip",
        "panel.timeline.tip.rename_clip" => "Rename this clip",
        "panel.timeline.tip.delete_clip" => "Delete this clip",
        "panel.timeline.tip.reverse" => "Reverse this clip's keys",
        "panel.timeline.tip.add_marker" => "Add a marker at the playhead",
        "panel.timeline.tip.onion_settings" => "Onion settings",
        "panel.timeline.tip.onion_keys" => "Ghosts on the nearest keys (off: on nearby frames)",
        "panel.timeline.loop" => "Loop",
        "panel.timeline.autoplay" => "Autoplay",
        // ⭐⭐ **`Ping-Pong`, com hífen — decisão do Enio (2026-09-19).** O app tinha DUAS grafias
        // para a mesma coisa: `PingPong` aqui e `Ping-Pong` nos outros quatro sítios que a
        // nomeiam (o menu desta mesma timeline, a tira do Flip, a direcção de animação do
        // Inspector, o eco do áudio). ⚠️ E a grafia colada era justamente a que NÃO CABIA na
        // coluna dela — `54,90 px` contra `52,0` —, o que a fazia pintar-se `PingPon…`:
        // *duas grafias de uma coisa só, e a que o artista via era a truncada*. A colada saiu;
        // há gate a exigir que estas duas concordem (`transport_toggle_column.rs`).
        "panel.timeline.ping_pong" => "Ping-Pong",
        "panel.timeline.autokey" => "AutoKey",
        "panel.timeline.record" => "Record",
        // Motion Path keying mode (ADR-0141): a new position key is a trajectory
        // point (on) or separate X/Y (off). "Path" fits the toggle's label column.
        "panel.timeline.motion_path" => "Path",
        "panel.timeline.snap" => "Snap",
        "panel.timeline.speed" => "Speed",
        // Says what the clock DRIVES, not what the scene contains — the scene
        // has physics bodies either way; this is whether Play steps them.
        "panel.timeline.physics" => "Physics",
        // Onion (ADR-0142): ghost poses of the selected object. "Onion" is the term
        // of art; "Keys" toggles pose-to-pose (neighbouring keyframes) vs t±k frames.
        "panel.timeline.onion" => "Onion",
        "panel.timeline.onion_keys" => "Keys",
        // Onion settings modal (ADR-0142 W3b): the floating card's title + row labels. Counts /
        // opacity / colours don't fit the transport bar, so the button opens this card.
        "panel.timeline.onion_settings" => "Onion Settings",
        // The Expression modal (plano 10 W1) — the card that replaces the inline
        // formula field with a searchable catalog and a tunable sheet.
        "panel.timeline.onion_opacity" => "Opacity",
        "panel.timeline.onion_before" => "Ghosts Before",
        "panel.timeline.onion_after" => "Ghosts After",
        "panel.timeline.onion_color_before" => "Past",
        "panel.timeline.onion_color_after" => "Future",
        // The three view tabs, in the order things are assembled: keys make a
        // clip, clips make a container, containers and clips make the scene.
        // Named for what you SEE there, not for a mode you enter.
        "panel.timeline.tab.keys" => "Keys",
        "panel.timeline.tab.containers" => "Containers",
        "panel.timeline.tab.arrange" => "Arrange",
        // Um canal de família que nenhum vocabulário desta build declara.
        "panel.timeline.prop.field" => "Property",
        "panel.timeline.prop.time" => "Time",
        "panel.timeline.prop.morph" => "Morph",
        "panel.timeline.prop.motor_target" => "Motor Target",
        "panel.timeline.prop.motor_speed" => "Motor Speed",
        "panel.timeline.prop.rest_length" => "Rest Length",
        "panel.timeline.prop.max_length" => "Max Length",
        // As duas alças de um osso que arqueia (ordem do dono, 2026-09-17). ⚠️ *In* e *Out* são as
        // palavras do *Bendy Bone* — a alça do lado da RAIZ e a do lado da PONTA —, e não entrada
        // e saída de nada.
        "panel.timeline.prop.bone_bend_in_x" => "Bend In X",
        "panel.timeline.prop.bone_bend_in_y" => "Bend In Y",
        "panel.timeline.prop.bone_bend_out_x" => "Bend Out X",
        "panel.timeline.prop.bone_bend_out_y" => "Bend Out Y",
        "panel.timeline.prop.ik_bend_side" => "IK Bend Side",
        "panel.timeline.prop.macro_time" => "Macro Time",
        // Per-track extrapolation badges (plan §6) — the dashed-region mode label,
        // shown on the dope-sheet only when the side is not the default Hold.
        "panel.timeline.extrap.loop" => "Loop",
        "panel.timeline.extrap.pingpong" => "Ping-Pong",
        "panel.timeline.extrap.continue" => "Continue",
        // Os dois chips do BUFFER do grafo (2026-09-16 — eram os dois textos que a timeline ainda
        // escrevia no fonte).
        "panel.timeline.buffer.swap" => "Swap",
        "panel.timeline.buffer.store" => "Store",
        // Dono 10/11 — a caixa da aba Keys (no lugar da vista Curves); nome curto «Edit Path» (dono 12/11).
        "panel.timeline.curves_on_canvas" => "Edit Path",
        // A pílula da sessão de animar (doc 17 item 9): «Animating jump» + «Done».
        "panel.timeline.animating" => "Animating {clip}",
        "panel.timeline.animating.done.tip" => "Put the scene back as it was. Your keys stay",
        "panel.timeline.curves_on_canvas.tip" => {
            "Show handles on the canvas to bend the object's path"
        }
        // O eixo do Flip image na faixa («Flip image H»), os glifos dos ícones do cartão.
        "panel.timeline.glyph.h" => "H",
        "panel.timeline.glyph.v" => "V",
        // O cartão «quem ouve» do marcador com sinal (doc 17 item 8, maquete B5).
        "panel.timeline.signal.title" => "Signal at frame {frame} \u{b7} {name}",
        "panel.timeline.signal.name" => "Name",
        "panel.timeline.signal.heard_by" => "Heard by {who}",
        "panel.timeline.signal.more" => "+{n} more",
        "panel.timeline.signal.row" => "\u{b7} {what}",
        "panel.timeline.signal.nobody" => "Nobody listens",
        "panel.timeline.signal.script_may" => "A script may listen",
        "panel.timeline.signal.who.signal_actions" => "Signal Actions",
        "panel.timeline.signal.who.weapon_fire" => "Weapon",
        "panel.timeline.signal.who.factory" => "Factory",
        "panel.timeline.signal.who.state_machine" => "State Machine",
        "panel.timeline.signal.who.shake_emitter" => "Camera Shake",
        "panel.timeline.signal.who.particle_emitter" => "Particles",
        "panel.timeline.signal.who.anim_rules" => "Animation Rules",
        "panel.timeline.signal.who.motion_graph" => "Motion input",
        "panel.timeline.signal.who.state_sets" => "UI state",
        "panel.timeline.signal.who.script" => "Script",
        _ => return None,
    })
}
