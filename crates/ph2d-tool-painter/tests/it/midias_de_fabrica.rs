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

/// O ruído do próprio Rebelle (recto × rápido da mesma foto, p90 sobre as 25 fotos, 11/10).
const RUIDO_COBERTURA: f64 = 0.012;
const RUIDO_TEXTURA: f64 = 0.37;

/// Os 8 s da cena + 22 s: seca, como a foto do Rebelle.
const QUADROS_SECOS: u32 = 1800;

/// Uma mídia: o preset, a foto do Rebelle (`docs/Painter/NewFeaturesWetPaint/fixtures/rebelle/`), o
/// raio que dá a largura dela (W2: o número é a largura) e os números da foto (`mede_traco.py`, recto).
struct Midia {
    preset: &'static str,
    foto: &'static str,
    raio: f32,
    cobertura: Option<f64>,
    textura: Option<f64>,
}

const MIDIAS: [Midia; 1] = [Midia {
    preset: "Wet Liner",
    foto: "wcliner",
    raio: 19.0,
    cobertura: Some(0.653),
    textura: Some(1.708),
}];

fn de_fabrica(nome: &str) -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("o preset de fábrica {nome:?}"))
        .preset
}

fn cena_seca(p: &Preset, raio: f32) -> Medida {
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
        std::fs::write(format!("{dir}/midia_{}.ppm", p.parametros.name), f).unwrap();
    }
    mede(&px, w, RECTO)
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c`.
fn no_vale(n: f64, r: f64, c: f64) -> bool {
    (n - r).abs() < 0.5 * (c - r).abs()
}

#[test]
fn cada_preset_parece_a_sua_midia() {
    let round = de_fabrica("Wet Round");
    let mut falhas = Vec::new();
    for m in &MIDIAS {
        let (n, c) = std::thread::scope(|s| {
            let a = s.spawn(|| cena_seca(&de_fabrica(m.preset), m.raio));
            let b = s.spawn(|| cena_seca(&round, m.raio));
            (a.join().unwrap(), b.join().unwrap())
        });
        println!(
            "[midia] {} vs {}: cobertura {:.3} (controlo {:.3}) · textura {:.2} % (controlo {:.2})",
            m.preset, m.foto, n.cobertura, c.cobertura, n.textura, c.textura
        );
        for (nome, r, nv, cv, ruido) in [
            (
                "cobertura",
                m.cobertura,
                n.cobertura,
                c.cobertura,
                RUIDO_COBERTURA,
            ),
            ("textura", m.textura, n.textura, c.textura, RUIDO_TEXTURA),
        ] {
            let Some(r) = r else { continue };
            if !no_vale(nv, r, cv) {
                falhas.push(format!(
                    "{}: {nome} {nv:.3} fora do vale do Rebelle {r} (controlo {cv:.3})",
                    m.preset
                ));
            }
            // O CONTROLO: o Wet Round tem de estar longe da foto (3× o ruído do Rebelle) — senão a
            // métrica não distingue a mídia e o vale aprovaria um Round com outro nome.
            assert!(
                (cv - r).abs() > 3.0 * ruido,
                "{}: o Wet Round dá {nome} {cv:.3}, a foto {} dá {r}: a métrica não distingue a mídia",
                m.preset,
                m.foto
            );
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
