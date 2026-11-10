//! **Color mixing** (a mistura opaca do Digital) no cartão próprio **Color Mixing** (logo depois do Basics, dono 10/11)
//! e no painel de antes: só onde a lei age, os números esmaecidos com `Off`, e o clique real chega ao barramento.

use super::h1_os_cartoes::{Mesa, pincel};
use super::h10_fotos::foto;
use ph2d_editor_core::panel::cards::grip_of;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{ModoMisturaOpaca, PaintMedia};

/// Os ids de controlo da mistura: o seletor, o Sample e os cinco números.
fn controlos() -> impl Iterator<Item = &'static ph2d_a11y::NodeId> {
    p::PAINTER_BRUSH_MIX_MODES
        .iter()
        .chain(&p::PAINTER_BRUSH_MIX_SAMPLES)
        .chain(&p::PAINTER_BRUSH_MIX_FIELDS)
}

/// Só Topo e Color Mixing abertos (`PAINTER_BRUSH_MIX_SECTION` nasce dobrado; no painel de antes é a dobra do bloco).
fn abre_o_basics(m: &mut Mesa) {
    m.so_abertos(&[p::PAINTER_CARD_TOPO, p::PAINTER_BRUSH_MIX_SECTION]);
}

/// ⭐ **Color Mixing é um cartão próprio** (dono 10/11): o cabeçalho dele vem logo depois do Basics, o título é
/// «Color Mixing», e com o Basics aberto e o Color Mixing dobrado nenhum controlo da mistura está à vista — o Basics já
/// não os tem.
#[test]
fn color_mixing_e_um_cartao_proprio_logo_depois_do_basics() {
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    assert_eq!(
        ph2d_i18n::tr("panel.painter_layers.brush.paint_mixing"),
        "Paint Mixing"
    );
    let mut m = Mesa::nova(true, b, false);
    m.so_abertos(&[p::PAINTER_CARD_BASICOS]);
    let basics = m.rect(grip_of(p::PAINTER_CARD_BASICOS)).expect("o Basics");
    let mistura = m
        .rect(grip_of(p::PAINTER_BRUSH_MIX_SECTION))
        .expect("o cartão Color Mixing");
    assert!(mistura.y > basics.y, "o Color Mixing vem depois do Basics");
    let entre: Vec<_> = super::h1_os_cartoes::TODAS
        .iter()
        .filter_map(|s| m.rect(grip_of(*s)).map(|r| (*s, r.y)))
        .filter(|(_, y)| *y > basics.y && *y < mistura.y)
        .collect();
    assert!(
        entre.is_empty(),
        "um cartão entre Basics e Color Mixing: {entre:?}"
    );
    for id in controlos() {
        assert!(m.rect(*id).is_none(), "{id:?} ainda no Basics");
    }
    abre_o_basics(&mut m);
    let mistura = m
        .rect(grip_of(p::PAINTER_BRUSH_MIX_SECTION))
        .expect("o cartão Color Mixing");
    for id in controlos() {
        assert!(m.rect(*id).is_some(), "{id:?} fora do cartão aberto");
        assert!(
            m.rect(*id).expect("pintado").y > mistura.y,
            "{id:?} acima do cabeçalho do Color Mixing"
        );
    }
}

/// ⭐ **Color Mixing dobra** (nasce dobrado): o clique no cabeçalho do cartão esconde o seletor, o Sample e os
/// números, e outro clique devolve-os.
#[test]
fn a_mistura_dobra_pelo_titulo() {
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mut m = Mesa::nova(true, b, false);
    abre_o_basics(&mut m);
    let _ = m.clica(grip_of(p::PAINTER_BRUSH_MIX_SECTION));
    m.assenta();
    for id in p::PAINTER_BRUSH_MIX_MODES
        .iter()
        .chain(&p::PAINTER_BRUSH_MIX_SAMPLES)
        .chain(&p::PAINTER_BRUSH_MIX_FIELDS)
    {
        assert!(m.rect(*id).is_none(), "{id:?} à vista com a dobra fechada");
    }
    let _ = m.clica(grip_of(p::PAINTER_BRUSH_MIX_SECTION));
    m.assenta();
    assert!(
        m.rect(p::PAINTER_BRUSH_MIX_MODES[1]).is_some(),
        "a dobra não reabriu"
    );
    assert!(
        m.rect(p::PAINTER_BRUSH_MIX_SAMPLES[0]).is_some(),
        "a dobra não reabriu"
    );
}

/// No Pincel do Digital, nos dois painéis: o seletor e os cinco números pintados; com `Off` os números esmaecidos e o
/// seletor vivo; o clique em «Blend» e o arrasto num número chegam à ferramenta.
#[test]
fn a_mistura_aparece_no_digital_e_o_clique_chega() {
    for cartoes in [true, false] {
        let (b, _) = pincel(PaintMedia::Digital, "brush");
        assert_eq!(b.mistura_opaca.modo, ModoMisturaOpaca::Off);
        let mut m = Mesa::nova(cartoes, b, false);
        abre_o_basics(&mut m);
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
            // Os CINCO números (Amount, Density, Color stretch e as duas pressões): cada arrasto chega com o id DELE.
            for id in p::PAINTER_BRUSH_MIX_FIELDS {
                let eventos = m.arrasta_a_pega(id, 40.0);
                let saida = m.aplica(eventos);
                let alvo = format!("{id:?}");
                assert!(
                    saida
                        .iter()
                        .any(|s| s.contains("SetValue") && s.contains(&alvo)),
                    "o número {alvo} não chegou: {saida:?}"
                );
            }
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
        abre_o_basics(&mut m);
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
        abre_o_basics(&mut m);
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
            abre_o_basics(&mut m);
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

/// `PH2D_FOTO_DIR=<pasta>`: o cartão Color Mixing do Digital com a mistura em Off e em Wet blend.
#[test]
#[ignore = "foto: PH2D_FOTO_DIR=<pasta>, --ignored"]
fn color_mixing_foto() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    for (nome, modo, meio) in [
        (
            "painel_mistura_off",
            ModoMisturaOpaca::Off,
            PaintMedia::Digital,
        ),
        (
            "painel_pigment_mix",
            ModoMisturaOpaca::WetBlend,
            PaintMedia::Digital,
        ),
        (
            "painel_paint_mixing_watercolor",
            ModoMisturaOpaca::Off,
            PaintMedia::Watercolor,
        ),
    ] {
        let (mut b, _) = pincel(meio, "brush");
        b.mistura_opaca.modo = modo;
        if modo != ModoMisturaOpaca::Off {
            b.inercias = ph2d_tool_painter::Inercias::NENHUMA;
        }
        let janela = Rect::new(0.0, 0.0, 1600.0, 2400.0);
        let mut m = Mesa::na_janela(true, b, false, janela);
        m.so_abertos(&[
            p::PAINTER_CARD_TOPO,
            p::PAINTER_CARD_BASICOS,
            p::PAINTER_BRUSH_MIX_SECTION,
        ]);
        foto(&gpu, &dir, nome, &mut m);
    }
}

/// ⭐ **Paint Mixing: UM cartão, UM Pigment** (dono 10/11: «junte e mude de nome»). Onde o Pigment age (Digital,
/// Watercolor, Impasto) o cartão aparece uma vez com uma linha Pigment só; o Color mixing (seletor, Sample, números)
/// só no Digital. Nos dois painéis.
#[test]
fn paint_mixing_e_um_cartao_com_um_pigment_so() {
    let casos = [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, false),
        (PaintMedia::Impasto, false),
    ];
    for (meio, com_wet_blend) in casos {
        for cartoes in [true, false] {
            let (b, _) = pincel(meio, "brush");
            assert!(b.pigment_offered, "{meio:?}: o Pigment age aqui");
            let mut m = Mesa::nova(cartoes, b, false);
            abre_o_basics(&mut m);
            m.quadro();
            let pigmentos = m
                .rects
                .iter()
                .filter(|(id, r)| *id == p::PAINTER_WATERCOLOR_MIX && r.w > 0.0 && r.h > 0.0)
                .count();
            assert_eq!(pigmentos, 1, "{meio:?} cartões={cartoes}: um Pigment só");
            if cartoes {
                let cartoes_mix = super::h1_os_cartoes::TODAS
                    .iter()
                    .filter(|s| m.rect(grip_of(**s)).is_some())
                    .filter(|s| **s == p::PAINTER_BRUSH_MIX_SECTION)
                    .count();
                assert_eq!(cartoes_mix, 1, "{meio:?}: o cartão Paint Mixing");
            }
            assert_eq!(
                m.rect(p::PAINTER_BRUSH_MIX_MODES[1]).is_some(),
                com_wet_blend,
                "{meio:?} cartões={cartoes}: o Color mixing só no Digital"
            );
        }
    }
}
