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
    /// A largura do traço DE PÉ (D4–D6) sobre a do DEITADO (A2–F2): o pincel chato (prompt 23).
    Chato,
    /// A tinta mediana no miolo do A2–F2 (0 = papel): a claridade do Express Oil (prompt 23).
    Tinta,
    /// O CRUZAMENTO vermelho × azul (18 §2): a identidade da tinta e do marcador (prompt 42).
    Cruzamento,
}

impl Metrica {
    fn de(self, (m, c): &(Medida, FamiliaC)) -> f64 {
        match self {
            Metrica::Cobertura => m.cobertura,
            Metrica::Textura => m.textura,
            Metrica::Borda => m.borda,
            Metrica::Orla => m.orla,
            Metrica::Chato => c.de_pe / c.deitado,
            Metrica::Tinta => c.tinta,
            Metrica::Cruzamento => c.cruz,
        }
    }
    fn ruido(self) -> f64 {
        match self {
            Metrica::Cobertura => 0.012,
            Metrica::Textura => 0.37,
            Metrica::Borda => 0.031,
            Metrica::Orla => 1.0,
            // Uma foto só tem um traço de pé: o ruído é o de 1 px de largura nos dois (≈ 0,03) e a tinta é
            // lida como a cobertura.
            Metrica::Chato => 0.05,
            Metrica::Tinta => 0.012,
            // O controlo da sessão de 16/10: o Dip Pen de 11/10 refeito deu 0,283 contra 0,223.
            Metrica::Cruzamento => 0.06,
        }
    }
}

/// A régua da família C (`ferramentas/mede_familia_c.py`, as mesmas janelas): a largura do traço vermelho DE
/// PÉ em D4 e D6 (fora do cruzamento), a do azul DEITADO A2–F2 e a tinta mediana no miolo dele.
#[derive(Debug)]
struct FamiliaC {
    de_pe: f64,
    deitado: f64,
    tinta: f64,
    cruz: f64,
}

impl FamiliaC {
    fn de(px: &[u8], w: u32) -> Self {
        let w = w as usize;
        let q = |x: usize, y: usize| &px[(y * w + x) * 4..(y * w + x) * 4 + 3];
        // O canal que a tinta mais apaga, e só a tinta da cor certa.
        let azul = |x, y| {
            let c = q(x, y);
            if c[2] < c[0] {
                0.0
            } else {
                1.0 - f64::from(c[0]) / 255.0
            }
        };
        let verm = |x, y| {
            let c = q(x, y);
            if c[0] < c[2] {
                0.0
            } else {
                1.0 - f64::from(c[1]) / 255.0
            }
        };
        // Px acima de metade do pico (o p95 do perfil).
        let acima = |perfil: &[f64]| {
            let mut o = perfil.to_vec();
            o.sort_by(f64::total_cmp);
            let pico = o[((o.len() - 1) as f64 * 0.95).round() as usize];
            perfil.iter().filter(|&&v| v > 0.5 * pico).count() as f64
        };
        let deitado: Vec<f64> = (80..160)
            .map(|y| (150..500).map(|x| azul(x, y)).sum::<f64>() / 350.0)
            .collect();
        let de_pe = [(395_usize, 445_usize), (515, 565)]
            .iter()
            .map(|&(y0, y1)| {
                let p: Vec<f64> = (240..360)
                    .map(|x| (y0..y1).map(|y| verm(x, y)).sum::<f64>() / (y1 - y0) as f64)
                    .collect();
                acima(&p)
            })
            .sum::<f64>()
            / 2.0;
        let mut o = deitado.clone();
        o.sort_by(f64::total_cmp);
        let pico = o[((o.len() - 1) as f64 * 0.95).round() as usize];
        let mut miolo: Vec<f64> = deitado
            .iter()
            .copied()
            .filter(|&v| v > 0.15 * pico)
            .collect();
        miolo.sort_by(f64::total_cmp);
        Self {
            de_pe,
            deitado: acima(&deitado),
            tinta: miolo[miolo.len() / 2],
            cruz: super::a_lei_da_tinta::cruzamento(px, w as u32),
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

use Metrica::{Borda, Chato, Cobertura, Cruzamento, Orla, Textura, Tinta};

// ⚠️ Os três do óleo levam os valores que o DONO afinou a olho contra o Rebelle (11/10, «vamos deixar esses
// valores»); a régua reprovava Textura do Oil Bristle (8,46 contra 5,86) e da Acrylic Round (30,4 contra 3,13)
// e a Borda do Express Oil (0,66 contra 0,80) — saíram daqui, a régua não mede o que o olho dele vê (18 §2).
// O Chato e a Tinta (prompt 23, aceites 12/10) são os dois pontos que o dono VIU: a mutação «preset de antes»
// sangra cada um sozinho (Oil 0,950 · Express 0,906).
#[rustfmt::skip]
const MIDIAS: [Midia; 13] = [
    Midia { preset: "Wet Liner", foto: "wcliner", raio: 19.0, metricas: &[(Cobertura, 0.653), (Textura, 1.708)] },
    Midia { preset: "Oil Bristle", foto: "oil", raio: 20.5, metricas: &[(Cobertura, 0.909), (Borda, 0.872), (Chato, 0.317)] },
    Midia { preset: "Acrylic Round", foto: "oilround", raio: 19.5, metricas: &[(Cobertura, 0.880), (Borda, 0.595)] },
    Midia { preset: "Express Oil", foto: "express", raio: 21.0, metricas: &[(Cobertura, 0.700), (Tinta, 0.772)] },
    Midia { preset: "Graphite 8B", foto: "pencil", raio: 15.0, metricas: &[(Cobertura, 0.798), (Textura, 9.093), (Borda, 0.320), (Orla, 7.0)] },
    Midia { preset: "Charcoal", foto: "charcoal", raio: 10.5, metricas: &[(Cobertura, 0.384), (Borda, 0.378), (Orla, 15.0)] },
    Midia { preset: "Soft Pastel", foto: "pastel", raio: 8.5, metricas: &[(Cobertura, 0.752), (Borda, 0.443), (Orla, 6.0)] },
    Midia { preset: "Oil Pastel", foto: "oilpastel2", raio: 15.0, metricas: &[(Cobertura, 0.654), (Textura, 17.354), (Borda, 0.417), (Orla, 11.0)] },
    // A tinta e o marcador (prompt 42): o Cruzamento é o da foto MOLHADA; a Borda e a Orla só onde o Round
    // não as tem (Ink Bullet: borda 1,00 nos dois; Orla 1 nos marcadores e nas canetas).
    Midia { preset: "Ink Bullet", foto: "ink", raio: 21.0, metricas: &[(Cobertura, 0.862), (Textura, 0.395), (Cruzamento, 0.117)] },
    Midia { preset: "Dip Pen", foto: "dippen", raio: 19.0, metricas: &[(Cobertura, 0.856), (Textura, 1.433), (Borda, 0.910), (Cruzamento, 0.223)] },
    Midia { preset: "Ink Brush", foto: "inkbrush", raio: 20.0, metricas: &[(Cobertura, 0.722), (Textura, 1.266), (Borda, 0.793), (Orla, 5.0), (Cruzamento, 0.239)] },
    Midia { preset: "Marker Bullet", foto: "markerbullet", raio: 20.0, metricas: &[(Cobertura, 0.909), (Textura, 0.658), (Borda, 0.768), (Cruzamento, 0.016)] },
    Midia { preset: "Marker Liner", foto: "marker", raio: 9.0, metricas: &[(Cobertura, 0.910), (Textura, 2.116), (Borda, 0.716), (Cruzamento, 0.016)] },
];

pub(super) fn de_fabrica(nome: &str) -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("o preset de fábrica {nome:?}"))
        .preset
}

fn cena_seca(p: &Preset, raio: f32, ficheiros: &[String]) -> (Medida, FamiliaC) {
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
    (mede(&px, w, RECTO), FamiliaC::de(&px, w))
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c`.
fn no_vale(n: f64, r: f64, c: f64) -> bool {
    (n - r).abs() < 0.5 * (c - r).abs()
}

#[test]
fn cada_preset_parece_a_sua_midia() {
    let round = &de_fabrica("Wet Round");
    // Todas as cenas de uma vez: as mídias e o controlo UMA vez por largura (o Wet Round não depende da
    // mídia). Medido 11/10: 57 → 46 s.
    let mut raios: Vec<f32> = MIDIAS.iter().map(|m| m.raio).collect();
    raios.sort_by(f32::total_cmp);
    raios.dedup();
    let (medidas, controlos): (Vec<_>, Vec<_>) = std::thread::scope(|s| {
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
        let junta = |h: Vec<std::thread::ScopedJoinHandle<'_, (Medida, FamiliaC)>>| {
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
            // A Tinta é uma FAIXA dos dois lados do Rebelle: o controlo (o Round, aquarela: 0,18) fica do lado
            // CLARO e o defeito medido do Express Oil era o opaco (0,91) — o vale aprovava-o (prompt 23).
            // O Cruzamento também: o vale (meio caminho do controlo, 0,96) aprovaria 0,49 para um marcador de
            // 0,016 — a faixa de 3× o ruído do Rebelle é o que o distingue da tinta.
            let dentro = match met {
                Tinta => (nv - r).abs() < 4.0 * met.ruido(),
                Cruzamento => (nv - r).abs() < 3.0 * met.ruido(),
                _ => no_vale(nv, r, cv),
            };
            if !dentro {
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
