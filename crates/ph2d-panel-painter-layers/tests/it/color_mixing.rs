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
            assert!(
                m.rect(*id).is_some(),
                "cartões={cartoes}: {id:?} não pintado"
            );
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
            saida
                .iter()
                .any(|s| s.contains("Click") && s.contains(&alvo)),
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

/// ⭐ **Sample** (This layer · All visible layers, Bug #79): nos dois painéis logo depois do seletor do modo, esmaecido
/// com `Off` (a dica da inércia), com a explicação no hover ligada, e o clique real em «All visible layers» chega à
/// ferramenta e muda `mistura_opaca.amostra`.
#[test]
fn o_sample_da_mistura_aparece_esmaece_e_o_clique_chega() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::tool::Tool;
    use ph2d_tool_painter::{AmostraDaMistura, Dependente, PainterTool};
    let explica = ph2d_i18n::tr("panel.painter_layers.brush.mix_sample_tip");
    let inerte = ph2d_i18n::tr(Dependente::MisturaDesligada.dica());
    for cartoes in [true, false] {
        let (b, _) = pincel(PaintMedia::Digital, "brush");
        assert_eq!(b.mistura_opaca.amostra, AmostraDaMistura::EstaCamada);
        // Off: à vista, esmaecido, com a dica da inércia.
        let mut m = Mesa::nova(cartoes, b, false);
        m.so_abertos(&[p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS]);
        let modo = m.rect(p::PAINTER_BRUSH_MIX_MODES[0]).expect("o modo");
        for id in p::PAINTER_BRUSH_MIX_SAMPLES {
            let r = m
                .rect(id)
                .unwrap_or_else(|| panic!("cartões={cartoes}: {id:?} não pintado"));
            assert!(
                r.y > modo.y,
                "cartões={cartoes}: o Sample vem depois do modo"
            );
            assert!(b.inercias.dica_de(id).is_some(), "com Off o Sample esmaece");
            assert_eq!(
                m.host.store().tooltip_for(id),
                Some(inerte),
                "cartões={cartoes}: a dica da inércia"
            );
        }
        // Wet blend: vivo, com a explicação; o clique chega à ferramenta.
        let mut b = b;
        b.mistura_opaca.modo = ModoMisturaOpaca::WetBlend;
        b.inercias = ph2d_tool_painter::Inercias::NENHUMA;
        let mut m = Mesa::nova(cartoes, b, false);
        m.so_abertos(&[p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS]);
        for id in p::PAINTER_BRUSH_MIX_SAMPLES {
            assert_eq!(
                m.host.store().tooltip_for(id),
                Some(explica),
                "cartões={cartoes}: a explicação do Sample"
            );
        }
        let r = m
            .rect(p::PAINTER_BRUSH_MIX_SAMPLES[1])
            .expect("All visible");
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(m.host.hit_at(cx, cy), Some(p::PAINTER_BRUSH_MIX_SAMPLES[1]));
        let _ = m.host.drained_actions();
        for e in m.host.click_at(cx, cy) {
            m.host
                .apply_panel_event::<ph2d_panel_painter_layers::PainterLayersPanel>(&mut m.st, e);
        }
        let mut t = PainterTool::default();
        for a in m.host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = a {
                t.handle_panel_event(pe);
            }
        }
        assert_eq!(
            t.brush_settings().mistura_opaca.amostra,
            AmostraDaMistura::TodasVisiveis,
            "cartões={cartoes}: o clique em «All visible layers» não chegou"
        );
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
                .chain(&p::PAINTER_BRUSH_MIX_SAMPLES)
            {
                assert!(
                    m.rect(*id).is_none(),
                    "{meio:?}/{modo} cartões={cartoes}: {id:?} à vista"
                );
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
