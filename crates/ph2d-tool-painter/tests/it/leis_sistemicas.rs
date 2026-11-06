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
    pinta_com(p, cena, nomes, false)
}

/// [`pinta`]; `sem_pressao`: a caneta pinta os traços com a pressão PRESA a 1 (os traços devolvidos guardam a
/// rampa nominal, a que a régua lê) — o controlo da lei da mão.
fn pinta_com(p: &Pincel, cena: &str, nomes: &[&str], sem_pressao: bool) -> (Vec<u8>, Vec<Value>) {
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
    let mut ops = vec![
        json!({"op": "pincel", "quem": "preset"}),
        json!({"op": "cor", "rgb": [23, 101, 179]}),
    ];
    ops.extend(tracos.iter().map(|t| {
        let mut t = t.clone();
        if sem_pressao {
            for pt in t["pts"].as_array_mut().unwrap() {
                pt[2] = json!(1.0);
            }
        }
        t
    }));
    ops.extend([
        json!({"op": "simula", "quadros": 240}),
        json!({"op": "foto", "nome": "A"}),
    ]);
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    // As leis sistémicas são do pigmento: sem o Real Oil dos `Oil/*` (J6, triagem (a)).
    let (_, px) = super::exame_bancada::pinta_cena_oleo(
        p,
        None,
        &json!({"nome": cena, "ops": ops}),
        tela,
        [255; 3],
        0,
        false,
        super::exame_bancada::Oleo::Desligado,
        &mut super::exame_bancada::Fases::default(),
    );
    (px, tracos)
}

/// Uma coluna do traço horizontal: a cobertura na normal (±h), a largura a ¼ do máximo dela e o centro (o do meio
/// da janela quando a coluna está vazia, `np.nan_to_num` do Python): vazia é a coluna cujo máximo fica abaixo de `piso`.
fn coluna_piso(px: &[u8], w: usize, x: usize, y: f64, h: f64, piso: f64) -> (Vec<f64>, f64, f64) {
    let (y0, y1) = ((y - h).max(0.0) as usize, (y + h) as usize);
    let c: Vec<f64> = (y0..=y1)
        .map(|yy| {
            let i = (yy * w + x) * 4;
            (0..3)
                .map(|k| 255.0 - f64::from(px[i + k]))
                .fold(0.0, f64::max)
                / 255.0
        })
        .collect();
    let mx = c.iter().copied().fold(0.0, f64::max);
    let meio = (c.len() / 2) as f64;
    if mx < piso {
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
    colunas_piso(px, w, t, None)
}

/// [`colunas`] com o piso da coluna RELATIVO ao topo do próprio traço (`rel` × a mediana do máximo das colunas a
/// pressão ≥ 0,9); `None`: o piso absoluto 0,08 do `exame_regua.py`.
fn colunas_piso(px: &[u8], w: usize, t: &Value, rel: Option<f64>) -> Vec<(f64, f64, f64)> {
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
    let piso = rel.map_or(0.08, |r| {
        let mut topo: Vec<f64> = (x0 as usize..=x1 as usize)
            .filter(|&x| pressao(x as f64) >= 0.9)
            .map(|x| {
                coluna_piso(px, w, x, y, h, 0.0)
                    .0
                    .iter()
                    .copied()
                    .fold(0.0, f64::max)
            })
            .collect();
        topo.sort_by(f64::total_cmp);
        r * topo[topo.len() / 2]
    });
    let cols: Vec<(f64, Vec<f64>, f64, f64)> = (x0 as usize..=x1 as usize)
        .map(|x| {
            let (c, l, cen) = coluna_piso(px, w, x, y, h, piso);
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
    // Mede o DEPÓSITO, com o Smooth Edges a 0 (o controlo): o 0,3 de fábrica dos 13 óleos (dono 05/11) amacia a imagem e
    // tira cobertura ao fino — Dabber razão 0,85 → 0,73, Express Oil 0,95 → 0,84 (Bug #61/#62 do BUGS_painter).
    let todos: Vec<Pincel> = com_origem(&o)
        .into_iter()
        .map(Pincel::sem_smooth_edges)
        .collect();
    let medidas: Vec<(String, f64, f64)> = std::thread::scope(|s| {
        let hs: Vec<_> = todos
            .iter()
            .map(|p| {
                s.spawn(move || {
                    let (px, ts) = pinta(p, "tamanhos", &["recto_5", "recto_40"]);
                    let c: Vec<f64> = ts
                        .iter()
                        .map(|t| cobertura(&colunas(&px, 1000, t)))
                        .collect();
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
            falhas.push(format!(
                "{nome}: o recto de 5 px cobre {r:.2} do de 40 px (o Rebelle {alvo:.2})"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// A cena inteira `cena` da bateria (com a base, as secagens e a foto molhada) pintada por `p`.
fn cena_inteira(p: &Pincel, cena: &str) -> Vec<u8> {
    let bat = bateria();
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    pinta_cena(p, None, c, tela, [255; 3], 0).1
}

fn rgb(px: &[u8], w: usize, x: usize, y: usize) -> [f64; 3] {
    let i = (y * w + x) * 4;
    [0, 1, 2].map(|k| f64::from(px[i + k]))
}

/// ⭐ **(d) A LENTE sobre a faixa molhada**: a largura do azul em cima da faixa vermelha MOLHADA ÷ em cima da SECA,
/// por cruzamento (a mediana das 41 linhas da faixa; o azul = `b > r + 30` nos ±60 px do cruzamento).
fn lente(px: &[u8], w: usize, x: usize, y_molhada: usize, y_seca: usize) -> f64 {
    let largura = |y: usize| {
        let mut v: Vec<f64> = (y - 20..=y + 20)
            .map(|yy| {
                let azul: Vec<usize> = (x - 60..=x + 60)
                    .filter(|&xx| {
                        let c = rgb(px, w, xx, yy);
                        c[2] > c[0] + 30.0
                    })
                    .collect();
                azul.last().map_or(0.0, |l| (l - azul[0] + 1) as f64)
            })
            .collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    largura(y_molhada) / largura(y_seca).max(1.0)
}

/// O fim do recto de 200 px (a cena `tamanhos`): a CURVA da ponta — o fim no eixo − o das bordas, ÷ a meia-largura
/// (a cápsula > 0, o corte recto ≈ 0; `leis_oraculo.py::pontas`) — e as ESTRIAS: a luminância p95 − p5 no miolo
/// (linhas 360..520, colunas 300..700) — o «brilho de tubo» que a revisão cega viu são as estrias claras do relevo.
fn ponta_e_tubo(px: &[u8], w: usize) -> (f64, f64) {
    let (x0, x1, y) = (60usize, 940usize, 440usize);
    let c = |x: usize, yy: usize| {
        let v = rgb(px, w, x, yy);
        v.iter().map(|k| 255.0 - k).fold(0.0, f64::max) / 255.0
    };
    let rows: Vec<usize> = (y - 130..y + 130).collect();
    let mut maxs: Vec<f64> = (x0 + 100..x1 - 100)
        .map(|x| rows.iter().map(|&r| c(x, r)).fold(0.0, f64::max))
        .collect();
    maxs.sort_by(f64::total_cmp);
    let nivel = maxs[maxs.len() * 9 / 10];
    let lin: Vec<f64> = rows
        .iter()
        .map(|&r| (x0 + 100..x1 - 100).map(|x| c(x, r)).sum::<f64>() / (x1 - x0 - 200) as f64)
        .collect();
    let lmax = lin.iter().copied().fold(0.0, f64::max);
    let dentro: Vec<usize> = (0..rows.len()).filter(|&i| lin[i] > 0.25 * lmax).collect();
    let (a, b) = (dentro[0] as f64, *dentro.last().unwrap() as f64);
    let (meio, hw) = (0.5 * (a + b), 0.5 * (b - a + 1.0));
    let (mut eixo, mut bordas) = (Vec::new(), Vec::new());
    for (i, &r) in rows.iter().enumerate() {
        let v = (i as f64 - meio) / hw;
        if v.abs() > 0.8 {
            continue;
        }
        let Some(fim) = (x0..w).rev().find(|&x| c(x, r) >= 0.25 * nivel) else {
            continue;
        };
        let e = fim as f64 - x1 as f64;
        if v.abs() < 0.25 {
            eixo.push(e);
        } else if v.abs() > 0.6 {
            bordas.push(e);
        }
    }
    let mediana = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let curva = (mediana(eixo) - mediana(bordas)) / hw;
    let mut lum: Vec<f64> = (360..=520)
        .flat_map(|r| (300..700).map(move |x| (r, x)))
        .map(|(r, x)| rgb(px, w, x, r).iter().sum::<f64>() / 3.0)
        .collect();
    lum.sort_by(f64::total_cmp);
    let q = |f: f64| lum[((lum.len() - 1) as f64 * f) as usize];
    let estrias = q(0.95) - q(0.05);
    (curva, estrias)
}

/// SONDA **(d) A LENTE sobre a faixa molhada** (03_MOTORES §7 d — FECHADA: `as_leis_da_escala::a_tinta_molhada_nao_alarga_o_traco`;
/// o porquê de antes): o azul do óleo que cruza a
/// faixa vermelha ainda molhada tem no Rebelle a largura que tem sobre a seca (12/12 px, 1,00); o nosso Oil Bristle
/// incha 1,58× numa lente clara. Mecanismo: o NIVELAMENTO do filme (`ph2d-wet-paint/src/solver.rs`, o termo
/// `level_k` da diferença do filme entre vizinhos) escoa o filme do dab novo pelas células que a faixa molhada mantém
/// activas (sobre o papel seco estão inactivas e nada escoa). `leveling` 0 no preset cura a lente (1,00) mas o mesmo
/// nivelamento é o que alisa o relevo dos fios no traço inclinado: a escada do W3b sobe para 2,87 (≤ 2,60), e
/// nenhum `emboss` (0,5–0,7) cabe nas duas barras do relevo (escada e contraste). A cura é de motor.
#[test]
#[ignore = "sonda: a lente do óleo sobre a tinta molhada (03_MOTORES §7 d; o portão é as_leis_da_escala)"]
fn sonda_a_lente_sobre_a_molhada() {
    let o = oraculo();
    for nome in ["Oil Bristle", "Dabber"] {
        let p = pinceis(nome).remove(0);
        let px = cena_inteira(&p, "sobre");
        for (k, x) in [
            ("cruza_1", 250usize),
            ("cruza_2", 500),
            ("cruza_3_rapido", 750),
        ] {
            let l = lente(&px, 1000, x, 150, 470);
            let alvo = o["leis"][nome]["molhada"][k]["lente"].as_f64().unwrap();
            println!("[lente] {nome} {k}: {l:.2} (Rebelle {alvo:.2})");
        }
    }
}

/// SONDA **(c) A ponta e as estrias do pincel CHATO de óleo** (03_MOTORES §7 c — FECHADA: `as_leis_da_escala`; o porquê
/// de antes): o recto de
/// 200 px do Rebelle acaba em corte recto e desfiado (curva −0,02, desfiado 6,5 px) e as estrias são escuras e
/// baixas (p95 − p5 = 29); o nosso acaba na meia-elipse da pegada achatada (curva 0,08, desfiado 0,7 px) e as
/// estrias claras do relevo têm o dobro (55). A ponta RETA do preset (`dab_quina` 1) corta recto (0,00) mas tira
/// o contraste do relevo a 41 px do vale (13,3 × 12,1 ± 1,1); o `emboss` 0,3 casa as estrias a 200 px mas parte o
/// contraste a 41 px (o relevo CRESCE com o tamanho no nosso e não no Rebelle). A cura é de motor.
#[test]
#[ignore = "sonda: a ponta e as estrias do óleo (03_MOTORES §7 c; o portão é as_leis_da_escala)"]
fn sonda_a_ponta_e_as_estrias_do_oleo() {
    let o = oraculo();
    let p = pinceis("Oil Bristle").remove(0);
    let (px, _) = pinta(&p, "tamanhos", &["recto_200"]);
    let (curva, estrias) = ponta_e_tubo(&px, 1000);
    let pontas = &o["leis"]["Oil Bristle"]["pontas"];
    println!(
        "[ponta] Oil Bristle: curva {curva:.3} (Rebelle {:.3}) · estrias {estrias:.1} (Rebelle {:.1})",
        pontas["fim"]["curva"].as_f64().unwrap(),
        pontas["estrias"].as_f64().unwrap()
    );
}

/// A resposta à pressão de uma rampa: por decil de pressão, a largura e a cobertura ÷ as de p ≥ 0,9
/// (`exame_regua.py`, `pressao_largura`/`pressao_cobertura`).
fn decis(cols: &[(f64, f64, f64)]) -> ([f64; 10], [f64; 10]) {
    let mediana = |mut v: Vec<f64>| {
        if v.is_empty() {
            return f64::NAN;
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let media = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let topo: Vec<&(f64, f64, f64)> = cols.iter().filter(|c| c.0 >= 0.9).collect();
    let wl = mediana(topo.iter().map(|c| c.1).collect());
    let cl = media(&topo.iter().map(|c| c.2).collect::<Vec<_>>());
    let (mut l, mut c) = ([f64::NAN; 10], [f64::NAN; 10]);
    for k in 0..10 {
        let sel: Vec<&(f64, f64, f64)> = cols
            .iter()
            .filter(|x| x.0 >= k as f64 / 10.0 && (x.0 < (k + 1) as f64 / 10.0 || k == 9))
            .collect();
        if sel.len() > 3 {
            l[k] = mediana(sel.iter().map(|x| x.1).collect()) / wl;
            c[k] = media(&sel.iter().map(|x| x.2).collect::<Vec<_>>()) / cl;
        }
    }
    (l, c)
}

/// A resposta na FAIXA DA MÃO DO DONO (0,24–0,86; `project-memory/feedback_pen_pressure_floor_must_keep_the_owners_
/// hand_range.md`): (largura · cobertura) no decil 8 ÷ no decil 2, na rampa 0→1.
/// `Err` quando o decil 2 não chega à régua (`a ≤ 1e-3`, ou sem colunas): um preset MUDO na mão leve é uma falha,
/// nunca uma resposta infinita (era 99, e passava sem a distância ao Rebelle).
fn mao(l: &[f64; 10], c: &[f64; 10]) -> Result<f64, String> {
    let (a, b) = (l[2] * c[2], l[8] * c[8]);
    if a > 1e-3 && b.is_finite() {
        Ok(b / a)
    } else {
        Err(format!(
            "a mão leve (decil 2 da rampa) não chega à régua: largura {:.2} × cobertura {:.2} (÷ as do topo)",
            l[2], c[2]
        ))
    }
}

/// A faixa da mão tem de responder pelo menos isto: o dono reprovou ~1,3 (o piso da W6, 18/10: «apertar muito e
/// pouco variam pouco») e aprovava a lei de antes (3,2–10 na régua); 2,5 fica perto da média geométrica dos dois.
const R_MAO: f64 = 2.5;

/// O piso da coluna na régua da mão, × o topo do PRÓPRIO traço: o absoluto (0,08) emudecia a aguada clara — o Wet
/// Wash da linha (topo 0,176) lia o decil 2 vazio com 0,059 de cobertura; a 0,08 × o topo lê 2,90× (o main 3,00).
const PISO_DA_MAO: f64 = 0.08;

/// A resposta da MÃO de `p` (a régua `rampa_0_1` da bateria `gestos`): quanto a faixa da mão do dono varia.
fn mao_do_pincel(p: &Pincel, sem_pressao: bool) -> Result<f64, String> {
    let (px, ts) = pinta_com(p, "gestos", &["rampa_0_1"], sem_pressao);
    let n: Vec<([f64; 10], [f64; 10])> = ts
        .iter()
        .map(|t| decis(&colunas_piso(&px, 1000, t, Some(PISO_DA_MAO))))
        .collect();
    let m = mao(&n[0].0, &n[0].1);
    println!(
        "[pressao] {} (sem pressão: {sem_pressao}): mão {m:.2?}",
        p.nome
    );
    m
}

/// A mão de cada preset com origem, em paralelo.
fn maos(sem_pressao: bool) -> Vec<(String, Result<f64, String>)> {
    let o = oraculo();
    let todos = com_origem(&o);
    std::thread::scope(|s| {
        let hs: Vec<_> = todos
            .iter()
            .map(|p| s.spawn(move || (p.nome.clone(), mao_do_pincel(p, sem_pressao))))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// A lei da MÃO (dono 18/10): todo preset com origem responde ≥ [`R_MAO`] na faixa da mão. A paridade com a rampa do
/// Rebelle (a antiga lei (a), `BARRA_DA_PRESSAO` 0,18 e o controlo `rampa_torta`) saiu: o Rebelle deixou de ser a
/// referência (dono 31/10, `project-memory/project_rebelle_no_longer_the_reference_2026_10_31.md`). Sem o 99 do decil
/// 2 mudo (02/11), com o [`PISO_DA_MAO`] relativo, reprovam 2 — iguais no `main`: Graphite 8B (máximo da coluna no
/// decil 2 0,063, topo 0,898) e Colored 4B (0,024, topo 0,792) quase não marcam a pressão 0,2–0,3 (a curva do preset).
/// **Mutação que sangra:** a pressão presa a 1 ([`a_regua_da_mao_reprova_a_pressao_desligada`]: 0,89–1,05×).
#[test]
fn a_pressao_de_cada_preset_responde_na_faixa_da_mao() {
    let falhas: Vec<String> = maos(false)
        .iter()
        .filter_map(|(nome, m)| match m {
            Err(e) => Some(format!("{nome}: {e}")),
            Ok(m) if *m < R_MAO => Some(format!(
                "{nome}: a faixa da mão do dono só responde {m:.2}× (< {R_MAO})"
            )),
            Ok(_) => None,
        })
        .collect();
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// CONTROLO da lei da mão: a MESMA rampa pintada com a pressão presa a 1 — a régua tem de ler ~1× em todo preset
/// (nenhum passa de [`R_MAO`]), senão ela mede outra coisa que não a pressão.
#[test]
fn a_regua_da_mao_reprova_a_pressao_desligada() {
    let medidas = maos(true);
    let passam: Vec<String> = medidas
        .iter()
        .filter(|(_, m)| !m.as_ref().is_ok_and(|m| *m < R_MAO))
        .map(|(nome, m)| format!("{nome}: {m:.2?}"))
        .collect();
    assert!(
        passam.is_empty(),
        "sem pressão a régua devia ler ~1×: {}",
        passam.join(" · ")
    );
}

/// ⭐ **No Wet o modo do grão que lê a PONTA morde** (L9 × Wet, limpeza da J2): o Wet carimba o grão com a ponta
/// CHEIA (`dab::grain_at`, `w = 1`), e com `w = 1` o Subtract (`w − d·(1 − t)`) e o Darken (`min(w, T)`) são o
/// Multiply ao bit e o Color Burn não tem grão — a linha `grainMode` do painel mentia (o censo
/// `a_linha_esmaecida_e_a_que_nao_age`). Agora o grão desses três lê a silhueta da célula
/// (`dab::grain_at_na_ponta`); o Multiply, o Height e o Linear Height ficam ao bit.
#[test]
fn no_wet_o_modo_do_grao_que_le_a_ponta_morde() {
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let p = pinceis("Graphite 8B").remove(0);
    let pinta_com = |modo: f64| -> Vec<u8> {
        let (w, h) = (400u32, 100u32);
        let mut t = ph2d_tool_painter::PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        p.aplica(&mut t, true);
        t.set_ponta(ph2d_tool_painter::LinhaDaPonta::GrainMode, modo);
        t.set_brush_size_px(16.0);
        t.set_brush_color_srgb8([23, 101, 179]);
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, 50.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(40.0, PointerPhase::Down));
        for k in 1..=100 {
            t.on_canvas_pointer(cp(40.0 + 3.2 * k as f32, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(360.0, PointerPhase::Up));
        for _ in 0..60 {
            t.on_tick(1000.0 / 60.0);
        }
        t.run_full().0
    };
    let multiply = pinta_com(0.0);
    let mut falhas = Vec::new();
    for (modo, nome) in [(1.0, "Subtract"), (2.0, "Darken"), (3.0, "Color Burn")] {
        let px = pinta_com(modo);
        let diferem = px
            .chunks(4)
            .zip(multiply.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        println!("[grao] {nome}: {diferem} px diferem do Multiply");
        if diferem < 200 {
            falhas.push(format!(
                "{nome}: só {diferem} px diferem do Multiply no Wet"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
