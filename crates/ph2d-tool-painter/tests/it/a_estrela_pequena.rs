//! ⭐ **O Stars pequeno não tem degraus** (o dono, 23/10: «o pincel Stars fica com baixa resolução quando o tamanho
//! do brush é reduzido»). Medido: o Stars toma a rota POR PÍXEL (Accumulate desligado), e a ponta de imagem de 128²
//! lida num ponto por píxel numa estrela de 10–40 px acendia e apagava o aro claro dela em degraus (erro contra a
//! área do píxel 0,55 a raio 5 — `ph2d-painter-brush::texture::shape::estrela_tests`). A cura: a imagem REDUZIDA
//! lê-se sempre pela área. Pela porta do artista: o Stars de fábrica (sem a lei `area`) pinta AO BIT o mesmo traço
//! que o Stars com a lei `area` ligada — a área exacta do píxel —, em cada tamanho pequeno.

use ph2d_brush_presets::{Biblioteca, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::path::PathBuf;

const W: usize = 400;
const H: usize = 160;

fn estrelas(area: bool) -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Stars")
        .expect("o Stars de fábrica")
        .preset;
    if area {
        let shape = p
            .parametros
            .brush
            .get_mut("shape")
            .and_then(serde_json::Value::as_object_mut)
            .expect("o Stars tem a Shape");
        assert!(
            !shape.contains_key("lei"),
            "o Stars de fábrica não pede a lei"
        );
        shape.insert("lei".into(), serde_json::json!({ "area": true }));
    }
    p
}

/// Um traço recto do Stars com o raio `raio`; a tela no fim.
fn pinta(p: &Preset, raio: f32) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.preset_aplicar(p).expect("aplica");
    t.set_brush_size_px(raio);
    let ponto = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut ultima = Vec::new();
    let mut tique = |t: &mut PainterTool| {
        t.on_tick(1000.0 / 60.0);
        if let Some((px, _, _)) = t.take_preview_arc() {
            ultima = px.as_ref().clone();
        }
    };
    t.on_canvas_pointer(ponto([40.0, 80.0], PointerPhase::Down));
    let mut pos = [40.0, 80.0];
    for k in 1..=30 {
        pos = [40.0 + 10.0 * k as f32, 80.0];
        t.on_canvas_pointer(ponto(pos, PointerPhase::Move));
        tique(&mut t);
    }
    t.on_canvas_pointer(ponto(pos, PointerPhase::Up));
    for _ in 0..5 {
        tique(&mut t);
    }
    ultima
}

/// ⭐ Em 10, 20 e 40 px de diâmetro o Stars de fábrica é a área exacta do píxel; o CONTROLO — o traço pinta
/// estrelas (senão a igualdade é vácua).
#[test]
fn a_estrela_pequena_nao_tem_degraus() {
    let (fabrica, area) = (estrelas(false), estrelas(true));
    for raio in [5.0_f32, 10.0, 20.0] {
        let (a, b) = (pinta(&fabrica, raio), pinta(&area, raio));
        let tinta = a.chunks(4).filter(|c| c[0] < 250).count();
        let difere = a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count();
        eprintln!("ESTRELA raio {raio}: tinta={tinta} difere={difere}");
        assert!(
            tinta > 200,
            "CONTROLO raio {raio}: o traço quase não pintou ({tinta} px)"
        );
        assert_eq!(
            difere, 0,
            "raio {raio}: o Stars de fábrica difere da área do píxel em {difere} px (degraus)"
        );
    }
}
