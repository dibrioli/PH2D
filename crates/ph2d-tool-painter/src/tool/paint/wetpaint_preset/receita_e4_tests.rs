//! E4 — a migração dos presets de fábrica (ADR-0197 §1.1): cada pincel Wet Paint de fábrica, carregado pelo
//! carregador do produto, pousa o MESMO pigmento e a MESMA água por unidade de depósito que na lei velha.

use super::*;
use ph2d_wet_paint::painter::receita::{Controlos, receita, veiculo};
use ph2d_wet_paint::tuning::Knob;

/// Relativa: a Opacity viaja em `f32` na ferramenta.
const TOL: f64 = 1e-6;

#[test]
fn cada_preset_de_fabrica_pousa_o_mesmo_pigmento_e_agua_por_dab() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let mut n = 0;
    let mut falhas = Vec::new();
    let mut agua_fora = 0;
    let mut borrachas_fora = 0;
    for e in bib
        .entradas
        .iter()
        .filter(|e| e.preset.parametros.media == MEIO_DO_PRESET)
    {
        // A ferramenta da água (Water Brush) não passa pelo depósito de tinta: só o corpo dela vai ao slot.
        let ferramenta = ferramenta_do_preset(&e.preset.parametros);
        if ferramenta.ferramenta_da_agua().is_some() {
            agua_fora += 1;
            continue;
        }
        // A Borracha também não (Bug #65 A5): a receita do Pincel fica como está.
        if ferramenta == crate::FerramentaDoPincel::Borracha {
            borrachas_fora += 1;
            continue;
        }
        let prm = &e.preset.parametros;
        let mut t = PainterTool::default();
        t.set_paint_media(PaintMedia::WetPaint);
        let d = WetKnobs::DEFAULT;
        let lei = |key: &str, kn: Knob| prm.engine.get(key).copied().unwrap_or(d.get(kn));
        let ppd = lei("pigmentPerDab", Knob::PigmentPerDab).clamp(0.0, 2000.0);
        let wpd = lei("waterPerDab", Knob::WaterPerDab).clamp(0.0, 4.0);
        let agua = prm.volumes.water.unwrap_or(t.paint.wetpaint.knobs.water);
        let op = prm
            .volumes
            .opacity
            .unwrap_or(f64::from(super::OPACIDADE_DE_FABRICA));
        let (pig_velho, agua_velha) = (ppd * op, veiculo(agua, wpd));

        t.wet_preset_aplicar(&e.preset);
        let k = &t.paint.wetpaint.knobs;
        // E10: a receita lê a Dilution e o Volume que a migração escreveu (o meio à parte: só a água o sente).
        let r = receita(&Controlos {
            diluicao: k.dilution(),
            volume_um: k.volume_um(),
            meio: 0.0,
        });
        let rel = |a: f64, b: f64| {
            if b == 0.0 {
                a.abs()
            } else {
                (a / b - 1.0).abs()
            }
        };
        let (dp, da) = (rel(r.pigmento(), pig_velho), rel(r.agua(), agua_velha));
        eprintln!(
            "{:<24} pig {pig_velho:>9.3} → {:>9.3} · água {agua_velha:.5} → {:.5} · carga {:.4}",
            prm.name,
            r.pigmento(),
            r.agua(),
            k.get(Knob::Intensity)
        );
        if dp > TOL || da > TOL {
            falhas.push(format!("{} (pig {dp:.2e}, água {da:.2e})", prm.name));
        }
        n += 1;
    }
    assert_eq!(
        (n, agua_fora, borrachas_fora),
        (46, 10, 1),
        "os 57 presets Wet Paint de fábrica (B36.1 somou o Oil Wash): 46 de tinta, 10 ferramentas da água, 1 Borracha"
    );
    assert!(falhas.is_empty(), "{} mudaram: {falhas:?}", falhas.len());
}

/// Um preset gravado DEPOIS da E4 (traz a marca) não volta a ser migrado: carregar → gravar → carregar é o mesmo.
#[test]
fn um_preset_gravado_na_lei_nova_nao_migra_outra_vez() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let e = bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.media == MEIO_DO_PRESET)
        .expect("um preset Wet Paint");
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.wet_preset_aplicar(&e.preset);
    let antes = t.paint.wetpaint.knobs;
    let (prm, texturas) = t.wet_preset_capturar(&e.preset.parametros);
    assert_eq!(prm.engine.get(CHAVE_RECEITA), Some(&1.0));
    let mut u = PainterTool::default();
    u.set_paint_media(PaintMedia::WetPaint);
    u.wet_preset_aplicar(&Preset {
        parametros: prm,
        texturas,
        ..e.preset.clone()
    });
    for kn in [Knob::PigmentPerDab, Knob::Intensity] {
        let (a, b) = (antes.get(kn), u.paint.wetpaint.knobs.get(kn));
        assert!(
            (a - b).abs() <= 1e-9 * a.abs().max(1.0),
            "{kn:?}: {a} → {b}"
        );
    }
}

/// O Wet Paint SEM preset (`WetKnobs::DEFAULT`, PPD 78 do Wet Round) pousa o mesmo que na lei velha com os
/// sliders de arranque (água 1, opacidade 1): a carga escrita à mão em `painter_default_knobs` é a da migração.
#[test]
fn o_wet_paint_sem_preset_pousa_o_mesmo_que_na_lei_velha() {
    let d = WetKnobs::DEFAULT;
    let s = ph2d_wet_paint::painter::Sliders::default();
    let (pig, carga) = ph2d_wet_paint::painter::receita::migra_da_lei_velha(
        d.get(Knob::PigmentPerDab),
        d.get(Knob::WaterPerDab),
        s.water,
        s.pigmento,
    );
    assert!(
        (pig - d.get(Knob::PigmentPerDab)).abs() < 1e-12,
        "pigmento {pig}"
    );
    assert!(
        (carga - d.get(Knob::Intensity)).abs() < 5e-4,
        "a carga de arranque devia ser {carga:.6}, é {}",
        d.get(Knob::Intensity)
    );
}

/// A FAIXA vermelha da miniatura das ferramentas da água (Blend, Smear, Water Brush, Blur…): os knobs dela eram uma
/// receita da lei velha escrita à mão (PPD 500, Water 0,05) sobre a carga do Wet Round — sem a migração pousava
/// `φ·h ≈ 427` em vez de `500` de pigmento (e a água `0,00075` em vez de `0,00087`): as quatro miniaturas Blender
/// mudaram 24–33 mil px (FECHO_31_10 §3). Pousa o mesmo que na lei velha, a qualquer Opacity.
#[test]
fn a_faixa_das_ferramentas_da_agua_pousa_o_mesmo_que_na_lei_velha() {
    use super::preview::{AGUA_DA_FAIXA, PIGMENTO_DA_FAIXA, knobs_da_faixa};
    for op in [1.0, 0.6, 0.25] {
        let k = knobs_da_faixa(op);
        let r = receita(&Controlos {
            diluicao: k.dilution(),
            volume_um: k.volume_um(),
            meio: k.medium(),
        });
        let (pig, agua) = (
            PIGMENTO_DA_FAIXA * op,
            veiculo(AGUA_DA_FAIXA, k.get(Knob::WaterPerDab)),
        );
        assert!(
            (r.pigmento() / pig - 1.0).abs() < TOL && (r.agua() / agua - 1.0).abs() < TOL,
            "Opacity {op}: pigmento {:.4} (velha {pig:.4}) · água {:.6} (velha {agua:.6})",
            r.pigmento(),
            r.agua()
        );
    }
}
