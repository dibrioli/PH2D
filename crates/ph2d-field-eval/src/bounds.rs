//! ⭐ **QUANTO ESPAÇO A PEÇA OCUPA** — a esfera que a contém, composta pela árvore (W33).
//!
//! # O defeito que isto existe para curar
//!
//! O extrator montava a grade sobre `[-1, 1]` **fixo** — a caixa que o motor assume por omissão. Duas
//! consequências, e a primeira é silenciosa:
//!
//! | | consequência |
//! |---|---|
//! | uma peça que sai da caixa | ⛔ **é CORTADA na exportação**, sem uma palavra |
//! | uma peça pequena no meio dela | a grade gasta a resolução em espaço vazio |
//!
//! # ⚠️ Conservador, e a assimetria é o critério
//!
//! Toda aproximação aqui erra **para cima**: um bordo maior do que a peça custa **resolução**; um
//! bordo menor **corta a peça e não diz nada**. Não é prudência genérica — é a única direção em que
//! o erro é recuperável (quem quiser mais nitidez sobe a qualidade da exportação; quem perdeu um
//! pedaço não tem como saber que o perdeu).
//!
//! # Por que uma ESFERA
//!
//! Ela é **invariante à rotação**: subir a cadeia de poses custa `centro' = pose(centro)` e
//! `raio' = raio · escala`. Uma caixa teria de ser re-envolvida a cada nível girado, e cada
//! re-envolvimento **cresce** — três agrupamentos rodados dariam uma caixa muito maior do que a peça.
//! *A moeda certa para compor bordos é a que a composição não estraga.*

use ph2d_field::{FieldDoc, NodeId, NodeKind, Op, Unary, Xform};

/// Uma esfera de bordo: centro e raio, no referencial de quem pergunta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ball {
    pub center: [f32; 3],
    pub radius: f32,
}

impl Ball {
    /// A caixa alinhada aos eixos que a contém — o que a grade do extrator precisa.
    #[must_use]
    pub fn aabb(self) -> ([f32; 3], [f32; 3]) {
        let r = self.radius.max(0.0);
        (
            [self.center[0] - r, self.center[1] - r, self.center[2] - r],
            [self.center[0] + r, self.center[1] + r, self.center[2] + r],
        )
    }

    /// A esfera que contém as duas — a união, sem re-envolvimento que cresça de mais.
    #[must_use]
    fn merge(self, other: Self) -> Self {
        let d = [
            other.center[0] - self.center[0],
            other.center[1] - self.center[1],
            other.center[2] - self.center[2],
        ];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        // Uma contém a outra: fica a maior.
        if dist + other.radius <= self.radius {
            return self;
        }
        if dist + self.radius <= other.radius {
            return other;
        }
        let radius = (dist + self.radius + other.radius) * 0.5;
        if dist <= f32::MIN_POSITIVE {
            return Self {
                center: self.center,
                radius,
            };
        }
        let t = (radius - self.radius) / dist;
        Self {
            center: [
                self.center[0] + d[0] * t,
                self.center[1] + d[1] * t,
                self.center[2] + d[2] * t,
            ],
            radius,
        }
    }
}

/// ⭐ **A esfera que contém a peça inteira**, ou `None` num documento sem geometria.
///
/// O registo entra porque uma **escultura** só sabe a caixa dela do lado do campo amostrado (ver
/// [`crate::hybrid::Sampled::bounding_radius`]).
#[must_use]
pub fn bounding_ball(doc: &FieldDoc, reg: &crate::hybrid::Registry) -> Option<Ball> {
    of_node(doc, reg, doc.root())
}

fn of_node(doc: &FieldDoc, reg: &crate::hybrid::Registry, id: NodeId) -> Option<Ball> {
    let node = doc.nodes().get(id.0 as usize)?;
    let local = match &node.kind {
        NodeKind::Leaf(p) => Some(Ball {
            center: [0.0; 3],
            radius: ph2d_field::bounding_radius(p),
        }),
        // ⚠️ Um nome que o registo não conhece lê como **espaço vazio** (`hybrid::ABSENT`), e um
        // vazio não ocupa lugar nenhum: o bordo dele é nada, não uma caixa inventada.
        NodeKind::Sampled { key } => reg.get(key).map(|f| Ball {
            center: [0.0; 3],
            radius: f.bounding_radius(),
        }),
        NodeKind::Combine { op, children } => {
            let mut it = children.iter().filter_map(|c| of_node(doc, reg, *c));
            match op {
                // ⭐ **A subtração é o PRIMEIRO filho e mais nada**: o que se corta não acrescenta
                // matéria, e um cortador enorme e distante inflaria a caixa da peça inteira.
                Op::Difference(_) => it.next(),
                // A interseção cabe em qualquer um dos lados: o MENOR é o bordo mais apertado que
                // continua a ser um bordo.
                Op::Intersection(_) => it.reduce(|a, b| if a.radius <= b.radius { a } else { b }),
                Op::Union(_) => it.reduce(Ball::merge),
            }
        }
    }?;
    Some(place(with_mods(local, &node.mods), node.xform))
}

/// O que os modificadores fazem ao bordo — **sempre para cima**.
fn with_mods(ball: Ball, mods: &[Unary]) -> Ball {
    let mut b = ball;
    for m in mods {
        b = match *m {
            // A parede é centrada na superfície: metade cresce para fora.
            Unary::Shell { thickness } => Ball {
                radius: b.radius + thickness.abs() * 0.5,
                ..b
            },
            Unary::Offset { distance } => Ball {
                radius: b.radius + distance.max(0.0),
                ..b
            },
            // O espelho é num plano do eixo LOCAL: a cópia está com aquela coordenada trocada de
            // sinal. ⚠️ **Uma função, três eixos** — três braços com a conta escrita à mão seriam
            // três sítios onde um índice errado dá uma caixa que **corta a peça** em silêncio.
            Unary::Mirror | Unary::MirrorY | Unary::MirrorZ => {
                let k = match m {
                    Unary::Mirror => 0,
                    Unary::MirrorY => 1,
                    _ => 2,
                };
                let mut c = b.center;
                c[k] = -c[k];
                b.merge(Ball {
                    center: c,
                    radius: b.radius,
                })
            }
            // A matriz linear anda ao longo do X local.
            Unary::Array { count, spacing } => {
                let span = f32::from(u16::try_from(count.saturating_sub(1)).unwrap_or(u16::MAX))
                    * spacing.abs();
                Ball {
                    center: [b.center[0] + span * 0.5, b.center[1], b.center[2]],
                    radius: b.radius + span * 0.5,
                }
            }
            // A matriz radial gira em torno do Z local: o bordo é o círculo que o centro descreve.
            Unary::Radial { .. } => {
                let arm = b.center[0].hypot(b.center[1]);
                Ball {
                    center: [0.0, 0.0, b.center[2]],
                    radius: arm + b.radius,
                }
            }
            // A secção cresce `slope` por unidade de altura, e a altura é no máximo o próprio raio.
            Unary::Taper { slope } => Ball {
                radius: b.radius * slope.abs().mul_add(b.radius, 1.0),
                ..b
            },
        };
    }
    b
}

/// A esfera vista do referencial do **pai** — e é aqui que a invariância à rotação se paga.
fn place(ball: Ball, xform: Xform) -> Ball {
    Ball {
        center: xform.apply(ball.center),
        radius: ball.radius * xform.scale.abs(),
    }
}

#[cfg(test)]
#[path = "bounds_tests.rs"]
mod tests;
