//! **A Borracha em cada meio** (dono 24/10: «a Borracha é especial: a SUA lista de presets e os meios Digital,
//! Watercolor e Wet Paint»). Oferecer um meio à Borracha só é honesto se ela apagar ali de outra maneira que no
//! Digital — e se o que ela apagou ficar apagado enquanto a água do meio seca.

use super::*;

const SIZE: u32 = 128;

/// A tinta da região (`x0..x1, y0..y1`): `Σ alfa·(1 − luz)`, em unidades de texel cheio.
fn tinta(t: &PainterTool, (x0, x1, y0, y1): (u32, u32, u32, u32)) -> f64 {
    let mut s = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = px(t, SIZE, x, y);
            let luz = (f64::from(p[0]) + f64::from(p[1]) + f64::from(p[2])) / 765.0;
            s += f64::from(p[3]) / 255.0 * (1.0 - luz);
        }
    }
    s
}

/// O miolo da coluna apagada (a borracha passa em `x = 64`, raio 8) dentro da aguada (`y ≈ 52..76`).
const APAGADO: (u32, u32, u32, u32) = (60, 69, 56, 73);

fn traco_h(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([20.0, y], PointerPhase::Down));
    let mut x = 20.0f32;
    while x < 108.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([108.0, y], PointerPhase::Up));
}

fn traco_v(t: &mut PainterTool, x: f32) {
    t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down));
    let mut y = 30.0f32;
    while y < 100.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x, 100.0], PointerPhase::Up));
}

/// A tela branca com o pincel de aguada em todos os slots.
pub(super) fn tela_de_aguada() -> PainterTool {
    let mut t = white_canvas(SIZE, 12.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.85],
        space_attenuation: false,
        watercolor: true,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    t
}

/// A Borracha na mão, no `meio`, com raio 8.
pub(super) fn borracha_em(t: &mut PainterTool, meio: PaintMedia) {
    t.set_paint_tool_mode("eraser");
    t.set_paint_media(meio);
    assert_eq!(
        t.paint_media(),
        meio,
        "fixtura: a borracha não ficou no meio"
    );
    assert!(t.paint.eraser, "fixtura: a borracha não está na mão");
    t.paint.brush.radius_px = 8.0;
}

/// A aguada molhada, apagada pela Borracha no `meio`; devolve a tela logo depois.
pub(super) fn aguada_apagada(meio: PaintMedia) -> PainterTool {
    let mut t = tela_de_aguada();
    traco_h(&mut t, 64.0);
    assert!(
        t.wet_session_continues(),
        "fixtura: a aguada não ficou molhada"
    );
    borracha_em(&mut t, meio);
    traco_v(&mut t, 64.0);
    t
}

#[test]
#[ignore = "sonda: a Borracha no Digital × Watercolor × Wet Paint (24/10)"]
fn sonda_borracha_por_meio() {
    // (a) sobre tinta SECA (o pincel Digital).
    let seca = |meio: PaintMedia| {
        let mut t = tela_de_aguada();
        t.set_paint_media(PaintMedia::Digital);
        traco_h(&mut t, 64.0);
        borracha_em(&mut t, meio);
        traco_v(&mut t, 64.0);
        t.canvas_rgba.to_vec()
    };
    let d = seca(PaintMedia::Digital);
    for m in [PaintMedia::Watercolor, PaintMedia::WetPaint] {
        let o = seca(m);
        let n = d.iter().zip(&o).filter(|(a, b)| a != b).count();
        eprintln!("BPM\tseca\t{m:?}\tbytes≠Digital\t{n}");
    }
    // (b) sobre a aguada MOLHADA: o apagado, depois a secagem, depois um Grain mexido com o pincel de volta.
    for m in [PaintMedia::Digital, PaintMedia::Watercolor] {
        let mut t = aguada_apagada(m);
        let logo = tinta(&t, APAGADO);
        for _ in 0..140 {
            t.paint_tick(0.5);
        }
        let seco = tinta(&t, APAGADO);
        let mut t2 = aguada_apagada(m);
        t2.set_paint_tool_mode("brush");
        t2.set_brush_texture_size_norm(0, 0.8);
        t2.paint_tick(1.0 / 60.0);
        let grain = tinta(&t2, APAGADO);
        let mut t3 = aguada_apagada(m);
        t3.set_paint_tool_mode("brush");
        traco_h(&mut t3, 96.0);
        let vizinho = tinta(&t3, APAGADO);
        eprintln!(
            "BPM\tmolhada\t{m:?}\tlogo {logo:.1}\tdepois de secar {seco:.1}\tGrain mexido {grain:.1}\t\
             traço vizinho {vizinho:.1}\tsessão viva {}",
            t3.wet_session_continues()
        );
    }
    // (a') a diferença do Wet Paint sobre tinta seca: onde, e quanta tinta.
    let mut td = tela_de_aguada();
    td.set_paint_media(PaintMedia::Digital);
    traco_h(&mut td, 64.0);
    let antes = td.canvas_rgba.to_vec();
    for m in [PaintMedia::Digital, PaintMedia::WetPaint] {
        let mut t = tela_de_aguada();
        t.set_paint_media(PaintMedia::Digital);
        traco_h(&mut t, 64.0);
        borracha_em(&mut t, m);
        traco_v(&mut t, 64.0);
        let (mut x0, mut x1, mut y0, mut y1) = (u32::MAX, 0, u32::MAX, 0);
        for (i, (a, b)) in antes.iter().zip(t.canvas_rgba.iter()).enumerate() {
            if a != b {
                let p = (i / 4) as u32;
                let (x, y) = (p % SIZE, p / SIZE);
                (x0, x1, y0, y1) = (x0.min(x), x1.max(x), y0.min(y), y1.max(y));
            }
        }
        eprintln!(
            "BPM\tseca\t{m:?}\tapagado x{x0}..{x1} y{y0}..{y1}\ttinta {:.1}\tmétodo {:?}\tsessão {}",
            tinta(&t, APAGADO),
            t.paint.brush.stroke_method,
            t.paint.wetpaint.session.is_some()
        );
    }
    // (c) sobre um traço Wet Paint VIVO: a Borracha do Wet Paint × a do Digital.
    for m in [PaintMedia::Digital, PaintMedia::WetPaint] {
        let mut t = tela_de_aguada();
        t.paint.brush.watercolor = false;
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.set_paint_media(PaintMedia::WetPaint);
        traco_h(&mut t, 64.0);
        for _ in 0..4 {
            t.paint_tick(1.0 / 40.0);
        }
        let viva = t.paint.wetpaint.session.is_some();
        let antes = tinta(&t, APAGADO);
        borracha_em(&mut t, m);
        traco_v(&mut t, 64.0);
        let logo = tinta(&t, APAGADO);
        for _ in 0..200 {
            t.paint_tick(1.0 / 40.0);
        }
        let depois = tinta(&t, APAGADO);
        let fora = tinta(&t, (30, 40, 56, 73));
        eprintln!(
            "BPM\tWet vivo\t{m:?}\tsessão antes {viva}\ttinta antes {antes:.1}\tlogo {logo:.1}\t\
             depois de 5 s {depois:.1}\tfora (x30..40) {fora:.1}\tsessão {}",
            t.paint.wetpaint.session.is_some()
        );
    }
    let a = aguada_apagada(PaintMedia::Digital).canvas_rgba.to_vec();
    let b = aguada_apagada(PaintMedia::Watercolor).canvas_rgba.to_vec();
    let n = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    eprintln!("BPM\tmolhada\tWatercolor\tbytes≠Digital\t{n}");
}

/// ⭐ **O que a Borracha apagou na aguada MOLHADA fica apagado** (24/10): a secagem, um Grain mexido com o pincel
/// de volta (o re-render da aguada editável) e o traço seguinte não trazem a tinta de volta. Guarda a lei que o
/// apagado depende: o traço da Borracha não continua a sessão molhada (a união esvazia no pen-down dele).
#[test]
fn o_apagado_na_aguada_molhada_fica_apagado() {
    for m in [PaintMedia::Digital, PaintMedia::Watercolor] {
        let mut t = aguada_apagada(m);
        assert_eq!(tinta(&t, APAGADO), 0.0, "{m:?}: a borracha não apagou");
        for _ in 0..140 {
            t.paint_tick(0.5);
        }
        assert_eq!(
            tinta(&t, APAGADO),
            0.0,
            "{m:?}: a secagem trouxe a tinta de volta"
        );
        let mut t = aguada_apagada(m);
        t.set_paint_tool_mode("brush");
        t.set_brush_texture_size_norm(0, 0.8);
        t.paint_tick(1.0 / 60.0);
        assert_eq!(
            tinta(&t, APAGADO),
            0.0,
            "{m:?}: o re-render da aguada repintou o apagado"
        );
        traco_h(&mut t, 96.0);
        assert_eq!(
            tinta(&t, APAGADO),
            0.0,
            "{m:?}: o traço vizinho repintou o apagado"
        );
    }
}

/// ⭐ **O Follow esmaecido não muda a tinta no Wet Paint** (o gate do painel `a_linha_esmaecida_e_a_que_nao_age`,
/// 24/10): com a ponta redonda (a Borracha do seu slot, o Pincel sem Shape) nada roda, mas o Rake armava o
/// aquecimento do traço, que retém os primeiros dabs e muda o momento em que a água os recebe (1 273 px).
/// **Mutação que sangra:** tirar o `FollowSemNada` de `authored_spec` (`dab_spec.rs`).
#[test]
fn o_follow_sem_lado_nao_muda_a_agua() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    fn quadro(t: &mut PainterTool, u: &mut Option<Vec<u8>>) {
        t.on_tick(1000.0 / 60.0);
        if let Some((px, _, _)) = t.take_preview_arc() {
            *u = Some(px.as_ref().clone());
        }
    }
    fn traco(t: &mut PainterTool, u: &mut Option<Vec<u8>>, de: [f32; 2], ate: [f32; 2]) {
        t.on_canvas_pointer(cp(de, PointerPhase::Down));
        let mut pos = de;
        for k in 1..=40 {
            let f = k as f32 / 40.0;
            let onda = (f * std::f32::consts::TAU * 1.5).sin() * 6.0;
            pos = [
                de[0] + (ate[0] - de[0]) * f,
                de[1] + (ate[1] - de[1]) * f + onda,
            ];
            t.on_canvas_pointer(cp(pos, PointerPhase::Move));
            quadro(t, u);
        }
        t.on_canvas_pointer(cp(pos, PointerPhase::Up));
        quadro(t, u);
    }
    for borracha in [true, false] {
        let corre = |follow: u8| {
            let mut t = white_canvas(SIZE, 12.0);
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_wet_relogio_fixo(true);
            let mut u = None;
            traco(&mut t, &mut u, [-8.0, 40.0], [136.0, 40.0]);
            if borracha {
                t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_WETPAINT_TOOL_IDS[1]));
                assert!(
                    t.paint.eraser,
                    "fixtura: a Borracha da água não está na mão"
                );
            } else {
                t.paint.brush.shape.kind = ph2d_painter_brush::TextureKind::None;
            }
            t.set_brush_shape_follow(follow);
            assert!(
                t.inercias().inerte(Dependente::FollowSemNada),
                "fixtura: o Follow tem um lado para rodar"
            );
            traco(&mut t, &mut u, [24.0, 120.0], [104.0, 8.0]);
            for _ in 0..30 {
                quadro(&mut t, &mut u);
            }
            u.expect("a água pintou a pré-visualização")
        };
        let (off, rake) = (corre(0), corre(1));
        let n = off.iter().zip(&rake).filter(|(a, b)| a != b).count();
        assert_eq!(
            n, 0,
            "borracha={borracha}: o Follow esmaecido mudou {n} bytes da água"
        );
    }
}
