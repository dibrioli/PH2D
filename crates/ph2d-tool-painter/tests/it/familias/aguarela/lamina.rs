//! **A lâmina de cada preset é a do oráculo** (J6, L3 — `docs/Painter/Pinceis/J6_lamina.md`). A medição refutou a
//! ponta que gira com inércia (06 §1, §10.1–10.2): os presets chatos do Rebelle são FIXOS na tela, cada um no seu
//! eixo. A lei, preset a preset: a largura do recto vertical ÷ a do horizontal = a do oráculo (os mesmos rectos da
//! bateria `gestos`: `leis_lacunas.py lamina`) e, onde o ajuste da lâmina fixa é justo (rms ≤ 1,3 px), também os
//! rectos a ±40° = os da lâmina ajustada (`w(θ) = L|sin(θ−φ)| + T|cos(θ−φ)|`). O controlo por dentro: a mesma ponta a SEGUIR o traço (`dabRotation` 1) tem
//! de reprovar — a largura dela não depende da direcção.

use crate::exame_bancada::{Fonte, Pincel, pinceis, pinta_cena};
use serde_json::{Value, json};

/// `(pincel, oráculo, tamanho px, w90 ÷ w0 lido, [w+40, w−40] ÷ w0 da lâmina ajustada)` (`J6_lamina.md` §1). O
/// tamanho é o nominal do oráculo. Fora: o Compressed Charcoal — o halo de pó soma ~10 px de cada lado em qualquer
/// direcção, então w90/w0 depende da escala (2,50 a 16 px, 1,50 a 24, 1,52 a 40; a SEGUIR o traço 1,6–1,8): mede-o a
/// régua do EXAME à escala do oráculo (`J6_lamina.md` §2).
type Lamina = (&'static str, &'static str, u32, f64, Option<[f64; 2]>);
const ORACULO: &[Lamina] = &[
    ("Flat Block-in", "Rebelle Digital/Flat", 40, 0.436, Some([0.907, 0.986])),
    ("Craft Flat", "Rebelle Graphite Pencil/Craft Flat", 40, 1.429, None),
    ("Broad Nib", "MyPaint classic/calligraphy", 40, 0.944, Some([0.241, 1.291])),
    ("Bristle Watercolor", "Rebelle Watercolor/Bristle Flat", 40, 0.348, None),
];
/// A folga relativa de cada razão: o ruído do oráculo (recto lento × rápido) é 0–1 px em 15–46 px, e o ajuste da
/// lâmina fixa tem rms 0,7–2,9 px.
const FOLGA: f64 = 0.15;

/// Os quatro rectos (0°, 90°, +40°, −40°, ângulos da imagem com y para baixo): `(início, fim)`.
const RECTOS: [([f64; 2], [f64; 2]); 4] = [
    ([100.0, 80.0], [500.0, 80.0]),
    ([850.0, 150.0], [850.0, 550.0]),
    ([100.0, 250.0], [330.0, 443.0]),
    ([450.0, 600.0], [680.0, 407.0]),
];

fn pincel(nome: &str, rotacao: Option<f64>) -> Pincel {
    let lapis = crate::familias::lapis::EM_ESTUDO.iter().find(|r| r.nome == nome);
    let wet = super::EM_ESTUDO
        .iter()
        .chain(crate::familias::pastel_carvao::EM_ESTUDO)
        .chain(crate::familias::caligrafia_marcadores::EM_ESTUDO_WET)
        .find(|r| r.nome == nome);
    let mut preset = match (lapis, wet) {
        (Some(r), _) => crate::presets_de_fabrica::preset_do_meio(r),
        (None, Some(r)) => ph2d_brush_presets::Preset {
            parametros: crate::presets_de_fabrica::parametros(r),
            origem: ph2d_brush_presets::Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas: Default::default(),
        },
        (None, None) => {
            let Fonte::Preset { preset, .. } = pinceis(nome).remove(0).fonte else {
                unreachable!()
            };
            *preset
        }
    };
    if let Some(v) = rotacao {
        preset.parametros.engine.insert("dabRotation".to_owned(), v);
    }
    Pincel {
        nome: rotacao.map_or_else(|| nome.to_owned(), |v| format!("{nome} rot{v}")),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: nome.to_owned(),
        },
    }
}

fn cena(tamanho: u32) -> Value {
    let mut ops = vec![
        json!({"op": "pincel", "quem": "preset"}),
        json!({"op": "cor", "rgb": [23, 101, 179]}),
    ];
    for (k, (a, b)) in RECTOS.iter().enumerate() {
        let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as usize;
        let pts: Vec<Value> = (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                json!([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, 1.0])
            })
            .collect();
        ops.push(json!({"op": "traco", "nome": format!("r{k}"), "size_px": tamanho, "passo": 3.0, "pts": pts}));
    }
    ops.push(json!({"op": "foto", "nome": "A"}));
    json!({"nome": "lamina", "ops": ops})
}

/// A largura mediana de cada recto: a perpendicular no miolo (30–70 % do comprimento), amostrada a 0,5 px; conta-se o
/// que passa ¼ do NÍVEL do traço (o p90 dos picos) — a régua `leis_lacunas.py::larguras`.
fn larguras(p: &Pincel, tamanho: u32) -> [f64; 4] {
    let px = pinta_cena(p, None, &cena(tamanho), [1000, 640], [255; 3], 0).1;
    if let Ok(d) = std::env::var("PH2D_LAMINA_FOTOS") {
        let f = std::path::Path::new(&d).join(format!("{}.ppm", p.nome.replace(' ', "_")));
        let mut ppm = b"P6\n1000 640\n255\n".to_vec();
        ppm.extend(px.as_chunks::<4>().0.iter().flat_map(|c| [c[0], c[1], c[2]]));
        std::fs::write(f, ppm).unwrap();
    }
    let cob = |x: f64, y: f64| -> f64 {
        let (xi, yi) = (x.round() as i64, y.round() as i64);
        if !(0..1000).contains(&xi) || !(0..640).contains(&yi) {
            return 0.0;
        }
        let i = (yi as usize * 1000 + xi as usize) * 4;
        (0..3).map(|k| 255.0 - f64::from(px[i + k])).fold(0.0, f64::max) / 255.0
    };
    RECTOS.map(|(a, b)| {
        let l = (b[0] - a[0]).hypot(b[1] - a[1]);
        let (t, n) = ([(b[0] - a[0]) / l, (b[1] - a[1]) / l], [-(b[1] - a[1]) / l, (b[0] - a[0]) / l]);
        let perfis: Vec<Vec<f64>> = (0..=40)
            .map(|i| {
                let s = l * (0.3 + 0.4 * f64::from(i) / 40.0);
                let c = [a[0] + t[0] * s, a[1] + t[1] * s];
                (-140..=140)
                    .map(|j| {
                        let d = f64::from(j) * 0.5;
                        cob(c[0] + n[0] * d, c[1] + n[1] * d)
                    })
                    .collect()
            })
            .collect();
        let mut picos: Vec<f64> = perfis.iter().map(|v| v.iter().copied().fold(0.0, f64::max)).collect();
        picos.sort_by(f64::total_cmp);
        let lim = (0.25 * picos[(picos.len() - 1) * 9 / 10]).max(1e-3);
        let mut w: Vec<f64> = perfis
            .iter()
            .map(|v| {
                let dentro: Vec<usize> = (0..v.len()).filter(|&j| v[j] >= lim).collect();
                dentro.last().map_or(0.0, |u| (u - dentro[0] + 1) as f64 * 0.5)
            })
            .collect();
        w.sort_by(f64::total_cmp);
        w[w.len() / 2]
    })
}

fn falhas(nome: &str, w: [f64; 4], l: &Lamina) -> Vec<String> {
    let mut alvo = vec![("w90", w[1] / w[0], l.3)];
    if let Some([a, b]) = l.4 {
        alvo.extend([("w+40", w[2] / w[0], a), ("w-40", w[3] / w[0], b)]);
    }
    alvo.into_iter()
        .filter(|(_, v, a)| ((v - a) / a).abs() > FOLGA)
        .map(|(k, v, a)| format!("{nome} {k}/w0 {v:.3} (oráculo {a:.3})"))
        .collect()
}

/// ⭐⭐ **A lâmina de cada preset chato é a do oráculo, FIXA na tela** (L3, decisão 1 do `ESTADO.md`).
#[test]
fn a_lamina_de_cada_preset_e_a_do_oraculo() {
    let medidas: Vec<([f64; 4], [f64; 4])> = std::thread::scope(|s| {
        let hs: Vec<_> = ORACULO
            .iter()
            .map(|&(nome, _, t, ..)| {
                s.spawn(move || (larguras(&pincel(nome, None), t), larguras(&pincel(nome, Some(1.0)), t)))
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut erros = Vec::new();
    for (l, (w, seg)) in ORACULO.iter().zip(&medidas) {
        let r = |w: &[f64; 4]| [w[1] / w[0], w[2] / w[0], w[3] / w[0]];
        println!(
            "[lamina] {:<20} w {w:.1?} → {:.3?} ({} {:.3} {:.3?}) · a seguir o traço {:.3?}",
            l.0,
            r(w),
            l.1,
            l.3,
            l.4,
            r(seg)
        );
        erros.extend(falhas(l.0, *w, l));
        assert!(
            !falhas(l.0, *seg, l).is_empty(),
            "{}: a ponta que SEGUE o traço também passa {:.3?} — o portão não mede a lâmina fixa",
            l.0,
            r(seg)
        );
    }
    assert!(erros.is_empty(), "lâminas fora do oráculo: {erros:#?}");
}
