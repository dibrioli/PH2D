//! B35.12 (PA-5..7): um knob que um pincel de fábrica mexe tem um controlo à vista — no cartão Wet, na linha da
//! ferramenta da água, no Inspector (caneta, Shape) ou no Tuning.

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

/// Os que têm casa noutra tabela do cartão (os controlos da E10 e os volumes, `set_controlo_e10`/`numeros_dos_volumes`),
/// no Shape › Bristles (o B35.13 junta-os lá) ou que se derivam de outro controlo (Volume, Dilution, o relevo pelo
/// tamanho): conferidos a 11/11 pelo primeiro vermelho deste gate. Um knob NOVO sem casa reprova.
const OUTRA_CASA: [&str; 18] = [
    "body",
    "dilution",
    "medium",
    "paperGate",
    "smoothEdges",
    "paperContrast",
    "volume",
    "intensity",
    "pigmentPerDab",
    "waterPerDab",
    "felt",
    "bristleCount",
    "bristleSize",
    "bristleStrength",
    "capsuleMaxPx",
    "capsuleRoundNearest",
    "pigmentSpacing",
    "embossSizeExp",
];

/// O knob tem casa à vista: cartão Wet, ferramenta da água, Inspector (B35.1), Shape (penteado) ou linha do Tuning.
fn tem_casa(knob: Knob) -> bool {
    let key = KNOB_DEFS[knob as usize].key;
    let id = crate::ids::wet_tuning_slider_id(key);
    super::super::KNOBS_NO_CARTAO
        .iter()
        .any(|(_, k)| *k == knob)
        || super::super::KNOBS_DA_FERRAMENTA
            .iter()
            .any(|(_, k, ..)| *k == knob)
        || crate::ids::NO_INSPECTOR.contains(&key)
        || crate::ids::CERDAS_DO_PENTEADO.contains(&key)
        || wet_tuning_id_map().contains_key(&id)
        || OUTRA_CASA.contains(&key)
}

/// ⭐ O GATE: nenhum knob que um preset de fábrica do Wet afasta da fábrica fica sem controlo à vista.
#[test]
fn todo_knob_de_fabrica_vivo_tem_linha_no_cartao_ou_no_tuning() {
    let mut caminhos = Vec::new();
    presets_do_wet(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes/WetPainter"),
        &mut caminhos,
    );
    assert!(caminhos.len() > 40, "{} presets", caminhos.len());
    let mut sem_casa = std::collections::BTreeMap::<&str, usize>::new();
    for c in &caminhos {
        let preset = ph2d_brush_presets::ler(&std::fs::read(c).unwrap()).unwrap();
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(PaintMedia::WetPaint);
        t.preset_aplicar(&preset).unwrap();
        let p = t.paint.wetpaint.knobs;
        for d in KNOB_DEFS.iter() {
            if p.get(d.knob) != WetKnobs::fabrica(d.knob) && !tem_casa(d.knob) {
                *sem_casa.entry(d.key).or_default() += 1;
            }
        }
    }
    assert!(
        sem_casa.is_empty(),
        "knob (pincéis que o mexem) sem controlo: {sem_casa:?}"
    );
}
