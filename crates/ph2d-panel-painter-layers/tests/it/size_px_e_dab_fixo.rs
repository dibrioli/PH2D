//! **Size em píxeis e Fixed dab size em todo meio** (dono 08/11): o Basics mostra o DIÂMETRO em px (a caixa
//! «Size (px)» do Wet Paint, em todo meio e nos dois painéis) e a entrada de `N` px dá o raio `N/2`; a caixa
//! «Fixed dab size» aparece no cartão Shape de todo meio, esmaecida com a dica num pincel que não espalha.

use super::censo_dos_controlos::Bancada;
use super::h1_os_cartoes::Mesa;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, PaintMedia};

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

fn pincel_em(meio: PaintMedia, count_norm: f64, across: f64) -> BrushSettings {
    let mut b = Bancada::nova(meio, &[]);
    b.no_modo("brush");
    b.tool.handle_panel_event(PanelEvent::SetValue(
        p::PAINTER_BRUSH_SPRAY_COUNT,
        count_norm,
    ));
    b.tool
        .handle_panel_event(PanelEvent::SetValue(p::PAINTER_MARCA_FIELDS[6], across));
    b.tool.brush_settings()
}

/// (a) O Size mostra o diâmetro em px de um raio conhecido, nos dois painéis e em todo meio; `N` px → raio `N/2`.
#[test]
fn o_size_do_basics_e_o_diametro_em_px() {
    for meio in MEIOS {
        let mut b = pincel_em(meio, 0.0, 0.0);
        b.size_px = 20.0;
        for cartoes in [true, false] {
            let m = Mesa::nova(cartoes, b, false);
            let v = m
                .host
                .store()
                .number_value(p::PAINTER_WETPAINT_VOL_SIZE)
                .unwrap_or_else(|| panic!("{meio:?} cartões {cartoes}: sem Size (px)"));
            assert!(
                (v - 40.0).abs() < 1e-3,
                "{meio:?} cartões {cartoes}: Size {v}"
            );
            assert!(
                m.rect(p::PAINTER_WETPAINT_VOL_SIZE).is_some(),
                "{meio:?} cartões {cartoes}: o Size (px) não foi pintado"
            );
            assert!(
                m.rect(p::PAINTER_BRUSH_SIZE_CHIP).is_none(),
                "{meio:?} cartões {cartoes}: ainda o Size fraccionário"
            );
        }
        let mut t = Bancada::nova(meio, &[]);
        t.no_modo("brush");
        t.tool
            .handle_panel_event(PanelEvent::SetValue(p::PAINTER_WETPAINT_VOL_SIZE, 50.0));
        let r = t.tool.brush_settings().size_px;
        assert!((r - 25.0).abs() < 1e-3, "{meio:?}: 50 px deu o raio {r}");
    }
}

/// (b) A caixa à vista no cartão Shape de todo meio (e no painel de antes); esmaecida só sem espalhar.
#[test]
fn a_caixa_do_ponto_fixo_aparece_em_todo_meio_e_so_age_a_espalhar() {
    let dica = "panel.painter_layers.inerte.fixed_dab_needs_scatter";
    for meio in MEIOS {
        for (count, across, inerte) in [(0.0, 0.0, true), (0.0, 0.4, false), (0.5, 0.0, false)] {
            let b = pincel_em(meio, count, across);
            assert_eq!(
                b.inercias.dica_de(p::PAINTER_BRUSH_DAB_FIXO) == Some(dica),
                inerte,
                "{meio:?} count {} across {across}",
                b.spray_count
            );
            for cartoes in [true, false] {
                let mut m = Mesa::nova(cartoes, b, false);
                if cartoes {
                    m.so_abertos(&[p::PAINTER_SHAPE_SECTION]);
                }
                assert!(
                    m.rect(p::PAINTER_BRUSH_DAB_FIXO).is_some(),
                    "{meio:?} cartões {cartoes} count {} across {across}: sem a caixa",
                    b.spray_count
                );
            }
        }
    }
}
