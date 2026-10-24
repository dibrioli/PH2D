//! ⭐ **A lei da orla do traço** (bug #58, 2.ª volta — fotos do dono depois de `c9c37e7c5f`: linhas finas dentro
//! das sobreposições, sobreposições mais claras nas voltas, placas rectangulares e linhas tortas num borrão grande,
//! com Wet ou Burnt > 0). Pela porta do ponteiro, sobre o fundo do smoke.
//!
//! ⚠️ **O oráculo é INDEPENDENTE do código sob teste**: a distância de cada texel de dentro (`v > τ`, `v` a máscara
//! VISÍVEL, `τ` a fronteira do traço — o gate (f) prova que é `½` do nível de UMA passagem) ao texel de fora mais
//! próximo, por força bruta sobre a tela. O contorno sub-píxel fica
//! entre os dois texels, logo `D − √2 ≤ d ≤ D`: com `D > W + √2` o texel está FUNDO (`e = 0`) e com `D ≤ 1` está
//! na borda (`e ≥ perfil(1)`). As cores esperadas são a composição do texel com `e = 0` e `e = 1`
//! (`TracoComposto::compoe`, a lei da cor — não a da distância).

use super::traco_orla_bordas_tests::{H, W, fundo, glaze_round, padrao};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Os nós ligados ao passo de 3 px do rato.
fn caminho(nos: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut out = vec![nos[0]];
    for par in nos.windows(2) {
        let (a, b) = (par[0], par[1]);
        let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as usize;
        for k in 1..=n {
            let s = k as f32 / n as f32;
            out.push([a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s]);
        }
    }
    out
}

/// O Z que se cruza (volta em diagonal por cima de si).
fn z() -> Vec<[f32; 2]> {
    caminho(&[
        [40.0, 40.0],
        [280.0, 40.0],
        [50.0, 200.0],
        [280.0, 200.0],
        [150.0, 20.0],
    ])
}

/// O borrão grande rabiscado: idas e voltas a 12 px umas das outras (um pincel de 40 px tapa-as todas).
fn rabisco() -> Vec<[f32; 2]> {
    let nos: Vec<[f32; 2]> = (0..12)
        .map(|k| {
            [
                if k % 2 == 0 { 60.0 } else { 260.0 },
                60.0 + 11.0 * k as f32,
            ]
        })
        .collect();
    caminho(&nos)
}

/// Pinta `pts` com um quadro a cada `lote(k)` eventos e deixa o traço ABERTO.
fn pinta(t: &mut PainterTool, pts: &[[f32; 2]], lote: impl Fn(usize) -> usize) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    let (mut k, mut desde) = (0, 0);
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        desde += 1;
        if desde >= lote(k) {
            t.paint_tick(1.0 / 60.0);
            (k, desde) = (k + 1, 0);
        }
    }
    t.paint_tick(1.0 / 60.0);
}

/// O que o oráculo precisa do traço aberto: a tela, a base, o plano (RGBA reto, f32) e a máscara visível.
struct Traco {
    tela: Vec<u8>,
    base: Vec<[f32; 4]>,
    plano: Vec<[f32; 4]>,
    v: Vec<f32>,
    /// A fronteira `τ` que o traço usa (o nível de uma passagem, ½).
    tau: f32,
}

fn traco(t: &mut PainterTool, pts: &[[f32; 2]]) -> Traco {
    pinta(t, pts, |_| 4);
    let tela = t.canvas_rgba.to_vec();
    let s = t
        .paint
        .plano_do_traco
        .as_ref()
        .expect("o traço pinta no plano");
    let dec = |c: &[u8]| [0, 1, 2, 3].map(|k| f32::from(c[k]) / 255.0);
    let base: Vec<[f32; 4]> = s.base.chunks(4).map(dec).collect();
    let plano: Vec<[f32; 4]> = s.plano.chunks(4).map(dec).collect();
    let tecto = t.paint.brush.traco.tecto;
    let v = plano
        .iter()
        .map(|p| if tecto > 0.0 { p[3].min(tecto) } else { p[3] })
        .collect();
    let tau = s.fronteira();
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    Traco {
        tela,
        base,
        plano,
        v,
        tau,
    }
}

/// ⭐ **O oráculo**: por texel de dentro, a distância (centro a centro) ao texel de fora mais próximo, por força
/// bruta num raio `alcance`; `INFINITY` além dele, `0` fora.
fn distancias(v: &[f32], tau: f32, alcance: i32) -> Vec<f32> {
    let (w, h) = (W as i32, H as i32);
    (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            if v[i as usize] <= tau {
                return 0.0;
            }
            let mut best = i32::MAX;
            for dy in -alcance..=alcance {
                for dx in -alcance..=alcance {
                    let (qx, qy) = (x + dx, y + dy);
                    if qx >= 0 && qy >= 0 && qx < w && qy < h && v[(qy * w + qx) as usize] <= tau {
                        best = best.min(dx * dx + dy * dy);
                    }
                }
            }
            if best == i32::MAX {
                f32::INFINITY
            } else {
                (best as f32).sqrt()
            }
        })
        .collect()
}

/// A cor que o texel `i` tem com o termo da orla `e`.
fn com_e(t: &PainterTool, tr: &Traco, i: usize, e: f32) -> [f32; 3] {
    let b = &t.paint.brush;
    let o = b.traco.compoe(
        b.blend,
        b.blend_sobre_vazio,
        b.pigmento(),
        tr.base[i],
        tr.plano[i],
        e,
    );
    [o[0] * 255.0, o[1] * 255.0, o[2] * 255.0]
}

fn tela_rgb(tr: &Traco, i: usize) -> [f32; 3] {
    [0, 1, 2].map(|k| f32::from(tr.tela[i * 4 + k]))
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0, f32::max)
}

/// O termo `e` que a tela mostra no texel `i` (entre a cor de `e = 0` e a de `e = 1`), se as duas se distinguem.
fn e_medido(t: &PainterTool, tr: &Traco, i: usize) -> Option<f32> {
    let (c0, c1, c) = (com_e(t, tr, i, 0.0), com_e(t, tr, i, 1.0), tela_rgb(tr, i));
    let k = (0..3).max_by(|&a, &b| (c1[a] - c0[a]).abs().total_cmp(&(c1[b] - c0[b]).abs()))?;
    let den = c1[k] - c0[k];
    (den.abs() >= 12.0).then(|| (c[k] - c0[k]) / den)
}

type Monta = fn() -> PainterTool;
/// Um caminho do rato.
type Caminho = fn() -> Vec<[f32; 2]>;

/// ⭐ **(a) Fundo do traço = sem orla**: no Glaze Round de fábrica (o Z que se cruza e o borrão rabiscado, Wet e
/// Burnt 0,8), todo texel a mais de `W + √2` do contorno visível tem a cor de `e = 0` — sem linhas finas nas
/// sobreposições, sem placas; e onde o tecto tapa (a sobreposição) a cor é UMA, nunca mais clara que uma passagem.
#[test]
fn o_fundo_do_traco_nao_tem_orla() {
    let casos: [(&str, Monta, Caminho); 6] = [
        (
            "Z · Wet 0,8",
            || glaze_round(|t| t.set_traco_molhada(0.8)),
            z,
        ),
        (
            "Z · Burnt 0,8",
            || glaze_round(|t| t.set_traco_queimada(0.8)),
            z,
        ),
        (
            "Z · Wet + Burnt 1",
            || {
                glaze_round(|t| {
                    t.set_traco_molhada(1.0);
                    t.set_traco_queimada(1.0);
                })
            },
            z,
        ),
        (
            "borrão · Wet 0,8",
            || glaze_round(|t| t.set_traco_molhada(0.8)),
            rabisco,
        ),
        (
            "borrão · Burnt 0,8",
            || glaze_round(|t| t.set_traco_queimada(0.8)),
            rabisco,
        ),
        (
            "borrão macio · Wet 0,8",
            || {
                padrao(|t| {
                    t.paint.brush.hardness = 0.2;
                    t.set_traco_molhada(0.8);
                })
            },
            rabisco,
        ),
    ];
    for (nome, mk, pts) in casos {
        let mut t = mk();
        let largura = t.paint.brush.traco.largura;
        let tr = traco(&mut t, &pts());
        let d = distancias(&tr.v, tr.tau, largura.ceil() as i32 + 3);
        let fundo_lim = largura + std::f32::consts::SQRT_2 + 0.01;
        let (mut fundos, mut erradas, mut pior) = (0, 0, 0.0f32);
        let tecto = t.paint.brush.traco.tecto;
        let liso = fundo();
        let mut tapados: Vec<[f32; 3]> = Vec::new();
        for i in 0..W * H {
            if d[i] <= fundo_lim {
                continue;
            }
            fundos += 1;
            let err = dist(tela_rgb(&tr, i), com_e(&t, &tr, i, 0.0));
            pior = pior.max(err);
            erradas += usize::from(err > 1.5);
            if tecto > 0.0 && tr.v[i] >= tecto - 1e-6 && liso[i * 4] == 40 {
                tapados.push(tela_rgb(&tr, i));
            }
        }
        assert!(fundos > 2_000, "{nome}: o traço não tem fundo ({fundos})");
        assert_eq!(
            erradas, 0,
            "{nome}: {erradas} de {fundos} texels do fundo têm orla (pior |Δ| {pior})"
        );
        if let Some(c) = tapados.first() {
            let fora = tapados.iter().filter(|q| dist(**q, *c) > 1.5).count();
            assert_eq!(fora, 0, "{nome}: a sobreposição tapada não é uma cor só");
        }
    }
}

/// ⭐ **(b) A orla existe em toda a borda visível**, com qualquer Strength (Accumulate desligado) e com Flow baixo:
/// cada texel de dentro encostado a um de fora (`D ≤ 1`) mostra `e ≥ perfil(1) − 0,1`.
#[test]
fn a_orla_existe_em_toda_a_borda_visivel() {
    let casos: [(&str, Monta); 5] = [
        ("Strength 0,3", || {
            padrao(|t| {
                t.set_brush_strength(0.3);
                t.set_traco_molhada(0.8);
            })
        }),
        ("Strength 0,6", || padrao(|t| t.set_traco_molhada(0.8))),
        ("Strength 1", || {
            padrao(|t| {
                t.set_brush_strength(1.0);
                t.set_traco_molhada(0.8);
            })
        }),
        ("Flow 0,3", || {
            padrao(|t| {
                t.set_brush_flow(0.3);
                t.set_traco_molhada(0.8);
            })
        }),
        ("Glaze Round", || glaze_round(|t| t.set_traco_molhada(0.8))),
    ];
    for (nome, mk) in casos {
        let mut t = mk();
        let largura = t.paint.brush.traco.largura;
        let tr = traco(&mut t, &z());
        let d = distancias(&tr.v, tr.tau, 2);
        let minimo = super::traco_orla::perfil(1.0, largura) - 0.1;
        let (mut borda, mut fracas) = (0, 0);
        for (i, &di) in d.iter().enumerate() {
            if di > 0.0 && di <= 1.0 {
                let Some(e) = e_medido(&t, &tr, i) else {
                    continue;
                };
                borda += 1;
                fracas += usize::from(e < minimo);
            }
        }
        assert!(borda > 500, "{nome}: borda pequena demais ({borda})");
        assert!(
            fracas * 100 <= borda,
            "{nome}: {fracas} de {borda} texels da borda sem orla (e < {minimo})"
        );
    }
}

/// ⭐ **(e) O perfil é monótono e tem a largura W**: num recto largo e horizontal, `e` ao longo da coluna do meio,
/// contra `perfil(d)` com `d` a distância à fronteira SUB-PÍXEL da coluna (o corte linear de `v = τ`).
#[test]
fn o_perfil_e_monotono_e_tem_a_largura() {
    for largura in [4.0, 7.0, 16.0] {
        let mut t = padrao(|t| {
            t.set_brush_size_px(90.0);
            t.set_brush_strength(1.0);
            t.paint.brush.hardness = 0.6;
            t.set_traco_molhada(0.8);
            t.set_traco_largura(largura);
        });
        let pts = caminho(&[[20.0, 120.0], [300.0, 120.0]]);
        let tr = traco(&mut t, &pts);
        let tau = tr.tau;
        let x = 160;
        let col: Vec<f32> = (0..H).map(|y| tr.v[y * W + x]).collect();
        let y0 = (1..H).find(|&y| col[y] > tau).expect("o recto");
        let yc = (y0 - 1) as f32 + (tau - col[y0 - 1]) / (col[y0] - col[y0 - 1]);
        let mut antes = 1.0f32;
        let mut pior = 0.0f32;
        for y in y0..120 {
            let Some(e) = e_medido(&t, &tr, y * W + x) else {
                continue;
            };
            let esperado = super::traco_orla::perfil(y as f32 - yc, largura);
            pior = pior.max((e - esperado).abs());
            assert!(
                e <= antes + 0.03,
                "W {largura}: o perfil sobe em y {y} ({antes} → {e})"
            );
            antes = antes.min(e);
            if y as f32 - yc >= largura + 0.5 {
                assert!(e.abs() <= 0.02, "W {largura}: orla além de W em y {y}: {e}");
            }
        }
        assert!(
            pior <= 0.05,
            "W {largura}: o perfil foge do analítico em {pior}"
        );
    }
}

/// `(máx |Δ|, texels > 2)` entre duas telas.
fn diferenca(a: &[u8], b: &[u8]) -> (u8, usize) {
    let mut max = 0u8;
    let mut n = 0;
    for (p, q) in a.chunks(4).zip(b.chunks(4)) {
        let d = (0..4).map(|k| p[k].abs_diff(q[k])).max().unwrap();
        max = max.max(d);
        n += usize::from(d > 2);
    }
    (max, n)
}

/// ⭐ **(d) Vivo = passagem única**, com lotes de tamanhos variados e o borrão muito maior que a janela antiga.
#[test]
fn o_vivo_e_a_passagem_unica_com_lotes_variados() {
    let casos: [(&str, Monta, Caminho); 3] = [
        (
            "Z · Wet + Burnt 0,8",
            || {
                glaze_round(|t| {
                    t.set_traco_molhada(0.8);
                    t.set_traco_queimada(0.8);
                })
            },
            z,
        ),
        (
            "borrão · Wet 0,8",
            || glaze_round(|t| t.set_traco_molhada(0.8)),
            rabisco,
        ),
        (
            "borrão macio · Burnt 1 · W 20",
            || {
                padrao(|t| {
                    t.paint.brush.hardness = 0.2;
                    t.set_traco_queimada(1.0);
                    t.set_traco_largura(20.0);
                })
            },
            rabisco,
        ),
    ];
    for (nome, mk, pts) in casos {
        let mut t = mk();
        let pts = pts();
        pinta(&mut t, &pts, |k| [1, 2, 5, 3, 8, 1, 13][k % 7]);
        let vivo = t.canvas_rgba.to_vec();
        let mut s = t.paint.plano_do_traco.clone().expect("plano");
        let tudo = super::Region {
            x: 0,
            y: 0,
            w: W as u32,
            h: H as u32,
        };
        t.compoe_o_plano(&mut s, tudo);
        let (max, n) = diferenca(&vivo, &t.canvas_rgba);
        assert!(
            max <= 1,
            "{nome}: vivo × passagem única máx |Δ| {max}, {n} texels > 2"
        );
    }
}

/// ⛔ Sonda + fotos (`PH2D_FOTO_DIR`, `PH2D_FOTO_SUFIXO`): as duas cenas do dono — o Glaze Round no Z com Wet e
/// Burnt, e o borrão macio rabiscado com Wet — inteiras (×2) e ampliadas (×5).
#[test]
#[ignore = "sonda: escreve PNG em PH2D_FOTO_DIR"]
fn foto_das_cenas_do_dono() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let sufixo = std::env::var("PH2D_FOTO_SUFIXO").unwrap_or_else(|_| "antes".into());
    let cenas: [(&str, Monta, Caminho, [usize; 2]); 4] = [
        (
            "glaze_z_wet08",
            || glaze_round(|t| t.set_traco_molhada(0.8)),
            z,
            [120, 60],
        ),
        (
            "glaze_z_burnt08",
            || glaze_round(|t| t.set_traco_queimada(0.8)),
            z,
            [120, 60],
        ),
        (
            "glaze_borrao_wet08_burnt08",
            || {
                glaze_round(|t| {
                    t.set_traco_molhada(0.8);
                    t.set_traco_queimada(0.8);
                })
            },
            rabisco,
            [40, 40],
        ),
        (
            "macio_borrao_wet08",
            || {
                padrao(|t| {
                    t.set_brush_size_px(60.0);
                    t.paint.brush.hardness = 0.2;
                    t.set_traco_molhada(0.8);
                })
            },
            rabisco,
            [40, 40],
        ),
    ];
    for (nome, mk, pts, [x0, y0]) in cenas {
        let mut t = mk();
        let pts = pts();
        pinta(&mut t, &pts, |_| 4);
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        let px = t.canvas_rgba.to_vec();
        let grava = |sub: &str, x0: usize, y0: usize, lw: usize, lh: usize, zoom: usize| {
            let mut c = super::tests::look_watercolor_arc::Canvas::new(lw * zoom, lh * zoom);
            for y in 0..lh * zoom {
                for x in 0..lw * zoom {
                    let i = ((y0 + y / zoom) * W + x0 + x / zoom) * 4;
                    c.set(x, y, [px[i], px[i + 1], px[i + 2]]);
                }
            }
            let p = std::path::Path::new(&dir).join(format!("{nome}_{sub}_{sufixo}.png"));
            super::tests::look_watercolor_arc::write_png(&p, &c).expect("PNG");
        };
        grava("tela", 0, 0, W, H, 2);
        grava("zoom_x5", x0, y0, 100, 80, 5);
    }
}

/// ⛔ Sonda do custo por quadro (`--profile smoke`): tela 1600×1000, pincel de 300 px, um rabisco ao passo do rato
/// com um quadro a cada 4 eventos; o tecto só (sem orla) contra Wet com `W` 7 e 48, intercalados, o MÍNIMO de 3.
#[test]
#[ignore = "sonda: mede o custo por quadro"]
fn mede_o_custo_da_orla() {
    let (tw, th) = (1600u32, 1000u32);
    let nos: Vec<[f32; 2]> = (0..10)
        .map(|k| {
            [
                if k % 2 == 0 { 250.0 } else { 1350.0 },
                200.0 + 60.0 * k as f32,
            ]
        })
        .collect();
    let pts = caminho(&nos);
    let variantes: [(&str, Option<f32>); 3] = [
        ("só tecto", None),
        ("Wet W 7", Some(7.0)),
        ("Wet W 48", Some(48.0)),
    ];
    let mut melhor = [(f64::MAX, f64::MAX, f64::MAX); 3];
    for _ in 0..3 {
        for (k, (_, w)) in variantes.iter().enumerate() {
            let mut t = PainterTool::default();
            t.set_source(vec![200u8; (tw * th * 4) as usize], tw, th);
            t.set_paint_media(crate::PaintMedia::Digital);
            t.set_brush_size_px(300.0);
            t.set_brush_strength(0.6);
            t.set_traco_tecto(0.9);
            if let Some(w) = w {
                t.set_traco_molhada(0.8);
                t.set_traco_largura(*w);
            }
            let mut quadros = Vec::new();
            t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
            let inicio = std::time::Instant::now();
            let mut q = std::time::Instant::now();
            for (i, p) in pts[1..].iter().enumerate() {
                t.on_canvas_pointer(cp(*p, PointerPhase::Move));
                if i % 4 == 3 {
                    t.paint_tick(1.0 / 60.0);
                    quadros.push(q.elapsed().as_secs_f64() * 1e3);
                    q = std::time::Instant::now();
                }
            }
            t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
            let total = inicio.elapsed().as_secs_f64() * 1e3;
            quadros.sort_by(f64::total_cmp);
            let ativos: Vec<f64> = quadros.iter().copied().filter(|&q| q > 0.05).collect();
            let med = ativos[ativos.len() / 2];
            let max = *quadros.last().unwrap();
            let m = &mut melhor[k];
            *m = (m.0.min(med), m.1.min(max), m.2.min(total));
        }
    }
    for (k, (nome, _)) in variantes.iter().enumerate() {
        let (med, max, total) = melhor[k];
        eprintln!(
            "{nome}: mediana dos quadros que pintam {med:.2} ms · pior {max:.2} ms · traço {total:.0} ms ({} eventos)",
            pts.len()
        );
    }
}

/// Os pincéis do dono para o traço que VOLTA: o Glaze Round de fábrica e o Digital por omissão (Accumulate
/// ligado e desligado, Strength 1 e 0,6, Flow 1 e 0,3), cada um com Wet 0,8 e com Burnt 0,8.
fn pinceis_da_volta() -> Vec<(String, PainterTool)> {
    let mut out = Vec::new();
    for (orla, liga) in [
        (
            "Wet 0,8",
            PainterTool::set_traco_molhada as fn(&mut PainterTool, f32),
        ),
        ("Burnt 0,8", PainterTool::set_traco_queimada),
    ] {
        out.push((
            format!("Glaze Round · {orla}"),
            glaze_round(|t| liga(t, 0.8)),
        ));
        for acum in [false, true] {
            for forca in [1.0, 0.6] {
                for flow in [1.0, 0.3] {
                    let t = padrao(|t| {
                        t.paint.brush.accumulate = acum;
                        t.set_brush_strength(forca);
                        t.set_brush_flow(flow);
                        liga(t, 0.8);
                    });
                    out.push((
                        format!("Digital · Acc {acum} · S {forca} · F {flow} · {orla}"),
                        t,
                    ));
                }
            }
        }
    }
    out
}

/// A posição (px, sub-píxel) em que `e` desce de `½` na coluna `x`, de cima para dentro (`None` sem orla à vista).
fn borda_na_coluna(t: &PainterTool, tr: &Traco, x: usize, y0: usize, y1: usize) -> Option<f32> {
    let mut antes: Option<(usize, f32)> = None;
    for y in y0..y1 {
        let Some(e) = e_medido(t, tr, y * W + x) else {
            continue;
        };
        if let Some((ya, ea)) = antes
            && ea >= 0.5
            && e < 0.5
        {
            return Some(ya as f32 + (ea - 0.5) / (ea - e) * (y - ya) as f32);
        }
        antes = Some((y, e));
    }
    None
}

/// O retrato do traço ABERTO agora (sem levantar).
fn retrato(t: &PainterTool) -> Traco {
    let s = t
        .paint
        .plano_do_traco
        .as_ref()
        .expect("o traço pinta no plano");
    let dec = |c: &[u8]| [0, 1, 2, 3].map(|k| f32::from(c[k]) / 255.0);
    let plano: Vec<[f32; 4]> = s.plano.chunks(4).map(dec).collect();
    let tecto = t.paint.brush.traco.tecto;
    Traco {
        tela: t.canvas_rgba.to_vec(),
        base: s.base.chunks(4).map(dec).collect(),
        v: plano
            .iter()
            .map(|p| if tecto > 0.0 { p[3].min(tecto) } else { p[3] })
            .collect(),
        plano,
        tau: s.fronteira(),
    }
}

/// ⭐ **(f) O contorno da primeira perna não se mexe quando o traço volta** (smoke do dono depois de `3574ae0122`:
/// *«no segundo movimento (quando o traço muda de direção) faz os edges mudarem de forma estranha, se
/// encolhendo»*): um traço para a direita que volta para a esquerda por cima de si; na faixa da 1.ª perna onde a
/// 2.ª não deposita nada, a tela antes e depois da volta é a MESMA (`|Δ| ≤ 1`) e a borda (`e = ½`) não anda mais
/// de 0,5 px.
#[test]
fn o_contorno_da_primeira_perna_nao_se_mexe_quando_o_traco_volta() {
    let ida = caminho(&[[30.0, 100.0], [290.0, 100.0]]);
    let volta = caminho(&[[290.0, 100.0], [150.0, 110.0]]);
    // A faixa só da 1.ª perna: x < 150 − raio (20) − W e a célula (≥ 10).
    let (x0, x1) = (40usize, 110usize);
    let mut falhas = Vec::new();
    for (nome, mut t) in pinceis_da_volta() {
        pinta(&mut t, &ida, |_| 4);
        let antes = retrato(&t);
        for p in &volta[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        let depois = retrato(&t);
        t.on_canvas_pointer(cp(*volta.last().unwrap(), PointerPhase::Up));
        let maxv = |tr: &Traco| tr.v.iter().copied().fold(0.0, f32::max);
        // A fronteira é o nível de UMA passagem: ½ do máximo da 1.ª perna, e a volta não o sobe.
        let uma = 0.5 * maxv(&antes);
        if (antes.tau - uma).abs() > 0.5 / 255.0 || depois.tau.to_bits() != antes.tau.to_bits() {
            falhas.push(format!(
                "{nome}: τ {:.4} → {:.4}, ½ da 1.ª perna {uma:.4}",
                antes.tau, depois.tau
            ));
            continue;
        }
        let (mut pior, mut mudados) = (0u8, 0usize);
        for y in 0..H {
            for x in x0..x1 {
                let i = (y * W + x) * 4;
                let d = (0..4)
                    .map(|k| antes.tela[i + k].abs_diff(depois.tela[i + k]))
                    .max()
                    .unwrap();
                pior = pior.max(d);
                mudados += usize::from(d > 1);
            }
        }
        let xc = (x0 + x1) / 2;
        let (ba, bd) = (
            borda_na_coluna(&t, &antes, xc, 60, 100),
            borda_na_coluna(&t, &depois, xc, 60, 100),
        );
        let anda = match (ba, bd) {
            (Some(a), Some(b)) => (b - a).abs(),
            (None, None) => 0.0,
            _ => f32::INFINITY,
        };
        eprintln!(
            "{nome}: max v {:.3} → {:.3} · borda {ba:?} → {bd:?} (anda {anda:.2} px) · tela máx |Δ| {pior}, {mudados} texels",
            maxv(&antes),
            maxv(&depois)
        );
        if pior > 1 || anda > 0.5 {
            falhas.push(nome.clone());
        }
    }
    assert!(falhas.is_empty(), "a 1.ª perna mexe: {falhas:#?}");
}

/// ⛔ Sonda + fotos (`PH2D_FOTO_DIR`, `PH2D_FOTO_SUFIXO`): o traço que VOLTA por cima de si, o quadro na volta
/// (`1_na_volta`) e o quadro depois de voltar (`2_voltou`), ×2.
#[test]
#[ignore = "sonda: escreve PNG em PH2D_FOTO_DIR"]
fn foto_do_traco_que_volta() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let sufixo = std::env::var("PH2D_FOTO_SUFIXO").unwrap_or_else(|_| "antes".into());
    let ida = caminho(&[[30.0, 100.0], [290.0, 100.0]]);
    let volta = caminho(&[[290.0, 100.0], [150.0, 110.0]]);
    let cenas: [(&str, Monta); 3] = [
        ("glaze_wet08", || glaze_round(|t| t.set_traco_molhada(0.8))),
        ("glaze_burnt08", || {
            glaze_round(|t| t.set_traco_queimada(0.8))
        }),
        ("digital_acc_s06_f03_wet08", || {
            padrao(|t| {
                t.paint.brush.accumulate = true;
                t.set_brush_flow(0.3);
                t.set_traco_molhada(0.8);
            })
        }),
    ];
    for (nome, mk) in cenas {
        let mut t = mk();
        let grava = |t: &PainterTool, sub: &str| {
            let px = &t.canvas_rgba;
            let (y0, lh, zoom) = (50, 100, 2);
            let mut c = super::tests::look_watercolor_arc::Canvas::new(W * zoom, lh * zoom);
            for y in 0..lh * zoom {
                for x in 0..W * zoom {
                    let i = ((y0 + y / zoom) * W + x / zoom) * 4;
                    c.set(x, y, [px[i], px[i + 1], px[i + 2]]);
                }
            }
            let p = std::path::Path::new(&dir).join(format!("{nome}_{sub}_{sufixo}.png"));
            super::tests::look_watercolor_arc::write_png(&p, &c).expect("PNG");
        };
        pinta(&mut t, &ida, |_| 4);
        grava(&t, "1_na_volta");
        for p in &volta[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        grava(&t, "2_voltou");
        t.on_canvas_pointer(cp(*volta.last().unwrap(), PointerPhase::Up));
    }
}
