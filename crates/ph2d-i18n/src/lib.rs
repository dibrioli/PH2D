#![forbid(unsafe_code)]
//! ph2d-i18n — internationalization.
//!
//! **M13 stub.** This crate currently ships a **static English string
//! table** keyed by Fluent-style identifiers (`tool.trim_transparency.label`,
//! `tool.make_square.label`, …). The full Fluent/ICU MessageFormat
//! implementation is deferred per the milestone plan; this stub
//! gives consumers a stable call-site shape (`tr("key")`) that the
//! eventual Fluent bundle replaces transparently.
//!
//! # Why a stub, not Fluent right now
//!
//! Fluent requires runtime locale state + per-locale bundles +
//! plural-form resolution + a bundler-source story. None of those are
//! load-bearing for the current milestone (single-locale en-US editor).
//! Shipping the table now centralizes the **strings** so the Fluent
//! migration is a one-touch impl swap, not a 100-callsite rename.
//!
//! # Usage
//!
//! ```
//! use ph2d_i18n::tr;
//! // Image-tool labels are abreviados (cap 5 chars) pra caber no chip;
//! // o tooltip mantém o nome legível por extenso.
//! assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
//! assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
//! assert_eq!(tr("tool.unknown.key"), "tool.unknown.key"); // missing-key passthrough
//! ```

/// Look up a string by Fluent-style key. Missing keys round-trip the
/// key itself so missing entries are visible in the UI (debugging
/// aid) rather than silently rendering as empty.
///
/// Returns `&'static str` — current implementation is a compile-time
/// table. The Fluent migration will widen this to `String` (formatted
/// with arguments) at that point.
pub fn tr(key: &str) -> &'static str {
    match key {
        // Image Tools — action row pills. Labels abreviados (Enio
        // 2026-05-25): cabem na coluna do chip (44 px) sem clip; o
        // tooltip mantém o nome completo + descrição.
        "tool.trim_transparency.label" => "TRIM",
        "tool.trim_transparency.tooltip" => "Trim Transparency",
        "tool.make_square.label" => "SQUAR",
        "tool.make_square.tooltip" => "Make Square",
        "tool.bgremoval.label" => "BGRMV",
        "tool.bgremoval.tooltip" => "Background Removal · 3",
        "tool.real_size.label" => "SIZE",
        "tool.real_size.tooltip" => "Real Size · reset scale to 1:1",
        "tool.padding.label" => "PAD",
        "tool.padding.tooltip" => "Padding · expand or crop canvas edges",
        "tool.color_equalization.label" => "CEQ",
        "tool.color_equalization.tooltip" => {
            "Color Equalization · CLAHE + brightness/contrast/saturation + auto-WB"
        }
        "tool.equalize_sizes.label" => "EQSZ",
        "tool.equalize_sizes.tooltip" => {
            "Equalize Sizes · normalize selection to Max / Fixed / Grid target"
        }
        "tool.rasterize.label" => "RASTR",
        "tool.rasterize.tooltip" => {
            "Rasterize · bake scale + rotation into pixels (reset Transform)"
        }
        "tool.upscale.label" => "UPSC",
        "tool.upscale.tooltip" => "Upscale · resize image up 1x..16x (Lanczos3 / Nearest / xBR)",
        "tool.painter.label" => "PNTR",
        "tool.painter.tooltip" => {
            "Painter · sucessor do Procreate (brush engine GPU, history vetorial, MCP)"
        }
        // Image-edit undo affordance.
        "edit.undo.label" => "Undo",
        "edit.undo.image_edit.toast_hint" => "Undo: Cmd+Z",
        "edit.undo.image_edit.toast_done" => "Undone",
        "edit.undo.image_edit.toast_nothing_to_undo" => "Nothing to undo",
        // Toast strings — Trim / Make Square outcomes. Wiring goes
        // through `tr()` so the shell drainer doesn't hardcode the
        // English copies (HR-15). Format strings ("Trimmed → {w} × {h} px")
        // stay at call-site for now; the Fluent migration moves them
        // here as `format(key, args)`.
        "tool.trim_transparency.toast.nothing" => "Nothing to trim",
        "tool.trim_transparency.toast.unavailable" => "Trim unavailable for this sprite",
        "tool.make_square.toast.already_square" => "Sprite is already square",
        "tool.make_square.toast.unavailable" => "Make Square unavailable for this sprite",
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
        "panel.timeline.ping_pong" => "PingPong",
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
        // Per-track extrapolation badges (plan §6) — the dashed-region mode label,
        // shown on the dope-sheet only when the side is not the default Hold.
        "panel.timeline.extrap.loop" => "Loop",
        "panel.timeline.extrap.pingpong" => "Ping-Pong",
        "panel.timeline.extrap.continue" => "Continue",
        // ── Vector panel (ADR-0108/0112) — section headers, tool modes and the
        // shape catalogue. The panel is a 17-section stack; every section title
        // and every chrome word routes through here (the shape NAMES themselves
        // stay in the `ph2d-tool-vector` catalogue, which is their single source).
        "panel.vector.title" => "Vector",
        "panel.vector.section.tool" => "Tool",
        "panel.vector.section.shape" => "Shape",
        "panel.vector.section.blend" => "Blend",
        "panel.vector.section.morph" => "Morph",
        "panel.vector.section.envelope" => "Envelope",
        "panel.vector.section.textpath" => "Text on Path",
        "panel.vector.section.patternpath" => "Pattern on Path",
        "panel.vector.section.contour" => "Contour",
        "panel.vector.section.effects" => "Effects",
        // O Falloff modula a FORÇA do deformador abaixo dele na pilha; o card diz para onde ele
        // aponta, para que um Falloff sozinho (sem deformador abaixo) não pareça quebrado.
        "panel.vector.fx.falloff.modulates" => "modulates the effect below",
        "panel.vector.fx.falloff.inert" => "add a deformer below (Bulge/Warp/Zig Zag)",
        "panel.vector.section.stroke" => "Stroke",
        "panel.vector.section.fill" => "Fill",
        "panel.vector.section.fill_type" => "Fill Type",
        "panel.vector.section.snap" => "Snap",
        "panel.vector.section.pencil" => "Pencil",
        // A SIMETRIA de desenho (W6.3). ⚠️ Os rótulos dos TIPOS não estão aqui: eles moram em
        // `ph2d_symmetry::SymmetryKind::label`, ao lado do enum, porque uma segunda lista
        // divergiria da primeira no dia em que o vocabulário ganhasse o quinto tipo.
        "panel.vector.section.symmetry" => "Symmetry",
        // A MOLDURA (plano UI/UX W0) — o contêiner.
        "panel.vector.section.frame" => "Frame",
        // **Os TOKENS** (plano UI/UX W4): a row que diz de que token a propriedade segue.
        "panel.vector.token" => "Token",
        // A linha que SOLTA a propriedade — ela volta ao literal do documento.
        "panel.vector.token.none" => "None (use literal)",
        "panel.vector.frame.clip" => "Clip content",
        "panel.vector.transform.resize_box" => "Resize Box",
        "panel.vector.frame.clip.off" => "Off",
        "panel.vector.frame.clip.on" => "On",
        // O interruptor do painel autorado (plano UI/UX W8b.2) — a moldura descreve um painel, e
        // este chip o mostra ao lado, docado.
        "panel.vector.frame.panel" => "Show as Panel",
        "panel.vector.frame.panel.off" => "Off",
        "panel.vector.frame.panel.on" => "On",
        // **AS ÂNCORAS** (plano UI/UX W3) — a regra do filho que NÃO está num fluxo.
        // ⚠️ A vertical é nomeada pelo que se VÊ ("Top"/"Bottom"), e não pelo sinal: o documento é
        // Y-up, então "Top" é a âncora 1. A tradução mora numa tabela só, na shell.
        "panel.vector.section.anchors" => "Constraints",
        "panel.vector.anchors.h" => "Horizontal",
        "panel.vector.anchors.v" => "Vertical",
        "panel.vector.anchors.left" => "Left",
        "panel.vector.anchors.right" => "Right",
        "panel.vector.anchors.top" => "Top",
        "panel.vector.anchors.bottom" => "Bottom",
        "panel.vector.anchors.center" => "Center",
        "panel.vector.anchors.stretch" => "Stretch",
        // **OS COMPONENTES** (plano UI/UX W5) — o prefab: mestre, instância, override.
        // ⚠️ "Main" e não "Master": é a palavra que o Figma passou a usar, e é a que aparece no
        // readout de órfã — os dois lados têm de falar a mesma.
        "panel.vector.section.component" => "Component",
        "panel.vector.component.create" => "Create Component",
        "panel.vector.component.place" => "Place Instance",
        "panel.vector.component.detach" => "Detach Instance",
        "panel.vector.component.reset" => "Reset Overrides",
        "panel.vector.component.missing" => "Main missing",
        // **AS DIFERENÇAS** (W5b) — a lista de peças, o absorver e a troca de mestre.
        "panel.vector.component.pieces" => "Pieces",
        "panel.vector.component.piece_colour" => "Colour",
        // ⚠️ Rótulo PRÓPRIO para a cor que esta cópia autorou: sem ele, *"esta peça está
        // diferente"* só se descobre carregando em Reset e vendo o que muda.
        "panel.vector.component.piece_colour_own" => "Colour (own)",
        "panel.vector.component.pieces_beyond" => "more pieces (edit them in the main)",
        // **OS VARIANTS** (W5c) — que versão do componente esta cópia é.
        // ⚠️ Este rótulo só é usado no modo de NOMES CRUS: quando os mestres irmãos declaram
        // propriedades no nome (`Size=Small`), o rótulo de cada fileira é a propriedade, que é
        // palavra do ARTISTA e nunca passa por aqui.
        "panel.vector.component.variant" => "Variant",
        "panel.vector.component.variants_beyond" => "more variant options (use Swap Main)",
        "panel.vector.component.update_main" => "Update Main",
        "panel.vector.component.swap" => "Swap Main",
        "panel.vector.component.swap_armed" => "Click a component to swap",
        // **A PELE POR-WIDGET** (plano UI/UX W6.2) — a forma veste um widget do catálogo, e o
        // pintor REAL desenha no lugar dela.
        // ⚠️ Os nomes dos tipos são os do catálogo (`ph2d_editor_core::widget`) e passam por aqui
        // como qualquer outro rótulo: eles aparecem na tela, e a lei do repo não abre exceção para
        // substantivo próprio de design system.
        // **OS ESTADOS de UI** (plano UI/UX W7) — os quatro papéis e os três verbos.
        // ⚠️ Os papéis são NOMES DE PAPEL, não de estado livre: "Hover" descreve o que aconteceu
        // com o rato, e é isso que torna o gatilho derivável em vez de autorado.
        "panel.vector.section.states" => "States",
        "panel.vector.states.role.default" => "Default",
        "panel.vector.states.role.hover" => "Hover",
        "panel.vector.states.role.pressed" => "Pressed",
        "panel.vector.states.role.disabled" => "Disabled",
        "panel.vector.states.rec" => "Rec",
        "panel.vector.states.show" => "Show",
        "panel.vector.states.clear" => "Clear",
        // O readout da pré-visualização: sem ele uma cena parada num hover parece o repouso, e a
        // gravação seguinte do Default o sobrescreve com a pose errada.
        "panel.vector.states.showing" => "Showing",
        "panel.vector.states.duration" => "Duration",
        "panel.vector.states.spring" => "Spring",
        "panel.vector.states.stiffness" => "Stiffness",
        "panel.vector.states.damping" => "Damping",
        // **O MODO DE PREVIEW** (W7r). O segundo rótulo diz como SAIR, e não é cortesia: um modo
        // que toma o rato e não anuncia a porta de saída é um modo em que o artista fica preso.
        "panel.vector.states.preview" => "Preview",
        "panel.vector.states.preview.on" => "Preview on — Esc to exit",
        // ⚠️ O rótulo diz o que ACONTECE, não o que a caixa é: *"Move All States"* descreve o
        // efeito do próximo arrasto, e é isso que o artista precisa de decidir antes de arrastar.
        "panel.vector.states.move_all" => "Move All States",
        // **O SELETOR DE CURVA** (W7). *"Curve"* e não *"Easing"*: o artista escolhe a FORMA do
        // movimento, e *easing* é o nome que a implementação lhe dá. Os rótulos dos chips não
        // estão aqui — vêm do `EasingFamily::label()`, porque são o vocabulário do catálogo e não
        // texto deste painel; uma segunda lista aqui divergiria do menu da timeline.
        "panel.vector.states.curve" => "Curve",
        "panel.vector.states.curve.mode" => "Direction",
        "panel.vector.section.widget" => "Widget Skin",
        "panel.vector.widget.wear" => "Wear a Widget",
        "panel.vector.widget.remove" => "Back to Drawing",
        // ⚠️ O rótulo do widget é o NOME da entidade (a Hierarquia), nunca um campo próprio — esta
        // linha é o que torna essa lei visível ao artista em vez de descoberta por acidente.
        "panel.vector.widget.label_is_name" => "Label follows the object name",
        "panel.vector.widget.unknown" => "Unknown widget — drawn as a shape",
        "panel.vector.widget.drives" => "Drives",
        "panel.vector.widget.drives_none" => "nothing yet",
        "panel.vector.widget.bind" => "Bind Shape...",
        "panel.vector.widget.unbind" => "Unbind",
        "panel.vector.widget.kind.button" => "Button",
        "panel.vector.widget.kind.toggle" => "Toggle",
        "panel.vector.widget.kind.checkbox" => "Checkbox",
        "panel.vector.widget.kind.slider" => "Slider",
        "panel.vector.widget.kind.progress" => "Progress",
        "panel.vector.widget.kind.tag" => "Tag",
        "panel.vector.widget.kind.text_input" => "Text Input",
        "panel.vector.widget.kind.card" => "Card",
        "panel.vector.widget.kind.section" => "Section",
        "panel.vector.widget.kind.list_item" => "List Item",
        "panel.vector.widget.kind.spinner" => "Spinner",
        "panel.vector.widget.kind.divider" => "Divider",
        // **O AUTO LAYOUT** (plano UI/UX W2, ADR-0153) — a moldura que empilha os filhos.
        // ⚠️ Os rótulos de direção incluem o "Off" porque *"esta moldura flui?"* e *"em que
        // direção?"* são a MESMA pergunta (o `display` do CSS) — ver `VECTOR_LAYOUT_DIR_OFF`.
        "panel.vector.section.layout" => "Layout",
        "panel.vector.layout.dir" => "Direction",
        "panel.vector.layout.dir.off" => "Off",
        "panel.vector.layout.dir.row" => "Row",
        "panel.vector.layout.dir.col" => "Column",
        "panel.vector.layout.dir.wrap" => "Wrap",
        "panel.vector.layout.gap" => "Gap",
        "panel.vector.layout.padding" => "Padding",
        "panel.vector.layout.padding.all" => "All",
        "panel.vector.layout.padding.each" => "Each",
        "panel.vector.layout.align" => "Align",
        "panel.vector.layout.align.start" => "Start",
        "panel.vector.layout.align.center" => "Center",
        "panel.vector.layout.align.end" => "End",
        "panel.vector.layout.align.stretch" => "Stretch",
        "panel.vector.layout.justify" => "Distribute",
        "panel.vector.layout.justify.start" => "Start",
        "panel.vector.layout.justify.center" => "Center",
        "panel.vector.layout.justify.end" => "End",
        "panel.vector.layout.justify.between" => "Between",
        "panel.vector.layout.justify.around" => "Around",
        "panel.vector.layout.item" => "In parent",
        "panel.vector.symmetry.off" => "Off",
        "panel.vector.symmetry.on" => "On",
        "panel.vector.symmetry.enable" => "Enable",
        "panel.vector.symmetry.axis" => "Axis",
        "panel.vector.symmetry.segments" => "Segments",
        "panel.vector.symmetry.fuse" => "Fuse",
        "panel.vector.symmetry.apply" => "Apply Symmetry",
        // A FONTE da largura de um traço de lápis (W1d). "Pen" e não "Pressure": o rótulo diz o
        // DISPOSITIVO, e é ele que hoje não existe nesta shell — o artista escolhe e não vê
        // diferença nenhuma, o que é a resposta honesta enquanto o caminho do tablet não chega.
        "panel.vector.pencil.width.uniform" => "Uniform",
        "panel.vector.pencil.width.speed" => "Speed",
        "panel.vector.pencil.width.pressure" => "Pen",
        // **O catálogo de perfis de largura** (W2b) — os rótulos da tabela
        // `ph2d_stroke_width::PRESETS`, na ordem em que ela os lista. São VERBOS sobre a curva
        // ("afina", "engrossa"), não números: um nome só serve se descrever a forma, e a tabela
        // foi medida para que descreva (o doc dela traz o multiplicador em cinco pontos do arco).
        "panel.vector.width.preset.uniform" => "Uniform",
        "panel.vector.width.preset.taper" => "Taper",
        "panel.vector.width.preset.both" => "Both",
        "panel.vector.width.preset.bulge" => "Bulge",
        "panel.vector.section.transform" => "Transform",
        "panel.vector.section.vertex" => "Vertex",
        "panel.vector.section.boolean" => "Boolean",
        "panel.vector.section.expand" => "Expand",
        "panel.vector.section.align" => "Align",
        "panel.vector.section.arrange" => "Arrange",
        // **O Z-INDEX** — o lugar da forma na pilha dos IRMÃOS, maior = mais à frente
        // (a convenção do Godot/Unity). Readout: o numero e' derivado da arvore.
        "panel.vector.arrange.z" => "Z",
        "panel.vector.section.path" => "Path",
        "panel.vector.section.text" => "Text",
        "panel.vector.section.font" => "Font",
        "panel.vector.section.paragraph" => "Paragraph",
        "panel.vector.section.axes" => "Axes",
        // A seção do CONECTOR — só aparece com um conector na seleção. Os RÓTULOS dos três
        // campos (Route / Jetty / Spread) vêm do catálogo em `ph2d-tool-vector::connector`,
        // que é a fonte única deles (a mesma regra do catálogo de formas).
        "panel.vector.section.connector" => "Connector",

        // ── Physics world panel (ADR-0131 D8 / W2b) — the WORLD half of physics
        // authoring. The per-BODY half is the Inspector's "Physics Body" section.
        // Labels say what the number DOES — but ⚠️ that cuts both ways: calling
        // the UNIFORM damping "Air Drag" is exactly what made the first smoke
        // fail (Enio: "todos os objetos grandes e pequenos caem na mesma
        // velocidade"). A label has to promise what the model can deliver.
        // Wet Tuning side panel (doc 22) — labels are the model app's own.
        "panel.wet_tuning.title" => "Wet Tuning",
        "panel.wet_tuning.group.paint" => "Paint",
        "panel.wet_tuning.group.water" => "Water",
        "panel.wet_tuning.group.physics" => "Physics",
        "panel.wet_tuning.group.tools" => "Tools",
        "panel.wet_tuning.group.paper" => "Paper",
        "panel.wet_tuning.group.experimental" => "Experimental",
        "panel.wet_tuning.km_mixing" => "Pigment mixing (K-M)",
        "panel.wet_tuning.km_glaze" => "Glaze layering (K-M)",
        "panel.wet_tuning.note" => {
            "Kubelka-Munk subtractive color. Further gated extensions (diffusion, backrun, fingering, dry-brush, render extras) ship neutral; see the tuning registry's hidden group."
        }
        "panel.wet_tuning.knob.pigmentPerDab" => "Pigment per dab",
        "panel.wet_tuning.knob.paperGate" => "Paper gate",
        "panel.wet_tuning.knob.felt" => "Felt (pores)",
        "panel.wet_tuning.knob.bristleCount" => "Bristle count",
        "panel.wet_tuning.knob.drag" => "Drag",
        "panel.wet_tuning.knob.pickup" => "Pickup",
        "panel.wet_tuning.knob.intensity" => "Intensity",
        "panel.wet_tuning.knob.bristleStrength" => "Bristle strength",
        "panel.wet_tuning.knob.bristleSize" => "Bristle size",
        "panel.wet_tuning.knob.spacing" => "Spacing",
        "panel.wet_tuning.knob.tipClean" => "Tip clean",
        "panel.wet_tuning.knob.blendForce" => "Blend force",
        "panel.wet_tuning.knob.gateSaturation" => "Gate saturation",
        "panel.wet_tuning.knob.waterPerDab" => "Water per dab",
        "panel.wet_tuning.knob.waterCap" => "Water cap",
        "panel.wet_tuning.knob.evaporation" => "Evaporation",
        "panel.wet_tuning.knob.rewet" => "Re-wet",
        "panel.wet_tuning.knob.retention" => "Retention",
        "panel.wet_tuning.knob.edgeDarkening" => "Edge darkening",
        "panel.wet_tuning.knob.baseEvaporation" => "Base evaporation",
        "panel.wet_tuning.knob.leveling" => "Leveling",
        "panel.wet_tuning.knob.capillary" => "Capillary",
        "panel.wet_tuning.knob.brake" => "Brake",
        "panel.wet_tuning.knob.gravity" => "Gravity",
        "panel.wet_tuning.knob.levelClamp" => "Level clamp",
        "panel.wet_tuning.knob.viscosity" => "Viscosity",
        "panel.wet_tuning.knob.maxVelocity" => "Max velocity",
        "panel.wet_tuning.knob.projection" => "Projection",
        "panel.wet_tuning.knob.brakeReach" => "Brake reach",
        "panel.wet_tuning.knob.capillaryGate" => "Capillary gate",
        "panel.wet_tuning.knob.eraser" => "Eraser",
        "panel.wet_tuning.knob.dryer" => "Dryer",
        "panel.wet_tuning.knob.blow" => "Blow",
        "panel.wet_tuning.knob.smear" => "Smear",
        "panel.wet_tuning.knob.wetLift" => "Rewet lift",
        "panel.wet_tuning.knob.extStaining" => "Staining",
        "panel.wet_tuning.knob.paperContrast" => "Contrast",
        "panel.wet_tuning.knob.paperFibres" => "Fibres",
        "panel.wet_tuning.knob.paperGrooves" => "Grooves",
        "panel.wet_tuning.knob.visualGrain" => "Visual grain",
        "panel.wet_tuning.knob.emboss" => "Emboss",
        "panel.wet_tuning.knob.paperVisibility" => "Paper visibility",
        // **O painel de TOKENS** (plano UI/UX W6) — a tabela de cor do design system, autorável.
        // ⚠️ Os NOMES dos tokens (`bg-0`, `accent`, …) NÃO passam por aqui: eles são as chaves do
        // `tokens.json`, o endereço que o artista digita no picker de binding e que o arquivo
        // guarda — traduzi-los partiria o endereço.
        "panel.tokens.title" => "Tokens",
        "panel.tokens.authored" => "authored",
        "panel.tokens.reset" => "Reset",
        "panel.tokens.reset_all" => "Reset This Mode",
        // O readout de CONTRASTE (plano UI/UX W4b). ⚠️ O nome do CRITÉRIO ("WCAG 2.2 AA 1.4.3")
        // não passa por aqui: ele é o endereço de uma norma, e traduzi-lo tornaria a coisa que o
        // artista precisa de PROCURAR impossível de procurar — a mesma lei que mantém as chaves
        // dos tokens fora desta tabela.
        "panel.tokens.contrast.title" => "Contrast below WCAG",
        "panel.tokens.contrast.on" => "on",
        // A família NUMÉRICA (plano UI/UX W4c.1) — a escala que se mede em px.
        // ⚠️ O cabeçalho diz a UNIDADE, e é o que separa esta lista da de cima: as duas listam
        // "tokens", e sem a unidade um chip com `8` ao lado de uma swatch não diz de que grandeza
        // se está a falar. A unidade é a razão de as três escalas serem UMA família.
        "panel.tokens.numeric" => "Scale (px)",
        "panel.tokens.formula.hint" => "e.g. {spacing.md} * 2",
        // O INTEROP DTCG (plano UI/UX W9). ⚠️ **"DTCG" não é traduzido**: é o nome próprio do
        // formato W3C que o Tokens Studio / Style Dictionary / Penpot falam, e é a palavra que o
        // artista procura no menu da OUTRA ferramenta — a mesma lei que mantém "WCAG 2.2 AA" e as
        // chaves dos tokens fora desta tabela.
        //
        // ⚠️ E as reticências são ASCII (`...`), como as do `Import Font...` do painel de vetor —
        // este painel não tem outro botão que abra um diálogo com quem ser consistente, e a fonte
        // agrupada cobre o `\u{2026}` mas o gate de tofu não o vigia.
        "panel.tokens.dtcg.export" => "Export DTCG...",
        "panel.tokens.dtcg.import" => "Import DTCG...",
        // O painel da cena 3D (ADR-0150 W12). Os NOMES dos verbos e das curvas
        // NÃO estão aqui: eles vêm de `Verb::label()` / `Falloff::label()`, que
        // é a mesma porta que o log do teclado usa — duas tabelas de nomes para
        // a mesma lista divergiriam no dia em que um verbo fosse renomeado.
        "panel.sculpt3d.title" => "Sculpt 3D",
        "panel.sculpt3d.section.tool" => "Tool",
        "panel.sculpt3d.section.brush" => "Brush",
        "panel.sculpt3d.section.symmetry" => "Symmetry",
        "panel.sculpt3d.section.topology" => "Topology",
        "panel.sculpt3d.section.shading" => "Shading",
        "panel.sculpt3d.section.scene" => "Scene",
        "panel.sculpt3d.radius" => "Radius",
        "panel.sculpt3d.strength" => "Strength",
        "panel.sculpt3d.falloff" => "Falloff",
        "panel.sculpt3d.plane_offset" => "Plane Offset",
        "panel.sculpt3d.pinch" => "Pinch",
        "panel.sculpt3d.alpha" => "Alpha",
        "panel.sculpt3d.alpha.none" => "None",
        "panel.sculpt3d.alpha_scale" => "Alpha Scale",
        "panel.sculpt3d.mask" => "Mask",
        "panel.sculpt3d.mask.clear" => "Clear",
        "panel.sculpt3d.mask.invert" => "Invert",
        "panel.sculpt3d.mask.blur" => "Blur",
        "panel.sculpt3d.mask.sharpen" => "Sharpen",
        "panel.sculpt3d.sym.x" => "X",
        "panel.sculpt3d.sym.y" => "Y",
        "panel.sculpt3d.sym.z" => "Z",
        "panel.sculpt3d.dyntopo" => "Dynamic Topology",
        "panel.sculpt3d.detail" => "Detail",
        "panel.sculpt3d.detail.coarse" => "Coarse",
        "panel.sculpt3d.detail.medium" => "Medium",
        "panel.sculpt3d.detail.fine" => "Fine",
        "panel.sculpt3d.level" => "Level",
        "panel.sculpt3d.subdivide" => "Subdivide",
        "panel.sculpt3d.reverse" => "Reverse",
        "panel.sculpt3d.remesh" => "Remesh",
        "panel.sculpt3d.close_holes" => "Close Holes",
        // "Cavity" e não "Curvature": é o nome que Blender, ZBrush e Substance
        // dão ao MESMO canal, e o artista o procura por ele.
        "panel.sculpt3d.cavity" => "Cavity",
        "panel.sculpt3d.ao" => "Ambient Occlusion",
        "panel.sculpt3d.bake_ao" => "Bake AO",
        "panel.sculpt3d.ssao" => "Screen Occlusion",
        "panel.sculpt3d.sss" => "Subsurface",
        "panel.sculpt3d.sss_scatter" => "Scatter",
        "panel.sculpt3d.ao_stale" => "AO describes the previous shape",
        "panel.sculpt3d.matcap" => "Material",
        // ⚠️ "Rig" e não "None": a primeira opção NÃO é a ausência de luz, é a
        // luz do DOCUMENTO — a mesma lâmpada que acende a tinta ao lado. Chamá-la
        // de "None" faria o artista ler o modo default como "sem sombreamento".
        "panel.sculpt3d.matcap.rig" => "Rig",
        "panel.sculpt3d.wireframe" => "Wireframe",
        "panel.sculpt3d.accumulate" => "Accumulate",
        "panel.sculpt3d.light_az" => "Light Angle",
        "panel.sculpt3d.light_elev" => "Light Height",
        "panel.sculpt3d.add" => "Add",
        "panel.sculpt3d.add.sphere" => "Sphere",
        "panel.sculpt3d.add.cube" => "Cube",
        "panel.sculpt3d.add.cylinder" => "Cylinder",
        "panel.sculpt3d.add.torus" => "Torus",
        "panel.sculpt3d.duplicate" => "Duplicate",
        "panel.sculpt3d.delete" => "Delete",
        "panel.sculpt3d.isolate" => "Isolate",
        "panel.sculpt3d.merge" => "Merge Visible",
        "panel.sculpt3d.pieces" => "Pieces",
        "panel.sculpt3d.verts" => "Vertices",
        "panel.physics.title" => "Physics",
        "panel.physics.section.world" => "World",
        "panel.physics.section.solver" => "Solver",
        // Two DIFFERENT models, and the section headers are what keeps them
        // apart: "Air Drag" scales with a body's cross-section and is resisted
        // by its mass (big things fall faster); "Damping" is a uniform velocity
        // decay that mass cannot enter (everything slows equally). Labelling
        // the uniform one "Air Drag" is what made the first smoke fail.
        "panel.physics.section.air" => "Air Drag",
        "panel.physics.section.damping" => "Damping",
        "panel.physics.air_drag" => "Density",
        "panel.physics.section.layers" => "Collision Layers",
        "panel.physics.section.sleep" => "Sleep",
        "panel.physics.section.debug" => "Debug",
        "panel.physics.gravity_x" => "Gravity X",
        "panel.physics.gravity_y" => "Gravity Y",
        "panel.physics.substeps" => "Sub-steps",
        "panel.physics.iterations" => "Iterations",
        "panel.physics.contact_hz" => "Contact Hz",
        "panel.physics.linear_damping" => "Linear",
        "panel.physics.angular_damping" => "Angular",
        "panel.physics.sleep_speed" => "Speed",
        "panel.physics.sleep_spin" => "Spin",
        "panel.physics.sleep_delay" => "Delay",
        // ── Interaction tool (W-Hand): what the POINTER does to a running scene.
        "panel.physics.section.interact" => "Interaction",
        "panel.physics.tool" => "Tool",
        "panel.physics.tool.hand" => "Hand",
        "panel.physics.tool.explode" => "Blast",
        "panel.physics.tool.attract" => "Pull",
        "panel.physics.hold" => "Hold",
        "panel.physics.hold.spring" => "Spring",
        "panel.physics.hold.rigid" => "Rigid",
        "panel.physics.hold.rope" => "Rope",
        "panel.physics.hold_stiffness" => "Stiffness",
        "panel.physics.hold_damping" => "Damping",
        "panel.physics.hold_slack" => "Slack",
        "panel.physics.blast_radius" => "Radius",
        "panel.physics.blast_force" => "Impulse",
        "panel.physics.pull_radius" => "Radius",
        "panel.physics.pull_force" => "Force",
        "panel.physics.ik_damping" => "Smoothing",
        "panel.physics.ik_angle" => "Tip Angle",
        "panel.physics.ik_angle.free" => "Free",
        "panel.physics.ik_angle.match" => "Match",
        "panel.physics.interact_hint" => "Play + drag on the canvas",
        // ── A seção JOINTS (W-JointTools) ──────────────────────────────────
        // ⚠️ As duas seções de interação existem porque as duas famílias querem
        // estados OPOSTOS do transporte: aquelas três empurram o solver e pedem
        // Play, estas cinco autoram a cena e pedem Pause. A dica de cada modo
        // diz qual — uma dica só mandaria metade dos artistas fazer exatamente
        // o que não funciona.
        "panel.physics.section.joint" => "Joints",
        "panel.physics.joint_tool" => "Drag",
        "panel.physics.joint_tool.body" => "Body",
        "panel.physics.joint_tool.rig" => "Rig",
        "panel.physics.joint_tool.links" => "Links",
        "panel.physics.joint_tool.ik" => "IK",
        "panel.physics.joint_tool.fk" => "FK",
        "panel.physics.joint_hint.body" => "Drag moves only the body you grab",
        "panel.physics.joint_hint.rig" => "Paused: drag carries the whole rig, anchors included",
        "panel.physics.joint_hint.links" => "Paused: drag carries the moving links; anchors stay",
        "panel.physics.joint_hint.ik" => "Paused: drag the tip and the chain bends behind it",
        "panel.physics.joint_hint.fk" => "Paused: drag a link and it swings about its joint",
        "panel.physics.joint_hint.alt" => "Alt while dragging always carries the whole rig",
        "panel.physics.show_colliders" => "Show Colliders",
        "panel.physics.reset_defaults" => "Reset to Defaults",
        // The world scale is `ProjectSettings::pixels_per_meter` — a PROJECT
        // setting. This panel shows it so the metre-valued knobs above can be
        // read in pixels, and deliberately does not own or duplicate it (D4).
        "panel.physics.scale" => "Scale",
        "panel.physics.bodies" => "Bodies",
        "panel.vector.mode.build" => "Build",
        "panel.vector.mode.select" => "Select",
        "panel.vector.mode.node" => "Node",
        "panel.vector.mode.pen" => "Pen",
        "panel.vector.mode.shape" => "Shape",
        "panel.vector.mode.text" => "Text",
        "panel.vector.mode.connect" => "Connect",
        "panel.vector.mode.fillet" => "Fillet",
        "panel.vector.mode.chamfer" => "Chamfer",
        "panel.vector.mode.width" => "Width",
        "panel.vector.mode.pencil" => "Pencil",
        "panel.vector.mode.cut" => "Cut",
        "panel.vector.mode.frame" => "Frame",
        // As oito operações do Pathfinder. As quatro primeiras eram literais no painel até a W5;
        // passam por aqui agora porque a fileira é UMA e metade dela em i18n seria o pior dos dois.
        "panel.vector.bool.union" => "Union",
        "panel.vector.bool.subtract" => "Subtract",
        "panel.vector.bool.intersect" => "Intersect",
        "panel.vector.bool.exclude" => "Exclude",
        "panel.vector.bool.minus_back" => "Minus Back",
        "panel.vector.bool.trim" => "Trim",
        "panel.vector.bool.crop" => "Crop",
        "panel.vector.bool.merge" => "Merge",
        // A BOOLEANA VIVA (plano UI/UX W1): o modo dos oito acima + o commit.
        "panel.vector.bool.live" => "Live",
        "panel.vector.bool.live.off" => "Off",
        "panel.vector.bool.live.on" => "On",
        "panel.vector.bool.apply" => "Apply Boolean",
        "panel.vector.cut.apply" => "Cut",
        "panel.vector.cut.discard" => "Discard Cut Line",
        "panel.vector.section.cut" => "Cut",
        "panel.vector.category" => "Category",
        "panel.vector.shape.no_params" => "No parameters",
        // Stroke markers (arrowheads) — the two selectors in the STROKE section.
        // Only the ROW labels live here: the marker NAMES ("Arrow", "Diamond",
        // "Bar"…) come from `ph2d_vec_scene::Marker::label()`, which is their
        // single source — the same rule the shape catalogue follows.
        "panel.vector.marker.start" => "Start",
        "panel.vector.marker.end" => "End",
        "panel.vector.group.basic" => "Basic",
        "panel.vector.group.round" => "Round",
        "panel.vector.group.arrows" => "Arrows",
        "panel.vector.group.flow" => "Flow",
        "panel.vector.group.bubbles" => "Bubbles",
        "panel.vector.group.symbols" => "Symbols",
        "panel.vector.group.iso" => "3D",
        // Pass-through for unknown keys so the missing entry is
        // visible in the UI (the rendered key is intentionally ugly).
        _ => leak_key(key),
    }
}

/// Stub for the unknown-key path: leak the input into a `&'static`
/// so the return type stays uniform. Only fires on developer typos
/// (unknown keys at runtime), so the small per-typo leak is fine.
/// The Fluent migration replaces this with a `String` return.
fn leak_key(key: &str) -> &'static str {
    Box::leak(key.to_string().into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keys_round_trip_to_english() {
        // Image-tool labels were abbreviated 2026-05-25 to fit the
        // 44-px chip column; tooltips keep the long English form.
        assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
        assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
        assert_eq!(tr("tool.make_square.label"), "SQUAR");
        assert_eq!(tr("edit.undo.label"), "Undo");
        // Timeline panel chrome (W2.E9).
        assert_eq!(tr("panel.timeline.title"), "Timeline");
        assert_eq!(tr("panel.timeline.loop"), "Loop");
        assert_eq!(tr("panel.timeline.prop.translate_x"), "Translate X");
        // Vector panel chrome — section headers + the shape-catalogue words.
        assert_eq!(tr("panel.vector.section.tool"), "Tool");
        assert_eq!(tr("panel.vector.mode.shape"), "Shape");
        assert_eq!(tr("panel.vector.category"), "Category");
        assert_eq!(tr("panel.vector.shape.no_params"), "No parameters");
        assert_eq!(tr("panel.vector.group.iso"), "3D");
    }

    #[test]
    fn unknown_key_returns_the_key_itself() {
        let leaked = tr("tool.nonexistent.foo");
        assert_eq!(leaked, "tool.nonexistent.foo");
    }
}
