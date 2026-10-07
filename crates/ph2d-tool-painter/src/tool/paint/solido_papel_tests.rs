//! **O Solid com papel escuro** (smoke do dono 2026-10-06, com fotos: *«solid com papel escuro não
//! preenche corretamente»* — o miolo mais escuro que o traço, a borda do miolo em escada, e buracos
//! triangulares onde os traços se cruzam).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, Tool};

const LADO: usize = 160;
const PAPEL: [u8; 3] = [64, 28, 30];
const TINTA: [u8; 3] = [220, 30, 30];

fn papel(t: &mut PainterTool) {
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
}

/// Um gesto de pontos, com um evento por ponto.
fn gesto(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(*pts.last().expect("pontos"), PointerPhase::Up));
}

/// Uma elipse em 48 pontos à volta de (80, 80).
fn elipse() -> Vec<[f32; 2]> {
    (0..=48)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let a = k as f32 / 48.0 * std::f32::consts::TAU;
            [80.0 + 55.0 * a.cos(), 80.0 + 40.0 * a.sin()]
        })
        .collect()
}

/// Um rabisco que se cruza e VOLTA para trás (sentidos opostos): um zigue-zague em laço.
fn rabisco() -> Vec<[f32; 2]> {
    let mut v = Vec::new();
    for k in 0..=120 {
        #[allow(clippy::cast_precision_loss)]
        let s = k as f32 / 120.0 * std::f32::consts::TAU * 2.0;
        v.push([80.0 + 55.0 * (s * 1.5).sin(), 80.0 + 50.0 * (s * 1.0).cos()]);
    }
    v
}

fn traco(meio: PaintMedia, s: f32, papel_antes: bool, pts: &[[f32; 2]]) -> PainterTool {
    let mut t = tool(LADO as u32, meio, 10.0);
    t.set_brush_color_srgb8(TINTA);
    t.paint.brush.strength = s;
    t.paint.brush.style_solid = true;
    if papel_antes {
        papel(&mut t);
    }
    gesto(&mut t, pts);
    if !papel_antes {
        papel(&mut t);
    }
    t
}

fn imagem(t: &mut PainterTool) -> Vec<u8> {
    t.invalidate_composite();
    t.take_preview_arc().expect("preview").0.to_vec()
}

/// SONDA — (a) o miolo e a borda do miolo sobre a camada transparente do papel; (b) os buracos de um
/// rabisco que se cruza, com papel e sobre o branco.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_solid_com_papel -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_solid_com_papel() {
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for s in [0.6f32, 1.0] {
            for papel_antes in [true, false] {
                let mut t = traco(meio, s, papel_antes, &elipse());
                let v = imagem(&mut t);
                let c = &t.canvas_rgba;
                let px = |b: &[u8], x: usize, y: usize| {
                    let i = (y * LADO + x) * 4;
                    [b[i], b[i + 1], b[i + 2], b[i + 3]]
                };
                eprintln!(
                    "{meio:?} S{s} papel {}: anel camada {:?} mostrado {:?} · miolo camada {:?} mostrado {:?}",
                    if papel_antes { "antes" } else { "depois" },
                    px(c, 135, 80),
                    px(&v, 135, 80),
                    px(c, 80, 80),
                    px(&v, 80, 80)
                );
            }
        }
    }
    // (b) buracos: píxeis do interior do rabisco que ficaram SEM tinta.
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::Watercolor,
        PaintMedia::WetPaint,
    ] {
        for (nome, com_papel) in [("com papel antes", true), ("no branco, sem papel", false)] {
            let mut t = tool(LADO as u32, meio, 10.0);
            t.set_brush_color_srgb8(TINTA);
            t.paint.brush.style_solid = true;
            if com_papel {
                papel(&mut t);
            }
            gesto(&mut t, &rabisco());
            for _ in 0..30 {
                t.paint_tick(1.0 / 30.0);
            }
            let c = &t.canvas_rgba;
            let vazio = |i: usize| {
                if com_papel {
                    c[i * 4 + 3] == 0
                } else {
                    c[i * 4..i * 4 + 3] == [255, 255, 255]
                }
            };
            let buracos = cercados(LADO, LADO, vazio);
            eprintln!("rabisco {meio:?} {nome}: {buracos} píxeis sem tinta CERCADOS pelo gesto");
        }
    }
}

/// Os píxeis vazios que NÃO se ligam à borda da tela por outros vazios (4-vizinhança): cercados.
pub(super) fn cercados(w: usize, h: usize, vazio: impl Fn(usize) -> bool) -> usize {
    onde_cercados(w, h, vazio).len()
}

/// Os índices dos píxeis vazios cercados (ver [`cercados`]).
fn onde_cercados(w: usize, h: usize, vazio: impl Fn(usize) -> bool) -> Vec<usize> {
    let mut fora = vec![false; w * h];
    let mut fila: Vec<usize> = (0..w * h)
        .filter(|&i| {
            let (x, y) = (i % w, i / w);
            (x == 0 || y == 0 || x == w - 1 || y == h - 1) && vazio(i)
        })
        .collect();
    fila.iter().for_each(|&i| fora[i] = true);
    while let Some(i) = fila.pop() {
        let (x, y) = (i % w, i / w);
        let viz = [
            (x > 0).then(|| i - 1),
            (x + 1 < w).then(|| i + 1),
            (y > 0).then(|| i - w),
            (y + 1 < h).then(|| i + w),
        ];
        for k in viz.into_iter().flatten() {
            if !fora[k] && vazio(k) {
                fora[k] = true;
                fila.push(k);
            }
        }
    }
    (0..w * h).filter(|&i| vazio(i) && !fora[i]).collect()
}

/// ⭐ **COM PAPEL, A MANCHA DO SOLID TEM A TINTA DO TRAÇO** (smoke do dono 2026-10-06, foto: o miolo
/// mais escuro que o traço, a borda dele em escada) — com o papel escolhido ANTES a camada é
/// transparente, e o `over` da mancha misturava a tinta com o preto transparente por baixo
/// (`[132, 18, 18, 153]` contra o traço `[220, 30, 30, 153]` em Strength 0,6). No Digital e no Impasto,
/// Strength 0,3 · 0,6 · 1: o miolo tem a cor e o alfa do traço, e NENHUM texel pintado da camada tem
/// outra cor que a tinta (a borda AA da mancha incluída).
#[test]
fn com_papel_a_mancha_tem_a_tinta_do_traco() {
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for s in [0.3f32, 0.6, 1.0] {
            let t = traco(meio, s, true, &elipse());
            let c = &t.canvas_rgba;
            let px = |x: usize, y: usize| {
                let i = (y * LADO + x) * 4;
                [c[i], c[i + 1], c[i + 2], c[i + 3]]
            };
            let (anel, miolo) = (px(135, 80), px(80, 80));
            assert!(
                miolo[3] > 0,
                "controlo: {meio:?} S{s}: a mancha pintou o miolo"
            );
            // A cor a ±1 (o arredondamento da mistura) e o ALFA EXACTO: é o tecto do traço.
            assert!(
                (0..3).all(|k| anel[k].abs_diff(miolo[k]) <= 1) && anel[3] == miolo[3],
                "{meio:?} S{s}: o miolo {miolo:?} não é o traço {anel:?}"
            );
            let outra = c
                .chunks(4)
                .filter(|p| p[3] > 0 && (0..3).any(|k| p[k].abs_diff(TINTA[k]) > 1))
                .count();
            assert_eq!(
                outra, 0,
                "{meio:?} S{s}: {outra} texels pintados com outra cor que a tinta (a mancha escureceu)"
            );
        }
    }
}

/// ⭐ **O `over` da mancha sobre um destino TRANSLÚCIDO é o de alfa directo, arredondado** — o píxel
/// de junção da mancha com o traço na camada transparente do papel: destino `[220, 30, 30, 100]`, a
/// mancha a `a = 128` da mesma tinta. Alfa `128 + 100·127/255 = 177,8` ⇒ `178` (truncar dava `177`), e a
/// cor fica a tinta. Num destino OPACO a conta é a de sempre, ao byte.
#[test]
fn o_over_da_mancha_num_destino_translucido() {
    use crate::tool::paint::solid_deposit::{SolidBand, blend_solid_rows};
    let corre = |destino: [u8; 4]| {
        let mut buf = destino.to_vec();
        blend_solid_rows(
            &mut buf,
            None,
            SolidBand {
                cov: &[255],
                cov_stride: 1,
                row_bytes: 4,
                x0: 0,
                cols: 1,
                rgb: [220, 30, 30],
                strength: 128,
                no_tecto: false,
            },
            false,
        );
        buf
    };
    assert_eq!(corre([220, 30, 30, 100]), [220, 30, 30, 178]);
    // Sobre OUTRA tinta translúcida (azul): cada canal pesado pelos dois alfas e arredondado —
    // vermelho `(220·32640 + 30·12700 + 22670)/45340 = 167,3` ⇒ `167` (truncar dava `166`).
    assert_eq!(corre([30, 30, 220, 100]), [167, 30, 83, 178]);
    // Opaco: `(c·a + d·(255 − a)) / 255` truncado, a lei aprovada sobre o branco —
    // `(220·128 + 255·127)/255 = 237`, `(30·128 + 255·127)/255 = 142`.
    assert_eq!(corre([255, 255, 255, 255]), [237, 142, 142, 255]);
}

/// ⭐ **O SOLID ENCHE O QUE O RABISCO CERCA** (decisão do dono 2026-10-06: *«então vamos corrigir»*,
/// nos quatro meios) — um rabisco que se cruza e volta para trás em sentidos opostos não deixa nenhum
/// píxel de BURACO (vazio cercado, os 8 vizinhos vazios) cercado pelo gesto, com papel e no branco.
/// Vermelho antes (regra não-zero, vazios cercados): Digital `3 113`, Impasto `3 022`, Aquarela
/// `3 265`, Wet Paint `4 139`.
#[test]
fn o_solid_enche_o_que_o_rabisco_cerca() {
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::Watercolor,
        PaintMedia::WetPaint,
    ] {
        for com_papel in [true, false] {
            let mut t = tool(LADO as u32, meio, 10.0);
            t.set_brush_color_srgb8(TINTA);
            t.paint.brush.style_solid = true;
            if com_papel {
                papel(&mut t);
            }
            gesto(&mut t, &rabisco());
            for _ in 0..30 {
                t.paint_tick(1.0 / 30.0);
            }
            let c = &t.canvas_rgba;
            let vazio = |i: usize| {
                if com_papel {
                    c[i * 4 + 3] == 0
                } else {
                    c[i * 4..i * 4 + 3] == [255, 255, 255]
                }
            };
            assert!(
                (0..LADO * LADO).filter(|&i| !vazio(i)).count() > 3000,
                "controlo: {meio:?}: o rabisco pintou"
            );
            // Um BURACO: vazio cercado com os 8 vizinhos vazios. ⚠️ O Wet Paint deixa `24`–`41` píxeis
            // vazios SOLTOS na orla macia do fluido (numa elipse simples também, com e sem papel —
            // `diag_os_vazios_do_wet_paint`): a textura da borda dele, não um buraco da mancha.
            let vazio8 = |i: usize| {
                let (x, y) = (i % LADO, i / LADO);
                x > 0
                    && y > 0
                    && x + 1 < LADO
                    && y + 1 < LADO
                    && (0..3).all(|dy| (0..3).all(|dx| vazio((y + dy - 1) * LADO + x + dx - 1)))
            };
            let buracos = onde_cercados(LADO, LADO, vazio)
                .into_iter()
                .filter(|&i| vazio8(i))
                .count();
            assert_eq!(
                buracos, 0,
                "{meio:?} papel {com_papel}: {buracos} píxeis de buraco cercados pelo gesto"
            );
        }
    }
}

/// SONDA — os vazios cercados do Wet Paint num rabisco e numa elipse SIMPLES (sem buraco na regra),
/// com e sem papel, e onde estão.
#[test]
#[ignore = "diagnóstico"]
fn diag_os_vazios_do_wet_paint() {
    for (nome, pts) in [("rabisco", rabisco()), ("elipse", elipse())] {
        for com_papel in [true, false] {
            let mut t = tool(LADO as u32, PaintMedia::WetPaint, 10.0);
            t.set_brush_color_srgb8(TINTA);
            t.paint.brush.style_solid = true;
            if com_papel {
                papel(&mut t);
            }
            gesto(&mut t, &pts);
            for _ in 0..30 {
                t.paint_tick(1.0 / 30.0);
            }
            let c = t.canvas_rgba.clone();
            let vazio = |i: usize| {
                if com_papel {
                    c[i * 4 + 3] == 0
                } else {
                    c[i * 4..i * 4 + 3] == [255, 255, 255]
                }
            };
            let n = cercados(LADO, LADO, vazio);
            let alguns: Vec<(usize, usize, [u8; 4])> = onde_cercados(LADO, LADO, vazio)
                .into_iter()
                .take(10)
                .map(|i| {
                    let viz = [
                        c[(i + 1) * 4],
                        c[(i + 1) * 4 + 1],
                        c[(i + 1) * 4 + 2],
                        c[(i + 1) * 4 + 3],
                    ];
                    (i % LADO, i / LADO, viz)
                })
                .collect();
            eprintln!(
                "Wet Paint {nome} papel {com_papel}: {n} cercados · onde (x, y, o vizinho da direita) {alguns:?}"
            );
        }
    }
}

/// SONDA — os laços que o Solid enche para o rabisco: quantos, quantos pontos, a área de cada um, e
/// os primeiros pontos contra o gesto.
#[test]
#[ignore = "diagnóstico"]
fn diag_os_lacos_do_rabisco() {
    let mut t = tool(LADO as u32, PaintMedia::Digital, 10.0);
    t.paint.brush.style_solid = true;
    let pts = rabisco();
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..60] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    let lacos = t.solid_fill_loops();
    for l in &lacos {
        let area: f32 = (0..l.len())
            .map(|i| {
                let (a, b) = (l[i], l[(i + 1) % l.len()]);
                a[0] * b[1] - b[0] * a[1]
            })
            .sum::<f32>()
            / 2.0;
        eprintln!(
            "laço: {} pontos · área {area:.1} · primeiros {:?}",
            l.len(),
            &l[..4.min(l.len())]
        );
    }
    eprintln!("gesto (60 eventos): primeiros {:?}", &pts[..4]);
}

/// O winding dos laços no centro do píxel `(x, y)` (raio para a direita).
fn winding(lacos: &[Vec<[f32; 2]>], x: usize, y: usize) -> i32 {
    #[allow(clippy::cast_precision_loss)]
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    let mut w = 0;
    for l in lacos {
        for i in 0..l.len() {
            let (a, b) = (l[i], l[(i + 1) % l.len()]);
            if (a[1] <= py) != (b[1] <= py) {
                let xi = a[0] + (b[0] - a[0]) * (py - a[1]) / (b[1] - a[1]);
                if xi > px {
                    w += if b[1] > a[1] { 1 } else { -1 };
                }
            }
        }
    }
    w
}

/// SONDA — de que são os vazios cercados do rabisco: buracos da REGRA (winding 0 cercado pelo
/// polígono) ou BOLSOS que o traço grosso fecha (winding 0 ligado ao exterior pelo polígono)?
#[test]
#[ignore = "diagnóstico"]
fn diag_de_que_sao_os_vazios() {
    for (nome, n) in [("meia volta", 60usize), ("rabisco inteiro", 121)] {
        let mut t = tool(LADO as u32, PaintMedia::Digital, 10.0);
        t.paint.brush.style_solid = true;
        let pts = rabisco();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for p in &pts[1..n] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        }
        let lacos = t.solid_fill_loops();
        t.on_canvas_pointer(cp(pts[n - 1], PointerPhase::Up));
        let w0 = |i: usize| winding(&lacos, i % LADO, i / LADO) == 0;
        let regra = onde_cercados(LADO, LADO, w0).len();
        let c = t.canvas_rgba.clone();
        let vazio = |i: usize| c[i * 4..i * 4 + 3] == [255, 255, 255];
        let vazios = onde_cercados(LADO, LADO, vazio);
        let dentro = vazios.iter().filter(|&&i| !w0(i)).count();
        let caixa = vazios
            .iter()
            .fold((usize::MAX, usize::MAX, 0, 0), |(a, b, c, d), &i| {
                (
                    a.min(i % LADO),
                    b.min(i / LADO),
                    c.max(i % LADO),
                    d.max(i / LADO),
                )
            });
        eprintln!(
            "{nome}: {} laço(s) · buracos da regra {regra} · vazios cercados pela tinta {} (dentro do laço final: {dentro}) · caixa dos vazios {caixa:?}",
            lacos.len(),
            vazios.len()
        );
    }
}
