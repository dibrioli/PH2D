//! **O smoke do módulo de modelagem 3D** — `PH2D_FIELD_SMOKE=1..3` (ADR-0161).
//!
//! Põe na tela o que o módulo de facto é: o **campo traçado**, não uma malha. É por aqui que o Enio
//! vê a quina de navalha e o filete liso que a W0 mediu.
//!
//! # É um PRATO GIRATÓRIO, e a escolha tem motivo
//!
//! A peça gira sozinha. Isso é a lei da casa — *feature nova = auto-play* — e aqui ela também evita
//! o que mais importa: **não toco no despacho de entrada**. A `line/sculpt3d` está viva e edita os
//! mesmos arquivos de `shells/desktop/`, e o `input_dispatch.rs` é exatamente onde duas linhas se
//! encontrariam. Órbita por mouse chega junto com a ferramenta (W4), onde a fiação de entrada é
//! parte do trabalho de qualquer forma.
//!
//! # Estado contido, de propósito
//!
//! O estado vive **neste arquivo**, num `thread_local`, em vez de num campo do `App`. Não é
//! preguiça: `app_state.rs` é compartilhado e a outra linha o edita: um campo novo lá é uma colisão
//! por conveniência. O que este módulo toca fora de si são **duas linhas** — o `mod` e a chamada.
//!
//! # A requisição em voo é UMA
//!
//! Traçar 640×480 custa ~25 ms (medido, `ph2d-field-render::measure_trace_cost`), e fazê-lo dentro
//! do laço de quadro comeria o orçamento inteiro (HR-4). Então o traçado roda **fora** da thread de
//! UI, com **uma requisição em voo por vez** — a mesma disciplina que o modelador original pagou
//! para descobrir (`docs/3DModeling/00_plano_port.md` §1.2.7): as respostas que chegam durante a
//! espera já nasceram velhas, e só a última interessa.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use ph2d_editor::zones::Rect as EditorRect;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Profile, Xform};
use ph2d_field_render::{Matcap, Orbit, shade, trace};
use ph2d_vec_scene::{VecPath, VecVertex};
use ph2d_vector::{Affine, ImageQuality, VectorScene};

/// O menor traçado que ainda é uma imagem — só para não pedir zero pixels a uma área degenerada.
const MIN_TRACE: u32 = 16;

/// Quanto a peça gira **por segundo**, em radianos.
///
/// ⚠️ **Por segundo, e não por quadro** (correção do smoke de 19/08). Com um passo por quadro, a
/// velocidade da peça era função do custo do traçado: baixar a resolução acelerava a rotação e
/// subi-la travava-a. Isso confunde as duas perguntas que um prato giratório responde — *"a forma
/// está certa?"* e *"isto corre depressa?"* — e faz a segunda mentir sobre a primeira.
const SPIN_RATE: f32 = 0.5;

/// O fundo do quadro: **transparente**.
///
/// ⚠️ **Correção de um smoke do Enio (19/08):** *"o fundo está cinza escuro e acima do canvas"*.
/// Um cinza opaco aqui era eu **inventando uma cor** — e uma cor de fundo inventada num app com
/// tema é a segunda resposta a uma pergunta que o tema já responde (HR-15). Com alfa zero o canvas
/// do app aparece por baixo, e o módulo deixa de ter opinião sobre o fundo.
const BACKGROUND: [u8; 4] = [0, 0, 0, 0];

struct Smoke {
    doc: FieldDoc,
    matcap: Arc<MatcapTexels>,
    cam: Orbit,
    /// O último quadro pronto — com o tamanho a que foi traçado, para o desenhar sem esticar
    /// enquanto o próximo (já do tamanho novo) não chega.
    frame: Option<(Arc<Vec<u8>>, u32, u32)>,
    inflight: Option<Receiver<Ready>>,
    /// Quando o pedido em voo saiu — é o relógio que faz a rotação ser por SEGUNDO.
    since: std::time::Instant,
    /// Já anunciou o primeiro quadro? Ver a nota do `boot`.
    announced: bool,
}

/// O que uma requisição de traçado devolve.
struct Ready {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    hits: usize,
    edges: usize,
    millis: f64,
}

struct MatcapTexels {
    side: u32,
    rgb: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<Option<Option<Smoke>>> = const { RefCell::new(None) };
}

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

/// As cenas. ⚠️ **Cada uma imprime o que montou** — se a linha não aparecer no terminal, o smoke
/// não chegou a construir nada, e a tela vazia é sintoma disso e não da geometria.
fn scene(n: u32) -> FieldDoc {
    let combine = |op: Op, children: Vec<NodeId>| Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
    };
    let leaf = |p: Primitive, x: Xform| Node {
        xform: x,
        kind: NodeKind::Leaf(p),
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

/// Carrega um matcap da casa e converte para linear f32.
///
/// ⚠️ **Os matcaps moram na `ph2d-mesh-render`, com os assets e a licença** — e é de lá que se
/// pegam, em vez de sintetizar aqui um sombreamento novo. O acoplamento é do **smoke**, não do
/// módulo: a `ph2d-field-render` recebe os texels por parâmetro e não conhece aquela crate.
#[cfg(feature = "sculpt3d")]
fn load_matcap() -> MatcapTexels {
    let id = 0usize;
    let side = ph2d_mesh_render::matcap::MATCAPS[id].side;
    let bytes = ph2d_mesh_render::matcap::decode(id);
    let n = (side as usize) * (side as usize);
    let mut rgb = Vec::with_capacity(n * 3);
    for texel in bytes.chunks_exact(8) {
        // RGBA em `f16` little-endian; o alfa é descartado (é 1 em toda parte, por construção).
        for c in 0..3 {
            let bits = u16::from_le_bytes([texel[c * 2], texel[c * 2 + 1]]);
            rgb.push(half::f16::from_bits(bits).to_f32());
        }
    }
    MatcapTexels { side, rgb }
}

/// Sem o módulo de escultura compilado não há matcap — e um cinza plano seria uma forma ilegível.
#[cfg(not(feature = "sculpt3d"))]
fn load_matcap() -> MatcapTexels {
    println!("[field-smoke] ⚠️ sem a feature `sculpt3d` não há matcap; o smoke fica sem cor");
    MatcapTexels {
        side: 0,
        rgb: Vec::new(),
    }
}

fn boot() -> Option<Smoke> {
    let n: u32 = std::env::var("PH2D_FIELD_SMOKE").ok()?.parse().unwrap_or(1);
    let doc = scene(n);
    println!(
        "[field-smoke] traçado no tamanho REAL da área, com anti-serrilhado — prato giratório, \
         feche a janela para sair"
    );
    Some(Smoke {
        doc,
        matcap: Arc::new(load_matcap()),
        cam: Orbit::default(),
        frame: None,
        inflight: None,
        since: std::time::Instant::now(),
        announced: false,
    })
}

/// Desenha o smoke sobre a área dada. No-op silencioso quando a variável não está posta.
pub(crate) fn draw(area: EditorRect, scene_out: &mut VectorScene) {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let smoke = slot.get_or_insert_with(boot);
        let Some(smoke) = smoke.as_mut() else {
            return;
        };

        // Colhe o traçado que ficou pronto, se ficou.
        if let Some(rx) = &smoke.inflight {
            match rx.try_recv() {
                Ok(r) => {
                    if !smoke.announced {
                        smoke.announced = true;
                        // ⚠️ Uma linha, uma vez. É ela que separa "o smoke subiu" de "o smoke
                        // DESENHOU": o boot já imprime acima, e um boot sem quadro é exatamente o
                        // modo de falha em que a janela fica vazia e ninguém sabe de quem é a culpa.
                        // Zero pixels aqui = a peça está fora do quadro ou o campo saiu vazio.
                        println!(
                            "[field-smoke] primeiro quadro desenhado — {}x{}, {} pixels de peça, \
                             {} de borda re-amostrada, {:.1} ms",
                            r.width, r.height, r.hits, r.edges, r.millis
                        );
                    }
                    smoke.frame = Some((Arc::new(r.rgba), r.width, r.height));
                    smoke.inflight = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => smoke.inflight = None,
            }
        }

        // ⚠️ **O traçado sai no tamanho REAL da área** (correção do smoke de 19/08: *"render
        // pixelado"*). Antes ele era fixo em 640×480 e o desenho reamostrava para a área — e uma
        // imagem reamostrada é uma imagem com metade da informação, num módulo cuja razão de
        // existir é a nitidez da aresta. *A resolução do traçado é a da tela, não a de um número
        // que alguém escolheu.*
        let (tw, th) = (
            (area.w.round().max(1.0) as u32).max(MIN_TRACE),
            (area.h.round().max(1.0) as u32).max(MIN_TRACE),
        );

        // Uma requisição em voo por vez: só pede a próxima quando a anterior chegou.
        if smoke.inflight.is_none() {
            let dt = smoke.since.elapsed().as_secs_f32();
            smoke.since = std::time::Instant::now();
            // Trava o passo: se a janela ficou minimizada meia hora, a peça não dá vinte voltas de
            // uma vez.
            smoke.cam.yaw += SPIN_RATE * dt.min(0.25);
            let (tx, rx) = channel::<Ready>();
            let doc = smoke.doc.clone();
            let cam = smoke.cam;
            let matcap = Arc::clone(&smoke.matcap);
            std::thread::spawn(move || {
                let t0 = std::time::Instant::now();
                let g = trace(&doc, &cam, tw, th);
                let rgba = shade(
                    &g,
                    &Matcap {
                        side: matcap.side,
                        rgb_linear: &matcap.rgb,
                    },
                    BACKGROUND,
                );
                // O receptor pode ter sumido (janela fechada): descartar é a resposta certa.
                let _ = tx.send(Ready {
                    rgba,
                    width: tw,
                    height: th,
                    hits: g.hits(),
                    edges: g.edges.len(),
                    millis: t0.elapsed().as_secs_f64() * 1000.0,
                });
            });
            smoke.inflight = Some(rx);
        }

        if let Some((frame, fw, fh)) = &smoke.frame {
            // O quadro cobre a área toda — ele foi traçado com a proporção dela. Enquanto o
            // primeiro traçado do tamanho novo não chega, o anterior estica; é um quadro só, e
            // esticar é melhor do que piscar.
            scene_out.draw_image_rgba_premultiplied_transformed(
                frame,
                *fw,
                *fh,
                Affine::translate((f64::from(area.x), f64::from(area.y)))
                    * Affine::scale_non_uniform(
                        f64::from(area.w) / f64::from(*fw),
                        f64::from(area.h) / f64::from(*fh),
                    ),
                // ⚠️ **Bilinear, não bicúbico.** No caso normal o mapeamento é 1:1 e os dois são a
                // identidade — mas o bicúbico **toca** (*ringing*) numa aresta de alto contraste, e
                // agora que a aresta sai anti-serrilhada do traçador, um halo posto pelo filtro
                // seria o próprio artefato que se acabou de remover.
                ImageQuality::Medium,
            );
        }
    });
}

/// ⚠️ A metade de shell da ponte ECS vive num arquivo irmão, pendurada aqui pelo padrão do
/// `joint_rig`: o que ela prova — que o componente sobrevive ao snapshot real — só o shell sabe.
#[cfg(test)]
#[path = "field3d_snapshot_tests.rs"]
mod snapshot_tests;

#[cfg(test)]
mod scene_tests {
    use super::*;

    /// ⭐ **Toda cena do smoke constrói E DESENHA.**
    ///
    /// O modo de falha deste smoke não é o pânico: é a **janela vazia** — a peça fora do quadro, o
    /// perfil recusado, o campo que saiu sem interior. A linha *"primeiro quadro desenhado — N
    /// pixels"* existe para o Enio conseguir ver isso; este gate existe para ninguém precisar de
    /// abrir a janela para saber.
    #[test]
    fn every_smoke_scene_builds_and_draws_something() {
        for n in 1..=5 {
            let doc = scene(n);
            let g = ph2d_field_render::trace(&doc, &Orbit::default(), 160, 120);
            assert!(
                g.hits() > 200,
                "a cena {n} traçou só {} pixels de peça em 160x120 — a peça está fora do quadro \
                 ou o campo saiu vazio",
                g.hits()
            );
        }
    }
}
