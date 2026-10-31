//! **Amarelo sobre azul SECO não deixa borda branca** (queixa do dono 31/10: «amarelo sobre azul seco fica com bordas
//! brancas», a família do BUGS #50 — a orla esbranquiçada). Cada preset de fábrica pinta uma faixa azul, a tela seca
//! (Dry canvas), e o MESMO pincel cruza-a com amarelo; na imagem mostrada (sobre o papel branco), nenhum píxel da faixa
//! azul fica mais claro que AMBAS as tintas, canal a canal.

use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const W: usize = 160;
const H: usize = 120;
const AZUL: [u8; 3] = [40, 90, 220];
const AMARELO: [u8; 3] = [245, 205, 25];

fn presets() -> Vec<(String, std::path::PathBuf)> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut v = Vec::new();
    let mut fila = vec![raiz.clone()];
    while let Some(d) = fila.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                fila.push(p);
            } else if p.to_string_lossy().ends_with(".ph2d-brush.png") {
                let nome = p
                    .strip_prefix(&raiz)
                    .unwrap()
                    .to_string_lossy()
                    .replace(".ph2d-brush.png", "");
                v.push((nome, p));
            }
        }
    }
    v.sort();
    v
}

fn ponteiro(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A imagem mostrada sobre o papel branco, RGB.
fn mostrada(t: &mut PainterTool) -> Vec<[u8; 3]> {
    t.invalidate_composite();
    let (px, _, _) = t.take_preview_arc().expect("a imagem mostrada");
    px.chunks(4)
        .map(|q| {
            let a = f32::from(q[3]) / 255.0;
            let m = |c: u8| (f32::from(c) * a + 255.0 * (1.0 - a)).round() as u8;
            [m(q[0]), m(q[1]), m(q[2])]
        })
        .collect()
}

/// `(antes, depois, orla)` de [`cena`].
pub(super) type CenaBorda = (Vec<[u8; 3]>, Vec<[u8; 3]>, usize);

/// `(antes, depois, orla)`: a faixa azul seca, a cena com o amarelo assentado e os píxeis da faixa mais claros que
/// ambas as tintas (canal a canal, +4). `None` = o preset não pinta no Wet.
pub(super) fn cena(
    caminho: &std::path::Path,
    relevo: bool,
    extra: &[(ph2d_wet_paint::tuning::Knob, f64)],
) -> Option<CenaBorda> {
    let p = ph2d_brush_presets::ler(&std::fs::read(caminho).unwrap()).unwrap();
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_wet_relogio_fixo(true);
    t.set_paint_tool_mode("wetpaint");
    let arma = |t: &mut PainterTool| {
        if t.preset_aplicar(&p).is_err() {
            return false;
        }
        t.set_brush_size_px(24.0);
        for &(k, v) in extra {
            t.paint.wetpaint.knobs.set(k, v);
        }
        if !relevo {
            t.paint
                .wetpaint
                .knobs
                .set(ph2d_wet_paint::tuning::Knob::Emboss, 0.0);
        }
        true
    };
    if !arma(&mut t) {
        return None;
    }
    let tick = |t: &mut PainterTool| t.on_tick(1000.0 / 60.0);
    t.set_brush_color_srgb8(AZUL);
    for y in [44.0f32, 56.0] {
        t.on_canvas_pointer(ponteiro(20.0, y, PointerPhase::Down));
        for k in 1..=30 {
            t.on_canvas_pointer(ponteiro(20.0 + 4.0 * k as f32, y, PointerPhase::Move));
            tick(&mut t);
        }
        t.on_canvas_pointer(ponteiro(140.0, y, PointerPhase::Up));
        for _ in 0..20 {
            tick(&mut t);
        }
    }
    t.wetpaint_dry_canvas();
    for _ in 0..10 {
        tick(&mut t);
    }
    let antes = mostrada(&mut t);
    assert!(arma(&mut t));
    t.set_brush_color_srgb8(AMARELO);
    for x in [60.0f32, 100.0] {
        t.on_canvas_pointer(ponteiro(x, 15.0, PointerPhase::Down));
        for k in 1..=22 {
            t.on_canvas_pointer(ponteiro(x, 15.0 + 4.0 * k as f32, PointerPhase::Move));
            tick(&mut t);
        }
        t.on_canvas_pointer(ponteiro(x, 105.0, PointerPhase::Up));
        for _ in 0..20 {
            tick(&mut t);
        }
    }
    for _ in 0..300 {
        tick(&mut t);
    }
    let depois = mostrada(&mut t);
    let teto: [u8; 3] = std::array::from_fn(|k| AZUL[k].max(AMARELO[k]));
    let azul = |q: &[u8; 3]| i16::from(q[2]) > i16::from(q[0]) + 30;
    let orla = (0..W * H)
        .filter(|&i| {
            azul(&antes[i]) && (0..3).all(|k| depois[i][k] > teto[k].saturating_add(4).min(250))
        })
        .count();
    Some((antes, depois, orla))
}

fn grava(p: &std::path::Path, px: &[[u8; 3]]) {
    use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
    write_png(
        p,
        &Canvas {
            w: W,
            h: H,
            px: px.to_vec(),
        },
    )
    .expect("png");
}

/// SONDA — a fábrica toda (`PH2D_SONDA_DIR` grava os PNG; `PH2D_SEM_RELEVO=1` sem o Emboss; `PH2D_SONDA_OPACO=1`).
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_borda_branca_na_fabrica() {
    let relevo = std::env::var("PH2D_SEM_RELEVO").is_err();
    let dir = std::env::var("PH2D_SONDA_DIR").ok();
    let filtro = std::env::var("PH2D_SONDA_FILTRO").ok();
    // `PH2D_SONDA_OPACO=1`: a tinta opaca ligada em todos (o «depois» antes de regerar os presets).
    let extra: Vec<_> = std::env::var("PH2D_SONDA_OPACO")
        .ok()
        .map(|_| (ph2d_wet_paint::tuning::Knob::Opaque, 1.0))
        .into_iter()
        .collect();
    let extra = &extra;
    let lista: Vec<_> = presets()
        .into_iter()
        .filter(|(n, _)| filtro.as_ref().is_none_or(|f| n.contains(f.as_str())))
        .collect();
    let fatias: Vec<_> = lista.chunks(lista.len().div_ceil(8).max(1)).collect();
    let mut linhas: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = fatias
            .iter()
            .map(|f| {
                let dir = dir.clone();
                s.spawn(move || {
                    f.iter()
                        .map(|(n, p)| match cena(p, relevo, extra) {
                            None => format!("     -  {n}"),
                            Some((_, depois, orla)) => {
                                if let Some(d) = &dir {
                                    let f = n.replace(['/', ' '], "_");
                                    grava(
                                        &std::path::Path::new(d).join(format!("{f}.png")),
                                        &depois,
                                    );
                                }
                                format!("{orla:>6}  {n}")
                            }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        h.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    linhas.sort();
    linhas.reverse();
    let txt = format!("  orla  preset (relevo {relevo})\n{}\n", linhas.join("\n"));
    eprintln!("{txt}");
    if let Some(d) = dir {
        std::fs::write(format!("{d}/orla_relevo_{relevo}.txt"), txt).unwrap();
    }
}

/// Os píxeis da faixa azul (`x` 36..124, as linhas onde a faixa é azul) que NENHUMA mistura das duas tintas explica:
/// mais claros, em todos os canais, que o ponto mais próximo do segmento entre o azul seco da linha (mediana de
/// `x` 124..136, fora do amarelo) e o amarelo sobre o papel (mediana de `x` 54..67 × `y` 92..101) — `> 20` níveis.
fn brancos_a_mais(px: &[[u8; 3]]) -> usize {
    let med = |v: Vec<[u8; 3]>| -> [f64; 3] {
        std::array::from_fn(|k| {
            let mut c: Vec<u8> = v.iter().map(|p| p[k]).collect();
            c.sort_unstable();
            f64::from(c[c.len() / 2])
        })
    };
    let am = med((54..67)
        .flat_map(|x| (92..101).map(move |y| px[y * W + x]))
        .collect());
    let mut n = 0;
    for y in 36..66 {
        let az = med((124..136).map(|x| px[y * W + x]).collect());
        if az[2] <= az[0] + 30.0 {
            continue;
        }
        for x in 36..124 {
            let c = px[y * W + x];
            let e = (0..=50)
                .map(|i| {
                    let t = f64::from(i) / 50.0;
                    (0..3)
                        .map(|k| f64::from(c[k]) - (t * am[k] + (1.0 - t) * az[k]))
                        .fold(f64::MIN, f64::max)
                })
                .fold(f64::MAX, f64::min);
            if e > 20.0 {
                n += 1;
            }
        }
    }
    n
}

/// O que a régua do [`a_tinta_opaca_mistura_como_pigmento`] lê de uma cena: `(brancos, cinzentos, miolo, verdes)`.
/// Cinzento = um píxel da faixa azul que a tinta nova MUDOU (> 10 níveis contra o azul seco sozinho) com a saturação
/// (`(max − min) ÷ max`) abaixo de [`SATURACAO_CINZENTA`]; verde = mudado, com `G > R + 10` e `G > B + 40`; o miolo = a mediana de
/// `x` 54..67 × `y` 46..55 (o 1.º traço amarelo sobre o meio da faixa).
fn regua_opaca(antes: &[[u8; 3]], depois: &[[u8; 3]]) -> (usize, usize, [u8; 3], usize) {
    let (mut cinza, mut verde) = (0, 0);
    for y in 36..66 {
        for x in 36..124 {
            let i = y * W + x;
            let (a, d) = (antes[i], depois[i]);
            let azul = i16::from(a[2]) > i16::from(a[0]) + 30;
            let mudou = (0..3).any(|k| (i16::from(a[k]) - i16::from(d[k])).abs() > 10);
            if !azul || !mudou {
                continue;
            }
            let (mx, mn) = (*d.iter().max().unwrap(), *d.iter().min().unwrap());
            if f64::from(mx - mn) < SATURACAO_CINZENTA * f64::from(mx) {
                cinza += 1;
            }
            if i16::from(d[1]) > i16::from(d[0]) + 10 && i16::from(d[1]) > i16::from(d[2]) + 40 {
                verde += 1;
            }
        }
    }
    let miolo: [u8; 3] = std::array::from_fn(|k| {
        let mut c: Vec<u8> = (54..67)
            .flat_map(|x| (46..55).map(move |y| depois[y * W + x][k]))
            .collect();
        c.sort_unstable();
        c[c.len() / 2]
    });
    (brancos_a_mais(depois), cinza, miolo, verde)
}

/// A saturação (`(max − min) ÷ max`) abaixo da qual um píxel mudado é CINZENTO. Medido 31/10: o amarelo (245,205,25)
/// 0,90, o azul (40,90,220) 0,82; a orla fina da mistura subtractiva é um verde-azulado ESCURO e ainda saturado —
/// (66,110,111) 0,41, o pior da Gouache Flat opaca —, o miolo (222,204,42) 0,81; o cinzento do segmento RGB
/// (194,198,200) 0,03 e o branco a mais da aguarela (239,205,138) 0,42 no miolo pêssego. (O croma `max − min` não serve:
/// 45 no verde escuro, que é o que a tinta real faz.)
const SATURACAO_CINZENTA: f64 = 0.25;

/// ⭐ **A tinta OPACA mistura-se como PIGMENTO com a de baixo** (dono 31/10: «amarelo sobre azul seco fica com bordas
/// brancas»; `Knob::Opaque`, a lei subtractiva em `assenta/ponto.rs` `mistura_opaca`). A Gouache Flat (a orla
/// `[194,198,200]` e o miolo pêssego) e a Gouache Opaque (com o `dryCover`): com `opaque` 0 (a lei da aguarela — o
/// CONTROLO) há branco a mais; com `opaque` 1: zero brancos, zero cinzentos ([`SATURACAO_CINZENTA`]), o miolo espesso
/// AMARELO (R e G > 180, B < 100) e a orla fina esverdeada (verdes > 0, onde o pincel esbate). Medido 31/10
/// (brancos, cinzentos, miolo, verdes): Flat 0 → (173, 172, (239,205,138), 0), 1 → (0, 0, (222,204,42), 232); Opaque
/// 0 → (66, 0, …), 1 → (0, 0, (246,208,40), 0) — com o `dryCover` ligado: a mistura cobre também o papel que ele afasta.
#[test]
fn a_tinta_opaca_mistura_como_pigmento() {
    use ph2d_wet_paint::tuning::Knob;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes/WetPainter");
    let casos = [
        // `true` = tem orla FINA (a Flat esbate; a Opaque tem a borda firme, `gate` 0 e dureza 0,95 — sem orla).
        (
            "Gouache Flat",
            raiz.join("Watercolor/Gouache/Gouache Flat.ph2d-brush.png"),
            true,
        ),
        (
            "Gouache Opaque",
            raiz.join("Watercolor/Gouache/Gouache Opaque.ph2d-brush.png"),
            false,
        ),
    ];
    let linhas: Vec<_> = std::thread::scope(|s| {
        let h: Vec<_> = casos
            .iter()
            .flat_map(|(n, p, fina)| {
                [0.0, 1.0].map(|o| {
                    s.spawn(move || {
                        let (antes, depois, _) =
                            cena(p, true, &[(Knob::Opaque, o)]).expect("o preset pinta no Wet");
                        (
                            n.to_string(),
                            o as usize,
                            *fina,
                            regua_opaca(&antes, &depois),
                        )
                    })
                })
            })
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    eprintln!("{linhas:?}");
    let controlo: usize = linhas.iter().filter(|l| l.1 == 0).map(|l| l.3.0).sum();
    assert!(
        controlo > 0,
        "controlo: com opaque 0 já não há branco a mais — a régua não vê o defeito"
    );
    for (n, o, fina, (brancos, cinza, miolo, verde)) in &linhas {
        if *o == 1 {
            assert_eq!(
                *brancos, 0,
                "{n} opaca: {brancos} px de branco que nenhuma mistura explica"
            );
            assert_eq!(
                *cinza, 0,
                "{n} opaca: {cinza} px cinzentos (saturação < {SATURACAO_CINZENTA})"
            );
            assert!(
                miolo[0] > 180 && miolo[1] > 180 && miolo[2] < 100,
                "{n} opaca: o miolo espesso {miolo:?} não é amarelo"
            );
            assert!(!fina || *verde > 0, "{n} opaca: a orla fina não esverdeia");
        }
    }
}
