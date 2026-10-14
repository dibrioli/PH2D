//! ⭐ **A lei B do #47** (plano 27 W3, [27 §6.8](../../../../../../docs/Painter/NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md)):
//! o padrão procedural do Shape multiplica DEPOIS do portão do papel — nunca entra no `stamp` que o dente
//! corta. O redondo (sem padrão) fica ao byte: é o portão das 12 miniaturas de fábrica
//! (`tests/it/presets_de_fabrica.rs`).
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::TextureKind;

const W: u32 = 320;
const H: u32 = 96;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela(media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(media);
    t.set_wet_relogio_fixo(true);
    t
}

/// O traço recto de 220 px e a água a assentar; devolve `Σ(255 − min(rgb))`.
fn tinta_do_traco(t: &mut PainterTool) -> u64 {
    use ph2d_editor_core::Tool;
    let y = H as f32 / 2.0;
    t.on_canvas_pointer(cp([50.0, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=44 {
        t.on_canvas_pointer(cp([50.0 + 5.0 * k as f32, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([270.0, y], PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
        .sum()
}

/// A tinta de `k` ÷ a do Shape `None`, com `prepara` aplicado a cada pincel.
fn razao(media: PaintMedia, k: TextureKind, prepara: &dyn Fn(&mut PainterTool)) -> f64 {
    let mede = |k: TextureKind| {
        let mut t = tela(media);
        prepara(&mut t);
        t.set_brush_shape_kind(k.to_u8());
        tinta_do_traco(&mut t)
    };
    let none = mede(TextureKind::None);
    assert!(none > 0, "{media:?}: o redondo não pintou");
    mede(k) as f64 / none as f64
}

/// **Sobre o GRÃO** (os pastéis: o `texv` é o grão e o portão só deixa passar os picos): o Shape Noise
/// pousava 0,000 do redondo e o Clouds 0,002 (medido 14/10); pela lei B 0,112 e 0,293. E nunca passa do
/// redondo — o padrão só multiplica.
///
/// **Mutação que sangra:** devolver o padrão à silhueta em `wetpaint/dab_route.rs` (`padrao_a_parte =
/// false`) ⇒ 0,000.
#[test]
fn o_padrao_do_shape_pinta_sobre_o_grao_dos_pasteis() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = Biblioteca::carregar(&raiz, None);
    let pastel = bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Soft Pastel")
        .expect("o Soft Pastel de fábrica");
    let prepara = |t: &mut PainterTool| {
        assert!(t.wet_preset_aplicar(&pastel.preset).is_empty());
        t.wet_preset_aplicar_volumes(&pastel.preset.parametros.volumes);
        t.paint.wetpaint.tilt_on = false;
        assert!(
            t.paint.brush.grain_samples(false),
            "o pastel pinta pelo grão"
        );
    };
    for (k, piso) in [(TextureKind::Noise, 0.05), (TextureKind::Clouds, 0.15)] {
        let r = razao(PaintMedia::WetPaint, k, &prepara);
        assert!(
            (piso..1.0).contains(&r),
            "Soft Pastel com o Shape {k:?}: {r:.3} do redondo (piso {piso}) — o portão corta o padrão"
        );
    }
}

/// **Sobre a ponta LISA** (o pincel Wet Paint padrão com o Shape procedural) o portão comia os vales do
/// padrão: Noise 0,337 do redondo antes da lei B, 0,605 depois (medido 14/10; o Digital dá 0,448 — a água
/// não é a mesma composição, a razão não se iguala).
#[test]
fn o_padrao_do_shape_sobre_a_ponta_lisa_nao_perde_os_vales() {
    let r = razao(
        PaintMedia::WetPaint,
        TextureKind::Noise,
        &|_: &mut PainterTool| {},
    );
    assert!(
        (0.5..1.0).contains(&r),
        "Shape Noise na ponta lisa: {r:.3} do redondo (antes da lei B 0,337)"
    );
}
