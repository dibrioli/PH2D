//! **AS STRINGS DO PAINEL TIMELINE** — `panel.timeline.*`.
//!
//! ⚠️ **Um corte por ASSUNTO, e por TECTO DE LOC** (integração de 2026-09-20): o `lib.rs` estava em
//! `696` de `700` no `main` e a `line/Vector` acrescentou-lhe as cinco chaves das alças do osso que
//! arqueia ⇒ `704`. ⛔ Nenhum dos dois lados o estoura sozinho — *um tecto por-ficheiro é a única
//! grandeza deste repo que SOMA entre linhas sem ninguém a contar* (`CLAUDE.md` §5.0) —, e a cura
//! de um tecto vermelho é **corte por responsabilidade**, nunca uma entrada nova no
//! `FILE_OVERAGE_OK`.
//!
//! ⭐ O corte é o que os irmãos já fizeram (`vector.rs`, `sculpt3d.rs`, `grid_snap.rs`, …): *ali
//! ficam as chaves do APP, aqui as de UM painel*. Elas saem VERBATIM — nenhuma palavra muda.

/// A tradução de uma chave do painel Timeline, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Timeline panel (W2.E9) — dope-sheet + graph editor + transport chrome.
        // English by canon (feedback_app_ui_english_only); routed through tr() so
        // the strings live in one table for the eventual Fluent migration.
        "panel.timeline.title" => "Timeline",
        "panel.timeline.summary" => "Summary",
        "panel.timeline.add_track" => "+ Track  \u{25be}",
        "panel.timeline.add_lane" => "+ Lane",
        "panel.timeline.add_container" => "+ Container",
        "panel.timeline.crumb_root" => "Scene",
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
        "panel.timeline.add_marker" => "+M",
        "panel.timeline.time_seconds" => "Time(s)",
        "panel.timeline.length" => "Dur(s)",
        "panel.timeline.frame" => "Frame",
        "panel.timeline.loop" => "Loop",
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
        "panel.timeline.prop.translate_x" => "Translate X",
        "panel.timeline.prop.translate_y" => "Translate Y",
        "panel.timeline.prop.rotation" => "Rotation",
        "panel.timeline.prop.scale_x" => "Scale X",
        "panel.timeline.prop.scale_y" => "Scale Y",
        "panel.timeline.prop.opacity" => "Opacity",
        "panel.timeline.prop.time" => "Time",
        "panel.timeline.prop.morph" => "Morph",
        "panel.timeline.prop.position" => "Position",
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
        // Per-track extrapolation badges (plan §6) — the dashed-region mode label,
        // shown on the dope-sheet only when the side is not the default Hold.
        "panel.timeline.extrap.loop" => "Loop",
        "panel.timeline.extrap.pingpong" => "Ping-Pong",
        "panel.timeline.extrap.continue" => "Continue",
        // Os dois chips do BUFFER do grafo (2026-09-16 — eram os dois textos que a timeline ainda
        // escrevia no fonte).
        "panel.timeline.buffer.swap" => "Swap",
        "panel.timeline.buffer.store" => "Store",
        _ => return None,
    })
}
