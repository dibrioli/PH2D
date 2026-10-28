//! **U3 da auditoria W3b**: um controlo do PINCEL vale para os traços seguintes (o painel só vale para o futuro), logo
//! o roteador do painel não recompõe a tela por ele. Censo derivado da tabela de knobs (os do Tuning, os do cartão e
//! os das ferramentas da água) mais os factos do pincel que o roteador escreve. Doc:
//! `docs/Painter/49_realshader/AUDITORIA_W3b.md`.
//!
//! Detector: a tela é ENVENENADA depois do traço; uma recomposição reescreve-a (controlo positivo abaixo).

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Rebuild;

const W: usize = 160;
const H: usize = 100;
const VENENO: u8 = 7;

fn tela(oleo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    // Dono 28/10: com tinta com óleo na tela também (a caixa, o Depth e o Gloss são do traço).
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_size_px(10.0);
    t.set_brush_color_srgb8([180, 30, 30]);
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, 50.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(20.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(p(20.0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(140.0, PointerPhase::Up));
    for _ in 0..10 {
        t.on_tick(1000.0 / 60.0);
    }
    assert!(t.paint.wetpaint.session.is_some(), "controlo: há sessão");
    t
}

fn envenena(t: &mut PainterTool) {
    std::sync::Arc::make_mut(&mut t.canvas_rgba).fill(VENENO);
    // A escrita é do teste, não alheia: sem isto a guarda da identidade da tela mata a sessão.
    t.wetpaint_rearm_after_own_write();
}

fn envenenada(t: &PainterTool) -> bool {
    t.canvas_rgba.iter().all(|b| *b == VENENO)
}

/// Os eventos do painel que escrevem um controlo do pincel: cada knob do registo que não refaz a folha e tem porta
/// no painel (o slider do Tuning, o cartão, a ferramenta da água) e os factos (as duas caixas K-M, Live Water, Heavy
/// Edge, o Tilt e os seis knobs do cartão básico).
fn eventos_do_pincel() -> Vec<(String, PanelEvent)> {
    let mut ev = Vec::new();
    let mapa = wet_tuning_id_map();
    for def in KNOB_DEFS
        .iter()
        .filter(|d| d.rebuild != Some(Rebuild::Paper))
    {
        let id = crate::ids::wet_tuning_slider_id(def.key);
        let meio = 0.5 * (def.min + def.max);
        if mapa.contains_key(&id) {
            ev.push((
                format!("tuning:{}", def.key),
                PanelEvent::SetValue(id, meio),
            ));
        }
        for (cid, k) in KNOBS_NO_CARTAO {
            if k == def.knob {
                ev.push((
                    format!("cartao:{}", def.key),
                    PanelEvent::SetValue(cid, meio),
                ));
            }
        }
        for (cid, k, ..) in KNOBS_DA_FERRAMENTA {
            if k == def.knob {
                ev.push((format!("agua:{}", def.key), PanelEvent::SetValue(cid, meio)));
            }
        }
    }
    use crate::ids as i;
    for (nome, id) in [
        ("km_glaze", i::WET_TUNING_KM_GLAZE),
        ("km_mixing", i::WET_TUNING_KM_MIXING),
        ("paint_texture", i::PAINTER_WETPAINT_PAPER_VISUAL),
        ("real_oil", i::PAINTER_WETPAINT_OLEO_REAL),
        ("live_water", i::PAINTER_WETPAINT_LIVE_WATER),
        ("tilt_toggle", i::PAINTER_WETPAINT_TILT_TOGGLE),
    ] {
        ev.push((nome.to_owned(), PanelEvent::Click(id)));
    }
    for (nome, id, v) in [
        ("heavy_edge", i::PAINTER_WETPAINT_HEAVY_EDGE, 0.7),
        ("tilt_ring", i::PAINTER_WETPAINT_TILT_RING, 6.0),
        ("tilt_spoke", i::PAINTER_WETPAINT_TILT_SPOKE, 5.0),
        ("water", i::PAINTER_WETPAINT_WATER, 0.3),
        ("pigment", i::PAINTER_WETPAINT_PIGMENT, 900.0),
        ("pickup", i::PAINTER_WETPAINT_PICKUP, 0.4),
        ("dry_speed", i::PAINTER_WETPAINT_DRY_SPEED, 0.4),
        ("edge", i::PAINTER_WETPAINT_EDGE, 0.4),
        ("gravity", i::PAINTER_WETPAINT_GRAVITY, 0.4),
        ("erase", i::PAINTER_WETPAINT_ERASE, 0.4),
    ] {
        ev.push((nome.to_owned(), PanelEvent::SetValue(id, v)));
    }
    ev
}

/// ⭐ O GATE: nenhum controlo do pincel, pelo roteador do painel, recompõe a tela — com e sem tinta com óleo.
#[test]
fn um_controlo_do_pincel_nao_recompoe_a_tela() {
    for oleo in [false, true] {
        um_controlo_do_pincel_nao_recompoe(oleo);
    }
}

fn um_controlo_do_pincel_nao_recompoe(oleo: bool) {
    let mut t = tela(oleo);
    // Controlo positivo: o detector vê uma recomposição.
    envenena(&mut t);
    t.wet_recomposite_full();
    assert!(!envenenada(&t), "controlo: recompor reescreve a tela");

    let ev = eventos_do_pincel();
    assert!(
        ev.len() > 40,
        "o censo tem de cobrir a tabela (só {} eventos)",
        ev.len()
    );
    let mut recompoem = Vec::new();
    for (nome, e) in &ev {
        envenena(&mut t);
        assert!(
            t.route_brush_wetpaint_event(e),
            "{nome}: o roteador não consumiu o evento"
        );
        if !envenenada(&t) {
            recompoem.push(nome.clone());
        }
    }
    // Os do cartão da FORMA que só o Wet Paint lê (J6, as pontas rectas): a rota da pegada.
    use crate::ids as i;
    for (nome, e) in [
        ("pontas_rectas", PanelEvent::Click(i::PAINTER_BRUSH_PONTAS_RECTAS)),
        ("pontas_alem", PanelEvent::SetValue(i::PAINTER_BRUSH_PONTAS_ALEM, 0.6)),
    ] {
        envenena(&mut t);
        assert!(t.route_brush_dab_event(&e), "{nome}: a rota da pegada não consumiu o evento");
        if !envenenada(&t) {
            recompoem.push(nome.to_owned());
        }
    }
    assert!(
        recompoem.is_empty(),
        "óleo {oleo}: estes controlos do pincel recompõem a tela: {recompoem:?}"
    );
}
