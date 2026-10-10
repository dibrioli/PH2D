//! **Sonda (ignorada, W1g, O3): os fios finos e compridos do Rebelle saem da COMPOSIÇÃO?** Antes de
//! construir uma lei, mede-se se os knobs que o motor já tem (fingering, rivulets, nivelamento,
//! viscosidade, capilaridade, freio, velocidade) dão ~10 fios de 3–7 px e 250–460 px no S com a tela no
//! máximo (o Rebelle, `ferramentas/fios_s.py`) sem fio a mais a meia e sem rectângulos. Uma rodada, no
//! MESMO processo; grava `f_<variante>_<anel>.ppm` em `PH2D_SONDA_OUT`.
use super::sonda_achatamento::{Variante, cena};
use ph2d_wet_paint::tuning::Knob;

fn variantes() -> Vec<(&'static str, Vec<(Knob, f64)>)> {
    use Knob::*;
    vec![
        ("base", vec![]),
        ("fing03", vec![(ExtFingering, 0.3)]),
        ("fing10", vec![(ExtFingering, 1.0)]),
        ("fing10_riv8", vec![(ExtFingering, 1.0), (ExtRivulets, 8.0)]),
        (
            "fing10_riv40",
            vec![(ExtFingering, 1.0), (ExtRivulets, 40.0)],
        ),
        ("nivel02", vec![(Leveling, 0.2)]),
        ("nivel01", vec![(Leveling, 0.1)]),
        ("clamp005", vec![(LevelClamp, 0.05)]),
        ("visc005", vec![(Viscosity, 0.05)]),
        ("capil06", vec![(Capillary, 0.6)]),
        ("vmax2", vec![(MaxVelocity, 2.0)]),
        ("freio05", vec![(Brake, 0.5)]),
        (
            "fing10_riv10_nivel02",
            vec![(ExtFingering, 1.0), (ExtRivulets, 10.0), (Leveling, 0.2)],
        ),
        ("nivel01_visc005", vec![(Leveling, 0.1), (Viscosity, 0.05)]),
    ]
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_fios_finos_pela_composicao() {
    let dir = std::env::var("PH2D_SONDA_OUT").ok();
    let vs = variantes();
    let linhas: Vec<String> = std::thread::scope(|sc| {
        let fios: Vec<_> = vs
            .iter()
            .flat_map(|(nome, knobs)| [(nome, knobs, 4u8), (nome, knobs, 8u8)])
            .map(|(nome, knobs, anel)| {
                let dir = dir.clone();
                sc.spawn(move || {
                    let v = Variante {
                        knobs: knobs.clone(),
                        ..Variante::default()
                    };
                    let px = cena(&v, Some((3, anel)), 30, &mut |_, _| {}).0;
                    if let Some(d) = dir {
                        let mut out = format!(
                            "P6\n{} {}\n255\n",
                            super::sonda_achatamento::W,
                            super::sonda_achatamento::H
                        )
                        .into_bytes();
                        for p in px.chunks(4) {
                            out.extend_from_slice(&p[..3]);
                        }
                        let _ = std::fs::write(format!("{d}/f_{nome}_{anel}.ppm"), out);
                    }
                    format!(
                        "[fios] {nome} anel {anel}: recto {:?}",
                        super::achatamento_tests::le(&px)
                    )
                })
            })
            .collect();
        fios.into_iter()
            .map(|f| f.join().expect("variante"))
            .collect()
    });
    for l in linhas {
        println!("{l}");
    }
}
