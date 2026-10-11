//! **Cada preset de mídia parece a SUA mídia** (prompt 17, Fase C; 05 §4b «Presets de todas as mídias»).
//!
//! A cena da bancada (`sonda_gestos::pinta_cena_ate`) com o preset à largura da foto do Rebelle da mesma
//! mídia, SECA (os 8 s da cena + 22 s: o Rebelle é fotografado depois do `DRY`), medida pela régua do
//! Python portada (`regua_midia`) no recto. A BARRA de cada métrica é o VALE (05 §3): o preset tem de
//! ficar do lado do Rebelle do ponto médio entre o Rebelle e o CONTROLO — o Wet Round à mesma largura,
//! medido aqui, que tem de ficar FORA (sem isso o portão aprovaria um Round com outro nome).
//!
//! Só entram as métricas que DISTINGUEM a mídia do Round acima do ruído do próprio Rebelle (a diferença
//! entre o recto e o rápido da mesma foto, p90 sobre as 25 fotos: cobertura 0,012 · textura 0,37 % ·
//! borda/miolo 0,031) — a borda do Liner (1,05 contra 1,00) não distingue e não conta.

use super::regua_midia::{Medida, RECTO, mede};
use super::sonda_gestos::{pinta_cena_ate, tela};
use ph2d_brush_presets::{Biblioteca, Preset};
use std::path::PathBuf;

/// Uma métrica da régua e o ruído do próprio Rebelle nela (recto × rápido da mesma foto, p90 sobre as 25
/// fotos, 11/10): só conta numa mídia se o controlo estiver a mais de 3× este ruído da foto.
#[derive(Clone, Copy, Debug)]
enum Metrica {
    Cobertura,
    Textura,
    Borda,
    Orla,
}

impl Metrica {
    fn de(self, m: &Medida) -> f64 {
        match self {
            Metrica::Cobertura => m.cobertura,
            Metrica::Textura => m.textura,
            Metrica::Borda => m.borda,
            Metrica::Orla => m.orla,
        }
    }
    fn ruido(self) -> f64 {
        match self {
            Metrica::Cobertura => 0.012,
            Metrica::Textura => 0.37,
            Metrica::Borda => 0.031,
            Metrica::Orla => 1.0,
        }
    }
}

/// Os 8 s da cena + 22 s: seca, como a foto do Rebelle.
const QUADROS_SECOS: u32 = 1800;

/// Uma mídia: o preset, a foto do Rebelle (`docs/Painter/NewFeaturesWetPaint/fixtures/rebelle/`), o
/// raio que dá a largura dela (W2: o número é a largura) e os números da foto (`mede_traco.py`, recto) nas
/// métricas que DISTINGUEM a mídia do Round (18 §3).
struct Midia {
    preset: &'static str,
    foto: &'static str,
    raio: f32,
    metricas: &'static [(Metrica, f64)],
}

use Metrica::{Borda, Cobertura, Orla, Textura};

#[rustfmt::skip]
const MIDIAS: [Midia; 8] = [
    Midia { preset: "Wet Liner", foto: "wcliner", raio: 19.0, metricas: &[(Cobertura, 0.653), (Textura, 1.708)] },
    Midia { preset: "Oil Bristle", foto: "oil", raio: 20.5, metricas: &[(Cobertura, 0.909), (Textura, 5.861), (Borda, 0.872)] },
    Midia { preset: "Acrylic Round", foto: "oilround", raio: 19.5, metricas: &[(Cobertura, 0.880), (Textura, 3.133), (Borda, 0.595)] },
    Midia { preset: "Express Oil", foto: "express", raio: 21.0, metricas: &[(Cobertura, 0.700), (Borda, 0.796)] },
    Midia { preset: "Graphite 8B", foto: "pencil", raio: 15.0, metricas: &[(Cobertura, 0.798), (Textura, 9.093), (Borda, 0.320), (Orla, 7.0)] },
    Midia { preset: "Charcoal", foto: "charcoal", raio: 10.5, metricas: &[(Cobertura, 0.384), (Borda, 0.378), (Orla, 15.0)] },
    Midia { preset: "Soft Pastel", foto: "pastel", raio: 8.5, metricas: &[(Cobertura, 0.752), (Borda, 0.443), (Orla, 6.0)] },
    Midia { preset: "Oil Pastel", foto: "oilpastel2", raio: 15.0, metricas: &[(Cobertura, 0.654), (Textura, 17.354), (Borda, 0.417), (Orla, 11.0)] },
];

fn de_fabrica(nome: &str) -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("o preset de fábrica {nome:?}"))
        .preset
}

fn cena_seca(p: &Preset, raio: f32, ficheiros: &[String]) -> Medida {
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    let (px, w, h) = pinta_cena_ate(&mut t, QUADROS_SECOS);
    if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
        let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
        px.as_chunks::<4>()
            .0
            .iter()
            .for_each(|q| f.extend_from_slice(&q[..3]));
        std::fs::create_dir_all(&dir).unwrap();
        for nome in ficheiros {
            std::fs::write(format!("{dir}/midia_{nome}.ppm"), &f).unwrap();
        }
    }
    mede(&px, w, RECTO)
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c`.
fn no_vale(n: f64, r: f64, c: f64) -> bool {
    (n - r).abs() < 0.5 * (c - r).abs()
}

#[test]
fn cada_preset_parece_a_sua_midia() {
    let round = &de_fabrica("Wet Round");
    // Todas as cenas de uma vez: as 8 mídias e o controlo UMA vez por largura (o Wet Round não depende da
    // mídia). Medido 11/10: 57 → 46 s.
    let mut raios: Vec<f32> = MIDIAS.iter().map(|m| m.raio).collect();
    raios.sort_by(f32::total_cmp);
    raios.dedup();
    let (medidas, controlos): (Vec<Medida>, Vec<Medida>) = std::thread::scope(|s| {
        let a: Vec<_> = MIDIAS
            .iter()
            .map(|m| s.spawn(|| cena_seca(&de_fabrica(m.preset), m.raio, &[m.preset.to_owned()])))
            .collect();
        let b: Vec<_> = raios
            .iter()
            .map(|&r| {
                let nomes: Vec<String> = MIDIAS
                    .iter()
                    .filter(|m| m.raio == r)
                    .map(|m| format!("{}_controlo", m.preset))
                    .collect();
                s.spawn(move || cena_seca(round, r, &nomes))
            })
            .collect();
        let junta = |h: Vec<std::thread::ScopedJoinHandle<'_, Medida>>| {
            h.into_iter().map(|h| h.join().unwrap()).collect()
        };
        (junta(a), junta(b))
    });
    let mut falhas = Vec::new();
    for (m, n) in MIDIAS.iter().zip(&medidas) {
        let c = &controlos[raios.iter().position(|&r| r == m.raio).unwrap()];
        println!(
            "[midia] {} vs {}: {:?} (controlo {:?})",
            m.preset, m.foto, n, c
        );
        for &(met, r) in m.metricas {
            let (nv, cv) = (met.de(n), met.de(c));
            if !no_vale(nv, r, cv) {
                falhas.push(format!(
                    "{}: {met:?} {nv:.3} fora do vale do Rebelle {r} (controlo {cv:.3})",
                    m.preset
                ));
            }
            // O CONTROLO: o Wet Round tem de estar longe da foto (3× o ruído do Rebelle) — senão a
            // métrica não distingue a mídia e o vale aprovaria um Round com outro nome.
            assert!(
                (cv - r).abs() > 3.0 * met.ruido(),
                "{}: o Wet Round dá {met:?} {cv:.3}, a foto {} dá {r}: a métrica não distingue a mídia",
                m.preset,
                m.foto
            );
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
