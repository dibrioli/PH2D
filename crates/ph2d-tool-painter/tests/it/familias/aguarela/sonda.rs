//! **Sonda do MIOLO LISO** (J4, motor da família `aguarela`; `docs/Painter/Pinceis/exames/aguarela.md` «Motor J4»).
//! Pinta o recto de 200 px da bateria (e a pressão fraca a 40 px) com um pincel e as suas variantes, e lê o GRID do
//! motor em três momentos — logo depois do traço, depois de 240 quadros, depois do Dry — para dizer ONDE nascem as
//! listras e o miolo oco: no depósito (o `susp` logo no fim do traço), no escoamento ou na secagem (o `sett`).
//!
//! ```text
//! PH2D_SONDA_PINCEL="Wet Wash" PH2D_SONDA_VARIANTES='[{}, {"edgeDarkening": 0}]' \
//!   cargo test -p ph2d-tool-painter --profile smoke --test it sonda_o_miolo_liso -- --ignored --nocapture
//! ```

use crate::exame_bancada::pinceis;
use crate::sonda_gestos::{AZUL, gesto, tela};
use ph2d_editor_core::Tool;
use ph2d_tool_painter::PainterTool;

/// O perfil ATRAVÉS de um campo do grid (média em x, das colunas `x0..x1` px) nas linhas `y0..y1` px.
pub(super) fn perfil(
    t: &mut PainterTool,
    y0: f64,
    y1: f64,
    x0: f64,
    x1: f64,
) -> Vec<(f64, [f64; 5])> {
    let mut out = Vec::new();
    t.wet_sonda_motor(|e| {
        let g = e.active_grid();
        let razao = 1000.0 / (g.w as f64);
        let cel = |v: f64| (v / razao) as usize + 1;
        for cy in cel(y0)..=cel(y1) {
            let mut soma = [0.0f64; 5];
            let n = (cel(x1) - cel(x0) + 1) as f64;
            for cx in cel(x0)..=cel(x1) {
                let i = cx + cy * g.s;
                let m = f64::from(g.susp[i]) + f64::from(g.sett[i]);
                soma[0] += f64::from(g.susp[i]);
                soma[1] += f64::from(g.sett[i]);
                soma[2] += f64::from(g.film[i]);
                soma[3] += f64::from(u8::from(m <= 0.0));
                soma[4] += m * m;
            }
            out.push(((cy - 1) as f64 * razao, soma.map(|s| s / n)));
        }
    });
    out
}

/// As medidas do perfil: `[largura (px), média, listras, oco, buracos, mancha]` — listras = desvio de P − P alisado
/// (9 células) no miolo ÷ a média; oco = ombros ÷ centro; buracos = fracção de células sem tinta no miolo; mancha =
/// desvio na direcção do traço (√(E[m²] − P²)) ÷ a média. `campo` 9 = `susp + sett`. `None` = vazio.
pub(super) fn medidas(p: &[(f64, [f64; 5])], campo: usize) -> Option<[f64; 6]> {
    let v: Vec<f64> = p
        .iter()
        .map(|(_, s)| if campo == 9 { s[0] + s[1] } else { s[campo] })
        .collect();
    let max = v.iter().cloned().fold(0.0, f64::max);
    if max <= 0.0 {
        return None;
    }
    let on: Vec<usize> = (0..v.len()).filter(|&i| v[i] > 0.25 * max).collect();
    let (a, b) = (on[0], on[on.len() - 1] + 1);
    let w = (b - a) as f64;
    let f = |t: f64| a + (t * w).round() as usize;
    let media = |r: std::ops::Range<usize>| r.clone().map(|i| v[i]).sum::<f64>() / r.len().max(1) as f64;
    let miolo = f(0.15)..f(0.85);
    let m = media(miolo.clone());
    let alis = |i: usize| {
        let (l, r) = (i.saturating_sub(4), (i + 5).min(v.len()));
        (l..r).map(|j| v[j]).sum::<f64>() / (r - l) as f64
    };
    let lis = (miolo.clone().map(|i| (v[i] - alis(i)).powi(2)).sum::<f64>() / miolo.len() as f64).sqrt() / m;
    let centro = media(f(0.40)..f(0.60));
    let ombros = (media(f(0.10)..f(0.25)) + media(f(0.75)..f(0.90))) / 2.0;
    let buracos = miolo.clone().map(|i| p[i].1[3]).sum::<f64>() / miolo.len() as f64;
    let mancha = miolo
        .clone()
        .map(|i| {
            let mm = p[i].1[0] + p[i].1[1];
            (p[i].1[4] - mm * mm).max(0.0).sqrt() / mm.max(1e-9)
        })
        .sum::<f64>()
        / miolo.len() as f64;
    Some([w * (p[1].0 - p[0].0), m, lis, ombros / centro, buracos, mancha])
}

fn resume(nome: &str, p: &[(f64, [f64; 5])], campo: usize) -> String {
    match medidas(p, campo) {
        None => format!("{nome}: vazio"),
        Some([w, m, lis, oco, buracos, mancha]) => format!(
            "{nome}: largura {w:.0} px · média {m:.1} · listras {lis:.4} · oco {oco:.3} · buracos {buracos:.3} · mancha {mancha:.3}"
        ),
    }
}

/// O campo `susp + sett` do grid na caixa (px) como PGM de 16 bits (`PH2D_SONDA_PGM` = a pasta), para OLHAR o depósito.
fn pgm(t: &mut PainterTool, caixa: [f64; 4], ficheiro: &str) {
    let Ok(pasta) = std::env::var("PH2D_SONDA_PGM") else {
        return;
    };
    t.wet_sonda_motor(|e| {
        let g = e.active_grid();
        let razao = 1000.0 / (g.w as f64);
        let cel = |v: f64| (v / razao) as usize + 1;
        let (x0, y0, x1, y1) = (cel(caixa[0]), cel(caixa[1]), cel(caixa[2]), cel(caixa[3]));
        let mut out = format!("P5\n{} {}\n65535\n", x1 - x0, y1 - y0).into_bytes();
        for cy in y0..y1 {
            for cx in x0..x1 {
                let i = cx + cy * g.s;
                let m = match std::env::var("PH2D_SONDA_CAMPO").as_deref() {
                    Ok("sett") => f64::from(g.sett[i]),
                    Ok("film") => f64::from(g.film[i]) * 100.0,
                    _ => f64::from(g.susp[i]) + f64::from(g.sett[i]),
                }
                .clamp(0.0, 6553.5);
                out.extend_from_slice(&((m * 10.0) as u16).to_be_bytes());
            }
        }
        std::fs::write(format!("{pasta}/{ficheiro}.pgm"), out).unwrap();
    });
}

fn variantes() -> Vec<serde_json::Map<String, serde_json::Value>> {
    let v = std::env::var("PH2D_SONDA_VARIANTES").unwrap_or_else(|_| "[{}]".into());
    serde_json::from_str::<Vec<serde_json::Map<String, serde_json::Value>>>(&v).expect("variantes")
}

#[test]
#[ignore = "sonda: onde nasce o miolo listrado e a orla oca da aguarela (J4); corre-se à mão"]
fn sonda_o_miolo_liso() {
    let spec = std::env::var("PH2D_SONDA_PINCEL").unwrap_or_else(|_| "Wet Wash".into());
    for p in pinceis(&spec) {
        for (k, v) in variantes().iter().enumerate() {
            let crate::exame_bancada::Fonte::Preset { preset, .. } = &p.fonte else {
                continue;
            };
            let mut pr = (**preset).clone();
            for (c, x) in v {
                pr.parametros.engine.insert(c.clone(), x.as_f64().unwrap());
            }
            // `PH2D_SONDA_TAMANHOS="5,40,200"`: os rectos a esses diâmetros, pressão 1 (por omissão o de 200 e o fraco de 40).
            let cenas: Vec<(String, f32, f32, f32)> = match std::env::var("PH2D_SONDA_TAMANHOS") {
                Ok(l) => l.split(',').map(|d| (d.to_owned(), d.parse::<f32>().unwrap() / 2.0, 320.0, 1.0)).collect(),
                Err(_) => vec![("200".into(), 100.0, 440.0, 1.0), ("fraca40".into(), 20.0, 190.0, 0.25)],
            };
            for (rotulo, raio, y, pressao) in cenas {
                let mut t = tela();
                t.set_semente_dos_tracos(0);
                t.set_wet_relogio_fixo(true);
                let falhas = t.preset_aplicar(&pr).expect("meio");
                assert!(falhas.is_empty(), "{falhas:?}");
                t.set_brush_size_px(raio);
                t.set_brush_color_srgb8(AZUL);
                let dabs: std::sync::Arc<std::sync::Mutex<Vec<[f64; 6]>>> = std::sync::Arc::default();
                if std::env::var("PH2D_SONDA_DABS").is_ok() {
                    let d2 = std::sync::Arc::clone(&dabs);
                    t.wet_sonda_motor(move |e| {
                        e.on_dab = Some(Box::new(move |b, d| {
                            d2.lock().unwrap().push([d.x, d.y, d.r, d.intensity, d.opacity, d.water_amount]);
                        }));
                    });
                }
                let env = |k: &str, o: f32| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(o);
                let (x0, x1) = (env("PH2D_SONDA_X0", 60.0), env("PH2D_SONDA_X1", 940.0));
                gesto(&mut t, &[[x0, y], [x1, y]], 3.0, |_| pressao);
                let ds = dabs.lock().unwrap().clone();
                if !ds.is_empty() {
                    let mut meio: Vec<&[f64; 6]> = ds.iter().filter(|d| d[0] > 300.0 && d[0] < 700.0).collect();
                    if meio.is_empty() {
                        meio = ds.iter().collect();
                    }
                    let passos: Vec<String> = meio.windows(2).take(12).map(|w| format!("{:.3}", w[1][0] - w[0][0])).collect();
                    let ys: Vec<String> = meio.iter().take(8).map(|d| format!("{:.4}", d[1])).collect();
                    let ex = meio[meio.len() / 2];
                    let ini: Vec<String> = ds.iter().take(12).map(|d| format!("x{:.0} r{:.1} op{:.3} ag{:.3}", d[0], d[2], d[4], d[5])).collect();
                    println!("[dabs] início: {}", ini.join(" · "));
                    println!(
                        "[dabs] {} {rotulo}: {} dabs ({} no meio) · passos {} · y {} · r {:.2} I {:.3} opac {:.3} água {:.3}",
                        p.nome, ds.len(), meio.len(), passos.join(" "), ys.join(" "), ex[2], ex[3], ex[4], ex[5]
                    );
                }
                let m = f64::from(raio) * 1.6;
                let (y0, y1) = (f64::from(y) - m, f64::from(y) + m);
                let nome = format!("{}~v{k} {rotulo}", p.nome);
                let p0 = perfil(&mut t, y0, y1, 300.0, 700.0);
                println!("[liso] {nome} 0q   susp  {}", resume("", &p0, 0));
                let slug = nome.replace([' ', '~'], "_");
                pgm(&mut t, [40.0, y0, 960.0, y1], &format!("{slug}_0q"));
                for _ in 0..240 {
                    t.on_tick(1000.0 / 60.0);
                }
                pgm(&mut t, [40.0, y0, 960.0, y1], &format!("{slug}_240q"));
                let p1 = perfil(&mut t, y0, y1, 300.0, 700.0);
                println!("[liso] {nome} 240q massa {}", resume("", &p1, 9));
                println!("[liso] {nome} 240q sett  {}", resume("", &p1, 1));
                if std::env::var("PH2D_SONDA_PERFIL").is_ok() {
                    let s: Vec<String> = p0.iter().map(|(yy, s)| format!("{yy:.0}:{:.0}/{:.0}", s[0] + s[1], p1.iter().find(|q| q.0 == *yy).map_or(0.0, |q| q.1[0] + q.1[1]))).collect();
                    println!("[perfil] {nome} {}", s.join(" "));
                }
            }
        }
    }
}
