//! **Bristles no Shape** (plano 27 W1): o tapete de cerdas do Wet Paint como o 30.º tipo do Shape. No
//! Digital e no Impasto ele é uma ponta de cerdas — pinta, e Count e Size mudam a ponta.

use super::super::media::PaintMedia;
use super::*;
use ph2d_painter_brush::TextureKind;

const N: u32 = 160;

fn pincel(media: PaintMedia, kind: TextureKind) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(media);
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_brush_shape_kind(kind.to_u8());
    t
}

fn risca(t: &mut PainterTool) -> Vec<u8> {
    t.on_canvas_pointer(cp([20.0, 80.0], PointerPhase::Down));
    for k in 1..=28 {
        t.on_canvas_pointer(cp([20.0 + 4.0 * k as f32, 80.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([132.0, 80.0], PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

fn tinta(px: &[u8]) -> u64 {
    px.as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
        .sum()
}

/// No Digital e no Impasto a ponta de cerdas PINTA, não é a redonda, e os sliders Count e Size dela
/// mudam o traço. Controlo: a redonda (`None`) e o mesmo pincel sem mexer em nada pinta igual duas vezes.
#[test]
fn digital_e_impasto_pintam_com_a_ponta_de_cerdas() {
    for media in [PaintMedia::Digital, PaintMedia::Impasto] {
        let redonda = risca(&mut pincel(media, TextureKind::None));
        let cerdas = risca(&mut pincel(media, TextureKind::Bristles));
        assert_eq!(
            cerdas,
            risca(&mut pincel(media, TextureKind::Bristles)),
            "{media:?}: determinístico"
        );
        let (r, c) = (tinta(&redonda), tinta(&cerdas));
        assert!(c > 0, "{media:?}: as cerdas não pintaram");
        assert!(
            c < r,
            "{media:?}: cerdas {c} contra a redonda {r} — a ponta tem de ser porosa"
        );
        for (slot, valor) in [(0usize, 0.1f32), (2, 0.9)] {
            let mut t = pincel(media, TextureKind::Bristles);
            t.set_brush_shape_param_norm(slot, valor);
            assert_ne!(
                risca(&mut t),
                cerdas,
                "{media:?}: o slider {slot} não mudou a ponta"
            );
        }
    }
}

/// HR-15: o nome de CADA tipo do dropdown (agora 30) e os rótulos dos parâmetros dele vêm da tabela —
/// nenhuma chave crua no ecrã.
#[test]
fn cada_tipo_e_cada_parametro_tem_texto() {
    assert_eq!(TextureKind::COUNT, 30);
    assert_eq!(TextureKind::from_u8(29), TextureKind::Bristles);
    for k in (0..TextureKind::COUNT).map(TextureKind::from_u8) {
        assert_eq!(TextureKind::from_u8(k.to_u8()), k);
        let chaves = std::iter::once(k.name_key())
            .chain(ph2d_painter_brush::param_specs(k).iter().map(|s| s.label));
        for chave in chaves {
            assert_ne!(ph2d_i18n::tr(chave), chave, "{k:?}: «{chave}» sem texto");
        }
    }
}
