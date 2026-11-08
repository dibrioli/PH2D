//! **Size em píxeis e Fixed dab size em todo meio** (dono 08/11): o Basics mostra o DIÂMETRO em px (a caixa
//! «Size (px)» do Wet Paint, em todo meio e nos dois painéis) e a entrada de `N` px dá o raio `N/2`; a caixa
//! «Fixed dab size» aparece no Jitter de todo meio, esmaecida com a dica num pincel que não espalha.

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

/// (b) A caixa e o campo moram no Jitter, logo depois do Scatter (dono 08/11), em todo meio e nos dois painéis; fora
/// do Shape; esmaecidos só sem espalhar; e com a dica de hover sempre (a explicação, ou a da inércia esmaecidos).
#[test]
fn a_caixa_do_ponto_fixo_aparece_em_todo_meio_e_so_age_a_espalhar() {
    let inerte = "panel.painter_layers.inerte.fixed_dab_needs_scatter";
    let explica = [
        (
            p::PAINTER_BRUSH_DAB_FIXO,
            "panel.painter_layers.stroke.jitter.fixed_dab_tip",
        ),
        (
            p::PAINTER_BRUSH_DAB_PX,
            "panel.painter_layers.stroke.jitter.dab_px_tip",
        ),
    ];
    let scatter = (p::PAINTER_MARCA_FIELDS[6], p::PAINTER_MARCA_CHIPS[6]);
    for meio in MEIOS {
        for (count, across, esmaecida) in [(0.0, 0.0, true), (0.0, 0.4, false), (0.5, 0.0, false)] {
            let mut b = pincel_em(meio, count, across);
            assert_eq!(
                b.inercias.dica_de(p::PAINTER_BRUSH_DAB_FIXO) == Some(inerte),
                esmaecida,
                "{meio:?} count {} across {across}",
                b.spray_count
            );
            b.dab_fixo = true;
            for cartoes in [true, false] {
                let ctx = format!("{meio:?} cartões {cartoes} count {count} across {across}");
                let mut m = Mesa::nova(cartoes, b, false);
                if cartoes {
                    m.so_abertos(&[p::PAINTER_SHAPE_SECTION]);
                    assert!(
                        m.rect(p::PAINTER_BRUSH_DAB_FIXO).is_none(),
                        "{ctx}: a caixa ainda no Shape"
                    );
                    m.so_abertos(&[p::PAINTER_BRUSH_STROKE_SECTION]);
                }
                let caixa = m
                    .rect(p::PAINTER_BRUSH_DAB_FIXO)
                    .unwrap_or_else(|| panic!("{ctx}: sem a caixa"));
                let campo = m
                    .rect(p::PAINTER_BRUSH_DAB_PX)
                    .unwrap_or_else(|| panic!("{ctx}: sem o campo"));
                let antes = m
                    .rect(scatter.0)
                    .or_else(|| m.rect(scatter.1))
                    .unwrap_or_else(|| panic!("{ctx}: sem Scatter"));
                let depois = m
                    .rect(p::PAINTER_BRUSH_JITTER_SPACING)
                    .or_else(|| m.rect(p::PAINTER_BRUSH_JITTER_SPACING_CHIP))
                    .unwrap_or_else(|| panic!("{ctx}: sem o Spacing do Jitter"));
                assert!(
                    antes.y < caixa.y && caixa.y <= campo.y && campo.y < depois.y,
                    "{ctx}: fora do Jitter, entre Scatter e Spacing: {antes:?} {caixa:?} {campo:?} {depois:?}"
                );
                for (id, chave) in explica {
                    let quer = ph2d_i18n::tr(if esmaecida { inerte } else { chave });
                    assert_eq!(
                        m.host.store().tooltip_for(id),
                        Some(quer),
                        "{ctx}: a dica de {id:?}"
                    );
                }
            }
        }
    }
}
