//! B40.1 (DF-54): o 1.º dab de Line/Arc/Free Hand (`fill_polyline_preview`), Ellipse e Polygon passa
//! pela porta única `stamp_at` — leva o espelho da Symmetry como os outros.

use super::super::*;
use crate::falloff::Falloff;
use crate::symmetry::{MirrorAxis, SymmetrySettings};

fn spec(sym: bool) -> BrushSpec {
    BrushSpec {
        radius_px: 6.0,
        spacing: 0.5,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        symmetry: SymmetrySettings {
            enabled: sym,
            axis: MirrorAxis::X,
            center: [50.0, 0.0],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn fill(method: u8, sym: bool) -> Vec<Dab> {
    let dynamics = Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    };
    let mut s = Stroke::new(spec(sym), dynamics, 7);
    let mut out = Vec::new();
    match method {
        0 => s.fill_polyline_preview(&[[10.0, 10.0], [40.0, 30.0], [80.0, 12.0]], &mut out),
        1 => s.fill_ellipse_preview([30.0, 40.0], [1.0, 0.0], 20.0, 12.0, &mut out),
        _ => s.fill_polygon_preview([30.0, 40.0], [1.0, 0.0], 20.0, 12.0, 5, &mut out),
    }
    out
}

#[test]
fn cada_metodo_de_forma_espelha_o_primeiro_dab() {
    for (method, nome) in [(0u8, "polyline"), (1, "ellipse"), (2, "polygon")] {
        let base = fill(method, false);
        let sym = fill(method, true);
        assert!(!base.is_empty(), "{nome}: fixture pinta");
        assert_eq!(
            sym.len(),
            base.len() * 2,
            "{nome}: com Mirror X cada dab ganha a sua cópia — o 1.º incluído"
        );
        assert_eq!(sym[0].center, base[0].center, "{nome}: o 1.º dab é a base");
        assert!(
            (sym[1].center[0] - (100.0 - base[0].center[0])).abs() < 1e-3,
            "{nome}: o 2.º dab é o espelho do 1.º ({:?})",
            sym[1].center
        );
    }
}
