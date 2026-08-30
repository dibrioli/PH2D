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
    /// A bola de um nó que não ocupa lugar nenhum — o degenerado seguro de um nome desconhecido.
    pub const EMPTY: Ball = Ball {
        center: [0.0; 3],
        radius: 0.0,
    };

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
    pub fn merge(self, other: Self) -> Self {
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

/// ⭐⭐⭐ **A bola LOCAL de cada nó** — antes dos modificadores e antes da pose —, numa passagem só.
///
/// ⚠️ **É a mesma lei do [`bounding_ball`], e não uma segunda:** o `of_node` dobra exactamente estas
/// respostas. Ela existe porque a [`crate::stacked`] precisa de saber **de onde parte** a pilha de
/// modificadores de um nó, e ninguém lho podia dizer — a torção tira daí o divisor que a mantém uma
/// distância honesta.
///
/// ⚠️ **Uma passagem PARA A FRENTE basta**, e é a mesma invariante que o [`crate::compile`] usa: a
/// arena garante que todo filho tem índice menor que o do pai.
#[must_use]
pub fn local_balls(doc: &FieldDoc, reg: &crate::hybrid::Registry) -> Vec<Option<Ball>> {
    let mut local: Vec<Option<Ball>> = Vec::with_capacity(doc.nodes().len());
    let mut placed: Vec<Option<Ball>> = Vec::with_capacity(doc.nodes().len());
    for node in doc.nodes() {
        let here = match &node.kind {
            NodeKind::Leaf(p) => Some(Ball {
                center: [0.0; 3],
                radius: ph2d_field::bounding_radius(p),
            }),
            NodeKind::Sampled { key } => reg.get(key).map(|f| Ball {
                center: [0.0; 3],
                radius: f.bounding_radius(),
            }),
            NodeKind::Combine { op, children } => {
                fold_children(doc, *op, children, |c| placed[c.0 as usize])
            }
        };
        local.push(here);
        placed.push(here.map(|b| place(with_mods(b, &node.mods), node.xform)));
    }
    local
}

/// ⭐ **A DOBRA dos filhos de um `Combine`**, com o verbo EFECTIVO de cada um — a lei, num sítio só.
///
/// ⚠️ **Ela saiu do `of_node` porque ganhou um segundo leitor** ([`local_balls`]): a recursão e a
/// passagem para a frente respondem à mesma pergunta, e escrevê-la duas vezes seria a forma de as
/// duas divergirem no dia seguinte.
///
/// ⛔ **A pergunta é o verbo do FILHO, não o do grupo** (2026-08-29). O defeito era silencioso e
/// assimétrico: com o grupo em `Difference` e um filho a pedir `Union`, o bordo ficava só com o
/// primeiro filho, e o que o segundo **acrescenta** caía fora da caixa do mundo ⇒ a peça sai
/// **cortada**, sem uma palavra.
///
/// ⚠️ O primeiro filho **semeia** e o verbo dele não é perguntado — a lei do `fold_verb`, a mesma
/// que o `combine_trees` e o `gradient_bound` pagam.
///
/// ⚠️ **O raio do filete não entra**, e é medido pela geometria: um arredondamento enche o vinco
/// côncavo, que fica **dentro** da união dos dois bordos. Ele não cresce a peça.
fn fold_children(
    doc: &FieldDoc,
    op: Op,
    children: &[NodeId],
    mut ball_of: impl FnMut(NodeId) -> Option<Ball>,
) -> Option<Ball> {
    let mut acc: Option<Ball> = None;
    for c in children {
        let Some(ball) = ball_of(*c) else {
            continue;
        };
        let Some(a) = acc else {
            acc = Some(ball);
            continue;
        };
        let verb = doc.node(*c).and_then(|n| n.verb);
        acc = Some(match ph2d_field::fold_verb(op, verb) {
            // ⭐ **O que se corta não acrescenta matéria** — e um cortador enorme e distante
            // inflaria a caixa da peça inteira.
            Op::Difference(_) => a,
            // A interseção cabe em qualquer um dos lados: o MENOR é o bordo mais apertado que
            // continua a ser um bordo.
            Op::Intersection(_) => {
                if a.radius <= ball.radius {
                    a
                } else {
                    ball
                }
            }
            Op::Union(_) => Ball::merge(a, ball),
        });
    }
    acc
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
        // ⭐⭐⭐ **A DOBRA, PASSO A PASSO, com o verbo EFECTIVO de cada filho** (2026-08-29).
        //
        // ⛔ **Isto perguntava `op` — o verbo DO GRUPO — e a W97 pôs um verbo em cada forma.** O
        // defeito era silencioso e assimétrico: com o grupo em `Difference` e um filho a pedir
        // `Union`, o bordo ficava só com o primeiro filho, e o que o segundo **acrescenta** caía
        // fora da caixa do mundo ⇒ a peça sai **cortada**, sem uma palavra.
        //
        // ⚠️ **Irmão do defeito que o Enio viu na marcha** (`step::gradient_bound`), achado ao
        // perguntar *«quem MAIS lê a mistura do grupo?»*. *Achar uma metade de uma família é motivo
        // para procurar as outras.*
        //
        // ⚠️ O primeiro filho **semeia** e o verbo dele não é perguntado — a lei do `fold_verb`, a
        // mesma que o `combine_trees` e o `gradient_bound` pagam.
        //
        // ⚠️ **O raio do filete não entra**, e é medido pela geometria: um arredondamento enche o
        // vinco côncavo, que fica **dentro** da união dos dois bordos. Ele não cresce a peça.
        NodeKind::Combine { op, children } => {
            fold_children(doc, *op, children, |c| of_node(doc, reg, c))
        }
    }?;
    Some(place(with_mods(local, &node.mods), node.xform))
}

/// O que os modificadores fazem ao bordo — **sempre para cima**.
///
/// ⚠️ **Pública desde 2026-08-30**: a pilha ([`crate::stacked`]) precisa do bordo do **FIM** dela para
/// preçar cada passo, e não do bordo corrente. Ver a nota do `step_divisor`.
#[must_use]
pub fn with_mods(ball: Ball, mods: &[Unary]) -> Ball {
    mods.iter().fold(ball, |b, m| step_mod(b, *m))
}

/// ⭐⭐⭐ **O ENVELOPE da pilha** — a bola que contém **todos** os estados intermédios dela.
///
/// ⛔ **A bola do FIM não serve, e isso foi medido:** a repetição radial **re-centra** no eixo, logo
/// a pilha não é monótona — `[Taper, Radial]` acabava com um bordo mais apertado do que o do passo
/// do meio, e um divisor calculado sobre ele lia `‖∇f‖ = 730,5`. *Uma cerca tem de conter o percurso
/// todo, e não o destino.*
#[must_use]
pub fn envelope(ball: Ball, mods: &[Unary]) -> Ball {
    let mut corrente = ball;
    let mut env = ball;
    for m in mods {
        corrente = step_mod(corrente, *m);
        env = env.merge(corrente);
    }
    env
}

/// ⭐⭐⭐ **O que UM modificador faz ao bordo** — a lei, num sítio só.
///
/// ⚠️ **Ela saiu do `with_mods` porque ganhou um SEGUNDO leitor** (2026-08-30): a pilha de
/// modificadores ([`crate::stacked`]) precisa de saber, a cada passo, quão longe do eixo a peça vai
/// — é disso que a torção tira o divisor que a mantém uma distância honesta. *Uma lei com dois
/// leitores é uma porta; escrita duas vezes, são duas respostas que começam a divergir no dia em que
/// alguém acrescentar um modificador.*
#[must_use]
pub fn step_mod(b: Ball, m: Unary) -> Ball {
    match m {
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
        // ⭐ **A torção varre em torno do Z local, e a bola dela é a MESMA da matriz radial** — cada
        // fatia de `z` é uma rotação em torno da origem, logo `‖(x,y)‖` e `z` são preservados. Uma
        // bola já centrada no eixo fica **inalterada ao bit**; uma descentrada varre o círculo que o
        // centro descreve. *Não se escreve lei nova: aponta-se para a que existe.*
        Unary::Radial { .. } | Unary::Twist { .. } => {
            let arm = b.center[0].hypot(b.center[1]);
            Ball {
                center: [0.0, 0.0, b.center[2]],
                radius: arm + b.radius,
            }
        }
        // A secção cresce `slope` por unidade de altura, e a altura é no máximo o próprio raio.
        // ⛔⛔ **ELA IGNORAVA O CENTRO DA BOLA, e a peça saía da caixa do mundo** (auditoria de
        // 2026-08-30, defeito **pré-existente** desde a W18).
        //
        // A secção escala por `k(y) = 1 + s·y` com o `y` **absoluto**, e a conta antiga (`r·(1+|s|r)`)
        // só está certa para uma bola centrada na origem. Medido: uma caixa `half = 0,2` em `x = 3`
        // com `Taper 1,0` dava bordo até `x = 3,4664`, e a peça chega a **`3,8400`** — *«um bordo
        // menor CORTA a peça e não diz nada»*, que é o modo de falha que o topo deste ficheiro diz
        // que nunca pode acontecer.
        //
        // ⚠️ E desde a torção isto ficou pior do que um corte na exportação: o `axis_reach` bebe
        // daqui, logo um `R` pequeno de menos dá um divisor pequeno de menos e o campo **fura**.
        //
        // A lei: o maior factor na bola é `k_max = 1 + |s|·(|c_y| + r)`; o raio cresce por ele e o
        // centro **afasta-se** por `(k_max − 1)` vezes a distância dele ao eixo `Y`. Conservadora nos
        // dois termos, que é a assimetria declarada deste ficheiro.
        Unary::Taper { slope } => {
            if slope == 0.0 || !slope.is_finite() {
                return b;
            }
            let s = slope.abs();
            let k_max = s.mul_add(b.center[1].abs() + b.radius, 1.0);
            let fora_do_eixo = b.center[0].hypot(b.center[2]);
            Ball {
                radius: (k_max - 1.0).mul_add(fora_do_eixo, b.radius * k_max),
                ..b
            }
        }
        // ⭐⭐ **A DOBRA move a peça, e é o único modificador que o faz de forma não-linear.**
        //
        // ⚠️ **A conta INGÉNUA — «cabe numa bola centrada no centro do arco, de raio `ρ + R`» —
        // EXPLODE quando `κ → 0`**: com `κ = 0,001` ela dá `1000`. E o doc do topo deste ficheiro
        // diz o que isso custa: *«um bordo maior do que a peça custa RESOLUÇÃO»* — a grade da malha
        // passaria a gastar tudo em vazio.
        //
        // ⇒ fica-se com a **MENOR** de duas cercas, e cada uma é boa num regime:
        //  · a do centro do arco, apertada com `κ` grande;
        //  · a da própria peça mais a **corda** que o arco descreve, apertada com `κ` pequeno — e
        //    ela tende para `R` quando `κ → 0`, que é a identidade.
        //
        // ⚠️ **Conservadora de propósito nos dois casos** (a assimetria do ficheiro: um bordo a mais
        // custa resolução, um bordo a menos **corta a peça e não diz nada**).
        Unary::Bend { turns, .. } => {
            let k = f64::from(turns) * std::f64::consts::TAU;
            if k == 0.0 || !k.is_finite() {
                return b;
            }
            let rho = (1.0 / k).abs();
            let alcance = f64::from(b.center[0].abs() + b.radius);
            let meia_altura = f64::from(b.center[2].abs() + b.radius);
            // A corda máxima que um ponto descreve ao varrer o ângulo da peça inteira.
            let corda = 2.0 * (rho + alcance) * (k.abs() * meia_altura * 0.5).sin().abs();
            let pela_corda = f64::from(b.radius) + corda;
            let pelo_centro = rho + alcance;
            #[allow(clippy::cast_possible_truncation)]
            let raio = pela_corda.min(pelo_centro) as f32;
            Ball {
                // ⚠️ O centro fica onde estava: a bola é grande o suficiente para conter o arco, e
                // mover o centro exigiria a imagem dele — que é a conta que se está a evitar.
                center: b.center,
                radius: raio.max(b.radius),
            }
        }
    }
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
