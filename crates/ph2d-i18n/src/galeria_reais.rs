//! A Widget Gallery: os cartões de casos reais (doc `CartoesUI/08` §1.3). Os rótulos que o código real
//! já tem usam a chave dele (`component.field.*`, `card.bones.*`…); aqui ficam os com unidade no rótulo
//! e os dados fictícios. Irmão do `galeria.rs`: uma linha nova não colide no `match` de outra.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Títulos.
        "panel.widget_gallery.card.image" => "Image",
        "panel.widget_gallery.card.collider" => "Collider",
        "panel.widget_gallery.card.animation" => "Animation",
        "panel.widget_gallery.card.brush" => "Brush",
        "panel.widget_gallery.card.bones" => "Bones",
        "panel.widget_gallery.card.constraint" => "Constraint",
        "panel.widget_gallery.card.audio" => "Audio",
        "panel.widget_gallery.card.pixel" => "Pixel Art",
        // Image.
        "panel.widget_gallery.real.use" => "Use",
        "panel.widget_gallery.real.asset.tile" => "grass_tile.png",
        "panel.widget_gallery.real.asset.sheet" => "hero_sheet.png",
        "panel.widget_gallery.real.asset.sky" => "sky.png",
        "panel.widget_gallery.real.width" => "Width (px)",
        "panel.widget_gallery.real.height" => "Height (px)",
        "panel.widget_gallery.real.columns" => "Columns",
        "panel.widget_gallery.real.rows" => "Rows",
        // Physics Body e Collider.
        "panel.widget_gallery.real.body.dynamic" => "Dynamic",
        "panel.widget_gallery.real.body.kinematic" => "Kinematic",
        "panel.widget_gallery.real.body.static" => "Static",
        "panel.widget_gallery.real.body.static_hint" => "It never moves; others bump into it.",
        "panel.widget_gallery.real.tip.shape_box" => "Box",
        "panel.widget_gallery.real.tip.shape_circle" => "Circle",
        "panel.widget_gallery.real.tip.shape_capsule" => "Capsule",
        "panel.widget_gallery.real.radius" => "Radius (px)",
        "panel.widget_gallery.real.half_height" => "Half height (px)",
        "panel.widget_gallery.real.offset_x" => "Offset X (px)",
        "panel.widget_gallery.real.offset_y" => "Offset Y (px)",
        // Health e Platform Player.
        "panel.widget_gallery.real.regen_delay" => "Regen Delay (s)",
        "panel.widget_gallery.real.speed" => "Speed (m/s)",
        "panel.widget_gallery.real.accel" => "Accel (m/s\u{b2})",
        "panel.widget_gallery.real.air_accel" => "Air (m/s\u{b2})",
        "panel.widget_gallery.real.jump_height" => "Jump Height (m)",
        "panel.widget_gallery.real.coyote" => "Coyote Time (s)",
        "panel.widget_gallery.real.buffer" => "Jump Buffer (s)",
        // Camera.
        "panel.widget_gallery.real.object.enemy" => "Enemy",
        "panel.widget_gallery.real.object.coin" => "Coin",
        "panel.widget_gallery.real.target_hint" => "Type a name",
        "panel.widget_gallery.real.tip.pick_target" => {
            "Pick the target: click here, then click the object on the canvas"
        }
        "panel.widget_gallery.real.target_picking" => "Click an object on the canvas\u{2026}",
        "panel.widget_gallery.real.target_follows" => "Follows {name}",
        "panel.widget_gallery.real.target_missing" => "No object named \u{201c}{name}\u{201d}",
        "panel.widget_gallery.real.format.wide" => "16:9",
        "panel.widget_gallery.real.format.classic" => "4:3",
        "panel.widget_gallery.real.format.square" => "1:1",
        "panel.widget_gallery.real.dead_zone" => "Dead Zone (px)",
        "panel.widget_gallery.real.lookahead" => "Lookahead (px)",
        "panel.widget_gallery.real.look_through" => "Look Through",
        // Animation.
        "panel.widget_gallery.real.clip" => "Clip",
        "panel.widget_gallery.real.clip.idle" => "idle",
        "panel.widget_gallery.real.clip.run" => "run",
        "panel.widget_gallery.real.clip.jump" => "jump",
        "panel.widget_gallery.real.clip.fall" => "fall",
        "panel.widget_gallery.real.when" => "When",
        "panel.widget_gallery.real.play" => "Play",
        "panel.widget_gallery.real.who_me" => "This clip starts when the game runs.",
        "panel.widget_gallery.real.who_script" => "A script picks the clip.",
        "panel.widget_gallery.real.no_rules" => "No rules yet: add one with +.",
        "panel.widget_gallery.real.tip.anim_play" => "Play this clip here",
        "panel.widget_gallery.real.tip.rule_add" => "Add a rule: when this happens, play that",
        // Brush.
        "panel.widget_gallery.real.preset" => "Preset",
        "panel.widget_gallery.real.preset.liner" => "Wet Liner",
        "panel.widget_gallery.real.preset.filbert" => "Filbert",
        "panel.widget_gallery.real.preset.fan" => "Fan Brush",
        "panel.widget_gallery.real.preset.knife" => "Knife Stain",
        "panel.widget_gallery.real.paper_press" => "Paper press",
        // Constraint.
        "panel.widget_gallery.real.influence" => "Influence (%)",
        "panel.widget_gallery.real.tip.constraint_add" => "Add a constraint",
        // Light 2D.
        "panel.widget_gallery.real.lamp_size" => "Size (m)",
        "panel.widget_gallery.real.lamp_range" => "Range (m)",
        "panel.widget_gallery.real.casts_shadows" => "Casts shadows",
        // Audio.
        "panel.widget_gallery.real.gain" => "Gain (dB)",
        "panel.widget_gallery.real.mix" => "Mix (%)",
        "panel.widget_gallery.real.tip.mute" => "Mute",
        "panel.widget_gallery.real.tip.rack_add" => "Add an audio effect",
        "panel.widget_gallery.real.tip.rack_visible" => "Turn this audio effect on or off",
        "panel.widget_gallery.real.tip.rack_remove" => "Remove this audio effect",
        // Pixel Art.
        "panel.widget_gallery.real.foreground" => "Foreground",
        "panel.widget_gallery.real.background" => "Background",
        "panel.widget_gallery.real.grid_size" => "Grid size (px)",
        "panel.widget_gallery.real.pixel_grid" => "Pixel grid",
        // Script.
        "panel.widget_gallery.real.script.player" => "player_move.luau",
        "panel.widget_gallery.real.script.enemy" => "enemy_patrol.luau",
        "panel.widget_gallery.real.script.coin" => "coin.luau",
        "panel.widget_gallery.real.prop.speed" => "speed",
        "panel.widget_gallery.real.prop.jump_power" => "jump_power",
        "panel.widget_gallery.real.prop.double_jump" => "double_jump",
        "panel.widget_gallery.real.prop.patrol" => "patrol_distance",
        "panel.widget_gallery.real.prop.damage" => "damage",
        "panel.widget_gallery.real.prop.value" => "value",
        _ => return None,
    })
}
