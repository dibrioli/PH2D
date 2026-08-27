//! ⭐⭐ **Quanto o preview CUSTA** — as sondas, irmãs dos gates em
//! [`field3d_preview_tests`](super::tests).
//!
//! ⚠️ **O corte é por ASSUNTO e nasceu de um teto:** o irmão passou os `600` LOC do HR-18, e o que
//! saiu foi o que responde *«quanto custa?»* — os gates que respondem *«a lei vale?»* ficaram lá.
//! Uma sonda traz consigo a tabela medida e o comando que a re-corre, e é isso que a faz crescer.

// ⚠️ **A resolução cheia é a MESMA dos gates** — uma cópia local seria duas respostas para
// *«qual é o alvo?»*, e a que envelhecesse seria a deste ficheiro.
use super::tests::FULL;

/// ⭐⭐⭐ **O QUE UM PERFIL FINO CUSTA EM CADA DIVISOR** — a coluna que a [`MEASURED_MS`] não tinha.
///
/// ⛔ **A tabela media duas cenas até `D=4`**, e o [`measured_cost`] escolhe a **linha mais
/// próxima** — então `D=5..8` recebiam todos o número do `D=4`. Enquanto o piso era `3` isso nunca
/// importou; quando o `MAX_PROFILE_RESOLUTION` subiu, o laço passou a precisar de divisores que a
/// tabela **não sabia medir**. *Uma tabela que satura na última linha responde a todas as perguntas
/// e só acerta nas que já sabia.*
///
/// ⚠️ A peça é uma extrusão no **teto** do `Resolution` — o pior caso que o produto consegue pedir.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bin ph2d-host-desktop -- \
///     --ignored --nocapture measure_a_fine_profile_at_every_divisor
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_a_fine_profile_at_every_divisor() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    // ⭐ **O PISO por nível**: o que a pré-visualização não consegue baixar, por mais grossa que
    // fique — medido em D=6, que a varredura achou ser o fundo (abaixo dele o custo volta a subir).
    println!("nivel | arestas | D=1 | D=3 | D=6 (o PISO)");
    for level in [1u32, 8, 16, 32, 64] {
        // ⛔ **A 1.ª versão desta fixtura construía um polígono de 168 pontos e passava a
        // tolerância como METADADO** — o `Profile::new` recebe uma polilinha **já achatada**, então
        // todos os níveis saíam com as mesmas 168 arestas e a tabela não media o nível nenhum.
        // *Uma fixtura que não contém o fenómeno mede outra coisa.* ⇒ aqui o contorno é **cozido**
        // de uma curva, que é o que o produto faz.
        let prof = {
            use ph2d_vec_scene::{VecPath, VecVertex};
            let (k, r) = (0.552_284_75_f64, 0.6_f64);
            let mut verts = Vec::new();
            for (i, (ax, ay)) in [(r, 0.0), (0.0, r), (-r, 0.0), (0.0, -r)]
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
            // ⚠️ **Por fora da trava**, como a sonda do teto: a pergunta é o que cada nível CUSTA,
            // e passar pelo clamp mediria o teto de hoje em todas as linhas.
            let tol = ph2d_field_profile::span_of(&path.cooked())
                * ph2d_field_profile::TOLERANCE_RATIO
                / f64::from(level);
            ph2d_field_profile::cook_path(&path, tol).expect("perfil")
        };
        let arestas = prof.segment_count();
        let d = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: prof,
                    half_height: 0.4,
                    round: 0.06,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");
        let r = ph2d_field_eval::hybrid::Registry::new();
        let c = ph2d_field_render::Orbit::default();
        let mut row = format!("{level:5} | {arestas:7} |");
        for dv in [1u32, 3, 6] {
            let (w, h) = (FULL.0 / dv, FULL.1 / dv);
            let _ = ph2d_field_render::trace(&d, &r, &c, w, h);
            let mut v: Vec<f64> = (0..3)
                .map(|_| {
                    let t = std::time::Instant::now();
                    let _ = ph2d_field_render::trace(&d, &r, &c, w, h);
                    t.elapsed().as_secs_f64() * 1000.0
                })
                .collect();
            v.sort_by(f64::total_cmp);
            row.push_str(&format!(" {:7.1} |", v[1]));
        }
        println!("{row}");
    }
    let n = 168usize;
    let contour: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    // A tolerância do TETO, pela porta do produto.
    let tol =
        ph2d_field_profile::TOLERANCE_RATIO / f64::from(ph2d_field::MAX_PROFILE_RESOLUTION) * 1.2;
    let profile = Profile::new(vec![contour], FillRule::NonZero, tol as f32).expect("perfil");
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
                half_height: 0.4,
                round: 0.06,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    println!("divisor | {}x{} | ms", FULL.0, FULL.1);
    for d in [1u32, 3, 6] {
        let (w, h) = (FULL.0 / d, FULL.1 / d);
        let _ = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let mut v: Vec<f64> = (0..3)
            .map(|_| {
                let t = std::time::Instant::now();
                let _ = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
                t.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        v.sort_by(f64::total_cmp);
        println!("{d:7} | {w:4}x{h:4} | {:8.1}", v[1]);
    }
}

/// ⭐⭐⭐ **QUANTO A CURA COMPRA** — o traçado de movimento, com e sem o contorno engrossado.
///
/// ⚠️ **As duas configurações no MESMO processo, por mediana** — a lição que a W64 pagou: subtrair
/// dois relógios de corridas separadas dá a soma dos dois ruídos.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bin ph2d-host-desktop -- \
///     --ignored --nocapture measure_what_the_coarse_contour_buys
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_coarse_contour_buys() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let median = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | pedido | sem a cura | com a cura | ganho");
    for n in [168usize, 472, 940] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.5 * a.cos()) as f32, (0.5 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Leaf(Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                }),
                mods: Vec::new(),
            }],
            NodeId(0),
        )
        .expect("extrusão");
        let asked = (640u32, 360u32);
        let grosso = super::coarse_doc(&doc, true).unwrap_or_else(|| doc.clone());
        for d in [&doc, &grosso] {
            let _ = ph2d_field_render::trace(d, &reg, &cam, asked.0, asked.1);
        }
        let mut sem = Vec::new();
        let mut com = Vec::new();
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&doc, &reg, &cam, asked.0, asked.1);
            sem.push(t.elapsed().as_secs_f64() * 1000.0);
            let t = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&grosso, &reg, &cam, asked.0, asked.1);
            com.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let (a, b) = (median(sem), median(com));
        println!(
            "{n:7} | {}x{} | {a:10.1} | {b:10.1} | {:5.2}x",
            asked.0,
            asked.1,
            a / b
        );
    }
}
