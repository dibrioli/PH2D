//! ⭐ **O painel de cada ferramenta em cada meio** (dono 24/10, as capturas: com a Borracha, o Blur, o Smear e o Clone na
//! mão viam-se «Blend + Color», «Composite Brush» e «Paint Mode»). Medido: o meio do painel era o ARME do Wet, que é
//! de todos — com a água armada no Pincel, o Blur mostrava a secção do Wet inteira. Agora o meio é o da ferramenta:
//! Blur/Smear/Clone/Máscara só no Digital (o Paint Mode sai), a Borracha nos seus três.

use super::censo_dos_controlos::Bancada;
use ph2d_a11y::NodeId;
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::ids::*;

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// O que uma ferramenta sem cor nunca mostra.
const DA_COR: [NodeId; 3] = [
    PAINTER_BRUSH_BLEND,
    PAINTER_BRUSH_COMPOSITE_ENABLE,
    ph2d_editor_core::ids::PAINTER_COLOR_THUMB,
];

/// O pincel no meio `do_pincel` (o estado do dono), depois a ferramenta pelo trilho e, se ela o tem, o meio dela.
fn bancada(do_pincel: PaintMedia, modo: &str, da_ferramenta: Option<PaintMedia>) -> Bancada {
    let mut b = Bancada::nova(do_pincel, &[]);
    b.no_modo(modo);
    if let Some(m) = da_ferramenta {
        b.tool.set_paint_media(m);
        b.no_modo(modo);
    }
    b
}

/// O controlo: o Pincel mostra a cor e o Paint Mode (senão o gate mediria o vazio).
#[test]
fn o_pincel_mostra_a_cor_e_o_paint_mode() {
    let ids = bancada(PaintMedia::Digital, "brush", None).ids();
    for id in DA_COR.iter().chain(&[PAINTER_BRUSH_MEDIA]) {
        assert!(ids.contains(id), "o Pincel não pinta {id:?}");
    }
    let wet = bancada(PaintMedia::WetPaint, "brush", None).ids();
    assert!(
        wet.contains(&PAINTER_WETPAINT_SECTION),
        "o Pincel no Wet pinta a secção do Wet"
    );
}

/// Vermelho antes (o pincel no Wet): o Blur/Smear/Clone/Máscara pintavam o Paint Mode, o Blend, o Color, o Composite e
/// a secção do Wet.
#[test]
fn as_ferramentas_de_um_meio_so_nao_mostram_o_paint_mode_nem_o_meio_do_pincel() {
    for do_pincel in MEIOS {
        for modo in ["blur", "smear", "clone", "mask"] {
            let b = bancada(do_pincel, modo, None);
            let ids = b.ids();
            let fora: Vec<_> = DA_COR
                .iter()
                .chain(&[PAINTER_BRUSH_MEDIA, PAINTER_WETPAINT_SECTION])
                .filter(|id| ids.contains(id))
                .collect();
            assert!(
                fora.is_empty(),
                "{modo} com o Pincel em {do_pincel:?}: pinta {fora:?}"
            );
            assert_eq!(
                b.tool.paint_media(),
                PaintMedia::Digital,
                "{modo} {do_pincel:?}"
            );
        }
    }
}

/// A Borracha nos seus três meios: o Paint Mode à vista (sem o Impasto), e nunca a cor.
#[test]
fn a_borracha_mostra_os_seus_tres_meios_e_nunca_a_cor() {
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::WetPaint,
    ] {
        let b = bancada(PaintMedia::Digital, "eraser", Some(meio));
        assert_eq!(b.tool.paint_media(), meio);
        let ids = b.ids();
        assert!(ids.contains(&PAINTER_BRUSH_MEDIA), "{meio:?}: o Paint Mode");
        let fora: Vec<_> = DA_COR.iter().filter(|id| ids.contains(id)).collect();
        assert!(fora.is_empty(), "Borracha em {meio:?}: pinta {fora:?}");
        assert_eq!(
            ids.contains(&PAINTER_WETPAINT_SECTION),
            meio == PaintMedia::WetPaint,
            "{meio:?}: a secção do Wet"
        );
        let b = b.tool.brush_settings();
        assert_eq!(b.meios.count_ones(), 3);
        assert_eq!(b.meios & (1 << PaintMedia::Impasto.to_u8()), 0);
    }
}

/// ⭐ **O Composite Brush é só do Digital** (dono 25/10): o cartão aparecia no Wet Paint e no Impasto (só a aguada o
/// escondia). Com o checkbox LIGADO (o preset antigo que o traz), o Pincel em cada meio: o cartão só no Digital, e a
/// Strength escondida pelo Composite só lá — nos outros meios ele não manda nada.
#[test]
fn o_composite_brush_so_aparece_no_digital() {
    for meio in MEIOS {
        let mut b = bancada(meio, "brush", None);
        b.tool.toggle_composite();
        b.no_modo("brush");
        assert!(b.tool.composite_enabled(), "{meio:?}: o checkbox ligado");
        let ids = b.ids();
        assert_eq!(
            ids.contains(&PAINTER_BRUSH_COMPOSITE_ENABLE),
            meio == PaintMedia::Digital,
            "{meio:?}: o cartão do Composite"
        );
        assert_eq!(
            b.tool.brush_settings().composite_on(),
            meio == PaintMedia::Digital,
            "{meio:?}: o Composite manda no pincel"
        );
    }
}
