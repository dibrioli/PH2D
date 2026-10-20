//! **As LEIS SISTÉMICAS** (JANELA 2 da missão Pincéis, `docs/Painter/Pinceis/03_MOTORES.md` §7): o que quase todo
//! pincel de fábrica fazia diferente do oráculo, medido nas fotos da bateria do Rebelle
//! (`docs/Painter/Pinceis/fixtures/leis_rebelle.json`, gerado por `ferramentas/leis_oraculo.py oraculo` com a régua
//! do EXAME) e pintado aqui pelos NOSSOS presets com os MESMOS pontos (`bateria.json`, a fonte única).
//!
//! A régua é a do `exame_regua.py` reduzida ao traço HORIZONTAL (todos os destas leis o são): em cada coluna a
//! cobertura `máx |px − papel| / 255` na normal (±H), a largura a ¼ do máximo da coluna e o miolo (|o − centro| ≤
//! 0,35 · largura).

use super::exame_bancada::{Pincel, bateria, pinceis, pinta_cena};
use serde_json::{Value, json};
use std::path::PathBuf;

fn oraculo() -> Value {
    let f = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/fixtures/leis_rebelle.json");
    serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap()
}

/// A cena `cena` da bateria só com os traços `nomes` (+ 240 quadros de água e a foto molhada), pintada por `p`.
fn pinta(p: &Pincel, cena: &str, nomes: &[&str]) -> (Vec<u8>, Vec<Value>) {
    let bat = bateria();
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap();
    let tracos: Vec<Value> = c["ops"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["op"] == "traco" && nomes.contains(&o["nome"].as_str().unwrap()))
        .cloned()
        .collect();
    let mut ops = vec![json!({"op": "pincel", "quem": "preset"}), json!({"op": "cor", "rgb": [23, 101, 179]})];
    ops.extend(tracos.iter().cloned());
    ops.extend([json!({"op": "simula", "quadros": 240}), json!({"op": "foto", "nome": "A"})]);
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let (_, px) = pinta_cena(p, None, &json!({"nome": cena, "ops": ops}), tela, [255; 3], 0);
    (px, tracos)
}

/// Uma coluna do traço horizontal: a cobertura na normal (±h), a largura a ¼ do máximo dela e o centro (o do meio
/// da janela quando a coluna está vazia, `np.nan_to_num` do Python).
fn coluna(px: &[u8], w: usize, x: usize, y: f64, h: f64) -> (Vec<f64>, f64, f64) {
    let (y0, y1) = ((y - h).max(0.0) as usize, (y + h) as usize);
    let c: Vec<f64> = (y0..=y1)
        .map(|yy| {
            let i = (yy * w + x) * 4;
            (0..3).map(|k| 255.0 - f64::from(px[i + k])).fold(0.0, f64::max) / 255.0
        })
        .collect();
    let mx = c.iter().copied().fold(0.0, f64::max);
    let meio = (c.len() / 2) as f64;
    if mx < 0.08 {
        return (c, 0.0, meio);
    }
    let dentro: Vec<usize> = (0..c.len()).filter(|&i| c[i] >= 0.25 * mx).collect();
    let larg = (dentro[dentro.len() - 1] - dentro[0] + 1) as f64;
    let soma: f64 = c.iter().sum();
    let cen = c.iter().enumerate().map(|(i, v)| i as f64 * v).sum::<f64>() / soma.max(1e-9);
    (c, larg, cen)
}

/// O traço `t` da bateria (horizontal): por coluna a pressão, a largura e a cobertura do miolo — a banda de ±0,35
/// da largura MEDIANA do traço em volta do centro da coluna (a do `exame_regua.py`: numa pressão baixa o miolo é
/// a mesma banda, e o papel que nela aparece é o esmaecer).
fn colunas(px: &[u8], w: usize, t: &Value) -> Vec<(f64, f64, f64)> {
    let pts: Vec<[f64; 3]> = t["pts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| [0, 1, 2].map(|i| p[i].as_f64().unwrap()))
        .collect();
    let (x0, x1, y) = (pts[0][0], pts[pts.len() - 1][0], pts[0][1]);
    let h = t["size_px"].as_f64().unwrap().max(6.0);
    let pressao = |x: f64| {
        let k = pts.partition_point(|p| p[0] < x).clamp(1, pts.len() - 1);
        let (a, b) = (pts[k - 1], pts[k]);
        let f = ((x - a[0]) / (b[0] - a[0]).max(1e-9)).clamp(0.0, 1.0);
        a[2] + (b[2] - a[2]) * f
    };
    let cols: Vec<(f64, Vec<f64>, f64, f64)> = (x0 as usize..=x1 as usize)
        .map(|x| {
            let (c, l, cen) = coluna(px, w, x, y, h);
            (pressao(x as f64), c, l, cen)
        })
        .collect();
    let mut ls: Vec<f64> = cols.iter().map(|c| c.2).collect();
    ls.sort_by(f64::total_cmp);
    let banda = 0.35 * ls[ls.len() / 2];
    cols.into_iter()
        .map(|(p, c, l, cen)| {
            let miolo: Vec<f64> = (0..c.len())
                .filter(|&i| (i as f64 - cen).abs() <= banda)
                .map(|i| c[i])
                .collect();
            (p, l, miolo.iter().sum::<f64>() / miolo.len().max(1) as f64)
        })
        .collect()
}

/// A cobertura do miolo do traço (as colunas de 10 % a 90 % do comprimento).
fn cobertura(cols: &[(f64, f64, f64)]) -> f64 {
    let n = cols.len();
    let v: Vec<f64> = cols[n / 10..n * 9 / 10].iter().map(|c| c.2).collect();
    v.iter().sum::<f64>() / v.len() as f64
}

/// Os pincéis de fábrica que têm origem no Rebelle (o fixture), com o nome.
fn com_origem(o: &Value) -> Vec<Pincel> {
    pinceis("todos")
        .into_iter()
        .filter(|p| o["leis"].get(&p.nome).is_some())
        .collect()
}

/// ⭐ **(b) O traço FINO pesa por área o que o largo pesa.** O recto de 5 px de cada preset tem de cobrir, em
/// fracção do recto de 40 px do MESMO pincel, o que o Rebelle cobre (`razao_5_40`, mediana 0,93) menos 0,20. Antes
/// da cura: 0,19–0,54 nos de grão e de água (o 8B 0,47 contra 0,93; o passo do caminhador preso numa célula abaixo
/// de 40 px de diâmetro pousava `d ÷ 40` do pigmento por área — `wetpaint/lote.rs::peso_do_passo_fino`).
#[test]
fn o_traco_fino_pesa_por_area_o_que_o_largo_pesa() {
    let o = oraculo();
    let todos = com_origem(&o);
    let medidas: Vec<(String, f64, f64)> = std::thread::scope(|s| {
        let hs: Vec<_> = todos
            .iter()
            .map(|p| {
                s.spawn(move || {
                    let (px, ts) = pinta(p, "tamanhos", &["recto_5", "recto_40"]);
                    let c: Vec<f64> = ts.iter().map(|t| cobertura(&colunas(&px, 1000, t))).collect();
                    (p.nome.clone(), c[0], c[1])
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (nome, c5, c40) in &medidas {
        let alvo = o["leis"][nome]["fino"]["razao_5_40"].as_f64().unwrap();
        let r = c5 / c40.max(1e-6);
        println!("[fino] {nome}: 5 px {c5:.3} · 40 px {c40:.3} · razão {r:.2} (Rebelle {alvo:.2})");
        if r < alvo.min(1.0) - 0.20 {
            falhas.push(format!("{nome}: o recto de 5 px cobre {r:.2} do de 40 px (o Rebelle {alvo:.2})"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
