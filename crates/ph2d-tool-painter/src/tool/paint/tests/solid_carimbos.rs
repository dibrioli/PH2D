//! **O Solid com uma imagem no Shape enche-se de CARIMBOS** (smoke do dono 12/10, com foto): com «Use
//! Texture Colors» o Color vai a branco, e a mancha lisa na cor do Color deixava o miolo BRANCO — só o
//! contorno tinha folhas. O miolo tem de se encher da própria ponta, com o jitter.

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::StrokeMethod;

const LADO: u32 = 256;

/// Uma ponta de imagem VERDE (RGB da textura), Solid ligado, jitter de rotação, escala e posição.
fn folhas(metodo: StrokeMethod) -> PainterTool {
    let s = 16u32;
    let rgba: Vec<u8> = std::iter::repeat_n([60u8, 160, 60, 255], (s * s) as usize)
        .flatten()
        .collect();
    folhas_com(metodo, &rgba, s)
}

fn folhas_com(metodo: StrokeMethod, rgba: &[u8], s: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(8.0);
    t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_LINE_SOLID));
    t.set_brush_shape_image_rgba(rgba, s, s, None);
    t.toggle_brush_shape_alpha_from_image();
    t.toggle_brush_shape_per_layer_color(); // Use Texture Colors ⇒ o Color vai a branco
    t.paint.brush.jitter_rotate = 1.0;
    t.paint.brush.jitter_scale = 0.3;
    t.paint.brush.jitter = 0.5;
    t.paint.brush.stroke_method = metodo;
    t
}

/// O laço à mão livre: um quadrado de meio-lado 60 à volta do centro, oito passos por aresta.
fn laco(t: &mut PainterTool) {
    let (c, s) = (128.0, 60.0);
    let pts = [
        [c - s, c - s],
        [c + s, c - s],
        [c + s, c + s],
        [c - s, c + s],
        [c - s, c - s],
    ];
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for w in pts.windows(2) {
        for k in 1..=8 {
            let f = k as f32 / 8.0;
            let p = [
                w[0][0] + (w[1][0] - w[0][0]) * f,
                w[0][1] + (w[1][1] - w[0][1]) * f,
            ];
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Up));
}

fn elipse(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([198.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([198.0, 128.0], PointerPhase::Up));
}

/// A fracção VERDE do miolo (80×80 à volta do centro, longe do contorno).
fn verde_no_miolo(t: &PainterTool) -> f32 {
    let mut verde = 0usize;
    for y in 88..168u32 {
        for x in 88..168u32 {
            let p = px(t, LADO, x, y);
            if i32::from(p[1]) > i32::from(p[0]) + 40 {
                verde += 1;
            }
        }
    }
    verde as f32 / 6400.0
}

/// **O caso da foto, nas duas famílias:** o traço à mão livre (cumulativo) e a Elipse (re-carimbo).
/// Antes: o miolo BRANCO (a mancha lisa na cor do Color, que «Use Texture Colors» pôs a branco).
#[test]
fn o_solid_com_imagem_no_shape_enche_o_miolo_de_carimbos() {
    for (nome, metodo) in [
        ("mão livre", StrokeMethod::Space),
        ("Elipse", StrokeMethod::Ellipse),
    ] {
        let mut t = folhas(metodo);
        if metodo == StrokeMethod::Ellipse {
            elipse(&mut t);
        } else {
            laco(&mut t);
        }
        let f = verde_no_miolo(&t);
        assert!(f > 0.9, "{nome}: só {:.0} % do miolo tem folhas", f * 100.0);
    }
}

/// **Um Ctrl+Z leva o traço E os carimbos da mancha** — eles são parte do mesmo gesto.
#[test]
fn desfazer_o_solid_de_carimbos_devolve_a_tela() {
    let mut t = folhas(StrokeMethod::Space);
    let antes = (*t.canvas_rgba).clone();
    laco(&mut t);
    assert!(verde_no_miolo(&t) > 0.9, "CONTROLO: o miolo encheu");
    assert!(t.undo_last());
    assert!(*t.canvas_rgba == antes, "o desfazer deixou tinta");
}

/// **Sem imagem no Shape a mancha continua LISA** (a ponta redonda não é o desenho): o miolo é a cor do
/// Color, uniforme.
#[test]
fn sem_imagem_a_mancha_continua_lisa() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(8.0);
    t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_LINE_SOLID));
    t.paint.brush.color = [1.0, 0.0, 0.0];
    t.paint.brush.jitter_rotate = 1.0;
    laco(&mut t);
    assert_eq!(px(&t, LADO, 128, 128), [255, 0, 0, 255]);
}

/// Sonda: o laço com uma ponta em FOLHA (alpha de uma folha pontiaguda, RGB verde) gravado em PPM em
/// `PH2D_SONDA_OUT` — para comparar com a foto do dono.
#[test]
#[ignore = "sonda: fotografa o Solid de folhas"]
fn sonda_solid_de_folhas() {
    let s = 32u32;
    let mut rgba = Vec::with_capacity((s * s * 4) as usize);
    for y in 0..s {
        for x in 0..s {
            let (u, v) = (
                (x as f32 + 0.5) / s as f32 * 2.0 - 1.0,
                (y as f32 + 0.5) / s as f32 * 2.0 - 1.0,
            );
            // Uma folha: a lente entre dois arcos, com a nervura mais escura.
            let dentro = v.abs() < 0.55 * (1.0 - u * u);
            let nervura = v.abs() < 0.04;
            let c = if nervura {
                [30u8, 90, 30]
            } else {
                [80, 170, 70]
            };
            rgba.extend_from_slice(&[c[0], c[1], c[2], if dentro { 255 } else { 0 }]);
        }
    }
    let mut t = folhas_com(StrokeMethod::Space, &rgba, s);
    t.set_brush_size_px(14.0);
    laco(&mut t);
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp/solid_folhas.ppm".into());
    let mut ppm = format!("P6 {LADO} {LADO} 255\n").into_bytes();
    for p in t.canvas_rgba.chunks(4) {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(&out, ppm).expect("escreve");
    println!("verde no miolo {:.0} % → {out}", verde_no_miolo(&t) * 100.0);
}

/// Um zigue-zague apertado (linhas a 10 px), para o Sketchy achar vizinhos no canvas e o Wire no percurso.
fn zigue(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([60.0, 90.0], PointerPhase::Down));
    for linha in 0..8 {
        let y = 70.0 + linha as f32 * 16.0;
        for k in 0..=17 {
            let x = if linha % 2 == 0 {
                60.0 + k as f32 * 8.0
            } else {
                196.0 - k as f32 * 8.0
            };
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
}

/// `(píxeis com tinta, píxeis com tinta que NÃO é da cor da ponta)`.
fn tinta(t: &PainterTool) -> (usize, usize) {
    let (mut n, mut fora) = (0, 0);
    for p in t.canvas_rgba.chunks(4) {
        if p[..3] != [255, 255, 255] {
            n += 1;
            if i32::from(p[1]) <= i32::from(p[0]) {
                fora += 1;
            }
        }
    }
    (n, fora)
}

/// **Todo Type pinta na cor da PONTA** (a pergunta do dono: *«o Type está coerente com a textura no Shape
/// e o jitter?»*). Speed, Rough e o corpo da Ribbon são carimbos — a ponta e o jitter valem por
/// construção. Sketchy, Wire e as travessas da Ribbon são FIOS: não têm ponta para carimbar, e pintavam
/// a cor do Color — que o «Use Texture Colors» põe a BRANCO: o fio sumia no papel (o Wire, que esconde
/// os carimbos, não deixava NADA). Agora o fio tem a cor média da ponta.
#[test]
fn todo_type_pinta_na_cor_da_ponta() {
    use ph2d_painter_brush::line_kind::LineKind;
    let medir = |k: LineKind| {
        let mut t = folhas(StrokeMethod::Space);
        t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_LINE_SOLID)); // Solid desligado
        t.set_brush_size_px(3.0);
        t.handle_panel_event(PanelEvent::SelectOption(
            crate::ids::PAINTER_LINE_TYPE,
            (k as u8).to_string(),
        ));
        assert_eq!(t.paint.brush.line_kind, k, "a porta do painel armou o Type");
        // Os fios e o Rough armados como no censo `measure_that_every_line_kind_does_something_under_solid`
        // (o Rough nasce a zero e o Sketchy com alcance curto).
        t.paint.brush.sketchy_reach = 3.0;
        t.paint.brush.sketchy_density = 1.0;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.5;
        t.paint.brush.rough_amount = 0.4;
        t.paint.brush.rough_bowing = 0.4;
        zigue(&mut t);
        tinta(&t)
    };
    let (base, _) = medir(LineKind::None);
    for k in [
        LineKind::None,
        LineKind::Speed,
        LineKind::Sketchy,
        LineKind::Wire,
        LineKind::Ribbon,
        LineKind::Rough,
    ] {
        let (n, fora) = medir(k);
        // ⚠️ A Ribbon é uma MOLA no relógio dos eventos: sem tempo entre eles (os do teste) a massa
        // quase não anda, também com a ponta redonda (medido: 32 px redonda, 34 com a folha).
        if !matches!(k, LineKind::Ribbon) {
            assert!(n > 500, "{k:?}: só {n} px de tinta");
        }
        assert!(
            fora * 50 < n,
            "{k:?}: {fora} de {n} px fora da cor da ponta"
        );
        if matches!(k, LineKind::Sketchy) {
            assert!(
                n > base + 300,
                "{k:?}: os fios não aparecem ({n} contra {base} sem eles)"
            );
        }
    }
}
