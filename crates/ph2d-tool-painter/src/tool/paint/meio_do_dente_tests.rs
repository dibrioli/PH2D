//! A altura média do dente (Bug #71): sem ela a tinta fica num degrau contra o papel à vista.

use super::*;
use crate::tool::PainterTool;

fn papel_de_imagem<'a>(lum: &'a [u8], lado: u32, rough: f32) -> Substrate<'a> {
    let tex = TextureSettings {
        kind: TextureKind::Image,
        mapping: TextureMapping::Tiled,
        ..TextureSettings::default()
    };
    let mask = ImageMask {
        lum,
        width: lado,
        height: lado,
    };
    Substrate::resolve(tex, 1.0, rough, Some(mask)).expect("papel ligado")
}

/// A média na grelha de `janela`² píxeis, em fracção da amplitude.
fn media_medida(s: &Substrate<'_>, janela: i64) -> f64 {
    let mut soma = 0.0;
    for y in 0..janela {
        for x in 0..janela {
            soma += f64::from(s.tooth_loads(x, y));
        }
    }
    soma / (janela * janela) as f64 / f64::from(s.amp_px / DEPTH_UNIT_PX)
}

/// A imagem clara satura no `contrasta`: à meia amplitude o degrau voltava, de sinal trocado.
#[test]
fn papel_de_imagem_claro_nao_recria_o_degrau() {
    let lum = vec![204u8; 64 * 64];
    let s = papel_de_imagem(&lum, 64, 1.0);
    let degrau = s.tooth_loads(13, 7) - s.meio_do_dente();
    assert!(
        degrau.abs() < 1e-4,
        "tinta contra o papel à vista: {degrau}"
    );
}

/// O sentido contrário: a imagem escura satura em baixo.
#[test]
fn papel_de_imagem_escuro_nao_recria_o_degrau() {
    let lum = vec![40u8; 64 * 64];
    let s = papel_de_imagem(&lum, 64, 1.0);
    let degrau = s.tooth_loads(13, 7) - s.meio_do_dente();
    assert!(
        degrau.abs() < 1e-4,
        "tinta contra o papel à vista: {degrau}"
    );
}

/// Numa imagem com grão, a grelha da média acerta na média de toda a tela.
#[test]
fn a_grelha_da_media_acerta_num_papel_de_imagem_com_grao() {
    let lum: Vec<u8> = (0..128u32 * 128)
        .map(|i| {
            let h = i.wrapping_mul(2_654_435_761) >> 24;
            (150 + h / 4) as u8
        })
        .collect();
    let s = papel_de_imagem(&lum, 128, 0.5);
    let media = media_medida(&s, 512);
    let grelha = f64::from(s.meio_do_dente() / (s.amp_px / DEPTH_UNIT_PX));
    assert!(
        (grelha - media).abs() < 0.02,
        "grelha {grelha} × tela {media}"
    );
}

/// O papel de fábrica fica na meia amplitude que o dono aprovou, ao bit.
#[test]
fn o_papel_de_fabrica_fica_na_meia_amplitude() {
    let s = Substrate::resolve(default_paper(), 1.0, 0.5, None).expect("papel ligado");
    assert_eq!(s.meio_do_dente(), 0.5 / DEPTH_UNIT_PX);
}

/// Os tipos que saturam (Dots 0,10 · Scales 0,80 a Roughness 1): a amostragem acerta a média da tela em todos.
#[test]
fn a_media_do_dente_de_cada_tipo_acerta_na_tela() {
    for k in 1..=u8::MAX {
        let kind = TextureKind::from_u8(k);
        if kind == TextureKind::None {
            break;
        }
        if kind == TextureKind::Image {
            continue;
        }
        let mut t = PainterTool::default();
        t.set_brush_paper_kind(k);
        t.set_substrate_depth(1.0);
        t.set_substrate_roughness(1.0);
        let s = t.substrate().expect("papel ligado");
        let media = media_medida(&s, 512);
        let lei = f64::from(s.meio_do_dente() / (s.amp_px / DEPTH_UNIT_PX));
        assert!(
            (lei - media).abs() < 0.03,
            "{kind:?}: {lei:.3} × {media:.3}"
        );
    }
}

/// A tabela do Bug #71: a média do dente, em fracção da amplitude, por tipo de papel e Roughness.
#[test]
#[ignore = "sonda: a tabela do Bug #71"]
fn sonda_media_do_dente_por_tipo() {
    for k in 1..=u8::MAX {
        let kind = TextureKind::from_u8(k);
        if kind == TextureKind::None {
            break;
        }
        if kind == TextureKind::Image {
            continue;
        }
        let mut t = PainterTool::default();
        t.set_brush_paper_kind(k);
        t.set_substrate_depth(1.0);
        let mut linha = format!("MEDIA\t{kind:?}");
        for r in [0.0, 0.25, 0.5, 1.0] {
            t.set_substrate_roughness(r);
            let s = t.substrate().expect("papel ligado");
            linha += &format!("\t{:.3}", media_medida(&s, 512));
        }
        println!("{linha}");
    }
}
