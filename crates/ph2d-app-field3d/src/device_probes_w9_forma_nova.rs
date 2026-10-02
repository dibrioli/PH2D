//! ⏱️⭐⭐⭐⭐ **O PREÇO DE UMA FORMA NOVA NO RENDER** — o report do dono de 2026-10-01:
//! *«ao acrescentar um box ele demora uns 5 segundos para aparecer … tb 5 seg para mudar de cor»*.
//!
//! A sonda refaz o gesto pelo caminho do produto ([`crate::gpu_frame::paint`], a `Sonda` de
//! omissão): uma peça de `n` caixas com a lei do dono (materiais distintos), um quadro de MOVIMENTO
//! e dois ASSENTES, e depois `n + 1` caixas — e mede o relógio de cada chamada. ⚠️ O que se lê é a
//! 1.ª chamada depois de cada mudança de estrutura: as seguintes acertam o cache e servem de
//! CONTROLO (o custo de desenhar, sem compilar). Com `PH2D_PIPELINE_LOG=1` cada falta no cache diz
//! quanto custou e de que entrada é.
//!
//! ```text
//! PH2D_PIPELINE_LOG=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib --release \
//!   -- --ignored diag_o_preco_de_uma_forma_nova --nocapture --test-threads=1
//! ```

use super::*;

#[test]
#[ignore = "sonda de diagnóstico: o relógio de acrescentar uma forma no Render"]
fn diag_o_preco_de_uma_forma_nova() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let caixa = |i: usize, dx: f32| {
        #[allow(clippy::cast_precision_loss)]
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.18; 3],
                round: 0.02,
                chamfer: 0.0,
            },
            Xform::at(i as f32 * 0.3 - 0.45 + dx, 0.0, 0.0),
        )
    };
    let peca = |n: usize, dx: f32| -> (FieldDoc, Vec<FieldDoc>) {
        let folhas: Vec<ph2d_field::Node> = (0..n).map(|i| caixa(i, dx)).collect();
        let mut nos = folhas.clone();
        let mut raiz = NodeId(0);
        for i in 1..n {
            nos.push(ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    #[allow(clippy::cast_possible_truncation)]
                    children: vec![raiz, NodeId(i as u32)],
                },
                mods: Vec::new(),
                verb: None,
            });
            #[allow(clippy::cast_possible_truncation)]
            {
                raiz = NodeId((nos.len() - 1) as u32);
            }
        }
        let postas = folhas
            .into_iter()
            .map(|f| FieldDoc::new(vec![f], NodeId(0)).expect("a folha"))
            .collect();
        (FieldDoc::new(nos, raiz).expect("a peça"), postas)
    };
    println!("\n  {}", contexto());
    println!("  passo                      · quadro    ·    relógio");
    for (rotulo, n, dx) in [
        ("3 caixas", 3usize, 0.0f32),
        ("4.ª caixa (forma NOVA)", 4, 0.0),
        ("arrasto (mesma estrutura)", 4, 0.05),
        ("5.ª caixa (forma NOVA)", 5, 0.0),
    ] {
        let (doc, postas) = peca(n, dx);
        let owners = ph2d_field_eval::owners::Owners::new(
            &postas,
            &reg,
            ph2d_field_render::hit_tolerance(
                cam.half_extent,
                f32::from(u16::try_from(LH).unwrap_or(u16::MAX)),
            ),
        );
        let mats: Vec<ph2d_material::Surface> = (0..n)
            .map(|_| ph2d_material::OpenPbr::default().prepare())
            .collect();
        let surfaces = ph2d_field_render::Surfaces {
            all: &mats,
            owners: Some(&owners),
        };
        for (quadro, assente) in [
            ("movimento", false),
            ("movimento", false),
            ("assente", true),
            ("assente", true),
        ] {
            let t0 = std::time::Instant::now();
            let saiu = crate::gpu_frame::paint(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
                None,
                LW,
                LH,
                assente,
            );
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            println!(
                "  {rotulo:<26} · {quadro:<9} · {}",
                if saiu.is_some() {
                    format!("{ms:9.2} ms")
                } else {
                    "  na CPU".to_string()
                }
            );
        }
    }
    println!();
}

/// ⏱️⭐⭐⭐ **O PREÇO DA LEI DO DONO POR QUADRO** — a outra metade do report (*«a resolução cai ao
/// rotacionar»* com três objectos). A mesma peça de `n` caixas, já compilada, pintada com e sem a
/// lei do dono, nos dois quadros e a `1280×720`: o que se lê é o MÍNIMO de cinco chamadas, que é a
/// régua desta casa para o relógio de um despacho.
#[test]
#[ignore = "sonda de diagnóstico: o relógio da lei do dono por quadro"]
fn diag_o_preco_do_dono_por_quadro() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let (w, h) = (1280u32, 720u32);
    println!("\n  {}", contexto());
    println!("  caixas · quadro    · sem dono ·  com dono");
    for n in [2usize, 3, 5, 8] {
        #[allow(clippy::cast_precision_loss)]
        let folhas: Vec<ph2d_field::Node> = (0..n)
            .map(|i| {
                ph2d_field_eval::leaf(
                    Primitive::Box {
                        half: [0.16; 3],
                        round: 0.02,
                        chamfer: 0.0,
                    },
                    Xform::at(i as f32 * 0.26 - 0.13 * (n as f32 - 1.0), 0.0, 0.0),
                )
            })
            .collect();
        let mut nos = folhas.clone();
        let mut raiz = NodeId(0);
        #[allow(clippy::cast_possible_truncation)]
        for i in 1..n {
            nos.push(ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![raiz, NodeId(i as u32)],
                },
                mods: Vec::new(),
                verb: None,
            });
            raiz = NodeId((nos.len() - 1) as u32);
        }
        let doc = FieldDoc::new(nos, raiz).expect("a peça");
        let postas: Vec<FieldDoc> = folhas
            .into_iter()
            .map(|f| FieldDoc::new(vec![f], NodeId(0)).expect("a folha"))
            .collect();
        let owners = ph2d_field_eval::owners::Owners::new(
            &postas,
            &reg,
            ph2d_field_render::hit_tolerance(
                cam.half_extent,
                f32::from(u16::try_from(h).unwrap_or(u16::MAX)),
            ),
        );
        let mats: Vec<ph2d_material::Surface> = (0..n)
            .map(|_| ph2d_material::OpenPbr::default().prepare())
            .collect();
        for (quadro, assente) in [("movimento", false), ("assente", true)] {
            let mut col = [0.0f32; 2];
            for (k, dono) in [None, Some(&owners)].into_iter().enumerate() {
                let surfaces = ph2d_field_render::Surfaces {
                    all: &mats,
                    owners: dono,
                };
                let mut melhor = f32::INFINITY;
                for _ in 0..6 {
                    let t0 = std::time::Instant::now();
                    let _ = crate::gpu_frame::paint(
                        t,
                        &doc,
                        &reg,
                        &cam,
                        &luz,
                        &surfaces,
                        &ph2d_field_render::Presentation::of(olhar),
                        [0, 0, 0, 0],
                        None,
                        w,
                        h,
                        assente,
                    );
                    melhor = melhor.min(t0.elapsed().as_secs_f32() * 1e3);
                }
                col[k] = melhor;
            }
            println!(
                "  {n:>6} · {quadro:<9} · {:>6.2} ms · {:>6.2} ms",
                col[0], col[1]
            );
        }
    }
    println!();
}
