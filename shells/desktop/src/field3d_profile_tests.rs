//! A sonda da tolerância do perfil — ver [`super`].

use ph2d_field_render::Orbit;

/// ⭐⭐ **A SONDA QUE ESCOLHEU A TOLERÂNCIA** (W54) — imprime a tabela que vive no doc do
/// [`ph2d_field_profile::TOLERANCE_RATIO`].
///
/// ⚠️ `#[ignore]` porque ela **mede relógio**: 35 traçados a 640×480 são ~8 s, e uma leitura desta
/// máquina não vale nada acima de `load ~5` (a lei do `CLAUDE.md` §5). Corre-se à mão, com a máquina
/// calma, quando o número tiver de ser reconferido:
///
/// ```text
/// cargo test -p ph2d-host-desktop --bin ph2d-host-desktop --release -- \
///     --exact field3d_profile::tests::the_table_that_chose_the_tolerance --ignored --nocapture
/// ```
///
/// ⭐ *Uma tabela num doc-comment sem a sonda ao lado envelhece em silêncio* — e foi exactamente o
/// que aconteceu com a de 2026-08-19, que esta wave desmentiu por 2,4×.
#[test]
#[ignore]
fn the_table_that_chose_the_tolerance() {
    use ph2d_vec_scene::{VecPath, VecVertex};
    // Um círculo desenhado com quatro segmentos de Bézier — o caso do torno do Enio.
    let k = 0.552_284_75_f64;
    let r = 0.5_f64;
    let mut verts = Vec::new();
    // ⚠️ **Deslocado do eixo**: o torno recusa um perfil com `x < 0` (regra do documento), e é essa
    // a forma de um perfil de torno a sério — um anel em volta do eixo.
    let cx = 0.9_f64;
    for (i, (ax, ay)) in [(cx + r, 0.0), (cx, r), (cx - r, 0.0), (cx, -r)]
        .into_iter()
        .enumerate()
    {
        let (tx, ty) = [(0.0, k * r), (-k * r, 0.0), (0.0, -k * r), (k * r, 0.0)][i];
        verts.push(VecVertex {
            in_handle: [ax - tx, ay - ty],
            out_handle: [ax + tx, ay + ty],
            ..VecVertex::corner([ax, ay])
        });
    }
    let path = VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    };

    println!("ratio | arestas | salto de normal | EXTRUSAO | TORNO | bandas");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    for ratio in [1e-3_f64, 3e-4, 1e-4, 3e-5, 1e-5] {
        let d = 2.0 * r;
        let prof = ph2d_field_profile::cook_path(&path, ratio * d).expect("perfil");
        let n = prof.segment_count();
        let jump = 360.0 / n as f64;
        let sag = r * (1.0 - (std::f64::consts::PI / n as f64).cos());
        // ⚠️ **AS DUAS**: a tabela de 19/08 no doc do `TOLERANCE_RATIO` diz 24,1 ms para 64
        // arestas, e o torno mede 66 ms para 56. Ou o traçado ficou mais caro, ou aquela mediu a
        // EXTRUSÃO. Medir as duas responde, em vez de sobrescrever.
        let mk = |k: ph2d_field::Primitive| {
            ph2d_field::FieldDoc::new(
                vec![ph2d_field::Node {
                    xform: ph2d_field::Xform::IDENTITY,
                    kind: ph2d_field::NodeKind::Leaf(k),
                    mods: Vec::new(),
                }],
                ph2d_field::NodeId(0),
            )
            .expect("a peça")
        };
        let ext = mk(ph2d_field::Primitive::Extrude {
            profile: prof.clone(),
            half_height: 0.2,
            round: 0.0,
        });
        let cam0 = Orbit::from_yaw_pitch(0.72, 0.52);
        let mut ext_ms = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&ext, &reg, &cam0, 640, 480);
            ext_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        ext_ms.sort_by(f64::total_cmp);
        let doc = mk(ph2d_field::Primitive::Revolve { profile: prof });
        let cam = Orbit::from_yaw_pitch(0.72, 0.52);
        // Sete corridas, a MEDIANA — uma leitura só é ruído, e três ainda deram uma tabela
        // não-monótona (305 arestas a 978 ms contra 528 a 594).
        let mut ms = Vec::new();
        let mut g = None;
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let out = ph2d_field_render::trace(&doc, &reg, &cam, 640, 480);
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            g = Some(out);
        }
        ms.sort_by(f64::total_cmp);
        // ⭐ A RÉGUA DAS BANDAS: pixels vizinhos, os dois na peça, cuja NORMAL salta mais que 3°.
        // É o que o olho lê como degrau — e não o erro da silhueta.
        let g = g.expect("traçou");
        let mut bands = 0usize;
        for y in 0..480usize {
            for x in 1..640usize {
                let (a, b) = (y * 640 + x - 1, y * 640 + x);
                if g.hit[a] && g.hit[b] {
                    let d = g.normal[a][0] * g.normal[b][0]
                        + g.normal[a][1] * g.normal[b][1]
                        + g.normal[a][2] * g.normal[b][2];
                    if d.clamp(-1.0, 1.0).acos().to_degrees() > 3.0 {
                        bands += 1;
                    }
                }
            }
        }
        let _ = sag;
        let _ = d;
        println!(
            "{ratio:>5.0e} | {n:>7} | {jump:>10.2}° | {:>7.1} ms | {:>6.1} ms | {bands:>6}",
            ext_ms[3], ms[3]
        );
    }
}
