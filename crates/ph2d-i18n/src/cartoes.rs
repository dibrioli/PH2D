//! **OS CARTÕES** (`docs/CartoesUI`): os painéis como listas do Kanban e o campo Figma. Irmão do
//! `vector.rs` pela razão de sempre — uma linha nova não colide no `match` de outra.

/// A tradução de uma chave dos cartões, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "card.insp.phys.body" => "Body",
        "card.insp.phys.shape" => "Shape",
        "card.insp.phys.collision_shape" => "Collision Shape",
        "card.insp.phys.collision_layers" => "Collision Layers",
        "card.insp.phys.width" => "Width",
        "card.insp.phys.height" => "Height",
        "card.insp.phys.touch" => "Touch",
        "card.insp.phys.start_moving" => "Start moving",
        "card.insp.phys.advanced" => "More",
        "card.insp.phys.joint" => "Joint",
        "card.insp.phys.bake" => "Bake to Timeline",
        "card.insp.phys.offset" => "Offset",
        "card.insp.phys.material" => "Material rules",
        "card.insp.phys.surface" => "Surface",
        "card.insp.phys.signals" => "Signals",
        "card.insp.phys.zone_force" => "Zone force",
        "card.insp.phys.bounce_friction" => "Bounce and Friction",
        "card.insp.phys.no_friction" => {
            "No Friction or One-Way on a player: the Platform Player walks and brakes."
        }
        "card.insp.phys.density" => "Density",
        "card.insp.phys.loose" => "No body above this shape. Nothing bumps into it yet.",
        "card.insp.phys.no_shape" => "This body has no collision shape, so nothing will touch it.",
        "card.insp.phys.add_shape" => "Add Shape",
        "card.insp.phys.drag_hint" => "Or drag it inside a body in the Hierarchy.",
        "card.insp.phys.part_of" => "Part of: {owner}",
        "card.insp.phys.shape_of" => "Shape of: {owner}",
        "card.insp.phys.move_to_child" => "Move shape to a child",
        "card.insp.phys.follows" => "Follows the picture",
        "card.insp.phys.custom" => "Custom size",
        "card.insp.phys.fit" => "Fit to picture",
        "card.insp.phys.outline_detail" => "Detail",
        "card.insp.phys.outline_alpha" => "Alpha cutoff",
        "card.insp.phys.outline_specks" => "Ignore specks under",
        "card.insp.phys.outline_detail_tip" => {
            "How far the outline may move out from the picture to stay simple. Bigger means fewer points."
        }
        "card.insp.phys.points_animated_title" => "This shape's points are animated",
        "card.insp.phys.points_animated_hint" => {
            "The animation controls these points. Delete the points animation to change them freely?"
        }
        "card.insp.phys.points_animated_delete" => "Delete animation",
        "card.insp.phys.outline_locked_tip" => {
            "Locked because you moved points by hand. Turn on \u{201c}Edit shape on canvas\u{201d} to change it."
        }
        "card.insp.phys.outline_alpha_tip" => {
            "How solid a pixel must be to count as part of the shape. 0.5 means half see-through."
        }
        "card.insp.phys.outline_specks_tip" => {
            "Leave out islands and fill holes smaller than this many pixels."
        }
        "card.insp.phys.outline_merge_dist" => "Merge by distance",
        "card.insp.phys.outline_merge" => "Merge",
        "card.insp.phys.outline_merge_tip" => {
            "Join points closer than this many pixels into one. Works on the selected points, or on all of them."
        }
        "card.insp.phys.outline_merged" => "Merged {n} {n?point|points}",
        "card.insp.phys.outline_merge_kept" => {
            "{k} {k?group was|groups were} left as {k?it is|they are} so the outline does not cross itself"
        }
        "card.insp.phys.outline_raised" => {
            "Too detailed — Detail raised to {x} px to stay under {n} pieces"
        }
        "card.insp.phys.edit_shape" => "Edit shape on canvas",
        "card.insp.phys.editing_shape" => "Editing shape · Esc to finish",
        "card.insp.phys.collides_with" => "Collides with",
        "card.insp.phys.collides_applies" => "Applies to everything on {layer}",
        "card.insp.phys.collides_differs" => "Differs from layer {layer}",
        "card.insp.phys.use_layer_default" => "Use layer default",
        "card.insp.phys.collides_on" => {
            "Things on {this} bump into things on {other}. Click to let them pass through."
        }
        "card.insp.phys.collides_off" => {
            "Things on {this} pass through things on {other}. Click to make them bump."
        }
        "card.insp.phys.collides_self_on" => {
            "Things on {this} bump into each other. Click to let them pass through."
        }
        "card.insp.phys.collides_self_off" => {
            "Things on {this} pass through each other. Click to make them bump."
        }
        "card.insp.phys.collides_add" => "+ Add layer",
        "card.insp.phys.collides_add_tip" => "Show one more layer here and in the matrix.",
        "card.insp.phys.edit_matrix" => "Open in tab…",
        "card.insp.phys.edit_matrix_tip" => "Open the Collision Layers tab at this layer.",
        "card.insp.phys.layer_rename" => "Rename layers…",
        "card.insp.phys.layer_rename_done" => "Done",
        "card.insp.phys.stretched" => "Scale stretches this round shape into an oval.",
        "card.insp.phys.scale_one" => "Set Scale to 1",
        "card.insp.phys.use_box" => "Use Box",
        "card.insp.phys.removed_body" => "Physics Body removed",
        "card.insp.phys.removed_shape" => "Shape removed",
        "card.insp.phys.undo" => "Undo",
        "card.insp.phys.still" => "Still",
        "card.insp.phys.moving" => "Moving",
        "card.insp.phys.none" => "None",
        "card.insp.phys.on" => "On",
        "card.insp.phys.dominance" => "Dominance {n}",
        "card.insp.phys.wall_cling_on" => "Wall Cling on",
        "card.insp.phys.wall_cling_off" => "Wall Cling off",
        "card.insp.phys.one_way_on" => "One-way on",
        "card.insp.phys.one_way_off" => "One-way off",
        "card.painter.preset.favorite_add" => "Add this preset to Favorites",
        "card.painter.preset.favorite_remove" => "Remove this preset from Favorites",
        "card.painter.wet.contact" => "Contact and drying",
        "card.painter.wet.thickness" => "Paint thickness",
        "card.painter.wet.pigment" => "Pigment and layers",
        "card.painter.wet.canvas" => "Whole canvas",
        "card.painter.watercolor.bleed" => "Bleed (px)",
        "card.painter.watercolor.ragged_edge" => "Ragged Edge (px)",
        "card.painter.watercolor.flow_angle" => "Flow Angle (°)",
        "card.painter.watercolor.drying_time" => "Drying Time (s)",
        "card.painter.impasto.tool" => "Tool",
        "card.painter.shape.angle" => "Angle (°)",
        "card.painter.shape.bristle_combing" => "Bristle Combing",
        "card.painter.shape.bristle_width" => "Bristle width (px)",
        "card.painter.shape.bristle_relief" => "Bristle relief",
        "card.painter.shape.ragged_bristle_tips" => "Ragged bristle tips",
        "card.painter.shape.bristle_follow_size" => "Follow brush size",
        "card.painter.stencil.rotation" => "Rotation (°)",
        "card.painter.randomize.when" => "New color",
        "card.painter.stroke.operation" => "Operation",
        "card.painter.stroke.delete" => "Delete",
        "card.painter.line.type" => "Line type",
        "card.painter.brush.sync" => "Sync tools",
        "card.painter.layers.relief" => "Relief depth",
        "card.painter.layers.relief_mode" => "Relief mode",
        "card.painter.adjust.channel" => "Channel",
        "card.painter.adjust.output" => "Output",
        "card.painter.adjust.color_group" => "Color group",
        "card.painter.adjust.stops" => "Stops",
        "card.painter.adjust.method" => "Method",
        "card.painter.adjust.tonal_range" => "Tonal range",
        "card.painter.adjust.kind" => "Type",
        "card.painter.adjust.add_point_tip" => "Add a point to the curve",
        "card.painter.adjust.remove_point_tip" => "Remove a point from the curve",
        "card.painter.adjust.reds" => "Reds",
        "card.painter.adjust.yellows" => "Yellows",
        "card.painter.adjust.greens" => "Greens",
        "card.painter.adjust.cyans" => "Cyans",
        "card.painter.adjust.blues" => "Blues",
        "card.painter.adjust.magentas" => "Magentas",
        "card.painter.adjust.whites" => "Whites",
        "card.painter.adjust.neutrals" => "Neutrals",
        "card.painter.adjust.blacks" => "Blacks",
        "card.painter.ramp.alpha" => "Alpha",
        "card.painter.sculpt.radius" => "Radius (px)",
        "card.painter.sculpt.angle" => "Angle (°)",
        "card.painter.sculpt.smooth" => "Smoothness (px)",
        "card.painter.composite.scope" => "Erases",
        "card.painter.selection.operation" => "Operation",
        "card.painter.selection.expand_contract" => "Expand or contract",
        "card.painter.pen.pressure" => "Pressure",
        "card.painter.pen.tilt" => "Tilt",
        "card.painter.pen.speed" => "Speed",
        "card.painter.pen.pulled_string" => "Pulled string (px)",
        "card.painter.pen.tilt_start" => "Tilt start (°)",
        "card.painter.pen.tilt_full" => "Tilt full (°)",
        "card.painter.pen.tilt_lag" => "Tilt lag (px)",
        "card.painter.pen.speed_lag" => "Speed lag (s)",
        "card.painter.impasto.angle" => "Angle (°)",
        "card.painter.impasto.elevation" => "Elevation (°)",
        "card.painter.wet.wet_tip" => "Wet the whole canvas",
        "card.painter.wet.dry_tip" => "Dry the whole canvas at once: the paint settles where it is",
        "card.painter.wet.fast_dry_tip" => {
            "Run the drying of the whole canvas fast, until it is dry"
        }
        "card.painter.wet.material" => "Material",
        "card.painter.performance.title" => "Performance",
        "card.painter.performance.guess" => "Faster wet paint",
        "card.painter.performance.guess_tip" => {
            "Wet paint flows faster when many strokes are wet at once. Some brushes look a little different"
        }

        "card.painter.ramp.color_mode" => "Color mode",
        "card.painter.ramp.interpolation" => "Interpolation",
        "card.painter.ramp.stop" => "Stop",
        "card.painter.ramp.position" => "Position",
        "card.painter.ramp.color" => "Color",
        "card.painter.ramp.add_tip" => "Add a color stop",
        "card.painter.ramp.remove_tip" => "Remove the selected stop",
        "card.painter.ramp.invert_tip" => "Flip the ramp end to end",
        "card.painter.ramp.layer_opacity" => "Opacity (%)",
        "card.ceq.histogram" => "Histogram",
        "card.ceq.auto" => "Auto",
        "card.ceq.tone" => "Tone",
        "card.ceq.color" => "Color",
        "card.ceq.sharpen" => "Sharpen",
        "card.ceq.local" => "Local Contrast",
        "card.ceq.lut" => "Color Lookup",
        "card.ceq.stylize" => "Stylize",
        "card.ceq.exposure" => "Exposure (EV)",
        "card.ceq.radius" => "Radius (px)",
        "card.ceq.amount" => "Amount",
        "card.bgr.mask" => "Mask",
        "card.bgr.colors" => "Background Colors",
        "card.bgr.islands" => "Islands",
        "card.bgr.protection" => "Protection",
        "card.bgr.falloff" => "Falloff",
        "card.bgr.protect_brush" => "Protection brush",
        "card.upscale.scale" => "Scale (\u{d7})",
        "card.equalize.target" => "Target",
        "card.equalize.size_from" => "Size from",
        "card.equalize.fixed_size" => "Size (px)",
        "card.equalize.offset" => "Offset (px)",
        "card.equalize.result" => "Result",
        // Cabem as duas caixas na linha, na largura de fábrica da lista.
        "card.equalize.upscale_if_small" => "Upscale if small",
        "card.vector.textpath.this" => "This",
        "card.vector.textpath.other" => "Other",
        "card.equalize.algorithm" => "Algorithm",
        "card.padding.edges" => "Edges",
        "card.padding.top" => "Top (px)",
        "card.padding.right" => "Right (px)",
        "card.padding.bottom" => "Bottom (px)",
        "card.padding.left" => "Left (px)",
        "card.padding.recenter_pivot" => "Recenter pivot",
        "card.field.mixed" => "Mixed",
        "card.glyph.in" => "In",
        "card.glyph.out" => "Out",
        "card.glyph.x" => "X",
        "card.glyph.y" => "Y",
        "card.glyph.z" => "Z",
        // O Inspector principal (02 §7): a unidade no rótulo (D13).
        "card.insp.name" => "Name",
        "card.insp.visible" => "Show / hide",
        "card.insp.lock" => "Lock / unlock",
        "card.insp.layer_visible" => "Show / hide this layer",
        "card.insp.layer_remove" => "Remove this layer",
        "card.field.drag_hint" => "Drag the icon to change · Shift ×10 · Alt fine",
        "card.field.drag_hint_named" => "{label} — {hint}",
        "card.insp.transform" => "Transform",
        "card.vector.shape" => "Shape",
        "card.insp.position_px" => "Position (px)",
        "card.insp.position_m" => "Position (m)",
        "card.insp.rotation_deg" => "Rotation (\u{b0})",
        "card.insp.rotation_rad" => "Rotation (rad)",
        "card.insp.scale" => "Scale",
        "card.insp.skew_deg" => "Skew (\u{b0})",
        "card.insp.skew_rad" => "Skew (rad)",
        "card.insp.scale_lock" => "Keep proportions",
        "card.insp.reset" => "Reset transform",
        "card.insp.reset_component" => "Reset to defaults",
        "card.insp.add_fill" => "Add fill",
        "card.insp.add_stroke" => "Add stroke",
        "card.insp.appearance" => "Appearance",
        "card.insp.opacity" => "Opacity (%)",
        "card.insp.look" => "Look",
        "card.insp.verb_painter" => "Painter",
        "card.insp.edit_in_painter_tip" => {
            "Edit in Painter. Every object using this image updates when you leave Draw"
        }
        "card.insp.verb_pixel" => "Pixel",
        "card.insp.pixel_pivot" => "Pivot",
        "card.insp.pixel_pivot_center" => "Center",
        "card.insp.pixel_pivot_tip" => {
            "The point that sits on the object position: the center, or a slice pivot"
        }
        "card.insp.edit_in_pixel_tip" => "Edit in Pixel. The thumbnail updates when you leave Draw",
        "card.insp.replace_receipt" => "{aw} \u{00d7} {ah} \u{2192} {bw} \u{00d7} {bh} px",
        "card.insp.keep_old_size" => "Keep old size",
        "card.insp.keep_old_size_tip" => {
            "The new image takes the size the old one had in the scene"
        }
        "card.insp.paint_new_tip" => "Start from a blank image and paint it",
        "card.insp.sheet_warning" => {
            "This image looks like a sheet of {c} \u{00d7} {l} cells. Still shows all of it."
        }
        "card.insp.sheet_make_animated" => "Make it Animated",
        "card.insp.sheet_make_animated_tip" => "Play the cells of this sheet as an animation",
        "card.insp.sheet_use_region" => "Use one Region",
        "card.insp.sheet_use_region_tip" => {
            "Show one part of this sheet: click a cell to choose it"
        }
        "card.insp.pivot" => "Pivot",
        "card.insp.pivot_px" => "Pivot (px)",
        "card.insp.shape_pivot_xy" => "Pivot from centre",
        "card.insp.shape_stroke" => "Stroke width (px)",
        "card.insp.shape_stroke_tip" => {
            "How thick the outline of this shape is drawn on the canvas"
        }
        "card.insp.shape_dashed" => "Dashed",
        "card.insp.shape_dashed_tip" => "Draw the outline of this shape as dashes",
        "card.insp.shape_dash_gap" => "Dash and gap (px)",
        "card.insp.shape_dash_tip" => "How long each dash of the outline is",
        "card.insp.shape_gap_tip" => "How long the space between two dashes is",
        "card.insp.pivot_nome_tip" => "Show the exact pivot position",
        "card.insp.pivot_in_editor" => "Edit in Animations",
        "card.insp.pivot_in_editor_tip" => {
            "An Animated Sprite's pivot is set in the Animation Editor, over every frame of every animation"
        }
        "card.insp.pivot.custom" => "Custom",
        "card.insp.pivot.top_left" => "Top Left",
        "card.insp.pivot.top" => "Top",
        "card.insp.pivot.top_right" => "Top Right",
        "card.insp.pivot.left" => "Left",
        "card.insp.pivot.center" => "Center",
        "card.insp.pivot.right" => "Right",
        "card.insp.pivot.bottom_left" => "Bottom Left",
        "card.insp.pivot.bottom" => "Bottom",
        "card.insp.pivot.bottom_right" => "Bottom Right",
        "card.insp.advanced" => "Advanced",
        "card.insp.flip" => "Flip image",
        "card.insp.flip_tip" => "Flips the picture only. Attached objects and collision stay put.",
        "card.insp.mirror_object" => "Mirror object",
        "card.insp.mirror_keep" => "Keep in place",
        "card.insp.mirror_keep_tip" => {
            "Mirror object also mirrors attached objects and collision, but the picture stays where it is."
        }
        "card.insp.mirror_object_tip" => {
            "Mirrors the whole object around its pivot, with its attached objects and collision."
        }
        "card.insp.image_size_repeated" => "{pw} \u{00d7} {ph} px ({size})",
        "card.insp.image_size_bare" => "{pw} \u{00d7} {ph}",
        "card.insp.image_size_cut" => "{pw} \u{00d7} {ph} px \u{00b7} {c} \u{00d7} {l}",
        "card.insp.image_size_cut_bare" => "{pw} \u{00d7} {ph} \u{00b7} {c} \u{00d7} {l}",
        "card.insp.image_size_cut_tight" => "{pw}\u{00d7}{ph} \u{00b7} {c}\u{00d7}{l}",
        "card.insp.image_size_tight" => "{pw}\u{00d7}{ph}",
        "card.insp.image_slot_tip" => {
            "Click to choose or replace the image, or drop one here from Assets"
        }
        "card.insp.image_unnamed" => "Untitled image",
        "card.insp.image_drop" => "Drop an image",
        "card.insp.sheet_drop" => "Drop a sprite sheet",
        "card.insp.new_image_old_cut" => {
            "New image, old cut: {c} × {r} makes cells of {w} × {h} px, and {n?this|these} {n} {n?animation|animations} {n?was|were} made for another image."
        }
        "card.insp.cut_again" => "Cut again",
        "card.insp.cut_again_tip" => {
            "Cut the new image into the grid its transparency suggests. If it suggests none, the editor opens, and this notice stays until the cut changes"
        }
        "card.insp.keep_cut" => "Keep the cut",
        "card.insp.keep_cut_tip" => "Leave the cut as it is and close this notice",
        // A linha Normal Map do cartão Image (P14): a normal é da IMAGEM.
        "card.insp.normal_map" => "Normal Map",
        "card.insp.normal_map_none" => "None",
        "card.insp.normal_map_tip" => {
            "Pick the image that tells 2D lights which way each pixel faces. Every object that shows this image uses it."
        }
        "card.insp.normal_map_none_tip" => "Take the normal map off this image",
        "card.insp.image_drop_hint" => "or click to choose",
        "card.insp.silhouette" => "Silhouette (Tint Fill)",
        "card.insp.tint_each_corner" => "Tint each corner",
        "card.insp.order_option" => "+ Order option…",
        "card.insp.transform_summary" => {
            "{x}, {y} \u{00b7} {r}\u{00b0} \u{00b7} {sx} \u{00d7} {sy}"
        }
        "card.insp.advanced_summary" => "{strategy} \u{00b7} {format}",
        "card.insp.blend" => "Blend",
        "card.insp.fill" => "Fill",
        "card.insp.stroke" => "Stroke",
        "card.insp.color" => "Color",
        "card.insp.gradient" => "Gradient",
        "card.insp.stroke_position" => "Position",
        "card.insp.weight" => "Weight (px)",
        "card.insp.align_center" => "Center",
        "card.insp.align_inside" => "Inside",
        "card.insp.align_outside" => "Outside",
        "card.insp.cap_butt" => "Butt",
        "card.insp.cap_round" => "Round",
        "card.insp.cap_square" => "Square",
        "card.insp.join_miter" => "Miter",
        "card.insp.join_round" => "Round",
        "card.insp.join_bevel" => "Bevel",
        "card.insp.cap" => "Cap",
        "card.insp.join" => "Join",
        "card.insp.ordering" => "Ordering",
        "card.insp.z_index" => "Z index",
        "card.insp.region_position_px" => "Region position (px)",
        "card.insp.region_size_px" => "Region size (px)",
        "card.insp.material" => "Material & Blend",
        "card.insp.mode" => "Mode",
        "card.insp.add_component" => "Add component",
        // O ícone ao lado do título da lista (D21).
        "card.menu.panel" => "Panel options",
        "card.menu.close" => "Close this panel",
        "card.search.clear" => "Clear the search",
        // O painel Physics: a unidade no rótulo (D13).
        "card.macros.record" => "Record",
        "card.macros.steps" => "Steps",
        "card.macros.step" => "Step",
        "card.macros.wait" => "Wait (s)",
        "card.macros.size" => "Size (px)",
        "card.grid.with_unit" => "{name} ({unit})",
        "card.grid.snap" => "Snap to the grid",
        "card.grid.snap_to" => "Snap to",
        "card.grid.not_available" => "n/a",
        "card.tokens.mode" => "Mode",
        "card.tokens.follows" => "Follows {alvo}",
        "card.tokens.app_colors" => "App Colors",
        "card.tokens.link" => "Follow another token: click here, then on the token to follow",
        "card.tokens.formula" => "Write a formula for this value",
        "card.wet_tuning.texture_size_ref" => "Texture size reference (px)",
        "card.wet_tuning.knife_ridge" => "Knife ridge (px)",
        "card.wet_tuning.subsurface_radius" => "Subsurface Radius (px)",
        "card.wet_tuning.wet_open_edge" => "Bloom edge softness (px)",
        "card.wet_tuning.reset_knob" => "Reset {knob}",
        "card.wet_tuning.reset_group" => "Reset all of {group}",
        "card.pixel.colors" => "Colors",
        "card.pixel.selected" => "{n} selected",
        "card.pixel.palette" => "Palette",
        "card.pixel.fg" => "Foreground",
        "card.pixel.bg" => "Background",
        "card.insp.behind_parent" => "Behind Parent",
        "card.flip.size" => "Size (px)",
        "card.flip.strength" => "Strength (%)",
        "card.flip.opacity" => "Opacity (%)",
        "card.flip.min_width" => "Min Width (%)",
        "card.flip.response" => "Response (%)",
        "card.flip.spacing" => "Spacing (× dot size)",
        "card.flip.trap" => "Trap (px)",
        "card.flip.grow" => "Grow (px)",
        "card.flip.bleed" => "Bleed (%)",
        "card.flip.depth" => "Depth (%)",
        "card.flip.blend" => "Blend",
        "card.flip.link_size" => "Eraser uses the brush size",
        "card.flip.link_strength" => "Eraser uses the brush opacity",
        "card.flip.layer_eye" => "Show or hide this layer",
        "card.flip.layer_lock" => "Lock this layer",
        "card.flip.layer_up" => "Move up",
        "card.flip.layer_down" => "Move down",
        "card.physics.gravity" => "Gravity (m/s\u{b2})",
        "card.physics.contact_hz" => "Contact (Hz)",
        "card.physics.sleep_speed" => "Speed (m/s)",
        "card.physics.sleep_delay" => "Delay (s)",
        "card.physics.hold_slack" => "Slack (m)",
        "card.physics.radius" => "Radius (m)",
        "card.physics.blast_impulse" => "Impulse (N\u{b7}s)",
        "card.physics.pull_force" => "Force (N)",
        "card.physics.layers" => "Layers that collide",
        "card.physics.layer_name" => "Layer {n} name",
        "card.physics.layer_add" => "+ Layer",
        "card.physics.layer_add_full" => {
            "All 32 layers are in use: 32 is the most the physics engine has."
        }
        "card.physics.layer_default" => "Layer {n}",
        "card.physics.matrix_title" => "Collision Layers",
        "card.physics.matrix_summary" => {
            "{n} {n?layer|layers} shown · {off} {off?pair that does|pairs that do} not collide"
        }
        "card.physics.layer_label_tip" => "Double-click to rename",
        "card.physics.layer_label_tip_named" => "{name}. Double-click to rename",
        "card.physics.layer_zero" => "Default",
        "card.physics.name_ellipsis" => "…",
        // As fotos do smoke da aba (`PH2D_PHYSICS_SMOKE_ABA`): os nomes da maquete B, separados por «|».
        "card.physics.smoke_layer_names" => "Ground|Player|Enemy|Pickup|Player Shots",
        "card.physics.smoke_layer_rename" => "Enemy Bats",
        "card.physics.smoke_long_name" => "Flying Enemies That Only Hit The Players",
        "card.physics.tab_layers" => "Layers",
        "card.physics.tab_add" => "+ Add layer",
        "card.physics.tab_add_tip" => {
            "Make a new layer. It bumps into everything until you switch pairs off."
        }
        "card.physics.tab_grid_title" => "Who bumps into whom",
        "card.physics.tab_pairs" => "{n} {n?pair|pairs}",
        // O par vai em negrito: a frase do pé é «{a} × {b}:» + o resto.
        "card.physics.tab_pair" => "{a} × {b}:",
        "card.physics.tab_pair_bump" => {
            "they bump into each other. Click to make them pass through."
        }
        "card.physics.tab_pair_pass" => "they pass through each other. Click to make them bump.",
        "card.physics.tab_bumps_into" => "Bumps into",
        "card.physics.tab_check" => "✓",
        "card.physics.tab_bump_tip" => {
            "Tick to make these two layers bump; untick to let them pass through."
        }
        "card.physics.tab_objects" => "Objects on this layer",
        "card.physics.tab_no_objects" => "Nothing is on this layer yet.",
        "card.physics.tab_object_tip" => "Select this object in the scene and the Inspector",
        "card.physics.tab_select_all" => "Select all",
        "card.physics.tab_select_all_tip" => "Select every object on this layer",
        "card.physics.tab_object_of" => "{body} › {shape}",
        "card.physics.tab_differs" => "Differ from this layer",
        "card.physics.tab_differs_tip" => {
            "Select this shape. Its own Collides with differs from this layer; Use layer default on its card puts it back."
        }
        "card.physics.tab_pin_on" => "{layer} on",
        "card.physics.tab_pin_off" => "{layer} off",
        "card.physics.tab_pins_sep" => " · ",
        "card.physics.tab_pin_from" => "on {layer}: {pins}",
        "card.physics.tab_pair_differs" => {
            " {n} {n?shape has its|shapes have their} own rule here (the dot)."
        }
        "card.physics.tab_more" => "+{n} more",
        "card.physics.tab_more_tip" => "Show every object on this layer",
        "card.physics.tab_remove" => "Remove",
        "card.physics.tab_remove_tip" => {
            "Remove this layer. If objects are on it, you choose where they go."
        }
        "card.physics.tab_remove_default_tip" => {
            "Default can't be removed: every object starts on it."
        }
        "card.physics.tab_remove_ask" => {
            "{n} {n?object is|objects are} on {layer}: {names}. Where should they go?"
        }
        "card.physics.tab_names_sep" => ", ",
        "card.physics.tab_names_last" => " and ",
        "card.physics.tab_move_to" => "Move them to",
        "card.physics.tab_move_to_tip" => "The objects go to this layer",
        "card.physics.tab_move_ok" => "Move and remove",
        "card.physics.tab_move_ok_tip" => {
            "Move the objects, then remove the layer. Ctrl+Z brings it all back."
        }
        "card.physics.tab_cancel" => "Cancel",
        "card.physics.tab_cancel_tip" => "Keep the layer as it is",
        "card.physics.tab_summary" => {
            "{n} {n?pair|pairs} · {off} pass through. Double-click a name to rename it."
        }
        "card.physics.tab_new_layer" => {
            "New layers bump into everything. Drag along the row to switch many cells at once."
        }
        "card.physics.tab_only_default" => {
            "Everything is on Default, so everything bumps into everything. Press + Add to make a layer, then switch off the pairs that should pass through."
        }
        "card.physics.tab_layer_title" => "{name} · {n} {n?object|objects}",
        "card.physics.tab_layer_pairs" => "{bump} bump · {pass} pass through",
        "card.physics.scale" => "Scale (px/m)",
        "card.physics.bodies" => "Bodies",
        "card.menu.reset_order" => "Reset Positions",
        "card.menu.reset_colors" => "Reset Colors",
        "card.menu.reset_all" => "Reset All",
        // O cartão solto no canvas (C5): a linha que fica na lista e o ícone que o devolve.
        "card.loose.on_canvas" => "on canvas",
        "card.loose.ghost_tip" => "Show this card on the canvas",
        "card.loose.return" => "Return to panel",
        // T7: o alfinete prende o cartão solto ao objeto de agora; sem ele, segue a seleção.
        "card.loose.pin" => "Pin to this object",
        "card.loose.rope" => "Show a rope to this object",
        "card.loose.unpin" => "Unpin (follow the selection)",
        // Os estilos de cor (D28/D30, maquete estilos.png).
        "card.styles.open" => "Color styles",
        "card.styles.title" => "Color styles",
        "card.styles.add" => "Save this color as a style",
        "card.styles.project" => "This project",
        "card.styles.empty" => "No styles yet. Press + to save this color.",
        "card.styles.search" => "Search...",
        "card.styles.no_match" => "No style has that name.",
        "card.styles.new_name" => "Color",
        "card.styles.copy" => "copy",
        "card.styles.detach" => "Detach style (keep the color)",
        "card.styles.edit_title" => "Edit style",
        "card.styles.name" => "Name",
        "card.styles.color" => "Color",
        "card.styles.used_by" => "Used by {n} {n?layer|layers}. All change.",
        "card.styles.apply" => "Apply",
        "card.styles.edit" => "Edit style...",
        "card.styles.rename" => "Rename",
        "card.styles.duplicate" => "Duplicate",
        "card.styles.delete" => "Delete",
        // O painel Bones (02 §6): títulos dos cartões e rótulos com a unidade no rótulo (D13).
        "card.bones.list_title" => "BONES",
        "card.bones.tool" => "Tool",
        "card.bones.weight_brush" => "Weight Brush",
        "card.bones.skin" => "Skin",
        "card.bones.bone" => "Bone",
        "card.bones.limit" => "Angle Limit",
        "card.bones.smart" => "Smart Bone",
        "card.bones.ik" => "IK",
        "card.bones.mode" => "Mode",
        "card.bones.direction" => "Direction",
        "card.bones.radius" => "Radius (px)",
        "card.bones.strength" => "Strength",
        "card.bones.target" => "Target Weight",
        "card.bones.deform_by" => "Deform by",
        // D27: o campo mostra px do MUNDO (o local × a escala da cadeia); a unidade no rótulo (D13).
        "card.bones.length" => "Length (px)",
        "card.bones.segments" => "Segments",
        "card.bones.reach" => "Reach Strength",
        "card.bones.handles" => "Curve handles",
        "card.bones.handles.chain" => "Chain",
        "card.bones.curve_in" => "Curve in",
        "card.bones.curve_out" => "Curve out",
        "card.bones.along_in" => "Along in",
        "card.assets.find" => "Find",
        "card.assets.catalogs" => "Catalogs",
        "card.assets.library" => "Library",
        "card.assets.kind" => "Type",
        "card.assets.sort" => "Sort by",
        "card.assets.thumb_size" => "Thumbnail size (px)",
        "card.assets.toggle_catalogs" => "Show or hide the catalogs",
        "card.assets.new_catalog" => "New catalog",
        "card.assets.clear_filter" => "Clear this filter",
        "card.assets.cancel_choice" => "Cancel choosing an image",
        "card.audio_mixer.output" => "Output",
        "card.audio_mixer.channels" => "Channels",
        "card.audio_mixer.loudness" => "Loudness (LUFS)",
        "card.audio_mixer.silent" => "-inf",
        "card.audio_mixer.pan" => "Pan",
        "card.audio_mixer.tone" => "Tone",
        "card.audio_mixer.low_cut" => "Low cut",
        "card.audio_mixer.gain_db" => "Gain (dB)",
        "card.audio_mixer.sends_hint" => "How much of each channel goes in:",
        "card.common.on" => "On",
        "card.common.enabled" => "Enabled",
        "card.common.off" => "Off",
        "card.widget_lab.bench" => "Bench",
        "card.hierarchy.add" => "Add an object",
        "card.hierarchy.filter" => "Show only the objects with this component",
        "card.hierarchy.eye" => "Show or hide this object",
        "card.hierarchy.lock" => "Lock this object so the canvas leaves it alone",
        "card.hierarchy.group" => {
            "Group the children: they move with the parent and can't be picked alone"
        }
        "card.hierarchy.expand" => "Open or close this row",
        "card.hierarchy.icon" => "Double-click to frame this object",
        "card.hierarchy.component" => "Open this component's card",
        "card.tags.all" => "All tags",
        "card.tags.selected" => "Selected tag",
        "card.tags.new" => "New tag",
        "card.tags.empty" => "No tags yet. Press + to make the first one.",
        "card.tags.pick_hint" => {
            "Choose a tag to rename, select or delete it. Drag a tag onto another to nest it."
        }
        "card.audio.sounds" => "Sounds",
        "card.audio.weight" => "Weight",
        "card.audio.remove_sound" => "Remove this sound",
        "card.audio.sound_on_tip" => "Off keeps the sound in the list but it never plays",
        "card.audio.weight_tip" => "How often this sound comes up, compared to the others",
        "card.audio.pick" => "Pick",
        "card.audio.pick_placeholder" => "Add a sound\u{2026}",
        "card.audio.pick_none" => "\u{2014}",
        "card.audio.pick_empty_project" => {
            "This project has no sounds yet. Browse\u{2026} imports one, then it can be picked here."
        }
        "card.audio.pick_all_in_list" => "Every sound of this project is already in the list.",
        "card.audio.pick_full" => "This list is full: remove a sound before adding another.",
        "card.audio.mode" => "Mode",
        "card.audio.no_repeat" => "No repeat",
        "card.audio.no_repeat_tip" => "Never the same sound twice in a row",
        "card.audio.no_repeat_sequence" => {
            "Sequence plays the list in order, so No repeat has no effect."
        }
        "card.audio.pitch_variation" => "Pitch variation (st)",
        "card.audio.volume_variation" => "Volume variation (dB)",
        "card.audio.list_decides" => "The Sounds list below decides what plays.",
        "card.audio.not_in_project" => "Not in this project, so it stays silent: {names}.",
        "card.audio.rows_hidden" => {
            "{n} more {n?sound|sounds} in this list {n?is|are} not shown here, and {n?it plays|they play} anyway."
        }
        "card.audio.offer_hidden" => "{n} more project {n?sound|sounds} cannot be offered here.",
        "card.anim.markers" => "Frame markers",
        "card.anim.marker_frame" => "Frame",
        "card.anim.marker_signal" => "Signal",
        "card.anim.marker_signal_placeholder" => "Signal name",
        "card.anim.marker_add" => "+ Marker",
        "card.anim.marker_add_open" => "Add marker",
        "card.anim.markers_open" => "{n} {n?marker|markers}",
        "card.anim.marker_del_tip" => "Remove this marker",
        "card.anim.marker_tip" => "When the animation enters this frame, it sends this signal",
        "card.anim.marker_past_end" => {
            "Frame {frame} is past the end ({n} {n?frame|frames}): this marker stays silent."
        }
        "card.anim.markers_hidden" => {
            "{n} more {n?marker|markers} on this animation {n?is|are} not shown here, and {n?it fires|they fire} anyway."
        }
        "card.bones.along_out" => "Along out",
        "card.bones.min" => "Min (\u{b0})",
        "card.bones.max" => "Max (\u{b0})",
        "card.bones.from" => "From (\u{b0})",
        "card.bones.to" => "To (\u{b0})",
        "card.bones.mix" => "Mix",
        "card.bones.chain" => "Chain length",
        "card.bones.softness" => "Softness",
        "card.bones.bend" => "Bend",
        "card.bones.bend_ccw" => "Bend counter-clockwise",
        "card.bones.bend_cw" => "Bend clockwise",
        "card.bones.aim" => "Aim offset (\u{b0})",
        _ => return None,
    })
}
