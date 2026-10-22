//! **As leis de ESCALA do Wet** (JANELA 2, `docs/Painter/Pinceis/03_MOTORES.md` §7 c, d e b-resto): o que o motor
//! fazia de diferente do Rebelle quando o MESMO pincel passa de 5 a 200 px, e por cima da tinta molhada. Oráculo:
//! as fotos A da bateria (`fixtures/rebelle_bateria/`, cena `tamanhos` e `sobre`) pela régua
//! `ferramentas/leis_escala.py`; aqui, o motor por dentro (o campo) e os portões.

use super::exame_bancada::{Pincel, bateria, pinceis};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use serde_json::Value;

/// Os pontos do traço `nome` da cena `cena` da bateria.
pub(super) fn pontos(cena: &str, nome: &str) -> Vec<[f32; 3]> {
    let bat = bateria();
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap();
    let t = c["ops"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["op"] == "traco" && o["nome"] == nome)
        .unwrap();
    t["pts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p: &Value| [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32))
        .collect()
}

/// Um traço com os pontos EXACTOS da bateria (um evento cada, 8 ms entre eles — o `exame_bancada::traco`).
pub(super) fn traco(t: &mut PainterTool, pts: &[[f32; 3]]) {
    let cp = |p: [f32; 3], phase| CanvasPointer {
        pos: [p[0], p[1]],
        pressure: p[2],
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for &p in &pts[1..] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(pts[pts.len() - 1], PointerPhase::Up));
}

/// Uma tela branca 1000×640 (a da bateria) com o pincel `p`, a cor azul da bateria e o diâmetro `d`.
pub(super) fn tela_com(p: &Pincel, d: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 1000 * 640 * 4], 1000, 640);
    p.aplica(&mut t, true);
    t.set_brush_color_srgb8([23, 101, 179]);
    t.set_brush_size_px(d * 0.5);
    t
}

/// O campo do relevo no miolo de um recto horizontal em `y` de diâmetro `d` (colunas 200..800, linhas a ±0,3 `d`):
/// o desvio dos DOIS termos do emboss do render (`ph2d_wet_paint::render`): o do gradiente da massa e o da altura
/// dos fios — e a massa média.
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct Relevo {
    pub massa: f64,
    pub emb_massa: f64,
    pub emb_altura: f64,
    pub emb_total: f64,
}

pub(super) fn relevo_no_miolo(t: &mut PainterTool, y: f64, d: f64) -> Relevo {
    let mut r = Relevo::default();
    t.wet_sonda_motor(|e| {
        let g = &e.layers[e.active_layer].grid;
        let s = g.s;
        let m = |i: usize| f64::from(g.sett[i] + g.susp[i]);
        let h = |i: usize| f64::from(ph2d_wet_paint::grid::altura::em(g, i));
        let (mut ms, mut em, mut ea, mut et) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let a = (0.3 * d).max(1.0) as i64;
        for yy in (y as i64 - a)..=(y as i64 + a) {
            for x in 200..800usize {
                let i = (yy as usize + 1) * s + x + 1;
                let k = f64::from(g.relevo[i]);
                let gm = ((m(i + 1) - m(i - 1)) * 0.5 + (m(i + s) - m(i - s))) * 0.008 * k;
                let ga = ((h(i + 1) - h(i - 1)) * 0.5 + (h(i + s) - h(i - s)))
                    * ph2d_wet_paint::bristles_relevo::LUZ
                    * k;
                ms.push(m(i));
                em.push(gm);
                ea.push(ga);
                et.push((gm + ga).clamp(-40.0, 40.0));
            }
        }
        let desvio = |v: &[f64]| {
            let n = v.len().max(1) as f64;
            let a = v.iter().sum::<f64>() / n;
            (v.iter().map(|x| (x - a).powi(2)).sum::<f64>() / n).sqrt()
        };
        r = Relevo {
            massa: ms.iter().sum::<f64>() / ms.len().max(1) as f64,
            emb_massa: desvio(&em),
            emb_altura: desvio(&ea),
            emb_total: desvio(&et),
        };
    });
    r
}

/// SONDA: o relevo do Oil Bristle (e das variantes `PH2D_SONDA_KNOBS`, `;`-separadas) a 40 e a 200 px.
#[test]
#[ignore = "sonda: o campo do relevo a 40 e 200 px; corre-se à mão"]
fn sonda_o_relevo_por_tamanho() {
    let base = pinceis("Oil Bristle").remove(0);
    let vs = std::env::var("PH2D_SONDA_KNOBS").unwrap_or_default();
    let mut casos: Vec<String> = vec![String::new()];
    casos.extend(vs.split(';').filter(|v| !v.is_empty()).map(str::to_owned));
    for kv in casos {
        let p = match &base.fonte {
            super::exame_bancada::Fonte::Preset { preset, relativo } => Pincel {
                nome: base.nome.clone(),
                fonte: super::exame_bancada::Fonte::Preset {
                    preset: Box::new(super::o_relevo_dos_fios::com_knobs((**preset).clone(), &kv)),
                    relativo: relativo.clone(),
                },
            },
            super::exame_bancada::Fonte::Meio(_) => unreachable!(),
        };
        let tams: Vec<f64> = std::env::var("PH2D_SONDA_TAMANHOS").map_or_else(
            |_| vec![40.0, 200.0],
            |v| v.split(',').map(|x| x.parse().unwrap()).collect(),
        );
        for d in tams {
            let (nome, y) = ("recto_200", 440.0);
            let mut t = tela_com(&p, d as f32);
            traco(&mut t, &pontos("tamanhos", nome));
            for _ in 0..240 {
                t.on_tick(1000.0 / 60.0);
            }
            let r = relevo_no_miolo(&mut t, y, d);
            println!(
                "[relevo] {kv:?} {nome}: Relevo {{ massa: {:?}, emb_massa: {:?}, emb_altura: {:?}, emb_total: {:?} }}",
                r.massa, r.emb_massa, r.emb_altura, r.emb_total
            );
        }
    }
}

/// SONDA: as barras do W3b (`o_relevo_dos_fios::o_relevo_nao_tem_escada`) e da W3 (a textura do recto) do Oil
/// Bristle de fábrica com as variantes `PH2D_SONDA_KNOBS` (`;`-separadas, `k=v,b.campo=v`), a 103 e a 41 px.
#[test]
#[ignore = "sonda: as barras do relevo com variantes; corre-se à mão"]
fn sonda_as_barras_do_relevo() {
    use super::midias_de_fabrica::de_fabrica;
    use super::o_relevo_dos_fios::{com_knobs, escada_de};
    let vs = std::env::var("PH2D_SONDA_KNOBS").unwrap_or_default();
    let mut casos: Vec<String> = vec![String::new()];
    casos.extend(vs.split(';').filter(|v| !v.is_empty()).map(str::to_owned));
    let todos: Vec<(String, f32)> = casos
        .iter()
        .flat_map(|kv| [51.5f32, 20.5, 20.0].map(|r| (kv.clone(), r)))
        .collect();
    std::thread::scope(|s| {
        let hs: Vec<_> = todos
            .iter()
            .map(|(kv, r)| {
                s.spawn(move || {
                    // `PH2D_SONDA_PRESET`: outro ficheiro no lugar do de fábrica (o de um commit antigo).
                    let base = std::env::var("PH2D_SONDA_PRESET").map_or_else(
                        |_| de_fabrica("Oil Bristle"),
                        |f| ph2d_brush_presets::ler(&std::fs::read(f).unwrap()).unwrap(),
                    );
                    let p = com_knobs(base, kv);
                    let c = com_knobs(p.clone(), "-bristleHeight,-bristleRemove,emboss=0.7");
                    let (m, cc) = (escada_de(&p, *r), escada_de(&c, *r));
                    let tex = super::o_relevo_dos_fios::cena_e_campo(&p, *r).0;
                    format!(
                        "[barras] {kv:?} r {r}: escada {:.2} {:.2} {:.2} · contraste {:.2} (controlo {:.2}) · textura {:.2} borda {:.3}",
                        m[0].escada, m[1].escada, m[2].escada, m[3].contraste, cc[3].contraste, tex.textura, tex.borda
                    )
                })
            })
            .collect();
        for h in hs {
            println!("{}", h.join().unwrap());
        }
    });
}

/// O pincel `nome` de fábrica com as variantes `kv` (`k=v,b.campo=v,-k`).
pub(super) fn com(nome: &str, kv: &str) -> Pincel {
    let base = pinceis(nome).remove(0);
    let super::exame_bancada::Fonte::Preset { preset, relativo } = base.fonte else {
        unreachable!()
    };
    Pincel {
        nome: base.nome,
        fonte: super::exame_bancada::Fonte::Preset {
            preset: Box::new(super::o_relevo_dos_fios::com_knobs(*preset, kv)),
            relativo,
        },
    }
}

/// A cena `cena` da bateria inteira (a base, as secagens, a foto A) pintada por `p`.
pub(super) fn cena(p: &Pincel, cena: &str) -> Vec<u8> {
    let bat = bateria();
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap();
    super::exame_bancada::pinta_cena(p, None, c, [1000, 640], [255; 3], 0).1
}

/// ⭐ **A LENTE**: a largura do azul (`b > r + 30`) em cima da faixa vermelha MOLHADA (y 150) e da SECA (y 470), no
/// cruzamento de coluna `x` — a mediana das 41 linhas de cada faixa (`leis_oraculo.py::lente`, ao bit).
pub(super) fn lente(px: &[u8], x: usize) -> (f64, f64) {
    let largura = |y: usize| {
        let mut v: Vec<f64> = (y - 20..=y + 20)
            .map(|yy| {
                let azul: Vec<usize> = (x - 60..=x + 60)
                    .filter(|&xx| {
                        let i = (yy * 1000 + xx) * 4;
                        f64::from(px[i + 2]) > f64::from(px[i]) + 30.0
                    })
                    .collect();
                azul.last().map_or(0.0, |l| (l - azul[0] + 1) as f64)
            })
            .collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    (largura(150), largura(470))
}

/// SONDA: a lente dos óleos sobre a faixa molhada (`PH2D_SONDA_PINCEIS`, `|`-separados; variantes `PH2D_SONDA_KNOBS`).
#[test]
#[ignore = "sonda: a lente sobre a molhada com variantes; corre-se à mão"]
fn sonda_a_lente_com_variantes() {
    let nomes = std::env::var("PH2D_SONDA_PINCEIS").unwrap_or_else(|_| "Oil Bristle".to_owned());
    let vs = std::env::var("PH2D_SONDA_KNOBS").unwrap_or_default();
    let mut casos: Vec<String> = vec![String::new()];
    casos.extend(vs.split(';').filter(|v| !v.is_empty()).map(str::to_owned));
    let todos: Vec<(String, String)> = nomes
        .split('|')
        .flat_map(|n| casos.iter().map(move |kv| (n.to_owned(), kv.clone())))
        .collect();
    std::thread::scope(|s| {
        let hs: Vec<_> = todos
            .iter()
            .map(|(n, kv)| {
                s.spawn(move || {
                    let px = cena(&com(n, kv), "sobre");
                    let l: Vec<String> = [250usize, 500, 750]
                        .iter()
                        .map(|&x| {
                            let (m, sc) = lente(&px, x);
                            format!("{m:.0}/{sc:.0}")
                        })
                        .collect();
                    format!("[lente] {n} {kv:?}: {}", l.join(" "))
                })
            })
            .collect();
        for h in hs {
            println!("{}", h.join().unwrap());
        }
    });
}

// ─── A régua (`ferramentas/leis_escala.py`, ao bit nos números que os portões leem) ─────────────────────────────

/// O oráculo das leis de escala (`fixtures/leis_escala_rebelle.json`, `leis_escala.py oraculo`): a métrica `q` do
/// recto `recto` do preset `pre` do Rebelle (`TOOL/prefixo`).
pub(super) fn rebelle(pre: &str, recto: &str, q: &str) -> f64 {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/fixtures/leis_escala_rebelle.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
    let r = &v["escala"][pre][recto];
    r.get(q)
        .or_else(|| r["fim"].get(q))
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{pre} {recto} {q}"))
}

pub(super) fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// O recto horizontal de `y` (as colunas de 10 % a 90 % de 60..940, a janela ±`h`): `(através, buraco)` — a
/// mediana do desvio da luminância ATRAVÉS do miolo e o maior buraco (colunas seguidas com o pico abaixo de metade
/// do mediano; uma coluna só é o grão, duas já se vêem).
pub(super) fn recto(px: &[u8], y: usize, h: usize) -> (f64, f64) {
    let c = |x: usize, yy: usize| {
        let i = (yy * 1000 + x) * 4;
        (0..3)
            .map(|k| 255.0 - f64::from(px[i + k]))
            .fold(0.0, f64::max)
            / 255.0
    };
    let lum = |x: usize, yy: usize| {
        let i = (yy * 1000 + x) * 4;
        (0..3).map(|k| f64::from(px[i + k])).sum::<f64>() / 3.0
    };
    let (mut larg, mut cen, mut mx) = (Vec::new(), Vec::new(), Vec::new());
    for x in 148..852 {
        let col: Vec<f64> = (y - h..=y + h).map(|yy| c(x, yy)).collect();
        let m = col.iter().copied().fold(0.0, f64::max);
        mx.push(m);
        if m < 0.08 {
            larg.push(0.0);
            cen.push(0.0);
            continue;
        }
        let a = col.iter().position(|&v| v >= 0.25 * m).unwrap();
        let b = col.iter().rposition(|&v| v >= 0.25 * m).unwrap();
        larg.push((b - a + 1) as f64);
        let s: f64 = col.iter().sum();
        cen.push(
            col.iter()
                .enumerate()
                .map(|(i, v)| (i as f64 - h as f64) * v)
                .sum::<f64>()
                / s.max(1e-9),
        );
    }
    let banda = (0.35 * mediana(larg)).max(0.5);
    let atr: Vec<f64> = (148..852)
        .filter_map(|x| {
            let l: Vec<f64> = (y - h..=y + h)
                .filter(|&yy| (yy as f64 - y as f64 - cen[x - 148]).abs() <= banda)
                .map(|yy| lum(x, yy))
                .collect();
            (l.len() >= 3).then(|| {
                let m = l.iter().sum::<f64>() / l.len() as f64;
                (l.iter().map(|v| (v - m).powi(2)).sum::<f64>() / l.len() as f64).sqrt()
            })
        })
        .collect();
    // O maior BURACO: a maior corrida de colunas com o pico abaixo de metade do mediano.
    let meio = mediana(mx.clone());
    let (mut buraco, mut corre) = (0usize, 0usize);
    for &m in &mx {
        corre = if m < 0.5 * meio { corre + 1 } else { 0 };
        buraco = buraco.max(corre);
    }
    (mediana(atr), buraco as f64)
}

/// O fim do recto horizontal de `y` (`leis_escala.py::ponta`): `(curva, desfiado)` — o fim no eixo − o das bordas ÷
/// a meia-largura (a cápsula > 0, o corte recto ≈ 0) e o desvio do fim de linha para linha sem a curva (px).
pub(super) fn ponta(px: &[u8], y: usize, h: usize) -> (f64, f64) {
    let c = |x: usize, yy: usize| {
        let i = (yy * 1000 + x) * 4;
        (0..3)
            .map(|k| 255.0 - f64::from(px[i + k]))
            .fold(0.0, f64::max)
            / 255.0
    };
    let (x0, x1, m) = (60usize, 940usize, 110usize);
    let mut maxs: Vec<f64> = (x0 + m..x1 - m)
        .map(|x| (y - h..=y + h).map(|r| c(x, r)).fold(0.0, f64::max))
        .collect();
    maxs.sort_by(f64::total_cmp);
    let k = 0.9 * (maxs.len() - 1) as f64;
    let nivel = maxs[k.floor() as usize]
        + (maxs[k.ceil() as usize] - maxs[k.floor() as usize]) * (k - k.floor());
    let lin: Vec<f64> = (y - h..=y + h)
        .map(|r| (x0 + m..x1 - m).map(|x| c(x, r)).sum::<f64>() / (x1 - x0 - 2 * m) as f64)
        .collect();
    let lmax = lin.iter().copied().fold(0.0, f64::max);
    let rows: Vec<usize> = (0..lin.len()).filter(|&i| lin[i] > 0.25 * lmax).collect();
    let (a, b) = (rows[0] as f64, *rows.last().unwrap() as f64);
    let (meio, hw) = (0.5 * (a + b), 0.5 * (b - a + 1.0));
    let mut fins: Vec<(f64, f64)> = Vec::new();
    for &r in &rows {
        let v = (r as f64 - meio) / hw;
        if v.abs() > 0.8 {
            continue;
        }
        if let Some(f) = (0..1000).rev().find(|&x| c(x, y - h + r) >= 0.25 * nivel) {
            fins.push((v, f as f64 - x1 as f64));
        }
    }
    let eixo = mediana(
        fins.iter()
            .filter(|p| p.0.abs() < 0.25)
            .map(|p| p.1)
            .collect(),
    );
    let bordas = mediana(
        fins.iter()
            .filter(|p| p.0.abs() > 0.6)
            .map(|p| p.1)
            .collect(),
    );
    // O resíduo da parábola de mínimos quadrados `e(v)` (o `np.polyfit(v, e, 2)`).
    let s = |p: i32, q: i32| fins.iter().map(|f| f.0.powi(p) * f.1.powi(q)).sum::<f64>();
    let m3 = [
        [s(4, 0), s(3, 0), s(2, 0)],
        [s(3, 0), s(2, 0), s(1, 0)],
        [s(2, 0), s(1, 0), s(0, 0)],
    ];
    let rhs = [s(2, 1), s(1, 1), s(0, 1)];
    let det = |m: [[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(m3);
    let coef: Vec<f64> = (0..3)
        .map(|j| {
            let mut mj = m3;
            (0..3).for_each(|i| mj[i][j] = rhs[i]);
            det(mj) / d
        })
        .collect();
    let res: Vec<f64> = fins
        .iter()
        .map(|&(v, e)| e - (coef[0] * v * v + coef[1] * v + coef[2]))
        .collect();
    let mr = res.iter().sum::<f64>() / res.len() as f64;
    let desfiado = (res.iter().map(|r| (r - mr).powi(2)).sum::<f64>() / res.len() as f64).sqrt();
    ((eixo - bordas) / hw, desfiado)
}

/// A cena `tamanhos` da bateria (os três rectos e a foto molhada) pintada por `p`.
pub(super) fn tamanhos(p: &Pincel) -> Vec<u8> {
    cena(p, "tamanhos")
}

/// ⭐⭐ **(c) O relevo NÃO CRESCE com o pincel** (03_MOTORES §7 c; `Knob::EmbossSizeExp`): o desvio da luminância
/// ATRAVÉS do miolo do recto de 200 px ÷ o do de 40, no Oil Bristle de fábrica, cabe na família do Rebelle (Bristle
/// Flat 0,74 · Wide 0,73 · Soft 0,68, o fixture) + 0,08 de ruído — e o de 200 px a ±2 do Bristle Flat (11,5). Antes
/// da lei o nosso CRESCIA: 15 → 17 (1,13), o «tubo» de estrias claras da revisão cega.
#[test]
fn o_relevo_nao_cresce_com_o_pincel() {
    let px = tamanhos(&pinceis("Oil Bristle").remove(0));
    let ((a40, _), (a200, _)) = (recto(&px, 190, 70), recto(&px, 440, 130));
    let fam = [
        "bristle_bristle_flat",
        "bristle_bristle_wide",
        "bristle_bristle_soft",
    ]
    .map(|n| {
        let pre = format!("OIL_AND_ACRYLIC/{n}");
        rebelle(&pre, "recto_200", "atraves") / rebelle(&pre, "recto_40", "atraves")
    });
    let tecto = fam.iter().copied().fold(0.0, f64::max) + 0.08;
    let alvo = rebelle(
        "OIL_AND_ACRYLIC/bristle_bristle_flat",
        "recto_200",
        "atraves",
    );
    println!(
        "[escala] através 40 {a40:.2} · 200 {a200:.2} · razão {:.3} (Rebelle {fam:.3?}, tecto {tecto:.3})",
        a200 / a40
    );
    assert!(
        a200 / a40 <= tecto,
        "o relevo cresce com o pincel: {a40:.2} → {a200:.2}"
    );
    assert!(
        (a200 - alvo).abs() <= 2.0,
        "o relevo a 200 px {a200:.2} contra o Rebelle {alvo:.2}"
    );
}

/// ⭐⭐ **(c) A ponta das cerdas é RECTA e DESFIADA** (03_MOTORES §7 c; `Knob::BristleTipRagged`, as faces da lâmina e o
/// desvio de cada fio): o fim do recto de 200 px do Oil Bristle com a curva a ±0,06 de zero (o Bristle Flat −0,02;
/// antes a meia-elipse lia 0,085 — a 40 px a curva não separa: a elipse 0,075, o desvio de ~27 fios ±0,05) e o
/// desfiado dos dois entre metade e 1,5× o do Bristle Flat (6,5 · 1,3 px; antes 0,7 · 0,3).
#[test]
fn a_ponta_das_cerdas_e_recta_e_desfiada() {
    let px = tamanhos(&pinceis("Oil Bristle").remove(0));
    let mut falhas = Vec::new();
    for (recto, y, h) in [("pontas_40", 190, 70), ("pontas_200", 440, 130)] {
        let (curva, desfiado) = ponta(&px, y, h);
        let alvo = rebelle("OIL_AND_ACRYLIC/bristle_bristle_flat", recto, "desfiado_px");
        println!("[ponta] {recto}: curva {curva:.3} · desfiado {desfiado:.2} (Rebelle {alvo:.2})");
        if recto == "pontas_200" && curva.abs() > 0.06 {
            falhas.push(format!("{recto}: a ponta curva {curva:.3}"));
        }
        if !(0.5 * alvo..=1.5 * alvo).contains(&desfiado) {
            falhas.push(format!("{recto}: desfiado {desfiado:.2} contra {alvo:.2}"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ **A ponta desfiada não depende da GRADE**: o desvio de cada fio é em px (sub-célula, a bilinear das quatro
/// células); o desfiado do recto de 40 px do Oil Bristle a 2 px por célula fica a ±25 % do de 1 px por célula
/// (1,15 · 1,05). ⚠️ A mutação «arredondado à célula» SOBREVIVE aqui (a 2 px o efeito fica abaixo do ruído da régua do
/// desfiado) e a 4 px a própria célula do motor domina (1,91 com ou sem ela): o portão guarda a ordem de grandeza.
#[test]
fn a_ponta_desfiada_nao_depende_da_grade() {
    let p = pinceis("Oil Bristle").remove(0);
    let des = |ratio: f64| {
        let mut t = tela_com(&p, 40.0);
        t.set_wet_grid_ratio(ratio);
        traco(&mut t, &pontos("tamanhos", "recto_40"));
        for _ in 0..240 {
            t.on_tick(1000.0 / 60.0);
        }
        ponta(&t.run_full().0, 190, 70).1
    };
    let (a, b) = std::thread::scope(|s| {
        let a = s.spawn(|| des(1.0));
        let b = s.spawn(|| des(2.0));
        (a.join().unwrap(), b.join().unwrap())
    });
    println!("[grade] desfiado a 40 px: 1 · 2 px/célula {a:.2} · {b:.2}");
    assert!(
        (b / a - 1.0).abs() <= 0.25,
        "o desfiado muda com a grade: {a:.2} → {b:.2}"
    );
}

/// ⭐⭐ **(d) A tinta molhada NÃO ALARGA o traço** (03_MOTORES §7 d; `Knob::LevelingOwnStroke`): o azul do Oil
/// Bristle que cruza a faixa vermelha AINDA MOLHADA tem a largura que tem sobre a SECA, ±1 px, nos três cruzamentos
/// (o Rebelle 12/12 · 14/14 · 12/12). O controlo é a seca do MESMO pincel SEM a lei (sobre o seco a lei não age:
/// a parede é só entre tintas molhadas). Antes: 19/12 nos três — o nivelamento escoava o filme do traço novo para as
/// células da faixa molhada, que o mantêm activo.
#[test]
fn a_tinta_molhada_nao_alarga_o_traco() {
    let (px, sem) = std::thread::scope(|s| {
        let a = s.spawn(|| cena(&pinceis("Oil Bristle").remove(0), "sobre"));
        let b = s.spawn(|| cena(&com("Oil Bristle", "levelingOwnStroke=0"), "sobre"));
        (a.join().unwrap(), b.join().unwrap())
    });
    let l: Vec<(f64, f64)> = [250usize, 500, 750]
        .iter()
        .map(|&x| (lente(&px, x).0, lente(&sem, x).1))
        .collect();
    println!("[lente] molhada (com a lei) / seca (sem a lei) {l:?}");
    for (m, s) in &l {
        assert!(
            *m <= s + 1.0,
            "a lente: {m} px sobre a molhada contra {s} sobre a seca sem a lei"
        );
    }
}

/// ⭐⭐ **(d) A parede é da TINTA, não do pincel activo**: o Oil Bristle pinta a cena `sobre` e o artista troca para um
/// pincel SEM a lei (o mesmo óleo, `levelingOwnStroke` 0) ANTES de a água correr — os traços do óleo continuam sem escoar uns nos outros (a lente
/// ±1 px da seca). Antes a lei era lida do pincel activo: trocar de pincel tirava as paredes da tinta já pousada.
#[test]
fn a_parede_e_da_tinta_e_nao_do_pincel_activo() {
    let bat = bateria();
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "sobre")
        .unwrap();
    let mut ops = c["ops"].as_array().unwrap().clone();
    let k = ops.iter().rposition(|o| o["op"] == "simula").unwrap();
    ops.insert(k, serde_json::json!({"op": "pincel", "quem": "base"}));
    let cena_troca = serde_json::json!({"nome": "sobre", "ops": ops});
    let oleo = pinceis("Oil Bristle").remove(0);
    // O outro pincel é o MESMO óleo sem a lei (o mesmo nivelamento: um Wet Round nivela pouco e não provaria nada).
    let outro = com("Oil Bristle", "levelingOwnStroke=0");
    let px = super::exame_bancada::pinta_cena(
        &oleo,
        Some(&outro),
        &cena_troca,
        [1000, 640],
        [255; 3],
        0,
    )
    .1;
    let l: Vec<(f64, f64)> = [250usize, 500, 750]
        .iter()
        .map(|&x| lente(&px, x))
        .collect();
    println!("[parede] com o pincel sem a lei activo: molhada/seca {l:?}");
    for (m, s) in &l {
        assert!(
            *m <= s + 1.0,
            "trocar de pincel tirou a parede: {m} px contra {s}"
        );
    }
}

/// ⭐⭐ **(b) O traço FINO do lápis não ABRE buracos** (03_MOTORES §7 b-resto; `PainterTool::grao_do_wet`): o recto de 5
/// px dos lápis e do carvão de fábrica sem nenhum buraco de 2 colunas ou mais (os 7 do Rebelle: nenhuma coluna
/// sequer). Antes: o 8B 2 · HB 3 · 2B 3 · carvão 2 colunas — o grão de período maior que um terço da ponta pousava
/// um vale atravessado no traço inteiro.
#[test]
fn o_traco_fino_do_lapis_nao_abre_falhas() {
    let nomes = [
        "Graphite 8B",
        "Graphite HB",
        "Graphite 2H",
        "Graphite 2B",
        "Colored 4B",
        "Charcoal",
    ];
    let r: Vec<(String, f64)> = std::thread::scope(|s| {
        let hs: Vec<_> = nomes
            .iter()
            .map(|n| {
                s.spawn(move || {
                    (
                        n.to_string(),
                        recto(&tamanhos(&pinceis(n).remove(0)), 70, 40).1,
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!("[fino] o maior buraco a 5 px (colunas): {r:?}");
    let abre: Vec<_> = r.iter().filter(|(_, b)| *b >= 2.0).collect();
    assert!(abre.is_empty(), "o traço de 5 px abre buracos: {abre:?}");
}
