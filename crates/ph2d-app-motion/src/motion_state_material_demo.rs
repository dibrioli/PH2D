//! ⭐⭐⭐ **DE QUE A PEÇA É FEITA** (`PH2D_GPU_COOK_DEMO=115`) — o MATERIAL
//! ([doc 109 §7](../../docs/Motion%20Nodes/109_o_colisor_na_forma.md)).
//!
//! ## O report que a escreveu
//!
//! *«Os círculos não rotacionam com a colisão, talvez por falta de atrito. Precisamos de parâmetros
//! do material»* (dono, 2026-09-13) — e ele tinha razão pela conta exacta: a correcção de
//! não-penetração empurra ao longo da NORMAL, e num disco a alavanca da normal é **zero**. Nenhum
//! número dado àquela metade podia rodar um círculo. O atrito é a outra metade, e nela a alavanca
//! do mesmo disco é o raio inteiro.
//!
//! ## O que se vê — quatro quadrantes, e em cada par muda UM número
//!
//! ```text
//!   EM CIMA   a mesma rampa, a mesma bola      Friction 0  →  DESLIZA sem virar
//!                                              Friction 1  →  ROLA
//!   EM BAIXO  o mesmo chão, a mesma queda      Bounciness 0   →  morre onde cai
//!                                              Bounciness 0,9 →  SALTA
//! ```
//!
//! ⚠️ **A bola tem um tracejado no contorno, e ele é a razão de ela ser desenhada assim:** um
//! círculo liso rodado é indistinguível de um círculo parado. *Uma cena que demonstra rotação tem
//! de desenhar uma coisa cuja rotação se VEJA.*
//!
//! ⚠️ **O que difere entre as duas metades de um par é UM param do cartão da forma** — a rampa, o
//! chão, o tamanho, a gravidade e a duração são os mesmos nós com os mesmos números. Se a
//! diferença estivesse no obstáculo, a cena ensinaria que o material é do mundo; ele é da PEÇA.
//!
//! ⚠️ **Ela precisa de Play** — é uma simulação, como a `=113` e a `=114`.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_motion_shape::param;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// O raio da bola (o `size` do `source.shape`: a geometria do círculo nasce em raio 1).
const RAIO: f32 = 0.22;
/// A inclinação da rampa, em graus. ⚠️ **Suave de propósito:** a `30°` a bola sai do quadrante
/// antes de o laço reiniciar, e o par deixa de se comparar lado a lado.
const RAMPA_GRAUS: f32 = 12.0;
/// Onde a superfície da rampa passa, na vertical do quadrante.
const RAMPA_Y: f32 = 0.7;
/// A que distância do centro do quadrante a bola é largada — do lado de CIMA da rampa.
const PARTIDA_X: f32 = 0.95;
/// O chão da fileira de baixo, e de que altura a bola cai nele.
const CHAO_Y: f32 = -2.35;
const QUEDA_Y: f32 = -0.85;

/// Quanto as duas colunas se afastam.
const VAO: f32 = 2.3;
/// A linha de cada fileira no grafo.
const LINHA_RAMPA: f32 = 120.0;
const LINHA_CHAO: f32 = 700.0;

/// A gravidade (`force.wind` para baixo, sem rajada — o doc dele diz que assim ele **é**
/// gravidade), e o relógio do laço.
const GRAVIDADE: f32 = 4.0;
const DURACAO: f32 = 2.2;
const PAUSA: f32 = 0.6;

/// O atrito da RAMPA e do CHÃO — o mesmo nos quatro quadrantes, de propósito: o que muda é a peça.
///
/// ⚠️ **`1` na rampa para o par do atrito ser sobre a BOLA**: com um obstáculo escorregadio
/// `√(μ_rampa · μ_bola)` seria pequeno nos dois lados e a cena mostraria duas bolas a deslizar.
const ATRITO_DO_MUNDO: f32 = 1.0;

/// **A NORMAL de um plano inclinado `graus`** — pela MESMA porta que o solver usa para girar um
/// colisor (`Colisor::girado`), e não por dois literais que envelhecem no dia em que o ângulo mudar.
fn normal_da_rampa(graus: f32) -> [f32; 2] {
    match ph2d_contact::Colisor::caixa([1.0, 0.0], [1.0, 0.0])
        .girado(graus)
        .forma
    {
        ph2d_contact::Forma::Caixa { eixo, .. } => [0.0 - eixo[1], eixo[0]],
        // Inalcançável: o construtor acima é uma caixa. Um chão horizontal é a resposta honesta.
        ph2d_contact::Forma::Disco(_) => [0.0, 1.0],
    }
}

/// O deslocamento de Hesse (o param `height` do `sim.collide`) de um plano com normal `n` que
/// passa pelo ponto `p`.
fn offset_de(n: [f32; 2], p: [f32; 2]) -> f32 {
    p[0] * n[0] + p[1] * n[1]
}

/// Onde a superfície de um plano `(n, offset)` está na vertical de `x`.
fn superficie_em(n: [f32; 2], offset: f32, x: f32) -> f32 {
    (offset - x * n[0]) / n[1]
}

/// Um quadrante: o que ele mostra, e qual é o número que o separa do irmão.
struct Quadrante {
    x: f32,
    rampa: bool,
    atrito: f32,
    salto: f32,
    rotulo: &'static str,
}

fn quadrantes() -> [Quadrante; 4] {
    [
        Quadrante {
            x: -VAO,
            rampa: true,
            atrito: 0.0,
            salto: 0.0,
            rotulo: "Friction 0: desliza sem virar",
        },
        Quadrante {
            x: VAO,
            rampa: true,
            atrito: 1.0,
            salto: 0.0,
            rotulo: "Friction 1: ROLA",
        },
        Quadrante {
            x: -VAO,
            rampa: false,
            atrito: 0.5,
            salto: 0.0,
            rotulo: "Bounciness 0: morre onde cai",
        },
        Quadrante {
            x: VAO,
            rampa: false,
            atrito: 0.5,
            salto: 0.9,
            rotulo: "Bounciness 0,9: SALTA",
        },
    ]
}

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    quadrantes()
        .iter()
        .map(|q| {
            let y = if q.rampa {
                RAMPA_Y - 1.05
            } else {
                CHAO_Y - 0.35
            };
            Caption::new([q.x, y], q.rotulo)
        })
        .collect()
}

/// Monta os quatro quadrantes. Devolve os sinks pela ordem de [`quadrantes`].
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};

    // ⚠️ Os índices de enum são PERGUNTADOS ao registo, nunca digitados — a porta da cena `=113`.
    let plano = super::sim_demo::indice_de(reg, "sim.collide", "shape", "Plane")?;
    let circulo = super::sim_demo::indice_de(reg, "source.shape", "kind", "Circle")?;
    let colisor_redondo =
        super::sim_demo::indice_de(reg, "source.shape", param::COLLIDER_SHAPE, "Circle")?;
    let em_laco = super::sim_demo::indice_de(reg, "sim.zone", "mode", "Loop")?;
    let n = normal_da_rampa(RAMPA_GRAUS);

    let mut sinks = Vec::new();
    for (i, q) in quadrantes().into_iter().enumerate() {
        let g = &mut doc.graph;
        let forma = g.add_node("source.shape");
        g.set_param(forma, param::KIND, circulo);
        g.set_param(forma, param::SIZE, RAIO);
        g.set_param(forma, param::COLLIDE, 1.0);
        // O colisor É o círculo que a arte desenha — o índice PERGUNTADO ao registo, como o `kind`.
        g.set_param(forma, param::COLLIDER_SHAPE, colisor_redondo);
        // ⭐ **O que a cena pergunta** — um número por quadrante, e mais nada muda.
        g.set_param(forma, param::FRICTION, q.atrito);
        g.set_param(forma, param::BOUNCE, q.salto);
        // ⚠️ **O TRACEJADO é o que torna a rotação visível** — ver o cabeçalho.
        g.set_param(forma, param::STROKE_WIDTH, RAIO * 0.28);
        g.set_param(forma, param::DASH, 2.0);
        g.set_param(forma, param::DASH_GAP, 2.0);

        let (offset, partida) = if q.rampa {
            let off = offset_de(n, [q.x, RAMPA_Y]);
            let x = q.x + PARTIDA_X;
            let y = superficie_em(n, off, x);
            // Pousada na rampa, com uma folga que o primeiro tique fecha.
            (off, [x + n[0] * RAIO, y + n[1] * RAIO + 0.02])
        } else {
            (CHAO_Y, [q.x, QUEDA_Y])
        };

        let alto = g.add_node("motion.transform");
        g.set_param(alto, "offset_x", partida[0]);
        g.set_param(alto, "offset_y", partida[1]);

        let zone = g.add_node("sim.zone");
        g.set_param(zone, "mode", em_laco);
        g.set_param(zone, "duration", DURACAO);
        g.set_param(zone, "loop_delay", PAUSA);

        let vento = g.add_node("force.wind");
        g.set_param(vento, "angle", 270.0);
        g.set_param(vento, "strength", GRAVIDADE);
        g.set_param(vento, "gust", 0.0);

        let passo = g.add_node("sim.step");

        let chao = g.add_node("sim.collide");
        g.set_param(chao, "shape", plano);
        g.set_param(chao, "height", offset);
        g.set_param(chao, "angle", if q.rampa { RAMPA_GRAUS } else { 0.0 });
        // ⚠️ **Os mesmos números nos quatro**: o que muda é a peça, nunca o mundo.
        g.set_param(chao, "friction", ATRITO_DO_MUNDO);
        g.set_param(chao, "restitution", 0.0);

        let out = g.add_node("motion.output");

        #[expect(clippy::cast_precision_loss, reason = "um indice de quadrante")]
        let y_linha = if q.rampa { LINHA_RAMPA } else { LINHA_CHAO } + (i % 2) as f32 * 250.0;
        for (k, no) in [forma, alto, zone].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
            let x = 40.0 + k as f32 * 176.0;
            g.set_pos(no, Pos { x, y: y_linha });
        }
        for (k, no) in [vento, passo, chao, out].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
            let x = 40.0 + k as f32 * 176.0;
            g.set_pos(
                no,
                Pos {
                    x,
                    y: y_linha + 125.0,
                },
            );
        }
        // ⚠️ A aresta `zone -> vento` é `delayed`: é a entrada de estado que fecha o laço.
        for (a, ap, b, bp, delayed) in [
            (forma, 0, alto, 0, false),
            (alto, 0, zone, 0, false),
            (zone, 0, vento, 0, true),
            (vento, 0, passo, 0, false),
            (passo, 0, chao, 0, false),
            (chao, 0, zone, 1, false),
            (zone, 0, out, 0, false),
        ] {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed,
            })
            .ok()?;
        }
        g.set_label(forma, q.rotulo);
        sinks.push(out);
    }
    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_material_demo_tests.rs"]
mod tests;
