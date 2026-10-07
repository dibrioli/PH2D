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
    (0..w * h).filter(|&i| vazio(i) && !fora[i]).count()
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
