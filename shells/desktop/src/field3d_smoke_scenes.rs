//! ⭐ **As cenas do smoke** — o catálogo do que o `PH2D_MODEL3D_SMOKE=<n>` encena.
//!
//! ⚠️ **Cada cena imprime o que montou.** O modo de falha deste módulo não é o pânico: é a **janela
//! vazia** — a peça fora do quadro, o perfil recusado, o campo que saiu sem interior. Uma linha no
//! terminal é o que separa *"não construiu"* de *"construiu e não se vê"*.
//!
//! ⚠️ É um **módulo-filho** de [`super`] e não um irmão de topo: `field3d_smoke::scene` continua a
//! ser o caminho, pelo re-export. Cortar um arquivo não pode custar uma reescrita em cada sítio que
//! o chamava.

use super::*;

/// **Um contorno desenhado como a caneta o desenharia** — âncoras de quina com **raio vivo** — e
/// depois cozido em perfil.
///
/// ⭐ É esta função que fecha a costura da W3: o caminho é `VecPath` → `cooked()` (os Live Corners
/// do editor vetorial correm aqui) → `Profile`. Nenhum arco é escrito à mão; o arredondamento das
/// arestas verticais do sólido **é** o *corner widget* do editor de vetores.
fn drawn_profile(pts: &[([f64; 2], f64)]) -> Profile {
    let path = VecPath {
        verts: pts
            .iter()
            .map(|&(p, radius)| VecVertex {
                corner_radius: radius,
                ..VecVertex::corner(p)
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    };
    ph2d_field_profile::cook_path_auto(&path).expect("os contornos do smoke são perfis válidos")
}

/// **Uma bolha ORGÂNICA** — uma esfera deslocada por uma onda suave, para servir de escultura.
///
/// ⚠️ **A primeira versão usava a `uv_sphere_noisy` da casa e o smoke reprovou** (Enio, 21/08:
/// *"bastante estranho"*): aquela função desloca cada vértice por ruído **branco**, o que dá uma
/// superfície **espinhosa** — ela existe para os gates da escultura medirem malha irregular, não para
/// alguém a olhar. Uma escultura de verdade tem forma **grande e lisa**, e é isso que torna visível o
/// que esta cena existe para mostrar: o filete a acompanhar a curvatura da peça.
///
/// A onda é `1 + 0,16·sin(3·azimute)·sin(2·polar)` — três lobos em volta, dois de cima a baixo.
#[cfg(test)]
pub(crate) fn organic_blob_for_probe(
    rings: usize,
    segments: usize,
    radius: f32,
) -> ph2d_mesh::Mesh {
    organic_blob(rings, segments, radius)
}

fn organic_blob(rings: usize, segments: usize, radius: f32) -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(rings, segments, radius);
    for p in mesh.positions_mut() {
        let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        if l <= 1e-6 {
            continue;
        }
        let azimuth = p[2].atan2(p[0]);
        let polar = (p[1] / l).clamp(-1.0, 1.0).acos();
        let k = 0.16f32.mul_add((3.0 * azimuth).sin() * (2.0 * polar).sin(), 1.0) * radius / l;
        for v in p.iter_mut() {
            *v *= k;
        }
    }
    mesh.rebuild();
    mesh
}

/// As cenas. ⚠️ **Cada uma imprime o que montou** — se a linha não aparecer no terminal, o smoke
/// não chegou a construir nada, e a tela vazia é sintoma disso e não da geometria.
pub(crate) fn scene(n: u32) -> FieldDoc {
    let combine = |op: Op, children: Vec<NodeId>| Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
        mods: Vec::new(),
        verb: None,
    };
    let leaf = |p: Primitive, x: Xform| Node {
        xform: x,
        kind: NodeKind::Leaf(p),
        mods: Vec::new(),
        verb: None,
    };
    let s = std::f32::consts::FRAC_1_SQRT_2;

    let doc = match n {
        2 => {
            println!("[field-smoke] cena 2 — cubo com aresta arredondada (r = 0,08)");
            FieldDoc::new(
                vec![leaf(
                    Primitive::Box {
                        half: [0.45; 3],
                        round: 0.08,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
        }
        3 => {
            println!(
                "[field-smoke] cena 3 — caixa arredondada MENOS cilindro, boca do furo em 0,05"
            );
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Box {
                            half: [0.5; 3],
                            round: 0.06,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.24,
                            half_height: 1.2,
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    combine(
                        Op::Difference(Blend::Exact { radius: 0.05 }),
                        vec![NodeId(0), NodeId(1)],
                    ),
                ],
                NodeId(2),
            )
        }
        4 => {
            // A cantoneira: o perfil é DESENHADO (quinas com raio vivo), extrudado com aro
            // arredondado, e furado por dois cilindros cuja boca ganha filete.
            let profile = drawn_profile(&[
                ([-0.35, -0.35], 0.05),
                ([0.35, -0.35], 0.05),
                ([0.35, -0.13], 0.05),
                ([-0.10, -0.13], 0.05),
                ([-0.10, 0.35], 0.05),
                ([-0.35, 0.35], 0.05),
            ]);
            println!(
                "[field-smoke] cena 4 — cantoneira DESENHADA: perfil de {} arestas (quinas vivas \
                 r=0,05), extrusão com aro 0,03, dois furos com a boca em 0,02",
                profile.segment_count()
            );
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Extrude {
                            profile,
                            half_height: 0.14,
                            round: 0.03,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.07,
                            half_height: 1.0,
                            round: 0.0,
                        },
                        Xform::at(0.18, -0.24, 0.0),
                    ),
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.07,
                            half_height: 1.0,
                            round: 0.0,
                        },
                        Xform::at(-0.225, 0.20, 0.0),
                    ),
                    combine(
                        Op::Difference(Blend::Exact { radius: 0.02 }),
                        vec![NodeId(0), NodeId(1), NodeId(2)],
                    ),
                ],
                NodeId(3),
            )
        }
        5 => {
            // O torno: o mesmo tipo de contorno, agora GIRADO em torno de Y. A silhueta é a de um
            // vaso oco — parede externa a subir, lábio, parede interna a descer, e o fecho no eixo.
            let profile = drawn_profile(&[
                ([0.00, -0.45], 0.0),
                ([0.26, -0.45], 0.05),
                ([0.30, -0.34], 0.05),
                ([0.15, -0.10], 0.06),
                ([0.33, 0.22], 0.06),
                ([0.27, 0.44], 0.04),
                ([0.33, 0.52], 0.02),
                ([0.27, 0.52], 0.02),
                ([0.21, 0.44], 0.04),
                ([0.09, -0.08], 0.05),
                ([0.19, -0.32], 0.04),
                ([0.00, -0.32], 0.0),
            ]);
            println!(
                "[field-smoke] cena 5 — TORNO: o mesmo contorno desenhado ({} arestas) girado em \
                 torno de Y — um vaso oco, com a parede a sair do desenho",
                profile.segment_count()
            );
            FieldDoc::new(
                vec![leaf(Primitive::Revolve { profile }, Xform::IDENTITY)],
                NodeId(0),
            )
        }
        6 => {
            // ⭐ **A PONTE DA ESCULTURA (plano W5)**: uma malha orgânica entra na booleana do campo
            // implícito e leva um furo preciso, com a boca arredondada.
            //
            // ⚠️ **A malha é gerada aqui de propósito.** O que esta cena tem de provar é a *ponte* —
            // que um campo amostrado se combina com um analítico —, e uma escultura vinda do módulo
            // de sculpt traria consigo a pergunta de **autoria** (como o artista cria um destes),
            // que é wave própria e tem UI. Uma esfera ruidosa é o mínimo que já não é analítico.
            let blob = organic_blob(64, 128, 0.5);
            let t0 = std::time::Instant::now();
            let field = ph2d_field_mesh::SampledField::from_mesh(&blob, 112)
                .expect("a bolha não é uma malha vazia");
            let cell = field.cell();
            crate::field3d_smoke::register_sampled("blob", std::sync::Arc::new(field));
            println!(
                "[field-smoke] cena 6 — A PONTE: uma ESCULTURA de {} triângulos virou campo em \
                 {:.0} ms (célula {cell:.4}) e leva um furo de 0,20 com a boca em 0,05",
                blob.faces().len(),
                t0.elapsed().as_secs_f64() * 1000.0,
            );
            FieldDoc::new(
                vec![
                    Node {
                        xform: Xform::IDENTITY,
                        kind: ph2d_field::NodeKind::Sampled { key: "blob".into() },
                        mods: Vec::new(),
                        verb: None,
                    },
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.20,
                            half_height: 1.2,
                            round: 0.0,
                        },
                        Xform {
                            rotation: [s, 0.0, 0.0, s],
                            ..Xform::IDENTITY
                        },
                    ),
                    combine(
                        Op::Difference(Blend::Exact { radius: 0.05 }),
                        vec![NodeId(0), NodeId(1)],
                    ),
                ],
                NodeId(2),
            )
        }
        7 => {
            // ⭐⭐⭐ **UM VERBO POR FORMA** (W97) — a receita numa lista PLANA.
            //
            // ⚠️ **O que esta cena tem de provar é a AUSÊNCIA de parentescos.** Antes desta wave,
            // dois furos com raios de junção diferentes exigiam **dois grupos aninhados** — a queixa
            // que abriu a wave. Aqui os quatro nós são irmãos de um grupo só, e a Hierarquia lê-se
            // de cima para baixo como a receita: `BSE` · `UNI` · `SUB` · `SUB`.
            //
            // ⚠️ A 2.ª forma é **CALADA de propósito**: ela herda o verbo *e o filete* do grupo
            // (`Union` a `0,08`), e é o que torna a herança visível ao lado das que se pronunciam.
            println!(
                "[field-smoke] cena 7 — UM VERBO POR FORMA: 4 irmãos, zero aninhamento. \
                 Bloco (BSE) · bossa CALADA que herda o filete 0,08 (UNI) · furo em pé com junção \
                 0,10 (SUB) · furo deitado de aresta VIVA (SUB)"
            );
            let mut furo_gordo = leaf(
                Primitive::Cylinder {
                    radius: 0.15,
                    half_height: 1.2,
                    round: 0.0,
                },
                Xform::at(-0.22, 0.0, 0.0),
            );
            // ⭐ O filete da junção viaja DENTRO do verbo — é o que faz «um raio por objeto» existir.
            furo_gordo.verb = Some(Op::Difference(Blend::Exact { radius: 0.10 }));
            let mut furo_vivo = leaf(
                Primitive::Cylinder {
                    radius: 0.15,
                    half_height: 1.2,
                    round: 0.0,
                },
                Xform {
                    translation: [0.3, 0.0, 0.0],
                    rotation: [s, 0.0, 0.0, s],
                    ..Xform::IDENTITY
                },
            );
            // ⚠️ O MESMO verbo, outro raio — o par que era inexprimível sem aninhar.
            furo_vivo.verb = Some(Op::Difference(Blend::Sharp));
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Box {
                            half: [0.55, 0.3, 0.4],
                            round: 0.04,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Sphere { radius: 0.26 },
                        Xform::at(0.0, 0.34, 0.0),
                    ),
                    furo_gordo,
                    furo_vivo,
                    combine(
                        Op::Union(Blend::Exact { radius: 0.08 }),
                        vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
                    ),
                ],
                NodeId(4),
            )
        }
        8 => {
            // ⭐⭐⭐ **OS TRÊS CARACTERES LADO A LADO** (W99) — a mesma junta, três formas.
            //
            // ⚠️ **O mesmo número nos três, de propósito**: é a única disposição em que se vê o que
            // a calibração compra. O filete e o orgânico põem a silhueta do canto **no mesmo
            // sítio** (medido, `the_four_characters`); o chanfro come mais, e é essa a diferença
            // que o artista escolhe.
            println!(
                "[field-smoke] cena 8 — OS TRÊS CARACTERES: três colunas com a MESMA junta de \
                 0,18 — Fillet (arco) · Chamfer (corte reto) · Organic (derretido)"
            );
            let coluna = |x: f32, blend: Blend| {
                let mut poste = leaf(
                    Primitive::Box {
                        half: [0.16, 0.5, 0.16],
                        round: 0.0,
                    },
                    Xform::at(x, 0.0, 0.0),
                );
                // ⚠️ O verbo do poste é o que carrega o carácter: cada coluna junta-se à base com a
                // forma dela.
                poste.verb = Some(Op::Union(blend));
                poste
            };
            FieldDoc::new(
                vec![
                    // A base: uma laje comum às três, para haver junta que ver.
                    leaf(
                        Primitive::Box {
                            half: [1.1, 0.12, 0.35],
                            round: 0.0,
                        },
                        Xform::at(0.0, -0.5, 0.0),
                    ),
                    coluna(-0.7, Blend::Exact { radius: 0.18 }),
                    coluna(0.0, Blend::Chamfer { radius: 0.18 }),
                    coluna(0.7, Blend::Organic { radius: 0.18 }),
                    combine(
                        Op::Union(Blend::Sharp),
                        vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
                    ),
                ],
                NodeId(4),
            )
        }
        9 => {
            println!(
                "[field-smoke] cena 9 — AS TRÊS FORMAS NOVAS (W101): cone fechado · tronco de cone \
                 · cápsula · prisma de 6 lados, todos com o filete que nasceram a ter"
            );
            // ⚠️ **Lado a lado e no MESMO tamanho**, de propósito: a cena existe para se ver o que
            // cada uma é, e uma delas maior que as outras leria como a forma sendo diferente.
            let x = |v: f32| Xform {
                translation: [v, 0.0, 0.0],
                ..Xform::IDENTITY
            };
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Cone {
                            bottom: 0.26,
                            top: 0.0,
                            half_height: 0.32,
                            round: 0.026,
                        },
                        x(-0.82),
                    ),
                    leaf(
                        Primitive::Cone {
                            bottom: 0.26,
                            top: 0.13,
                            half_height: 0.32,
                            round: 0.026,
                        },
                        x(-0.27),
                    ),
                    leaf(
                        Primitive::Capsule {
                            radius: 0.16,
                            half_height: 0.26,
                        },
                        x(0.27),
                    ),
                    leaf(
                        Primitive::Prism {
                            sides: 6,
                            bottom: 0.26,
                            top: 0.26,
                            half_height: 0.32,
                            round: 0.026,
                        },
                        x(0.82),
                    ),
                    // ⚠️ **União de ARESTA VIVA** (`Blend::Sharp`): elas não se tocam, e um filete
                    // de junção aqui seria um número que não faz nada — a cena mostraria um
                    // controlo que o artista concluiria estar partido.
                    combine(
                        Op::Union(Blend::Sharp),
                        vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
                    ),
                ],
                NodeId(4),
            )
        }
        10 => {
            println!(
                "[field-smoke] cena 10 — O LOTE DA W102: pirâmide · tronco de pirâmide · cunha · \
                 arco de toro (meia volta), lado a lado"
            );
            let x = |v: f32| Xform {
                translation: [v, 0.0, 0.0],
                ..Xform::IDENTITY
            };
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Prism {
                            sides: 4,
                            bottom: 0.26,
                            top: 0.0,
                            half_height: 0.34,
                            round: 0.026,
                        },
                        x(-0.82),
                    ),
                    leaf(
                        Primitive::Prism {
                            sides: 4,
                            bottom: 0.26,
                            top: 0.13,
                            half_height: 0.32,
                            round: 0.026,
                        },
                        x(-0.27),
                    ),
                    leaf(
                        Primitive::Wedge {
                            half: [0.26, 0.18, 0.21],
                            round: 0.013,
                        },
                        x(0.27),
                    ),
                    leaf(
                        Primitive::TorusArc {
                            major: 0.26,
                            minor: 0.073,
                            angle: std::f32::consts::PI,
                        },
                        x(0.82),
                    ),
                    // ⚠️ Aresta viva na junção, como a cena 9: elas não se tocam, e um filete de
                    // junção seria um número que não faz nada.
                    combine(
                        Op::Union(Blend::Sharp),
                        vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
                    ),
                ],
                NodeId(4),
            )
        }
        _ => {
            println!(
                "[field-smoke] cena 1 — junção de 3 cilindros: filete interno 0,12 + aros externos 0,05"
            );
            let cyl = |rot: [f32; 4]| {
                leaf(
                    Primitive::Cylinder {
                        radius: 0.22,
                        half_height: 0.78,
                        round: 0.05,
                    },
                    Xform {
                        rotation: rot,
                        ..Xform::IDENTITY
                    },
                )
            };
            FieldDoc::new(
                vec![
                    cyl([0.0, 0.0, 0.0, 1.0]),
                    cyl([s, 0.0, 0.0, s]),
                    cyl([0.0, s, 0.0, s]),
                    combine(
                        Op::Union(Blend::Exact { radius: 0.12 }),
                        vec![NodeId(0), NodeId(1), NodeId(2)],
                    ),
                ],
                NodeId(3),
            )
        }
    };
    doc.expect("as cenas do smoke são documentos válidos")
}

/// ⭐⭐ **Os gates do roteador** — nenhum existia até à W97. Ver
/// [`field3d_smoke_scene_tests`](self::scene_tests).
#[cfg(test)]
#[path = "field3d_smoke_scene_tests.rs"]
mod scene_tests;
