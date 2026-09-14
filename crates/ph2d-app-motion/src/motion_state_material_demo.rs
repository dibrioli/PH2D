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
const RAIO: f32 = 0.2;
/// A inclinação da rampa, em graus. ⚠️ **Suave de propósito:** a `30°` a bola sai do quadrante
/// antes de o laço reiniciar, e o par deixa de se comparar lado a lado.
const RAMPA_GRAUS: f32 = 12.0;
/// Onde a superfície da rampa passa, na vertical do quadrante.
const RAMPA_Y: f32 = 1.05;
/// A que distância do centro do quadrante a bola é largada — do lado de CIMA da rampa.
const PARTIDA_X: f32 = 0.9;
/// O chão da fileira do meio, e de que altura a bola cai nele.
const CHAO_Y: f32 = 0.0;
const QUEDA_Y: f32 = 0.95;
/// A taça da fileira de baixo: onde ela está, que raio tem, e a grelha de bolas que cai dentro.
const TACA: [f32; 2] = [0.0, -1.8];
const TACA_R: f32 = 1.2;
const TACA_LADO: f32 = 4.0;
const TACA_VAO: f32 = 0.28;
/// O raio das bolas da taça — menores que as de cima, para caberem muitas.
const TACA_RAIO: f32 = 0.11;
/// Onde a grelha nasce, relativa ao centro da taça: ALTA e DE LADO, para elas caírem em
/// cascata por uma parede e esfregarem umas nas outras a descer.
const TACA_PARTIDA: [f32; 2] = [0.28, 0.34];

/// Quanto as duas colunas se afastam.
const VAO: f32 = 2.3;
/// A linha de cada fileira no grafo.
const LINHA: f32 = 120.0;
const ALTURA_DA_LINHA: f32 = 260.0;

/// A gravidade (`force.wind` para baixo, sem rajada — o doc dele diz que assim ele **é**
/// gravidade), e o relógio do laço.
const GRAVIDADE: f32 = 4.0;
const DURACAO: f32 = 2.2;
const PAUSA: f32 = 0.6;

/// O atrito da RAMPA e do CHÃO — o mesmo nos quatro quadrantes de cima, de propósito: o que muda
/// é a peça.
///
/// ⚠️ **`1` para o par do atrito ser sobre a BOLA**: com um obstáculo escorregadio
/// `√(μ_mundo · μ_bola)` seria pequeno nos dois lados e a cena mostraria duas bolas a deslizar.
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

/// O que um quadrante encena.
#[derive(Clone, Copy, PartialEq)]
enum Cena {
    /// Uma bola numa rampa — o material da peça contra o MUNDO.
    Rampa,
    /// Uma bola a cair no chão — idem, na outra propriedade.
    Queda,
    /// ⭐⭐ **Uma taça de bolas — o material de uma peça contra OUTRA PEÇA** (3.º report do dono,
    /// 2026-09-13: *«as propriedades entre as próprias shapes não funcionam»*).
    ///
    /// ⚠️ **A taça é ESCORREGADIA nas duas metades** (`friction = 0`), e é isso que torna o par
    /// honesto: com um obstáculo áspero a diferença que se vê seria metade do mundo e metade das
    /// peças, e o quadrante não estaria a dizer o que promete. Aqui o ÚNICO atrito em jogo é o
    /// de bola contra bola.
    ///
    /// ⚠️ **E uma taça, não um chão** — pela mesma razão que a `=114`: num plano as peças
    /// espalham-se e quase não se tocam; uma taça junta-as todas no mesmo ponto baixo.
    Taca,
}

/// Um quadrante: o que ele mostra, e qual é o número que o separa do irmão.
struct Quadrante {
    x: f32,
    cena: Cena,
    atrito: f32,
    salto: f32,
    rotulo: &'static str,
}

fn quadrantes() -> [Quadrante; 6] {
    [
        Quadrante {
            x: -VAO,
            cena: Cena::Rampa,
            atrito: 0.0,
            salto: 0.0,
            rotulo: "Friction 0: desliza sem virar",
        },
        Quadrante {
            x: VAO,
            cena: Cena::Rampa,
            atrito: 1.0,
            salto: 0.0,
            rotulo: "Friction 1: ROLA",
        },
        Quadrante {
            x: -VAO,
            cena: Cena::Queda,
            atrito: 0.5,
            salto: 0.0,
            rotulo: "Bounciness 0: morre onde cai",
        },
        Quadrante {
            x: VAO,
            cena: Cena::Queda,
            atrito: 0.5,
            salto: 0.9,
            rotulo: "Bounciness 0,9: SALTA",
        },
        Quadrante {
            x: -VAO,
            cena: Cena::Taca,
            atrito: 0.0,
            salto: 0.0,
            rotulo: "Entre bolas, Friction 0: escorregam",
        },
        Quadrante {
            x: VAO,
            cena: Cena::Taca,
            atrito: 1.0,
            salto: 0.0,
            rotulo: "Entre bolas, Friction 1: ROLAM umas nas outras",
        },
    ]
}

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    quadrantes()
        .iter()
        .map(|q| {
            let y = match q.cena {
                Cena::Rampa => RAMPA_Y + 0.5,
                Cena::Queda => CHAO_Y - 0.3,
                Cena::Taca => TACA[1] - TACA_R - 0.3,
            };
            Caption::new([q.x, y], q.rotulo)
        })
        .collect()
}

/// Monta os seis quadrantes. Devolve os sinks pela ordem de [`quadrantes`].
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};

    // ⚠️ Os índices de enum são PERGUNTADOS ao registo, nunca digitados — a porta da cena `=113`.
    let plano = super::sim_demo::indice_de(reg, "sim.collide", "shape", "Plane")?;
    let taca = super::sim_demo::indice_de(reg, "sim.collide", "shape", "Bowl")?;
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
        g.set_param(
            forma,
            param::SIZE,
            if q.cena == Cena::Taca {
                TACA_RAIO
            } else {
                RAIO
            },
        );
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

        // ⭐ **A TAÇA precisa de muitas bolas** — uma peça só nunca toca noutra, e o quadrante
        // que promete *«entre bolas»* estaria a mostrar o mesmo que a fileira de cima.
        let carimbo = (q.cena == Cena::Taca).then(|| {
            let grid = g.add_node("motion.grid");
            g.set_param(grid, "rows", TACA_LADO);
            g.set_param(grid, "cols", TACA_LADO);
            g.set_param(grid, "gap_x", TACA_VAO);
            g.set_param(grid, "gap_y", TACA_VAO);
            (grid, g.add_node("motion.duplicator"))
        });

        let alto = g.add_node("motion.transform");
        let zone = g.add_node("sim.zone");
        g.set_param(zone, "mode", em_laco);
        g.set_param(zone, "duration", DURACAO);
        g.set_param(zone, "loop_delay", PAUSA);

        let vento = g.add_node("force.wind");
        g.set_param(vento, "angle", 270.0);
        g.set_param(vento, "strength", GRAVIDADE);
        g.set_param(vento, "gust", 0.0);

        let passo = g.add_node("sim.step");
        let mundo = g.add_node("sim.collide");
        let out = g.add_node("motion.output");

        match q.cena {
            Cena::Rampa => {
                let off = offset_de(n, [q.x, RAMPA_Y]);
                let x = q.x + PARTIDA_X;
                let y = superficie_em(n, off, x);
                // Pousada na rampa, com uma folga que o primeiro tique fecha.
                g.set_param(alto, "offset_x", x + n[0] * RAIO);
                g.set_param(alto, "offset_y", y + n[1] * RAIO + 0.02);
                g.set_param(mundo, "shape", plano);
                g.set_param(mundo, "height", off);
                g.set_param(mundo, "angle", RAMPA_GRAUS);
                g.set_param(mundo, "friction", ATRITO_DO_MUNDO);
            }
            Cena::Queda => {
                g.set_param(alto, "offset_x", q.x);
                g.set_param(alto, "offset_y", QUEDA_Y);
                g.set_param(mundo, "shape", plano);
                g.set_param(mundo, "height", CHAO_Y);
                g.set_param(mundo, "friction", ATRITO_DO_MUNDO);
            }
            Cena::Taca => {
                g.set_param(alto, "offset_x", q.x + TACA[0] + TACA_PARTIDA[0]);
                g.set_param(alto, "offset_y", TACA[1] + TACA_PARTIDA[1]);
                g.set_param(mundo, "shape", taca);
                g.set_param(mundo, "center_x", q.x + TACA[0]);
                g.set_param(mundo, "center_y", TACA[1]);
                g.set_param(mundo, "radius", TACA_R);
                // ⚠️ **ESCORREGADIA nas duas metades** — ver [`Cena::Taca`]: é o que faz deste
                // par uma pergunta sobre o material ENTRE PEÇAS e não sobre o mundo.
                g.set_param(mundo, "friction", 0.0);
            }
        }
        g.set_param(mundo, "restitution", 0.0);

        #[expect(clippy::cast_precision_loss, reason = "um indice de quadrante")]
        let y_linha =
            LINHA + (i / 2) as f32 * (ALTURA_DA_LINHA * 2.0) + (i % 2) as f32 * ALTURA_DA_LINHA;
        let fila: Vec<NodeId> = carimbo
            .map(|(grid, dup)| vec![forma, grid, dup, alto, zone])
            .unwrap_or_else(|| vec![forma, alto, zone]);
        for (k, no) in fila.into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
            let x = 40.0 + k as f32 * 176.0;
            g.set_pos(no, Pos { x, y: y_linha });
        }
        for (k, no) in [vento, passo, mundo, out].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
            let x = 40.0 + k as f32 * 176.0;
            g.set_pos(
                no,
                Pos {
                    x,
                    y: y_linha + 120.0,
                },
            );
        }
        // ⚠️ A aresta `zone -> vento` é `delayed`: é a entrada de estado que fecha o laço.
        let mut arestas = vec![
            (alto, 0, zone, 0, false),
            (zone, 0, vento, 0, true),
            (vento, 0, passo, 0, false),
            (passo, 0, mundo, 0, false),
            (mundo, 0, zone, 1, false),
            (zone, 0, out, 0, false),
        ];
        match carimbo {
            Some((grid, dup)) => {
                arestas.push((forma, 0, dup, 0, false));
                arestas.push((grid, 0, dup, 1, false));
                arestas.push((dup, 0, alto, 0, false));
            }
            None => arestas.push((forma, 0, alto, 0, false)),
        }
        for (a, ap, b, bp, delayed) in arestas {
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
