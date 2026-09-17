//! Os ROTULOS DE PARAMETRO do catalogo de nos — `node.<tipo>.param.<param>`.
//!
//! Irma da [`super::node_catalog`], que guarda o NOME de cada no. A lei e' a mesma e
//! e' a da fatia 1: **a chave deriva do id, o texto e' autorado**. Aqui o id e' o par
//! `(NodeTypeId, ParamSpec::name)` — exactamente o que um `ParamUiHint` anota —, logo
//! a chave le'-se do proprio sitio onde o rotulo vivia, sem tabela de traducao a meio.
//!
//! ⚠️ **O texto NAO e' derivavel do param, e e' por isso que ele esta' aqui:** `mode`
//! da' *«Aim At»* no `motion.look_at` e *«Acts As»* no `force.vortex` — o mesmo id,
//! duas perguntas diferentes, porque o rotulo nomeia o RESULTADO e nao o campo.
//!
//! ⚠️⚠️ **E `(tipo, param)` NAO e' unico — o gerador acusou a colisao, e o proprio
//! ficheiro que a causa ja' escrevia a razao:** o `motion.spline_wrap` declara DUAS
//! rows sobre o param `path` (*«Shape»*, que escolhe a forma, e *«Use Selected Path»*,
//! que e' um botao), e o comentario ao lado delas diz *«dois GESTOS para o mesmo
//! param, como arrastar um slider e digitar o numero»*. ⇒ a chave de uma ROW leva o
//! WIDGET **quando, e so' quando, o param declara mais do que uma** — uma regra que
//! depende do CONJUNTO de rows e nao da ORDEM delas, logo acrescentar uma terceira
//! nao renomeia as outras duas. ⛔ Nao existe `…param.path` sem sufixo: um so' dos
//! dois gestos a herdar a chave curta seria a ambiguidade a ficar escrita.
//!
//! ⛔ **A cauda foi emendada no CHAMADOR, e quem a nomeou foi o gate.** O campo `param`
//! aparece em tres formas neste repo — literal, caminho de `const`, e campo de
//! tuplo/indice de array/macro —, e uma varredura de texto so' ve' as duas primeiras.
//! O `every_param_label_is_a_key_derived_from_its_type_and_param` le' os hints
//! REGISTADOS: *uma varredura ve' a FORMA do fonte; o gate ve' o que o programa de
//! facto oferece ao painel.*
//!
//! ⚠️⚠️ **ESTE FICHEIRO ESTA' PARTIDO EM DOIS, e o corte e' por FAMILIA DE NO.** A tabela
//! inteira tem `828` chaves e estourava o tecto de LOC (`870` contra `700`); ⛔ a cura e'
//! **corte por responsabilidade** e nunca uma entrada no `FILE_OVERAGE_OK`. A `motion` sozinha
//! e' `430` das `828` — mais de metade —, logo ela sai para [`super::node_params_motion`] e as
//! outras dez familias (`value` `source` `force` `field` `sim` `pulse` `fx` `rig` `audio`
//! `debug`) ficam aqui. ⚠️ **A fronteira nao foi escolhida: ela e' o prefixo do proprio
//! `NodeTypeId`**, que e' como o catalogo ja' se organiza — e quando outra familia crescer
//! acima do tecto, sai da mesma maneira. O gate `o_corte_por_familia_e_honesto` prova que
//! nenhuma chave esta' do lado errado, com piso de populacao nos dois.

/// O texto ingles de cada rotulo de parametro das DEZ familias que nao sao a `motion`.
#[rustfmt::skip]
pub(crate) const ENTRADAS: &[(&str, &str)] = &[
    ("node.audio.bands.param.count", "Bands"),
    ("node.audio.bands.param.file", "Audio File"),
    ("node.audio.bands.param.floor_db", "Floor"),
    ("node.audio.bands.param.gain", "Gain"),
    ("node.audio.bands.param.max_hz", "High"),
    ("node.audio.bands.param.min_hz", "Low"),
    ("node.audio.bands.param.scale", "Scale"),
    ("node.audio.bands.param.smoothing", "Smoothing"),
    ("node.audio.bands.param.weighting", "Weighting"),
    ("node.debug.wave.param.gain", "Gain"),
    ("node.field.box.param.center_x", "Center X"),
    ("node.field.box.param.center_y", "Center Y"),
    ("node.field.box.param.curve", "Curve"),
    ("node.field.box.param.height", "Height"),
    ("node.field.box.param.invert", "Invert"),
    ("node.field.box.param.rotation", "Rotation"),
    ("node.field.box.param.soft", "Softness"),
    ("node.field.box.param.strength", "Strength"),
    ("node.field.box.param.width", "Width"),
    ("node.field.combine.param.clamp", "Clamp"),
    ("node.field.combine.param.mode", "Mode"),
    ("node.field.combine.param.strength", "Strength"),
    ("node.field.index_range.param.curve", "Curve"),
    ("node.field.index_range.param.end", "End"),
    ("node.field.index_range.param.invert", "Invert"),
    ("node.field.index_range.param.key", "Order By"),
    ("node.field.index_range.param.soft", "Softness"),
    ("node.field.index_range.param.start", "Start"),
    ("node.field.radial_sweep.param.center_x", "Center X"),
    ("node.field.radial_sweep.param.center_y", "Center Y"),
    ("node.field.radial_sweep.param.curve", "Curve"),
    ("node.field.radial_sweep.param.end_angle", "End Angle"),
    ("node.field.radial_sweep.param.inner_radius", "Inner Radius"),
    ("node.field.radial_sweep.param.invert", "Invert"),
    ("node.field.radial_sweep.param.radius", "Radius"),
    ("node.field.radial_sweep.param.repetitions", "Repetitions"),
    ("node.field.radial_sweep.param.rotation", "Rotation"),
    ("node.field.radial_sweep.param.soft", "Softness"),
    ("node.field.radial_sweep.param.soft_angular", "Angular Bias"),
    ("node.field.radial_sweep.param.start_angle", "Start Angle"),
    ("node.field.remap.param.clamp", "Clamp"),
    ("node.field.remap.param.contour", "Contour"),
    ("node.field.remap.param.curvature", "Curvature"),
    ("node.field.remap.param.curve", "Curve"),
    ("node.field.remap.param.curve_offset", "Curve Offset"),
    ("node.field.remap.param.inner_offset", "Inner Offset"),
    ("node.field.remap.param.invert", "Invert"),
    ("node.field.remap.param.max", "Max"),
    ("node.field.remap.param.min", "Min"),
    ("node.field.remap.param.multiplier", "Multiplier"),
    ("node.field.remap.param.probability", "Probability"),
    ("node.field.remap.param.seed", "Seed"),
    ("node.field.remap.param.steps", "Steps"),
    ("node.field.remap.param.strength", "Strength"),
    ("node.field.shape.param.curve", "Curve"),
    ("node.field.shape.param.distance", "Distance"),
    ("node.field.shape.param.invert", "Invert"),
    ("node.field.shape.param.mode", "Path Mode"),
    ("node.force.attractor.param.curve", "Curve"),
    ("node.force.attractor.param.inner", "Min Distance"),
    ("node.force.attractor.param.lead", "Predict"),
    ("node.force.attractor.param.peak", "Peak Distance"),
    ("node.force.attractor.param.radius", "Radius"),
    ("node.force.attractor.param.repel", "Repel"),
    ("node.force.attractor.param.reverse", "Reversal Distance"),
    ("node.force.attractor.param.strength", "Strength"),
    ("node.force.attractor.param.target_mode", "Target"),
    ("node.force.attractor.param.target_x", "Target X"),
    ("node.force.attractor.param.target_y", "Target Y"),
    ("node.force.buoyancy.param.density", "Density"),
    ("node.force.buoyancy.param.depth", "Depth"),
    ("node.force.buoyancy.param.drag", "Drag"),
    ("node.force.buoyancy.param.level", "Level"),
    ("node.force.buoyancy.param.wave_amplitude", "Wave Amplitude"),
    ("node.force.buoyancy.param.wave_length", "Wave Length"),
    ("node.force.buoyancy.param.wave_speed", "Wave Speed"),
    ("node.force.buoyancy.param.waves", "Waves"),
    ("node.force.curl.param.lacunarity", "Lacunarity"),
    ("node.force.curl.param.loop_period", "Loop Period"),
    ("node.force.curl.param.octaves", "Octaves"),
    ("node.force.curl.param.offset_x", "Offset X"),
    ("node.force.curl.param.offset_y", "Offset Y"),
    ("node.force.curl.param.roughness", "Roughness"),
    ("node.force.curl.param.scale", "Scale"),
    ("node.force.curl.param.seed", "Seed"),
    ("node.force.curl.param.speed", "Speed"),
    ("node.force.curl.param.strength", "Strength"),
    ("node.force.curl.param.type", "Noise Type"),
    ("node.force.drag.param.coefficient", "Coefficient"),
    ("node.force.drag.param.scale_x", "Drag X"),
    ("node.force.drag.param.scale_y", "Drag Y"),
    ("node.force.vortex.param.air_resist", "Air Resistance"),
    ("node.force.vortex.param.center_x", "Center X"),
    ("node.force.vortex.param.center_y", "Center Y"),
    ("node.force.vortex.param.clockwise", "Clockwise"),
    ("node.force.vortex.param.curve", "Curve"),
    ("node.force.vortex.param.mode", "Acts As"),
    ("node.force.vortex.param.radius", "Radius"),
    ("node.force.vortex.param.strength", "Strength"),
    ("node.force.wind.param.air_resist", "Air Resistance"),
    ("node.force.wind.param.angle", "Angle"),
    ("node.force.wind.param.gust", "Gust"),
    ("node.force.wind.param.gust_freq", "Gust Frequency"),
    ("node.force.wind.param.lacunarity", "Lacunarity"),
    ("node.force.wind.param.loop_period", "Loop Period"),
    ("node.force.wind.param.mode", "Acts As"),
    ("node.force.wind.param.octaves", "Octaves"),
    ("node.force.wind.param.roughness", "Roughness"),
    ("node.force.wind.param.seed", "Seed"),
    ("node.force.wind.param.strength", "Strength"),
    ("node.force.wind.param.type", "Noise Type"),
    ("node.fx.drop_shadow.param.direction", "Direction"),
    ("node.fx.drop_shadow.param.distance", "Distance"),
    ("node.fx.drop_shadow.param.r", "Color"),
    ("node.fx.drop_shadow.param.shadow_blend", "Shadow Blend"),
    ("node.fx.drop_shadow.param.softness", "Softness"),
    ("node.fx.glow.param.angle", "Streak Angle"),
    ("node.fx.glow.param.clamp", "Clamp"),
    ("node.fx.glow.param.dirt", "Dirt Texture"),
    ("node.fx.glow.param.dirt_intensity", "Dirt Intensity"),
    ("node.fx.glow.param.intensity", "Intensity"),
    ("node.fx.glow.param.knee", "Soft Knee"),
    ("node.fx.glow.param.operation", "Operation"),
    ("node.fx.glow.param.radius", "Radius"),
    ("node.fx.glow.param.ramp", "Halo Ramp"),
    ("node.fx.glow.param.saturation", "Saturation"),
    ("node.fx.glow.param.source", "Glow Based On"),
    ("node.fx.glow.param.stretch", "Anamorphic"),
    ("node.fx.glow.param.threshold", "Threshold"),
    ("node.fx.glow.param.tint_r", "Tint"),
    ("node.fx.rgb_split.param.center_x", "Axis X"),
    ("node.fx.rgb_split.param.center_y", "Axis Y"),
    ("node.fx.rgb_split.param.mode", "Mode"),
    ("node.fx.rgb_split.param.opacity", "Opacity"),
    ("node.fx.rgb_split.param.start", "Start Radius"),
    ("node.fx.rgb_split.param.strength", "Strength"),
    ("node.fx.rgb_split.param.x", "Offset X"),
    ("node.fx.rgb_split.param.y", "Offset Y"),
    ("node.pulse.adsr.param.attack", "Attack"),
    ("node.pulse.adsr.param.attack_shape", "Attack Shape"),
    ("node.pulse.adsr.param.decay", "Decay"),
    ("node.pulse.adsr.param.delay", "Delay"),
    ("node.pulse.adsr.param.hold", "Hold"),
    ("node.pulse.adsr.param.release", "Release"),
    ("node.pulse.adsr.param.release_shape", "Release Shape"),
    ("node.pulse.adsr.param.retrigger", "Retrigger"),
    ("node.pulse.adsr.param.sustain", "Sustain"),
    ("node.pulse.beat.param.bpm", "BPM"),
    ("node.pulse.beat.param.count", "Beat Count"),
    ("node.pulse.beat.param.offset", "Offset"),
    ("node.pulse.beat.param.period", "Period"),
    ("node.pulse.beat.param.phase_stagger", "Phase Stagger"),
    ("node.pulse.beat.param.time_mode", "Time Mode"),
    ("node.pulse.compare.param.edge", "Edge"),
    ("node.pulse.compare.param.fall", "Fall"),
    ("node.pulse.compare.param.rise", "Rise"),
    ("node.pulse.counter.param.count_max", "Count"),
    ("node.pulse.counter.param.mode", "Mode"),
    ("node.pulse.counter.param.reset_to", "Reset To"),
    ("node.pulse.counter.param.step", "Increment"),
    ("node.pulse.on_change.param.direction", "Direction"),
    ("node.pulse.on_change.param.epsilon", "Epsilon"),
    ("node.pulse.signal.param.name", "Signal Name"),
    ("node.pulse.threshold.param.channel", "Channel"),
    ("node.pulse.threshold.param.debounce", "Debounce"),
    ("node.pulse.threshold.param.edge", "Edge"),
    ("node.pulse.threshold.param.fall", "Fall"),
    ("node.pulse.threshold.param.rise", "Rise"),
    ("node.rig.fabrik.param.iterations", "Iterations"),
    ("node.rig.ik_2bone.param.flip", "Flip Elbow"),
    ("node.rig.ik_2bone.param.root", "Root Joint"),
    ("node.rig.rubber_hose.param.flip", "Flip Bend"),
    ("node.rig.skeleton.param.angle", "Bend"),
    ("node.rig.skeleton.param.branches", "Branches"),
    ("node.rig.skeleton.param.joints", "Joints"),
    ("node.rig.skeleton.param.length", "Bone Length"),
    ("node.rig.skeleton.param.root_angle", "Root Angle"),
    ("node.rig.skin_deformer.param.falloff", "Stiffness"),
    ("node.sim.collide.param.angle", "Angle"),
    ("node.sim.collide.param.box_height", "Box Height"),
    ("node.sim.collide.param.box_width", "Box Width"),
    ("node.sim.collide.param.center_x", "Center X"),
    ("node.sim.collide.param.center_y", "Center Y"),
    ("node.sim.collide.param.friction", "Friction"),
    ("node.sim.collide.param.height", "Offset"),
    ("node.sim.collide.param.particle_radius", "Particle Radius"),
    ("node.sim.collide.param.radius", "Radius"),
    ("node.sim.collide.param.radius_from", "Radius From"),
    ("node.sim.collide.param.restitution", "Bounce"),
    ("node.sim.collide.param.restitution_randomness", "Restitution Randomness"),
    ("node.sim.collide.param.seed", "Seed"),
    ("node.sim.collide.param.shape", "Shape"),
    ("node.sim.collide.param.size_scale", "Size Scale"),
    ("node.sim.lifetime.param.life", "Life"),
    ("node.sim.lifetime.param.seed", "Seed"),
    ("node.sim.lifetime.param.variance", "Variance"),
    ("node.sim.spawn.param.burst", "Burst"),
    ("node.sim.spawn.param.burst_speed", "Burst Speed"),
    ("node.sim.spawn.param.probability", "Probability"),
    ("node.sim.spawn.param.rate", "Rate"),
    ("node.sim.spawn.param.scatter", "Scatter"),
    ("node.sim.spawn.param.seed", "Seed"),
    ("node.sim.step.param.angular_damping", "Angular Damping"),
    ("node.sim.step.param.damping", "Damping"),
    ("node.sim.step.param.max_speed", "Speed Limit"),
    ("node.sim.step.param.min_speed", "Min Speed"),
    ("node.sim.zone.param.duration", "Duration"),
    ("node.sim.zone.param.loop_delay", "Loop Delay"),
    ("node.sim.zone.param.mode", "Life Cycle"),
    ("node.sim.zone.param.start", "Start"),
    ("node.sim.zone.param.substeps", "Substeps"),
    ("node.source.lsystem.param.angle", "Angle"),
    ("node.source.lsystem.param.axiom", "Axiom"),
    ("node.source.lsystem.param.bend", "Bend"),
    ("node.source.lsystem.param.branches", "Branches"),
    ("node.source.lsystem.param.continuous_angle", "Grow Angle"),
    ("node.source.lsystem.param.continuous_length", "Grow Length"),
    ("node.source.lsystem.param.generations", "Generations"),
    ("node.source.lsystem.param.geometry", "Geometry"),
    ("node.source.lsystem.param.growth", "Growth"),
    ("node.source.lsystem.param.leaf_angle", "Leaf Angle"),
    ("node.source.lsystem.param.leaf_effects", "Effects Reach Leaves"),
    ("node.source.lsystem.param.leaf_first_level", "First Level"),
    ("node.source.lsystem.param.leaf_front", "Leaves In Front"),
    ("node.source.lsystem.param.leaf_j", "Leaf (J)"),
    ("node.source.lsystem.param.leaf_k", "Leaf (K)"),
    ("node.source.lsystem.param.leaf_m", "Leaf (M)"),
    ("node.source.lsystem.param.leaf_pos_jitter", "Position Jitter"),
    ("node.source.lsystem.param.leaf_size", "Leaf Size"),
    ("node.source.lsystem.param.leaf_size_jitter", "Size Jitter"),
    ("node.source.lsystem.param.leaf_spread", "Leaf Spread"),
    ("node.source.lsystem.param.length_scale", "Length Scale"),
    ("node.source.lsystem.param.mode", "Mode"),
    ("node.source.lsystem.param.orient", "Shape Faces"),
    ("node.source.lsystem.param.preset", "Preset"),
    ("node.source.lsystem.param.root_angle", "Root Angle"),
    ("node.source.lsystem.param.rules", "Rules"),
    ("node.source.lsystem.param.seed", "Seed"),
    ("node.source.lsystem.param.segments", "Trunk Segments"),
    ("node.source.lsystem.param.step", "Step"),
    ("node.source.lsystem.param.step_scale", "Step Scale"),
    ("node.source.lsystem.param.tip_taper", "Tip Taper"),
    ("node.source.lsystem.param.tropism", "Tropism"),
    ("node.source.lsystem.param.tropism_angle", "Tropism Direction"),
    ("node.source.lsystem.param.variation", "Variation"),
    ("node.source.lsystem.param.width", "Width"),
    ("node.source.lsystem.param.width_scale", "Width Scale"),
    ("node.source.object.param.object", "Object"),
    ("node.source.object.param.space", "Transform"),
    ("node.source.object.param.time_offset", "Time Offset"),
    ("node.source.shape.param.aspect", "Aspect (H/W)"),
    ("node.source.shape.param.bounce", "Bounciness"),
    ("node.source.shape.param.cleft", "Cleft"),
    ("node.source.shape.param.collide", "Collide"),
    ("node.source.shape.param.collider_height", "Collider Height"),
    ("node.source.shape.param.collider_radius", "Collider Radius"),
    ("node.source.shape.param.collider_shape", "Collider Shape"),
    ("node.source.shape.param.collider_width", "Collider Width"),
    ("node.source.shape.param.corner", "Corner Radius"),
    ("node.source.shape.param.corner_bl", "Corner BL"),
    ("node.source.shape.param.corner_br", "Corner BR"),
    ("node.source.shape.param.corner_tr", "Corner TR"),
    ("node.source.shape.param.dash", "Dash"),
    ("node.source.shape.param.dash_gap", "Dash Gap"),
    ("node.source.shape.param.fill", "Own Fill"),
    ("node.source.shape.param.fill_r", "Fill"),
    ("node.source.shape.param.friction", "Friction"),
    ("node.source.shape.param.hole", "Hole"),
    ("node.source.shape.param.inner", "Inner"),
    ("node.source.shape.param.kind", "Shape"),
    ("node.source.shape.param.lock_rotation", "Lock Rotation"),
    ("node.source.shape.param.rolling", "Rolling Friction"),
    ("node.source.shape.param.rotation", "Rotation"),
    ("node.source.shape.param.show_collider", "Show Collider"),
    ("node.source.shape.param.sides", "Sides / Points / Teeth"),
    ("node.source.shape.param.size", "Size"),
    ("node.source.shape.param.smoothing", "Smoothing"),
    ("node.source.shape.param.star_depth", "Point Depth"),
    ("node.source.shape.param.start", "Start"),
    ("node.source.shape.param.stroke_r", "Stroke"),
    ("node.source.shape.param.stroke_width", "Stroke Width"),
    ("node.source.shape.param.sweep", "Sweep"),
    ("node.source.shape.param.tooth_depth", "Tooth Depth"),
    ("node.source.shape.param.trim_end", "Trim End"),
    ("node.source.shape.param.trim_offset", "Trim Offset"),
    ("node.source.shape.param.trim_start", "Trim Start"),
    ("node.source.table.param.file", "Table File"),
    ("node.source.table.param.spacing", "Spacing"),
    ("node.source.text.param.align", "Align"),
    ("node.source.text.param.font", "Font"),
    ("node.source.text.param.line_height", "Line Height"),
    ("node.source.text.param.pivot", "Pivot"),
    ("node.source.text.param.size", "Size"),
    ("node.source.text.param.text", "Text"),
    ("node.source.text.param.tracking", "Tracking"),
    ("node.source.text.param.weight", "Weight"),
    ("node.value.attribute.param.attr", "Read"),
    ("node.value.curve.param.curve", "Curve"),
    ("node.value.curve.param.factor", "Factor"),
    ("node.value.curve.param.in_hi", "In High"),
    ("node.value.curve.param.in_lo", "In Low"),
    ("node.value.curve.param.out_hi", "Out High"),
    ("node.value.curve.param.out_lo", "Out Low"),
    ("node.value.gain.param.mode", "Mode"),
    ("node.value.gain.param.strength", "Strength"),
    ("node.value.instance_field.param.key", "Key By"),
    ("node.value.instance_field.param.mode", "Mode"),
    ("node.value.instance_field.param.seed", "Seed"),
    ("node.value.instance_field.param.unique_per_node", "Unique Per Node"),
    ("node.value.lfo.param.amplitude", "Amplitude"),
    ("node.value.lfo.param.bpm", "BPM"),
    ("node.value.lfo.param.fade_in", "Fade In"),
    ("node.value.lfo.param.offset", "Offset"),
    ("node.value.lfo.param.period", "Period"),
    ("node.value.lfo.param.phase", "Phase"),
    ("node.value.lfo.param.phase_stagger", "Phase Stagger"),
    ("node.value.lfo.param.time_mode", "Time Mode"),
    ("node.value.lfo.param.wave", "Wave"),
    ("node.value.map_range.param.clamp", "Clamp"),
    ("node.value.map_range.param.in_hi", "In High"),
    ("node.value.map_range.param.in_lo", "In Low"),
    ("node.value.map_range.param.interpolation", "Interpolation"),
    ("node.value.map_range.param.out_hi", "Out High"),
    ("node.value.map_range.param.out_lo", "Out Low"),
    ("node.value.map_range.param.steps", "Steps"),
    ("node.value.math.param.distance", "Distance"),
    ("node.value.math.param.epsilon", "Epsilon"),
    ("node.value.math.param.op", "Op"),
    ("node.value.median.param.radius", "Radius"),
    ("node.value.median.param.tolerance", "Tolerance"),
    ("node.value.mix.param.blend", "Blend"),
    ("node.value.mix.param.clamp", "Clamp Factor"),
    ("node.value.mix.param.clamp_result", "Clamp Result"),
    ("node.value.mix.param.factor", "Factor"),
    ("node.value.noise.param.amplitude", "Amplitude"),
    ("node.value.noise.param.feature", "Cell"),
    ("node.value.noise.param.frequency", "Frequency"),
    ("node.value.noise.param.jitter", "Jitter"),
    ("node.value.noise.param.kernel", "Pattern"),
    ("node.value.noise.param.lacunarity", "Lacunarity"),
    ("node.value.noise.param.loop_period", "Loop"),
    ("node.value.noise.param.octaves", "Octaves"),
    ("node.value.noise.param.offset", "Offset"),
    ("node.value.noise.param.pan_x", "Pan X"),
    ("node.value.noise.param.pan_y", "Pan Y"),
    ("node.value.noise.param.roughness", "Roughness"),
    ("node.value.noise.param.seed", "Seed"),
    ("node.value.noise.param.space", "Sample"),
    ("node.value.noise.param.speed", "Speed"),
    ("node.value.normalize.param.mode", "Mode"),
    ("node.value.number.param.kind", "Kind"),
    ("node.value.number.param.state", "State"),
    ("node.value.number.param.value", "Value"),
    ("node.value.pattern.param.interp", "Interpolation"),
    ("node.value.pattern.param.offset", "Offset"),
    ("node.value.pattern.param.steps", "Steps"),
    ("node.value.pattern.param.table", "Table"),
    ("node.value.pattern.param.v0", "V0"),
    ("node.value.pattern.param.v1", "V1"),
    ("node.value.pattern.param.v2", "V2"),
    ("node.value.pattern.param.v3", "V3"),
    ("node.value.pattern.param.v4", "V4"),
    ("node.value.pattern.param.v5", "V5"),
    ("node.value.pattern.param.v6", "V6"),
    ("node.value.pattern.param.v7", "V7"),
    ("node.value.percentile.param.percentile", "Percentile"),
    ("node.value.percentile.param.radius", "Radius"),
    ("node.value.quantize.param.mode", "Mode"),
    ("node.value.quantize.param.offset", "Offset"),
    ("node.value.quantize.param.step", "Step"),
    ("node.value.reduce.param.mode", "Mode"),
    ("node.value.slope.param.scale", "Scale"),
    ("node.value.smooth.param.radius", "Radius"),
    ("node.value.smooth.param.weight", "Weight"),
    ("node.value.smooth.param.window", "Window"),
    ("node.value.step.param.invert", "Invert"),
    ("node.value.step.param.mode", "Mode"),
    ("node.value.step.param.threshold", "Threshold"),
    ("node.value.step.param.width", "Width"),
    ("node.value.switch.param.blend", "Blend"),
    ("node.value.switch.param.lazy", "Skip Unused Inputs"),
    ("node.value.table.param.file", "Table File"),
    ("node.value.table.param.interp", "Interpolation"),
    ("node.value.table.param.outside", "Outside"),
    ("node.value.table.param.time", "Time Column"),
    ("node.value.table.param.value", "Value Column"),
    ("node.value.time.param.offset", "Offset"),
    ("node.value.time.param.rate", "Rate"),
    ("node.value.time.param.stagger", "Phase Stagger"),
    ("node.value.unary.param.op", "Op"),
    ("node.value.wave.param.amplitude", "Amplitude"),
    ("node.value.wave.param.frequency", "Frequency"),
    ("node.value.wave.param.offset", "Offset"),
    ("node.value.wave.param.phase", "Phase"),
    ("node.value.wave.param.wave", "Wave"),
    ("node.value.wrap.param.hi", "Max"),
    ("node.value.wrap.param.lo", "Min"),
    ("node.value.wrap.param.mode", "Mode"),
];

/// Resolve uma chave `node.<tipo>.param.<param>` — as duas metades do corte, por esta ordem
/// porque a `motion` e' mais de metade das chaves. `None` deixa o chamador seguir para a
/// proxima tabela, que e' como o `lib.rs` encadeia as familias.
#[must_use]
pub(crate) fn tr(chave: &str) -> Option<&'static str> {
    super::node_params_motion::ENTRADAS
        .binary_search_by_key(&chave, |(k, _)| k)
        .ok()
        .map(|i| super::node_params_motion::ENTRADAS[i].1)
        .or_else(|| {
            ENTRADAS
                .binary_search_by_key(&chave, |(k, _)| k)
                .ok()
                .map(|i| ENTRADAS[i].1)
        })
}

#[cfg(test)]
mod testes_do_corte {
    /// ⭐⭐ **O CORTE POR FAMÍLIA É HONESTO, e sem isto ele seria uma convenção.**
    ///
    /// ⛔ Uma chave `node.motion.*` escrita neste ficheiro **resolve na mesma** — o `tr` procura
    /// nas duas metades —, logo o defeito seria **mudo** até alguém voltar a estourar o tecto de
    /// LOC e não perceber porquê. ⚠️ E o piso de população existe nas duas metades pela razão de
    /// sempre: *duas listas vazias satisfazem qualquer regra de pertença.*
    #[test]
    fn o_corte_por_familia_e_honesto() {
        let motion = super::super::node_params_motion::ENTRADAS;
        let resto = super::ENTRADAS;
        assert!(motion.len() >= 400, "só {} chaves `motion`", motion.len());
        assert!(
            resto.len() >= 350,
            "só {} chaves fora da `motion`",
            resto.len()
        );

        let intrusos: Vec<_> = resto
            .iter()
            .filter(|(k, _)| k.starts_with("node.motion."))
            .map(|(k, _)| *k)
            .collect();
        assert!(
            intrusos.is_empty(),
            "estas chaves da família `motion` estão no ficheiro das outras dez:\n  {}",
            intrusos.join("\n  ")
        );
        let fugidas: Vec<_> = motion
            .iter()
            .filter(|(k, _)| !k.starts_with("node.motion."))
            .map(|(k, _)| *k)
            .collect();
        assert!(
            fugidas.is_empty(),
            "estas chaves não são da família `motion` e estão no ficheiro dela:\n  {}",
            fugidas.join("\n  ")
        );
    }

    /// ⛔ **As duas metades têm de estar ORDENADAS** — o `tr` procura por `binary_search`, e uma
    /// entrada fora de ordem faz a busca devolver `None` para uma chave que ESTÁ na tabela.
    /// ⚠️ Isso pinta o identificador cru e faz `leak_key` por quadro, e nenhum outro gate o vê:
    /// os dois do registo afirmam que a chave é bem derivada, não que ela é encontrável.
    #[test]
    fn as_duas_metades_estao_ordenadas() {
        for (nome, t) in [
            ("motion", super::super::node_params_motion::ENTRADAS),
            ("resto", super::ENTRADAS),
        ] {
            assert!(
                t.windows(2).all(|w| w[0].0 < w[1].0),
                "a tabela `{nome}` não está estritamente ordenada"
            );
            // e a busca de facto acha cada uma — o controlo positivo da ordenação
            for (k, v) in t {
                assert_eq!(super::tr(k), Some(*v), "`{k}` não é encontrável");
            }
        }
    }
}
