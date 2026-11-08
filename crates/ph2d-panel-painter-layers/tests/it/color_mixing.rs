//! **Color mixing** (a mistura opaca do Digital) no cartão Basics e no painel de antes: só onde a lei age, os números
//! esmaecidos com `Off`, e o clique real chega ao barramento.

use super::h1_os_cartoes::{Mesa, pincel};
use super::h10_fotos::foto;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{ModoMisturaOpaca, PaintMedia};

/// No Pincel do Digital, nos dois painéis: o seletor e os cinco números pintados; com `Off` os números esmaecidos e o
/// seletor vivo; o clique em «Blend» e o arrasto num número chegam à ferramenta.
#[test]
fn a_mistura_aparece_no_digital_e_o_clique_chega() {
    for cartoes in [true, false] {
        let (b, _) = pincel(PaintMedia::Digital, "brush");
        assert_eq!(b.mistura_opaca.modo, ModoMisturaOpaca::Off);
        let mut m = Mesa::nova(cartoes, b, false);
        m.so_abertos(&[p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS]);
        for id in p::PAINTER_BRUSH_MIX_MODES
            .iter()
            .chain(&p::PAINTER_BRUSH_MIX_FIELDS)
        {
            assert!(m.rect(*id).is_some(), "cartões={cartoes}: {id:?} não pintado");
        }
        for id in p::PAINTER_BRUSH_MIX_FIELDS {
            assert!(b.inercias.dica_de(id).is_some(), "com Off o número esmaece");
        }
        for id in p::PAINTER_BRUSH_MIX_MODES {
            assert!(b.inercias.dica_de(id).is_none(), "o seletor nunca esmaece");
        }
        let saida = m.clica(p::PAINTER_BRUSH_MIX_MODES[1]);
        let alvo = format!("{:?}", p::PAINTER_BRUSH_MIX_MODES[1]);
        assert!(
            saida.iter().any(|s| s.contains("Click") && s.contains(&alvo)),
            "cartões={cartoes}: o Blend não chegou: {saida:?}"
        );
        if cartoes {
            let eventos = m.arrasta_a_pega(p::PAINTER_BRUSH_MIX_FIELDS[2], 40.0);
            let saida = m.aplica(eventos);
            assert!(
                saida.iter().any(|s| s.contains("SetValue")),
                "o Color stretch não chegou: {saida:?}"
            );
        }
    }
}

/// Fora do Pincel do Digital a lei não corre: o grupo não aparece (como o Fine Thread fora do Smear do Digital).
#[test]
fn a_mistura_some_onde_a_lei_nao_corre() {
    let fora = [
        (PaintMedia::WetPaint, "brush"),
        (PaintMedia::Watercolor, "brush"),
        (PaintMedia::Impasto, "brush"),
        (PaintMedia::Digital, "eraser"),
    ];
    for (meio, modo) in fora {
        for cartoes in [true, false] {
            let (b, _) = pincel(meio, modo);
            let mut m = Mesa::nova(cartoes, b, false);
            m.so_abertos(&[p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS]);
            for id in p::PAINTER_BRUSH_MIX_MODES
                .iter()
                .chain(&p::PAINTER_BRUSH_MIX_FIELDS)
            {
                assert!(m.rect(*id).is_none(), "{meio:?}/{modo} cartões={cartoes}: {id:?} à vista");
            }
        }
    }
}

/// `PH2D_FOTO_DIR=<pasta>`: o cartão Basics do Digital com a mistura em Off e em Pigment mix.
#[test]
#[ignore = "foto: PH2D_FOTO_DIR=<pasta>, --ignored"]
fn color_mixing_foto() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    for (nome, modo) in [
        ("painel_mistura_off", ModoMisturaOpaca::Off),
        ("painel_pigment_mix", ModoMisturaOpaca::WetBlend),
    ] {
        let (mut b, _) = pincel(PaintMedia::Digital, "brush");
        b.mistura_opaca.modo = modo;
        if modo != ModoMisturaOpaca::Off {
            b.inercias = ph2d_tool_painter::Inercias::NENHUMA;
        }
        let janela = Rect::new(0.0, 0.0, 1600.0, 2400.0);
        let mut m = Mesa::na_janela(true, b, false, janela);
        m.so_abertos(&[p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS]);
        foto(&gpu, &dir, nome, &mut m);
    }
}
