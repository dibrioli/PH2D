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

/// ⏱️⭐⭐⭐⭐ **O MESMO GESTO NA CENA DO DONO** — o report seguinte (2026-10-01): *«melhor mas ainda
/// com delay de 1 ou 2 segundos»*. A sonda irmã acrescenta caixas a caixas; a cena do smoke
/// (`PH2D_FIELD_SMOKE=28`) são QUATRO nós de toro, e a fita da marcha leva a peça INTEIRA — logo uma
/// caixa nova recompila os kernels da marcha com a fita dos quatro nós dentro. Mede-se aqui, com
/// `PH2D_PIPELINE_LOG=1` a dizer quanto custou cada entrada.
#[test]
#[ignore = "sonda de diagnóstico: o relógio de acrescentar uma forma ao lado dos nós de toro"]
fn diag_o_preco_de_uma_forma_nova_ao_lado_do_no() {
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
    let no = |winds: u32, loops: u32, x: f32| {
        let (radius, tube) = (0.20_f32, 0.085_f32);
        ph2d_field_eval::leaf(
            Primitive::TorusKnot {
                radius,
                tube,
                cord: ph2d_field::knot_cord_ceiling(radius, tube, winds, loops) * 0.85,
                winds,
                loops,
            },
            Xform::at(x, 0.0, 0.0),
        )
    };
    let caixa = |i: usize, dx: f32| {
        #[allow(clippy::cast_precision_loss)]
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.15; 3],
                round: 0.02,
                chamfer: 0.0,
            },
            Xform::at(i as f32 * 0.35 - 0.5 + dx, 0.6, 0.0),
        )
    };
    let peca = |caixas: usize, dx: f32| -> (FieldDoc, Vec<FieldDoc>) {
        let mut folhas = vec![
            no(2, 3, -0.72),
            no(3, 2, -0.24),
            no(2, 5, 0.24),
            no(5, 2, 0.72),
        ];
        folhas.extend((0..caixas).map(|i| caixa(i, dx)));
        let n = folhas.len();
        let mut nos = folhas.clone();
        #[allow(clippy::cast_possible_truncation)]
        nos.push(ph2d_field::Node {
            xform: Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: (0..n as u32).map(NodeId).collect(),
            },
            mods: Vec::new(),
            verb: None,
        });
        let postas = folhas
            .into_iter()
            .map(|f| FieldDoc::new(vec![f], NodeId(0)).expect("a folha"))
            .collect();
        #[allow(clippy::cast_possible_truncation)]
        let raiz = NodeId(n as u32);
        (FieldDoc::new(nos, raiz).expect("a peça"), postas)
    };
    println!("\n  {}", contexto());
    println!("  passo                      · quadro    ·    relógio");
    for (rotulo, caixas, dx) in [
        ("4 nós", 0usize, 0.0f32),
        ("+ 1.ª caixa (forma NOVA)", 1, 0.0),
        ("+ 2.ª caixa (forma NOVA)", 2, 0.0),
        ("arrasto (mesma estrutura)", 2, 0.04),
        ("arrasto (mesma estrutura)", 2, 0.08),
    ] {
        let (doc, postas) = peca(caixas, dx);
        let n = postas.len();
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
            // ⏱️ Com `PH2D_GPU_CRONOMETRO=1`, o relógio por passe NA PLACA deste quadro.
            if ph2d_field_gpu::cronometro::ligado() {
                let rel = t
                    .lock()
                    .map_or_else(|_| Vec::new(), |mut g| g.cronometro_relatorio());
                let mut linha = String::new();
                for (r, ms, n) in rel {
                    linha += &format!(" {r}={ms:.1}×{n}");
                }
                println!("      placa:{linha}");
            }
        }
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **O PREÇO DA MARCHA INTERPRETADA** — a condição que decide o *ubershader*: a caixa nova
/// só aparece já se um quadro com a fita interpretada couber no orçamento enquanto a compilada
/// compila. Na cena do dono (os quatro nós + `k` caixas): a 1.ª chamada de cada modo (compila) e o
/// MÍNIMO de cinco (o desenho), com a fita compilada e com a interpretada, e uma caixa nova a seguir
/// com a interpretada — que NÃO pode compilar nada.
#[test]
#[ignore = "sonda de diagnóstico: o preço da marcha interpretada"]
fn diag_o_preco_da_marcha_interpretada() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let no = |winds: u32, loops: u32, x: f32| {
        let (radius, tube) = (0.20_f32, 0.085_f32);
        ph2d_field_eval::leaf(
            Primitive::TorusKnot {
                radius,
                tube,
                cord: ph2d_field::knot_cord_ceiling(radius, tube, winds, loops) * 0.85,
                winds,
                loops,
            },
            Xform::at(x, 0.0, 0.0),
        )
    };
    let caixa = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.15; 3],
                round: 0.02,
                chamfer: 0.0,
            },
            Xform::at(i as f32 * 0.35 - 0.5, 0.6, 0.0),
        )
    };
    let peca = |caixas: usize| -> FieldDoc {
        let mut nos = vec![
            no(2, 3, -0.72),
            no(3, 2, -0.24),
            no(2, 5, 0.24),
            no(5, 2, 0.72),
        ];
        nos.extend((0..caixas).map(caixa));
        let n = nos.len();
        #[allow(clippy::cast_possible_truncation)]
        nos.push(ph2d_field::Node {
            xform: Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: (0..n as u32).map(NodeId).collect(),
            },
            mods: Vec::new(),
            verb: None,
        });
        #[allow(clippy::cast_possible_truncation)]
        FieldDoc::new(nos, NodeId(n as u32)).expect("a peça")
    };
    let mats = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    let quadro = |doc: &FieldDoc, assente: bool, interpretada: bool| -> (f32, Option<Vec<u8>>) {
        let sonda = crate::gpu_frame::Sonda {
            fita_interpretada: interpretada,
            ..crate::gpu_frame::Sonda::default()
        };
        let t0 = std::time::Instant::now();
        let saiu = crate::gpu_frame::paint_com(
            t,
            doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            [0, 0, 0, 0],
            None,
            LW,
            LH,
            assente,
            sonda,
        );
        (t0.elapsed().as_secs_f32() * 1e3, saiu.map(|p| p.rgba))
    };
    let doc = peca(1);
    let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg).expect("o campo");
    println!(
        "\n  {}\n  registos da fita: {:?} (tecto {})",
        contexto(),
        campo.registos_da_fita(),
        ph2d_field_eval::interp::REGISTOS_DA_MARCHA
    );
    println!(
        "  fita        · quadro    · 1.ª chamada · mínimo de 5 · píxeis diferentes contra a compilada"
    );
    for assente in [false, true] {
        let mut ref_img = None;
        for interpretada in [false, true] {
            let (primeira, _) = quadro(&doc, assente, interpretada);
            let mut minimo = f32::INFINITY;
            let mut img = None;
            for _ in 0..5 {
                let (ms, i) = quadro(&doc, assente, interpretada);
                minimo = minimo.min(ms);
                img = i;
            }
            let dif = match (&ref_img, &img) {
                (Some(a), Some(b)) => {
                    let a: &Vec<u8> = a;
                    a.chunks(4)
                        .zip(b.chunks(4))
                        .filter(|(x, y)| x.iter().zip(y.iter()).any(|(p, q)| p.abs_diff(*q) > 2))
                        .count()
                        .to_string()
                }
                _ => "—".to_string(),
            };
            if !interpretada {
                ref_img = img;
            }
            println!(
                "  {:<11} · {:<9} · {primeira:9.2} ms · {minimo:9.2} ms · {dif}",
                if interpretada {
                    "interpretada"
                } else {
                    "compilada"
                },
                if assente { "assente" } else { "movimento" },
            );
        }
    }
    // A caixa NOVA com a interpretada: o texto não muda, logo nada compila.
    let antes = t.lock().expect("o traçador").compiled();
    let doc2 = peca(2);
    let (mov, _) = quadro(&doc2, false, true);
    let (ass, _) = quadro(&doc2, true, true);
    let depois = t.lock().expect("o traçador").compiled();
    println!(
        "  2.ª caixa com a interpretada: movimento {mov:.2} ms · assente {ass:.2} ms · compilou {} pipeline(s)\n",
        depois - antes
    );
}
