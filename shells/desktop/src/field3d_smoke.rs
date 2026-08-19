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
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_render::{Matcap, Orbit, shade, trace};
use ph2d_vector::{ImageQuality, VectorScene};

/// Lado do traçado. ⚠️ Fixo e modesto de propósito: a pergunta deste smoke é *"a forma está
/// certa?"*, e 640×480 responde a ela em 25 ms. Subir a resolução aqui é subir o custo do prato
/// giratório, não a qualidade da resposta.
const TRACE_W: u32 = 640;
const TRACE_H: u32 = 480;

/// Quanto a peça gira por traçado, em radianos.
const SPIN: f32 = 0.035;

/// O fundo do quadro. ⚠️ Literal aqui não fere o HR-15: isto não é UI do produto — é um smoke
/// atrás de variável de ambiente, sem widget, sem tema e sem utilizador.
const BACKGROUND: [u8; 4] = [24, 26, 30, 255];

struct Smoke {
    doc: FieldDoc,
    matcap: Arc<MatcapTexels>,
    cam: Orbit,
    /// O último quadro pronto, guardado para desenhar enquanto o próximo não chega.
    frame: Option<Arc<Vec<u8>>>,
    inflight: Option<Receiver<Vec<u8>>>,
}

struct MatcapTexels {
    side: u32,
    rgb: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<Option<Option<Smoke>>> = const { RefCell::new(None) };
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
        "[field-smoke] traçado {TRACE_W}x{TRACE_H}, prato giratório — feche a janela para sair"
    );
    Some(Smoke {
        doc,
        matcap: Arc::new(load_matcap()),
        cam: Orbit::default(),
        frame: None,
        inflight: None,
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
                Ok(rgba) => {
                    smoke.frame = Some(Arc::new(rgba));
                    smoke.inflight = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => smoke.inflight = None,
            }
        }

        // Uma requisição em voo por vez: só pede a próxima quando a anterior chegou.
        if smoke.inflight.is_none() {
            smoke.cam.yaw += SPIN;
            let (tx, rx) = channel();
            let doc = smoke.doc.clone();
            let cam = smoke.cam;
            let matcap = Arc::clone(&smoke.matcap);
            std::thread::spawn(move || {
                let g = trace(&doc, &cam, TRACE_W, TRACE_H);
                let px = shade(
                    &g,
                    &Matcap {
                        side: matcap.side,
                        rgb_linear: &matcap.rgb,
                    },
                    BACKGROUND,
                );
                // O receptor pode ter sumido (janela fechada): descartar é a resposta certa.
                let _ = tx.send(px);
            });
            smoke.inflight = Some(rx);
        }

        if let Some(frame) = &smoke.frame {
            // Encaixa o quadro na área mantendo a proporção do traçado — esticar deformaria a peça,
            // que é exatamente o que este smoke existe para deixar ver.
            let scale = (area.w / TRACE_W as f32).min(area.h / TRACE_H as f32);
            let (dw, dh) = (TRACE_W as f32 * scale, TRACE_H as f32 * scale);
            let x0 = area.x + (area.w - dw) * 0.5;
            let y0 = area.y + (area.h - dh) * 0.5;
            scene_out.draw_image_rgba(
                frame,
                TRACE_W,
                TRACE_H,
                (
                    f64::from(x0),
                    f64::from(y0),
                    f64::from(x0 + dw),
                    f64::from(y0 + dh),
                ),
                // Bicúbico: a imagem é reamostrada para a tela, e o que este smoke mostra é
                // justamente a suavidade de uma superfície curva e a retidão de uma aresta.
                ImageQuality::High,
            );
        }
    });
}
