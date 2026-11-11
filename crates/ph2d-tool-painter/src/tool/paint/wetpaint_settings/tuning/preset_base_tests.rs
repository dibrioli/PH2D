//! B35.8: o ↺ da linha e o ↺ do grupo do Tuning voltam ao pincel que abriu (`preset_base`), não à fábrica.

use super::*;

fn presets_do_wet(dir: &std::path::Path, fora: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).expect("pasta de presets").flatten() {
        let p = e.path();
        if p.is_dir() {
            presets_do_wet(&p, fora);
        } else if p.to_string_lossy().ends_with(".ph2d-brush.png") {
            fora.push(p);
        }
    }
}

/// ⭐ O GATE: em cada preset de fábrica do Wet, um knob mexido volta pelo ↺ da linha e pelo ↺ do grupo ao valor
/// que o preset P deu — reset depois de carregar P devolve P.
#[test]
fn reset_depois_de_carregar_p_devolve_p() {
    let mut caminhos = Vec::new();
    presets_do_wet(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes/WetPainter"),
        &mut caminhos,
    );
    caminhos.sort();
    assert!(caminhos.len() > 40, "{} presets", caminhos.len());
    let mut diferem = Vec::new();
    for c in &caminhos {
        let preset = ph2d_brush_presets::ler(&std::fs::read(c).unwrap()).unwrap();
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(PaintMedia::WetPaint);
        t.preset_aplicar(&preset).unwrap();
        let p = t.paint.wetpaint.knobs;
        for (&id, &(i, kind)) in wet_tuning_id_map() {
            if kind != TuneWidget::Reset {
                continue;
            }
            let def = &KNOB_DEFS[i];
            let longe = if p.get(def.knob) == def.max {
                def.min
            } else {
                def.max
            };
            t.paint.wetpaint.knobs.set(def.knob, longe);
            assert!(t.route_wet_tuning_click(id), "{}", def.key);
            let da_linha = t.paint.wetpaint.knobs.get(def.knob);
            t.paint.wetpaint.knobs.set(def.knob, longe);
            let gi = crate::ids::WET_TUNING_RESET_GROUPS
                .iter()
                .position(|g| *g == def.group);
            let do_grupo = gi.map(|gi| {
                assert!(t.route_wet_tuning_click(crate::ids::WET_TUNING_GROUP_RESETS[gi]));
                t.paint.wetpaint.knobs.get(def.knob)
            });
            if da_linha != p.get(def.knob) || do_grupo.is_some_and(|v| v != p.get(def.knob)) {
                let nome = c.file_name().unwrap().to_string_lossy().into_owned();
                diferem.push((nome, def.key, p.get(def.knob), da_linha, do_grupo));
            }
            t.paint.wetpaint.knobs = p;
        }
    }
    assert!(
        diferem.is_empty(),
        "{} resets fora do preset (pincel, knob, P, linha, grupo): {:?}",
        diferem.len(),
        &diferem[..diferem.len().min(6)]
    );
}
