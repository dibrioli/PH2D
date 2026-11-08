//! ⭐ **Bristle Combing no cartão Shape** (dono 07/11): os quatro fios que penteiam a tinta, à parte do tapete, nos 4 modos,
//! esmaecidos onde não agem. Pelo gesto real nos rects pintados.

use super::h1_os_cartoes::{Mesa, pincel};
use super::h10_fotos::foto;
use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{PaintMedia, TextureKind};

fn chips() -> [NodeId; 4] {
    p::CERDAS_DO_PENTEADO.map(p::wet_tuning_chip_id)
}

const MODOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// Os quatro aparecem em todo modo, abaixo de tudo o que é da textura, e o scrub chega ao barramento.
#[test]
fn os_quatro_fios_estao_no_shape_abaixo_do_tapete_em_todo_modo() {
    for meio in MODOS {
        let (mut b, _) = pincel(meio, "brush");
        b.shape_kind = TextureKind::Bristles.to_u8();
        let mut m = Mesa::nova(true, b, false);
        m.so_abertos(&[p::PAINTER_SHAPE_SECTION]);
        m.assenta();
        let tapete = m.rect(p::PAINTER_SHAPE_PARAMS[3]);
        for id in chips() {
            let r = m.rect(id).unwrap_or_else(|| panic!("{meio:?} sem {id:?}"));
            if let Some(t) = tapete {
                assert!(r.y > t.y, "{meio:?}: o fio ficou no meio do tapete");
            }
        }
        let eventos = m.arrasta_a_pega(chips()[1], 40.0);
        let saida = m.aplica(eventos);
        assert!(
            saida.iter().any(|s| s.contains("SetValue")),
            "{meio:?}: o Bristle relief não chegou à ferramenta: {saida:?}"
        );
    }
}

/// O tapete não ganha linha nova: os quatro sliders de sempre continuam os do Shape, e são outros ids.
#[test]
fn o_tapete_e_o_penteado_nao_partilham_um_id() {
    for id in chips() {
        assert!(!p::PAINTER_SHAPE_PARAMS.contains(&id));
    }
}

/// `PH2D_FOTO_DIR=<pasta>`: o cartão Shape no Wet Paint e no Digital.
#[test]
#[ignore = "foto: PH2D_FOTO_DIR=<pasta>, --ignored"]
fn bristle_combing_foto() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    for (nome, meio) in [
        ("shape_wet", PaintMedia::WetPaint),
        ("shape_digital", PaintMedia::Digital),
    ] {
        let (mut b, _) = pincel(meio, "brush");
        b.shape_kind = TextureKind::Bristles.to_u8();
        let janela = Rect::new(0.0, 0.0, 1600.0, 2400.0);
        let mut m = Mesa::na_janela(true, b, false, janela);
        m.so_abertos(&[p::PAINTER_SHAPE_SECTION]);
        m.assenta();
        foto(&gpu, &dir, nome, &mut m);
    }
}
