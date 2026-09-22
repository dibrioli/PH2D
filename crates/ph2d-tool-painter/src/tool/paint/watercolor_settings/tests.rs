use crate::tool::PainterTool;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_painter_brush::{TextureKind, TextureMapping};

/// The full panel→tool seam EFFECT (the other half of the panel's `tests/seam.rs` forward proof):
/// the exact `PanelEvent`s the panel forwards, fed to `handle_panel_event`, mutate the observable
/// brush state (read back through the published `BrushSettings` snapshot). Also pins the clamps.
#[test]
fn panel_events_drive_watercolor_state() {
    let mut t = PainterTool::default();
    assert!(!t.brush_settings().watercolor, "default off");

    // The medium is picked from the Paint Mode dropdown (2026-07-22), not a section checkbox.
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_MEDIA,
        "1".into(),
    ));
    assert!(t.brush_settings().watercolor, "Wet edges toggled on");

    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_EDGE,
        3.0,
    ));
    assert_eq!(t.brush_settings().edge_gain, 3.0, "Edge slider set");

    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_GRANULATION,
        0.5,
    ));
    assert_eq!(t.brush_settings().granulation, 0.5, "Granulation set");

    // Pigment: the merged slider (Mix id) drives BOTH the amount and the on/off gate.
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_MIX,
        0.75,
    ));
    let b = t.brush_settings();
    assert!(b.pigment, "Pigment slider > 0 enables the gate");
    assert_eq!(b.pigment_mix, 0.75, "Pigment amount set");
    // Sliding to 0 turns the gate off but REMEMBERS the amount (zero-loss merge).
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_MIX,
        0.0,
    ));
    let b = t.brush_settings();
    assert!(!b.pigment, "Pigment slider 0 disables the gate");
    assert_eq!(
        b.pigment_mix, 0.75,
        "amount remembered while the gate is off"
    );
    // Re-enable for the rest of the sweep (so the reset assertion at the end has a gate to clear).
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_MIX,
        0.75,
    ));

    // Paper COLOUR from the shared picker's read-back (the document ground; "r,g,b" 8-bit).
    assert_eq!(
        t.brush_settings().paper_color,
        [1.0, 1.0, 1.0],
        "paper defaults to white"
    );
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        "239,233,220".into(),
    ));
    let pc = t.brush_settings().paper_color;
    assert_eq!(
        [
            (pc[0] * 255.0 + 0.5) as u8,
            (pc[1] * 255.0 + 0.5) as u8,
            (pc[2] * 255.0 + 0.5) as u8
        ],
        [239, 233, 220],
        "paper colour routed from the picker read-back"
    );

    // Render-path optics: Fill / Depth / Warp drive the same seam.
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_FILL,
        0.4,
    ));
    assert_eq!(t.brush_settings().fill, 0.4, "Fill set");
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_DEPTH,
        2.0,
    ));
    assert_eq!(t.brush_settings().depth, 2.0, "Depth set");
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_WARP,
        10.0,
    ));
    assert_eq!(t.brush_settings().warp, 10.0, "Warp set");

    // Wet Mix: the Smudge + Wet sliders drive the same seam (clamped 0..1).
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_SMUDGE,
        0.8,
    ));
    assert!(
        (t.brush_settings().wet_smudge - 0.8).abs() < 1e-6,
        "Smudge set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_WET,
        9.0,
    ));
    assert_eq!(t.brush_settings().wet_rewet, 1.0, "Wet clamped to 1");

    // Wet Mix mixer knobs: Charge / Dilution / Pull drive the same seam (clamped 0..1).
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_CHARGE,
        0.3,
    ));
    assert!(
        (t.brush_settings().wet_charge - 0.3).abs() < 1e-6,
        "Charge set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_DILUTION,
        0.6,
    ));
    assert!(
        (t.brush_settings().wet_dilution - 0.6).abs() < 1e-6,
        "Dilution set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_PULL,
        2.0,
    ));
    assert_eq!(t.brush_settings().wet_pull, 1.0, "Pull clamped to 1");

    // Paper + Granulation slots: kind picker, Size, Angle, and the "Same as Paper" toggle.
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_KIND,
        (TextureKind::PaperRough.to_u8()).to_string(),
    ));
    assert_eq!(
        t.paint.brush.paper.kind,
        TextureKind::PaperRough,
        "Paper kind picked"
    );
    assert_eq!(
        t.paint.brush.paper.mapping,
        TextureMapping::Tiled,
        "paper forced canvas-anchored"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_PAPER_SIZE_X,
        50.0,
    ));
    assert_eq!(
        t.paint.brush.paper.size[0], 50.0,
        "Paper Size X set (0.1..100)"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_PAPER_ANGLE,
        45.0,
    ));
    assert_eq!(t.paint.brush.paper.angle_deg, 45, "Paper Angle set");
    assert!(
        t.brush_settings().granulation_use_paper,
        "Same as Paper default on"
    );
    t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_WATERCOLOR_GRAN_SAME));
    assert!(
        !t.brush_settings().granulation_use_paper,
        "Same as Paper toggled off"
    );

    // Full Paper slot: Mapping / Rake / Random / Offset / Depth / param.
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_MAPPING,
        (TextureMapping::Random.to_u8()).to_string(),
    ));
    assert_eq!(
        t.paint.brush.paper.mapping,
        TextureMapping::Random,
        "Paper mapping picked"
    );
    // No Paper Rake: the paper is the canvas-anchored substrate under the paint, so a per-dab rotation has
    // nothing to rotate. The widgets never existed; the setters/ids that lingered "for the API" were dead
    // plumbing and were removed (2026-07-12) — an audit had already misread them as live knobs. Rake is
    // real on the **Grain** slot, which IS a stamp. (Per-slot Random Angle was retired everywhere 2026-07-19.)
    assert!(
        !t.paint.brush.paper.rake,
        "the Paper slot has no per-dab rotation, and nothing can turn it on"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_PAPER_OFFSET_X,
        0.3,
    ));
    assert!(
        (t.paint.brush.paper.offset[0] - 0.3).abs() < 1e-6,
        "Paper Offset X set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_PAPER_DEPTH,
        0.7,
    ));
    assert!(
        (t.paint.brush.paper_depth - 0.7).abs() < 1e-6,
        "Paper Depth set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_PAPER_PARAMS[2],
        0.8,
    ));
    assert!(
        (t.paint.brush.paper.params[2] - 0.8).abs() < 1e-6,
        "Paper param slot 2 set"
    );
    // Reset clears the Paper slot back to None.
    t.handle_panel_event(PanelEvent::Click(
        crate::ids::PAINTER_WATERCOLOR_PAPER_RESET,
    ));
    assert_eq!(
        t.paint.brush.paper.kind,
        TextureKind::None,
        "Paper reset to empty"
    );

    // Clamp: Edge caps at 8, Spread at 48.
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_EDGE,
        99.0,
    ));
    assert_eq!(t.brush_settings().edge_gain, 8.0, "Edge clamped to 8");
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WATERCOLOR_SPREAD,
        99.0,
    ));
    assert_eq!(t.brush_settings().edge_spread, 48.0, "Spread clamped to 48");

    // Reset returns the whole section to defaults — the `watercolor`/`pigment` gates OFF (which is
    // what makes a brush neutral); the params go back to their sensible when-enabled defaults.
    t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_WATERCOLOR_RESET));
    let b = t.brush_settings();
    assert!(
        !b.watercolor && !b.pigment,
        "reset turned the Watercolor + Pigment gates off"
    );
    assert_eq!(b.edge_gain, 1.5, "reset restored the default Edge gain");
}

/// The Preset dropdown seam: `SelectOption(PAINTER_BRUSH_PRESET, idx)` reconfigures the whole brush.
/// Watercolor Basic turns the render-path on with the wet_edges knobs; Digital Basic turns it back
/// off — both PRESERVING the user's colour + radius (a preset is a look, not a what/where reset).
#[test]
fn preset_dropdown_reconfigures_the_brush() {
    let mut t = PainterTool::default();
    // Give the brush a distinctive colour + size the preset must preserve.
    t.paint.brush.color = [0.2, 0.6, 0.9];
    t.paint.brush.radius_px = 40.0;

    // Watercolor Basic (idx 1): render-path on + wet_edges optics.
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_PRESET,
        "1".into(),
    ));
    let b = t.brush_settings();
    assert!(b.watercolor, "Watercolor Basic turns the render-path on");
    assert_eq!(b.edge_gain, 3.0, "wet_edges edge gain");
    assert_eq!(b.fill, 0.12, "wet_edges fill");
    assert_eq!(b.depth, 1.2, "wet_edges depth");
    assert_eq!(
        b.color,
        [0.2, 0.6, 0.9],
        "colour preserved across the preset"
    );
    assert_eq!(
        t.paint.brush.radius_px, 40.0,
        "radius preserved across the preset"
    );
    // Paper slot wired to a canvas-anchored cold-press paper (the substrate the wash sits on).
    assert_eq!(
        t.paint.brush.paper.kind,
        TextureKind::PaperCold,
        "Paper = cold-press"
    );
    assert_eq!(
        t.paint.brush.paper.mapping,
        TextureMapping::Tiled,
        "paper is canvas-anchored"
    );

    // Digital Basic (idx 0): back to the plain brush, colour + size still preserved.
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_PRESET,
        "0".into(),
    ));
    let b = t.brush_settings();
    assert!(!b.watercolor, "Digital Basic turns the render-path off");
    assert_eq!(b.color, [0.2, 0.6, 0.9], "colour still preserved");
    assert_eq!(t.paint.brush.radius_px, 40.0, "radius still preserved");
}

/// A tagged layer installs into the RIGHT slot: "Use as Paper" → the Paper slot; "Use as Granulation"
/// → the Granulation slot (Same-as-Paper off, its own map). The two are distinct destinations, not the
/// same Grain slot (the bug Enio caught).
#[test]
fn use_layers_routes_paper_and_granulation_to_separate_slots() {
    let lum = vec![128u8; 8 * 8];

    let mut t = PainterTool::default();
    t.use_layers_as_watercolor_paper(lum.clone(), 8, 8);
    let b = &t.paint.brush;
    assert_eq!(b.paper.kind, TextureKind::Image, "paper → Paper slot Image");
    assert_eq!(b.paper.mapping, TextureMapping::Tiled, "canvas-anchored");
    assert!(b.watercolor, "render-path on");
    // The Grain slot is untouched (Paper is its own slot now).
    assert_eq!(
        b.texture.kind,
        TextureKind::None,
        "the per-dab Grain slot is not touched"
    );

    let mut t2 = PainterTool::default();
    t2.use_layers_as_granulation(lum, 8, 8);
    let b2 = &t2.paint.brush;
    // Granulation = the GRAIN slot (its own section); "Same as Paper" turned off so the map is used.
    assert_eq!(
        b2.texture.kind,
        TextureKind::Image,
        "granulation → Grain slot Image"
    );
    assert!(
        !b2.granulation_use_paper,
        "granulation uses the Grain map, not the paper"
    );
    assert!(
        (b2.granulation - 0.65).abs() < 1e-6,
        "pronounced mineral-settling amount"
    );
    assert_eq!(
        b2.paper.kind,
        TextureKind::None,
        "the Paper slot is not touched by the granulation tag"
    );
}

/// **TODO `Click` DESPACHADO PELA SECÇÃO TEM DE SER ALCANÇÁVEL** — o censo da cura de 2026-09-22
/// ([doc 42](../../../../../../docs/Painter/42_auditoria_do_watercolor_2026-09-22.md)).
///
/// ⚠️ **Esta é a direcção que NENHUM censo de id deste repo media.** Os que existem perguntam *«o
/// que é PINTADO está registado?»* e *«o que é registado é ALCANÇÁVEL?»*; a que faltava é a terceira
/// — ***«o que é DESPACHADO chega a ser pintado?»*** —, e é por ela que o
/// `PAINTER_WATERCOLOR_PIGMENT` viveu meses com um braço que superfície nenhuma podia disparar.
///
/// ⛔ A régua local é a pertença ao `PAINTER_WATERCOLOR_CLICKS`, que é a lista que o `event.rs` do
/// painel varre para encaminhar um clique — *um id despachado aqui e ausente dela é inalcançável por
/// construção*.
///
/// ⚠️ **A isenção do `PAINTER_SHAPE_*` é MEDIDA e não suposta:** aquele id vive no módulo da secção
/// de FORMA (`ids/painter_shape.rs`) e é pintado (`paint_shape.rs`), populado (`populate.rs`) e
/// encaminhado (`event.rs`) por ela — *ele é alcançável pelo registo do vizinho, não pelo nosso*.
///
/// **Mutações que sangram** (2026-09-22): repor um braço de `Click` sobre um id que só está no
/// `…_FIELDS` (o defeito à letra) · partir a agulha da extracção do despacho (o piso de população).
#[test]
fn todo_click_despachado_pela_seccao_e_alcancavel() {
    let despacho = include_str!("../watercolor_settings.rs");
    let registo = include_str!("../../../ids/painter_watercolor.rs");

    // Os ids que o `route_brush_watercolor_event` consome como Click.
    let mut despachados: Vec<&str> = Vec::new();
    for l in despacho.lines() {
        if let Some(r) = l
            .split("PanelEvent::Click(id) if *id == crate::ids::")
            .nth(1)
        {
            despachados.push(r.trim_end_matches(" => {").trim());
        }
    }
    // PISO DE POPULAÇÃO: sem ele, uma extracção partida varre zero e fica trivialmente verde — a
    // armadilha que este repo já pagou num censo por prefixo de nome.
    assert!(
        despachados.len() >= 7,
        "a extracção do despacho partiu-se: achou {} braços de Click (esperados >= 7)",
        despachados.len()
    );

    let corpo_clicks = registo
        .split("PAINTER_WATERCOLOR_CLICKS")
        .nth(1)
        .and_then(|r| r.split_once('['))
        .and_then(|(_, r)| r.split_once("];"))
        .map(|(b, _)| b)
        .expect("o array PAINTER_WATERCOLOR_CLICKS tem de existir");
    // CONTROLO da extracção do OUTRO lado: o array não pode ler vazio.
    assert!(
        corpo_clicks.matches("PAINTER_WATERCOLOR_").count() >= 5,
        "a extracção do CLICKS partiu-se: leu {} membros",
        corpo_clicks.matches("PAINTER_WATERCOLOR_").count()
    );

    let orfaos: Vec<&&str> = despachados
        .iter()
        // A secção de FORMA regista os dela; ver o ⚠️ do doc acima (medido, não suposto).
        .filter(|id| id.starts_with("PAINTER_WATERCOLOR_"))
        .filter(|id| !corpo_clicks.contains(**id))
        .collect();
    assert!(
        orfaos.is_empty(),
        "estes ids são DESPACHADOS e não estão no PAINTER_WATERCOLOR_CLICKS ⇒ gesto nenhum os \
         alcança (apague o braço, ou registe o id e pinte a fileira): {orfaos:?}"
    );
}
