//! **A folha de contacto dos efeitos** — o aparelho que faltava (2026-07-18).
//!
//! Um PNG com uma linha por efeito e uma coluna por valor do parâmetro. Um olhar cobre o menu
//! inteiro, e foi a falta dele que deixou passar três efeitos maus de uma vez.
//!
//! ```text
//! PH2D_FX_LOOK_DIR=/tmp/look cargo test -p ph2d-vec-scene --test fx_look --release -- --ignored --nocapture
//! ```
//!
//! O **preenchimento** mostra a forma que o artista vê; a **linha fina** por cima mostra a
//! geometria; as **cruzes** mostram onde as âncoras caíram — é a diferença entre *"a curva está
//! errada"* e *"as âncoras estão no sítio errado"*, que foi exatamente a dúvida do Twist.

mod look;

use look::{Canvas, fill, mark, stroke, write_png};
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

const CELL: usize = 220;
const BG: [u8; 3] = [24, 24, 28];
const PAPER: [u8; 3] = [96, 132, 214];
const WIRE: [u8; 3] = [235, 235, 240];
const ANCHOR: [u8; 3] = [255, 168, 64];

/// Um círculo em quatro cúbicas, raio 60 na origem.
fn circle() -> VecPath {
    const K: f64 = 0.552_284_749_830_793_4;
    const R: f64 = 60.0;
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let t = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    VecPath {
        verts: (0..4)
            .map(|i| VecVertex {
                anchor: p[i],
                in_handle: [p[i][0] - t[i][0], p[i][1] - t[i][1]],
                out_handle: [p[i][0] + t[i][0], p[i][1] + t[i][1]],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Um quadrado de lado 80 centrado na origem — a forma que mostra ladrilhamento.
fn square() -> VecPath {
    VecPath {
        verts: [[-40.0, -40.0], [40.0, -40.0], [40.0, 40.0], [-40.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Um PENTAGRAMA `{5/2}` — cinco pontos ligados de dois em dois, cinco auto-interseções. É a forma
/// canônica de nó celta, e o caso de teste do Knot (o entrelace tem de alternar cima/baixo).
fn pentagram() -> VecPath {
    const R: f64 = 72.0;
    VecPath {
        verts: (0..5)
            .map(|k| {
                let a = (90.0 + f64::from(k) * 144.0).to_radians();
                VecVertex::corner([R * a.cos(), R * a.sin()])
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Achata os contornos COZIDOS em polilinhas, já em coordenadas de pixel da célula.
fn flatten(path: &VecPath, ox: f64, oy: f64, scale: f64) -> (Vec<Vec<[f64; 2]>>, Vec<[f64; 2]>) {
    const STEPS: usize = 24;
    let cooked = path.cooked();
    let mut polys = Vec::new();
    let mut anchors = Vec::new();
    let to_px = |p: [f64; 2]| [ox + p[0] * scale, oy - p[1] * scale];
    for k in 0..cooked.contour_count() {
        let Some((verts, closed)) = cooked.contour(k) else {
            continue;
        };
        let n = verts.len();
        if n < 2 {
            continue;
        }
        let segs = if closed { n } else { n - 1 };
        let mut poly = Vec::with_capacity(segs * STEPS);
        for i in 0..segs {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            anchors.push(to_px(a.anchor));
            for j in 0..STEPS {
                let t = j as f64 / STEPS as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                poly.push(to_px([
                    w0 * a.anchor[0]
                        + w1 * a.out_handle[0]
                        + w2 * b.in_handle[0]
                        + w3 * b.anchor[0],
                    w0 * a.anchor[1]
                        + w1 * a.out_handle[1]
                        + w2 * b.in_handle[1]
                        + w3 * b.anchor[1],
                ]));
            }
        }
        polys.push(poly);
    }
    (polys, anchors)
}

/// Desenha uma célula: preenchimento + arame + âncoras.
fn cell(canvas: &mut Canvas, path: &VecPath, col: usize, row: usize, scale: f64) {
    let ox = (col * CELL + CELL / 2) as f64;
    let oy = (row * CELL + CELL / 2) as f64;
    let (polys, anchors) = flatten(path, ox, oy, scale);
    fill(canvas, &polys, PAPER, false);
    stroke(canvas, &polys, WIRE);
    for a in anchors {
        mark(canvas, a, ANCHOR);
    }
}

/// As linhas da folha: `(nome, forma de origem, escala, valores do parâmetro que varia)`.
/// O índice do parâmetro que varia e o resto dos parâmetros vêm de `setup`.
struct Row {
    name: &'static str,
    kind: usize,
    scale: f64,
    /// Como armar o efeito para a coluna `c` (0 = neutro, à esquerda).
    arm: fn(&mut PathEffect, usize),
    shape: fn() -> VecPath,
}

fn arm_trim(fx: &mut PathEffect, c: usize) {
    if let Some(t) = fx.as_trim_mut() {
        t.end = 0.2 * c as f64;
    }
}
fn arm_zigzag(fx: &mut PathEffect, c: usize) {
    if let Some(z) = fx.as_zigzag_mut() {
        z.amplitude = 4.0 * c as f64;
        z.ridges = 12.0;
    }
}
/// Fileira em x: só a contagem cresce.
fn arm_repeat(fx: &mut PathEffect, c: usize) {
    fx.set(0, 1.0 + c as f64);
    fx.set(1, 100.0);
}
/// GRELHA: os dois eixos ligados, o segundo a crescer.
fn arm_grid(fx: &mut PathEffect, c: usize) {
    fx.set(0, 3.0);
    fx.set(1, 100.0);
    fx.set(2, 1.0 + c as f64);
    fx.set(3, 100.0);
}
/// SPIN: fileira fixa, cada cópia gira sobre si mesma.
fn arm_spin(fx: &mut PathEffect, c: usize) {
    fx.set(0, 4.0);
    fx.set(1, 110.0);
    fx.set(4, 15.0 * c as f64);
}
/// ORBIT: a mesma fileira, mas a rotação LEVA a cópia — o arranjo radial.
fn arm_orbit(fx: &mut PathEffect, c: usize) {
    fx.set(0, 6.0);
    fx.set(1, 60.0);
    fx.set(5, 12.0 * c as f64);
}
fn arm_bloat(fx: &mut PathEffect, c: usize) {
    fx.set(0, -60.0 + 30.0 * c as f64);
}
/// O TWIST sobre uma forma COM QUINAS (o caso de falha documentado): 0 → 360° na borda.
fn arm_twist(fx: &mut PathEffect, c: usize) {
    fx.set(0, 90.0 * c as f64);
}
/// O KNOT: o vão cresce 0 → 16% da forma (a espessura aparente do entrelace).
fn arm_knot(fx: &mut PathEffect, c: usize) {
    fx.set(0, 4.0 * c as f64);
}
/// O mesmo Knot com o SWAP ligado — quem passa por cima inverte em todo cruzamento.
fn arm_knot_swap(fx: &mut PathEffect, c: usize) {
    fx.set(0, 4.0 * c as f64);
    fx.set(1, 1.0);
}
fn arm_sketch(fx: &mut PathEffect, c: usize) {
    // 2 passadas; o tremor cresce por coluna (0 % → 12 %).
    fx.set(0, 2.0);
    fx.set(1, 3.0 * c as f64);
}
fn arm_hatch(fx: &mut PathEffect, c: usize) {
    // Espaçamento decresce por coluna (mais denso à direita): 14 % → 6 %. Coluna 0 = espaçamento
    // largo (poucas linhas).
    fx.set(1, 14.0 - 2.0 * c as f64);
}
fn arm_hatch_cross(fx: &mut PathEffect, c: usize) {
    fx.set(1, 14.0 - 2.0 * c as f64);
    fx.set(2, 1.0); // cross-hatch
}

const ROWS: &[Row] = &[
    Row {
        name: "Trim Path      (End 0 → 0.8)",
        kind: 0,
        scale: 1.2,
        arm: arm_trim,
        shape: circle,
    },
    Row {
        name: "Zig Zag        (Size 0 → 16)",
        kind: 1,
        scale: 1.2,
        arm: arm_zigzag,
        shape: circle,
    },
    Row {
        name: "Repeater eixo X  (1→5 cópias, Move X 100)",
        kind: 2,
        scale: 0.42,
        arm: arm_repeat,
        shape: square,
    },
    Row {
        name: "Repeater GRELHA  (3 em x · 1→5 em y)",
        kind: 2,
        scale: 0.30,
        arm: arm_grid,
        shape: square,
    },
    Row {
        name: "Repeater SPIN    (4 cópias · 0→60° sobre si mesma)",
        kind: 2,
        scale: 0.40,
        arm: arm_spin,
        shape: square,
    },
    Row {
        name: "Repeater ORBIT   (6 cópias · 0→48° em torno da origem)",
        kind: 2,
        scale: 0.30,
        arm: arm_orbit,
        shape: square,
    },
    Row {
        name: "Pucker & Bloat (-60 → +60)",
        kind: 3,
        scale: 1.1,
        arm: arm_bloat,
        shape: circle,
    },
    // ⚠️ Os QUATRO últimos KINDS, nesta ordem: Twist = `len()-4`, Knot = `len()-3`, Sketch =
    // `len()-2`, Hatch = `len()-1`. Se um KIND novo for apendado depois, estes índices deslocam-se —
    // mas a sonda é visual (`#[ignore]`) e o gate `every_effect_kind_is_reachable` cobre a
    // consistência KINDS<->from_kind à parte.
    // O Twist sobre um QUADRADO — o caso que a cerca do `fx_warp` diz ter rasgado. Se a coluna 360°
    // aparecer como um remoinho de tinta cheia (e não buracos/fios soltos), a volta é boa.
    Row {
        name: "Twist QUADRADO (0 → 360° na borda) -- o caso de falha da cerca",
        kind: PathEffect::KINDS.len() - 4,
        scale: 0.85,
        arm: arm_twist,
        shape: square,
    },
    // O mesmo Twist num CÍRCULO, para separar "a curva está errada" de "as quinas rasgam".
    Row {
        name: "Twist CIRCULO (0 → 360° na borda)",
        kind: PathEffect::KINDS.len() - 4,
        scale: 0.85,
        arm: arm_twist,
        shape: circle,
    },
    // O KNOT sobre um PENTAGRAMA (5 travessias) — o entrelace tem de ALTERNAR cima/baixo. A coluna 0
    // e' a estrela sólida; conforme o vão cresce, cada travessia mostra uma fita por baixo (o vão) e
    // a de cima inteira -- lido como tecido.
    Row {
        name: "Knot PENTAGRAMA (Gap 0 -> 16%) -- alterna cima/baixo",
        kind: PathEffect::KINDS.len() - 3,
        scale: 1.2,
        arm: arm_knot,
        shape: pentagram,
    },
    // O MESMO, com Swap -- quem passa por cima inverte em toda travessia.
    Row {
        name: "Knot PENTAGRAMA + Swap (Gap 0 -> 16%)",
        kind: PathEffect::KINDS.len() - 3,
        scale: 1.2,
        arm: arm_knot_swap,
        shape: pentagram,
    },
    // O SKETCH num CÍRCULO — o tremor cresce por coluna. Coluna 0 = 2 passadas coincidentes (o
    // círculo limpo); à direita as duas linhas se afastam e tremem = "desenhado à mão".
    Row {
        name: "Sketch CIRCULO (2 passadas, tremor 0 -> 12%)",
        kind: PathEffect::KINDS.len() - 2,
        scale: 1.1,
        arm: arm_sketch,
        shape: circle,
    },
    // O HATCH num QUADRADO — o espaçamento decresce por coluna (mais denso à direita). O outline
    // fica; as linhas enchem o interior a 45°.
    Row {
        name: "Hatch QUADRADO (espacamento 14% -> 6%)",
        kind: PathEffect::KINDS.len() - 1,
        scale: 1.1,
        arm: arm_hatch,
        shape: square,
    },
    // O MESMO com CROSS -- uma 2a familia a 90 graus.
    Row {
        name: "Hatch QUADRADO + Cross (14% -> 6%)",
        kind: PathEffect::KINDS.len() - 1,
        scale: 1.1,
        arm: arm_hatch_cross,
        shape: square,
    },
];

const COLS: usize = 5;

#[test]
#[ignore = "sonda visual: PH2D_FX_LOOK_DIR=<dir> ... -- --ignored --nocapture"]
fn probe_fx_look() {
    let dir = std::env::var("PH2D_FX_LOOK_DIR").unwrap_or_else(|_| "/tmp".into());
    let mut canvas = Canvas::new(CELL * COLS, CELL * ROWS.len(), BG);
    for (r, row) in ROWS.iter().enumerate() {
        for c in 0..COLS {
            let mut path = (row.shape)();
            let mut fx = PathEffect::from_kind(row.kind).expect("tipo");
            (row.arm)(&mut fx, c);
            path.effects = vec![FxEntry::new(fx)];
            cell(&mut canvas, &path, c, r, row.scale);
        }
    }
    let out = std::path::Path::new(&dir).join("fx_look.png");
    write_png(&out, &canvas).expect("escrever o PNG");
    println!("\n  {}\n", out.display());
    for (r, row) in ROWS.iter().enumerate() {
        println!("  linha {}: {}", r + 1, row.name);
    }
    println!("\n  preenchimento = a forma · linha = a geometria · cruzes = as ÂNCORAS\n");
}
