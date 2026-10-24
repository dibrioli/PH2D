//! `BUGS_painter_fechados_41_a_46.md` #53 (Enio 24/10): o «Export Image…» de um gato pintado no Painter saiu BRANCO — a
//! porta lia a textura da sprite, e a tinta ainda não assada vive no `PainterTool`.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const L: u32 = 256;
const E: u64 = 7;

fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2], levanta: bool) {
    let n = 12;
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let f = i as f32 / (n - 1) as f32;
        let phase = match i {
            0 => PointerPhase::Down,
            i if levanta && i == n - 1 => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        let pos = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        t.on_tick(16.667);
    }
}

/// O gato do dono: tela branca, Digital, um traço amarelo, uma camada nova com manchas escuras e um
/// traço cinzento ainda NO AR (sem Up). Nada assado — o artista não saiu da sprite.
fn o_gato() -> PainterTool {
    let mut t = PainterTool::default();
    t.bind_document(E, vec![255; (L * L * 4) as usize], L, L);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([230, 190, 40]); // LITERAL-COLOR-OK: a cor do artista, não da UI
    traco(&mut t, [40.0, 128.0], [216.0, 128.0], true);
    t.add_raster_layer("manchas");
    t.set_brush_color_srgb8([60, 40, 20]); // LITERAL-COLOR-OK: idem
    traco(&mut t, [80.0, 100.0], [120.0, 150.0], true);
    t.set_brush_color_srgb8([120, 120, 120]); // LITERAL-COLOR-OK: idem
    traco(&mut t, [60.0, 60.0], [200.0, 70.0], false);
    t
}

fn exporta(src: SpriteSource, nome: &str) -> Vec<u8> {
    let mut exporters = ExporterRegistry::new();
    ph2d_imageio_registry_init::register_all_exporters(&mut exporters);
    let mut importers = ph2d_imageio::ImporterRegistry::new();
    ph2d_imageio_registry_init::register_all_importers(&mut importers);
    // `PH2D_EXPORT_FOTO=docs/Painter/smoke_2026-10-24/export` grava as fotos antes/depois.
    let path = std::env::var_os("PH2D_EXPORT_FOTO").map_or_else(
        || std::env::temp_dir().join(format!("ph2d_export_{nome}.png")),
        |d| PathBuf::from(d).join(format!("{nome}.png")),
    );
    let mut toasts = ToastQueue::default();
    write_source(src, ExportFormat::Png, &path, &exporters, &mut toasts).expect("grava o PNG");
    let bytes = std::fs::read(&path).expect("lê o PNG");
    let back = importers
        .find_for(ph2d_imageio::MagicHint::Bytes(&bytes))
        .expect("importador")
        .import(&bytes, &ph2d_imageio::ImportOpts::default())
        .expect("importa");
    let DecodedImage::Flat(buf) = back else {
        panic!("um PNG de 8 bits volta Flat");
    };
    bytemuck::allocation::cast_vec(buf.pixels)
}

/// ⭐ **O ficheiro exportado é a imagem do ecrã**, byte a byte, pela mesma escolha de pixels que o menu
/// faz (`escolhe` — a sprite lida só quando o Painter não tem tinta por assar dela).
#[test]
fn o_export_de_uma_sprite_por_assar_e_a_imagem_do_ecra() {
    let mut t = o_gato();
    assert!(t.has_unbaked_edits(), "controlo: nada foi assado");
    let branca = vec![255u8; (L * L * 4) as usize];
    let antes = exporta(source_from_straight((branca.clone(), L, L)), "antes");
    let por_assar = ph2d_app_painter::painter_bridge_queries::imagem_por_assar_do(&t, E);
    let src = escolhe(por_assar, || {
        Some(source_from_straight((branca.clone(), L, L)))
    })
    .expect("pixels");
    let depois = exporta(src, "depois");
    let (ecra, w, h) = t.take_preview_arc().expect("o ecrã");
    assert_eq!((w, h), (L, L));
    assert!(
        antes != *ecra,
        "controlo: a sprite (o antigo) é branca e o ecrã tem o gato"
    );
    let difere = depois
        .as_chunks::<4>()
        .0
        .iter()
        .zip(ecra.as_chunks::<4>().0)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        difere, 0,
        "o PNG exportado difere do ecrã em {difere} texels"
    );
    let meio = t.sample_composite_at_uv(0.5, 0.5).expect("amostra");
    let i = ((L / 2 * L + L / 2) * 4) as usize;
    assert_eq!(&depois[i..i + 4], &meio, "o centro do traço amarelo");
    assert_ne!(meio, [255; 4], "controlo: o centro está pintado");
}

/// Outro documento, ou nada por assar: a sprite é a verdade (e com ela os 16 bits).
#[test]
fn sem_tinta_por_assar_o_export_le_a_sprite() {
    let mut t = o_gato();
    let por_outra = ph2d_app_painter::painter_bridge_queries::imagem_por_assar_do(&t, E + 1);
    assert!(por_outra.is_none(), "o Painter pinta outra sprite");
    t.mark_baked();
    let assada = ph2d_app_painter::painter_bridge_queries::imagem_por_assar_do(&t, E);
    assert!(assada.is_none(), "assado: a sprite já tem a tinta");
}
