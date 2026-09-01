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

/// Uma esfera de bordo: centro e raio, no referencial de quem pergunta — **mais** as meias-extensões
/// por eixo, que são a metade que a esfera não sabe dizer.
///
/// # ⭐⭐⭐ Por que as DUAS, e não uma caixa em vez da esfera (2026-08-31)
///
/// A nota do topo deste ficheiro continua verdadeira: a esfera é **invariante à rotação** e a caixa
/// não é, então a caixa é a moeda errada para **compor** bordos. ⇒ elas viajam **juntas**, e cada
/// consumidor lê a que responde à pergunta dele. A caixa nunca é maior do que a esfera
/// (`half[i] ≤ radius`, imposto na construção), então trocá-la pela esfera é sempre **seguro** —
/// o pior que acontece é ela ser folgada.
///
/// ⛔ **O `half` é PRIVADO de propósito.** Um valor pequeno demais **corta a peça e não diz nada**,
/// e um `Ball { radius: ..., ..b }` herdaria em silêncio o `half` de antes de a lei crescer a bola.
/// Sem literal de estrutura, cada sítio tem de escolher entre [`Ball::new`] (que assume o pior, e é
/// sempre seguro) e [`Ball::of`] (que sabe os eixos). *A cerca fica no lado perigoso.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ball {
    pub center: [f32; 3],
    pub radius: f32,
    /// ⚠️ Ver o doc da estrutura. **Nunca** maior do que o [`Ball::radius`].
    half: [f32; 3],
}

impl Ball {
    /// A bola de um nó que não ocupa lugar nenhum — o degenerado seguro de um nome desconhecido.
    pub const EMPTY: Ball = Ball {
        center: [0.0; 3],
        radius: 0.0,
        half: [0.0; 3],
    };

    /// ⭐ **Uma bola que não sabe os eixos** — a caixa dela é o cubo circunscrito, que é o que a
    /// esfera de facto garante. *Assumir o pior é o que torna este construtor sempre seguro.*
    #[must_use]
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Ball {
            center,
            radius,
            half: [radius; 3],
        }
    }

    /// ⭐⭐⭐ **O RAIO E A CAIXA, cada um pela lei dele** — o construtor que este ficheiro usa.
    ///
    /// ⛔⛔ **O raio NÃO se deriva da caixa, e a 1.ª versão deste módulo fazia-o.** Numa esfera a
    /// caixa é o cubo `[r, r, r]` e a diagonal dele é `r√3` — a bola crescia **73 %** e quatro gates
    /// reprovaram de uma vez (*«o bordo cresceu com a composição: 1,075 para uma esfera de 0,25»*).
    /// ⇒ cada lei mantém o **seu** raio, exactamente como antes, e a caixa é **acrescentada** ao
    /// lado. *Uma mudança que se diz aditiva tem de deixar o número antigo onde ele estava.*
    ///
    /// ⚠️ O `half` é preso ao raio: uma caixa maior do que a esfera que a contém é uma contradição.
    ///
    /// # ⭐⭐⭐ E o raio é preso à CAIXA — a outra metade, que faltava (2026-09-01)
    ///
    /// A regra acima só andava num sentido. Mas a caixa também **fala sobre a esfera**: se a peça
    /// cabe em `centro ± half`, então cabe na esfera que passa pelo canto dessa caixa, de raio
    /// `‖half‖`. ⇒ toda lei que sabe os eixos aperta o raio **de graça**, e a que não sabe
    /// ([`Ball::new`], com `half = [r, r, r]`) fica **inalterada ao bit** — ali `‖half‖ = r√3 > r`.
    ///
    /// ⚠️ Isto não é um atalho para a nota do topo do ficheiro: a esfera **continua** a ser a moeda
    /// da composição (ela é invariante à rotação e a caixa não). O que muda é que ela deixa de ser
    /// a única coisa que uma lei sabe dizer.
    #[must_use]
    pub fn of(center: [f32; 3], radius: f32, half: [f32; 3]) -> Self {
        let h = [
            half[0].min(radius),
            half[1].min(radius),
            half[2].min(radius),
        ];
        let canto = h[0].hypot(h[1]).hypot(h[2]);
        let radius = radius.min(canto);
        Ball {
            center,
            radius,
            half: [h[0].min(radius), h[1].min(radius), h[2].min(radius)],
        }
    }

    /// ⭐ **A MESMA bola, engordada por igual em todas as direcções** — uma parede, um afastamento.
    ///
    /// ⚠️ É o substituto do `Ball { radius: r + d, ..b }`, e a diferença é o que ele **impede**:
    /// aquele herdava o `half` de antes, que depois da engorda fica pequeno demais e **corta**.
    #[must_use]
    pub fn expanded_by(self, delta: f32) -> Self {
        Ball {
            center: self.center,
            radius: self.radius + delta,
            half: [
                self.half[0] + delta,
                self.half[1] + delta,
                self.half[2] + delta,
            ],
        }
    }

    /// As meias-extensões por eixo. ⚠️ Nunca maiores do que o [`Ball::radius`].
    #[must_use]
    pub fn half(self) -> [f32; 3] {
        self.half
    }

    /// ⭐⭐⭐ **A MESMA BOLA, vista do referencial CANÓNICO de um modificador** (Enio, 2026-08-31).
    ///
    /// ⚠️ Uma bola é uma **esfera**: levá-la a outro eixo é exactamente permutar as coordenadas do
    /// centro. *Não há lei nova a escrever* — é a mesma permutação que a
    /// [`ph2d_field_eval::stack`] aplica à árvore, e escrevê-la duas vezes seria pedir que as duas
    /// discordassem.
    #[must_use]
    pub fn to_canonical(self, s: usize) -> Self {
        Ball {
            center: ph2d_field::Axis::to_canonical(self.center, s),
            // ⚠️ **A caixa permuta com o centro** — ela tem eixos, e esquecê-la aqui daria uma
            // caixa do eixo errado a toda lei conjugada.
            half: ph2d_field::Axis::to_canonical(self.half, s),
            ..self
        }
    }

    /// O caminho de volta da [`Self::to_canonical`].
    #[must_use]
    pub fn from_canonical(self, s: usize) -> Self {
        Ball {
            center: ph2d_field::Axis::from_canonical(self.center, s),
            half: ph2d_field::Axis::from_canonical(self.half, s),
            ..self
        }
    }

    /// ⭐⭐⭐ **A CAIXA alinhada aos eixos que contém a peça** — o recorte da marcha e a grade do
    /// exportador.
    ///
    /// ⛔⛔⛔ **Ela era o CUBO circunscrito à esfera, e isso custava duas coisas de uma vez**
    /// (report do Enio, 2026-09-01: *«muitíssimo lento»*). O doc do
    /// `field3d_export_tests` já o nomeava — *«cúbico: um objeto fino reporta o lado maior nos
    /// três eixos»* — e o preço não era só resolução:
    ///
    /// | quem lê | o que o cubo custava |
    /// |---|---|
    /// | a marcha (`Scene::clip`) | o raio entra mais cedo e sai mais tarde, e **todo** `*_reach` dos deformadores é medido nele |
    /// | a grade do exportador | a resolução gasta-se em vazio |
    ///
    /// ⇒ ela devolve as **meias-extensões**, que é o que a peça de facto ocupa. ⚠️ **É seguro por
    /// construção**: `half[i] ≤ radius` (invariante da estrutura) e a caixa contém a peça (gate
    /// `the_box_of_a_bound_contains_the_piece_through_the_whole_stack`, 60 fixturas). Quem não
    /// sabe os eixos nasce por [`Ball::new`], cujo `half` **é** o raio — e aí ela é o cubo de
    /// sempre, ao bit.
    #[must_use]
    pub fn aabb(self) -> ([f32; 3], [f32; 3]) {
        let h = [
            self.half[0].max(0.0),
            self.half[1].max(0.0),
            self.half[2].max(0.0),
        ];
        (
            [
                self.center[0] - h[0],
                self.center[1] - h[1],
                self.center[2] - h[2],
            ],
            [
                self.center[0] + h[0],
                self.center[1] + h[1],
                self.center[2] + h[2],
            ],
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
        let center = if dist <= f32::MIN_POSITIVE {
            self.center
        } else {
            let t = (radius - self.radius) / dist;
            [
                self.center[0] + d[0] * t,
                self.center[1] + d[1] * t,
                self.center[2] + d[2] * t,
            ]
        };
        // ⭐ **A caixa fundida contém as DUAS**, medida do centro novo, e presa ao raio — que é a
        // invariante da estrutura. *Herdar a caixa de uma delas cortaria a outra.*
        let alcance = |b: &Self, e: usize| (b.center[e] - center[e]).abs() + b.half[e];
        Self {
            center,
            radius,
            half: std::array::from_fn(|e| alcance(&self, e).max(alcance(&other, e)).min(radius)),
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
            // ⭐⭐⭐ **A folha SABE os eixos** — é daqui que a caixa entra no sistema.
            NodeKind::Leaf(p) => Some(Ball::of(
                [0.0; 3],
                ph2d_field::bounding_radius(p),
                ph2d_field::bounding_half_extents(p),
            )),
            // ⚠️ Uma escultura só sabe dizer um raio — e o cubo circunscrito é o que ela garante.
            NodeKind::Sampled { key } => reg
                .get(key)
                .map(|f| Ball::new([0.0; 3], f.bounding_radius())),
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
        // ⭐⭐⭐ **A folha SABE os eixos** — é daqui que a caixa entra no sistema.
        NodeKind::Leaf(p) => Some(Ball::of(
            [0.0; 3],
            ph2d_field::bounding_radius(p),
            ph2d_field::bounding_half_extents(p),
        )),
        // ⚠️ Um nome que o registo não conhece lê como **espaço vazio** (`hybrid::ABSENT`), e um
        // vazio não ocupa lugar nenhum: o bordo dele é nada, não uma caixa inventada.
        // ⚠️ Uma escultura só sabe dizer um raio — e o cubo circunscrito é o que ela garante.
        NodeKind::Sampled { key } => reg
            .get(key)
            .map(|f| Ball::new([0.0; 3], f.bounding_radius())),
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
    // ⭐⭐⭐ **A LEI ESTÁ ESCRITA NO EIXO CANÓNICO; O EIXO ESCOLHIDO ENTRA POR FORA** (Enio,
    // 2026-08-31) — a mesma conjugação que a `stack::conjugado` aplica à árvore, e **a mesma
    // permutação**. Escrever a lei uma vez por eixo daria três sítios onde um índice errado
    // devolve uma caixa que **corta a peça** em silêncio, que é o modo de falha que o topo deste
    // ficheiro diz que nunca pode acontecer.
    let s = axis_shift_of(m);
    if s == 0 {
        return canonical_step(b, m);
    }
    canonical_step(b.to_canonical(s), m).from_canonical(s)
}

/// De quantos passos a lei de `m` tem de rodar para o eixo escolhido cair no canónico dela.
///
/// ⚠️ **`0` para quem não tem eixo** — a casca e o afastamento são isotrópicos, e o espelho tem os
/// três eixos como três variantes (ver [`ph2d_field::FIELD_DOC_VERSION`], v16).
pub(crate) fn axis_shift_of(m: Unary) -> usize {
    use ph2d_field::mods::{ARRAY_AXIS, BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    match m {
        Unary::Array { axis, .. } => axis.shift_to(ARRAY_AXIS),
        Unary::Taper { axis, .. } => axis.shift_to(TAPER_AXIS),
        Unary::Radial { axis, .. } => axis.shift_to(RADIAL_AXIS),
        Unary::Twist { axis, .. } => axis.shift_to(TWIST_AXIS),
        Unary::Bend { axis, .. } => axis.shift_to(BEND_AXIS),
        Unary::Shell { .. }
        | Unary::Offset { .. }
        | Unary::Mirror
        | Unary::MirrorY
        | Unary::MirrorZ => 0,
    }
}

/// ⭐⭐⭐ **Quão longe de um EIXO a bola chega** — a grandeza que a torção varre, que a repetição
/// radial varre e de que a inclinação tira o factor de secção. Uma lei, três leitores.
///
/// ⛔ **Cada um deles escrevia `‖(c_a, c_b)‖ + raio`, e o raio não é uma extensão de eixo**
/// (2026-09-01). Numa peça achatada no plano que interessa a esfera responde pela **espessura no
/// terceiro eixo**, que não participa: uma barra `0,05 × 0,05 × 0,80` centrada dizia `0,80` de braço
/// e o braço verdadeiro é `0,07` — **11×**, e o erro ia direito ao divisor da torção
/// (`σ = t/2 + √(1 + t²/4)` com `t = κ·R`), que é multiplicativo.
///
/// ⚠️ **As DUAS respostas são majorantes, e fica-se com a menor**: a da caixa ganha numa peça
/// achatada, a da esfera ganha numa peça longe do eixo (ali a caixa paga `√2` pelo canto).
#[must_use]
pub(crate) fn axis_distance(b: Ball, eixo: usize) -> f32 {
    let h = b.half();
    let (u, v) = ((eixo + 1) % 3, (eixo + 2) % 3);
    let pela_caixa = (b.center[u].abs() + h[u]).hypot(b.center[v].abs() + h[v]);
    let pela_esfera = b.center[u].hypot(b.center[v]) + b.radius.max(0.0);
    pela_caixa.min(pela_esfera)
}

/// A lei de cada modificador, **no eixo canónico dele** — ver [`step_mod`], que é a porta.
fn canonical_step(b: Ball, m: Unary) -> Ball {
    match m {
        // A parede é centrada na superfície: metade cresce para fora.
        Unary::Shell { thickness } => b.expanded_by(thickness.abs() * 0.5),
        Unary::Offset { distance } => b.expanded_by(distance.max(0.0)),
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
            // ⚠️ A cópia tem a **mesma** caixa — só o centro é que reflecte —, e o `merge` sabe
            // fundir as duas.
            b.merge(Ball::of(c, b.radius, b.half()))
        }
        // A matriz linear anda ao longo do X local.
        Unary::Array {
            count,
            spacing,
            joint,
            ..
        } => {
            let span = f32::from(u16::try_from(count.saturating_sub(1)).unwrap_or(u16::MAX))
                * spacing.abs();
            // ⭐ **A junta ACRESCENTA material no vinco** — um bordo que não a conte recorta a
            // peça na marcha e na exportação, que é o defeito que a inclinação já custou a esta
            // linha em 2026-08-30. Ver [`ph2d_field::Joint::reach`].
            //
            // ⭐⭐ **E a caixa cresce SÓ no eixo em que a matriz anda** — é a primeira lei deste
            // ficheiro que a esfera não sabia exprimir.
            let h = b.half();
            let j = joint.reach();
            Ball::of(
                [b.center[0] + span * 0.5, b.center[1], b.center[2]],
                b.radius + span * 0.5 + j,
                [h[0] + span * 0.5 + j, h[1] + j, h[2] + j],
            )
        }
        // ⭐ **A torção varre em torno do Z local, e a bola dela é a MESMA da matriz radial** — cada
        // fatia de `z` é uma rotação em torno da origem, logo `‖(x,y)‖` e `z` são preservados. Uma
        // bola já centrada no eixo fica **inalterada ao bit**; uma descentrada varre o círculo que o
        // centro descreve. *Não se escreve lei nova: aponta-se para a que existe.*
        Unary::Radial { joint, .. } => {
            // ⭐ Pela razão da matriz acima — a costura entre as cópias enche o vinco.
            //
            // ⭐⭐ **O varrimento é no plano XY; o Z não se mexe** — e é isso que a caixa diz e a
            // esfera não dizia.
            let h = b.half();
            let j = joint.reach();
            let raio_xy = axis_distance(b, 2) + j;
            let meia_z = h[2] + j;
            Ball::of(
                [0.0, 0.0, b.center[2]],
                raio_xy.hypot(meia_z),
                [raio_xy, raio_xy, meia_z],
            )
        }
        Unary::Twist { .. } => {
            // ⭐⭐ Como a radial: o giro é no plano XY e o Z é preservado **ao bit**.
            let h = b.half();
            let raio_xy = axis_distance(b, 2);
            Ball::of(
                [0.0, 0.0, b.center[2]],
                raio_xy.hypot(h[2]),
                [raio_xy, raio_xy, h[2]],
            )
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
        Unary::Taper { slope, .. } => {
            if slope == 0.0 || !slope.is_finite() {
                return b;
            }
            let s = slope.abs();
            // ⭐ **A altura é a do EIXO Y, e não o raio** (2026-09-01) — `k(y) = 1 + slope·y` só
            // pergunta pelo `y`, e a esfera responde pelo maior dos três eixos. Numa chapa larga e
            // baixa isso inflava o factor de secção inteiro.
            let k_max = s.mul_add(b.center[1].abs() + b.half()[1], 1.0);
            let fora_do_eixo = b.center[0].hypot(b.center[2]);
            // ⭐⭐ **A secção escala em X e Z; o Y não se mexe** — a inclinação é o exemplo mais
            // claro de uma lei que a esfera obrigava a arredondar para cima nos três eixos.
            let h = b.half();
            let cresce = |e: usize| (k_max - 1.0).mul_add(fora_do_eixo, h[e] * k_max);
            Ball::of(
                b.center,
                (k_max - 1.0).mul_add(fora_do_eixo, b.radius * k_max),
                [cresce(0), h[1], cresce(2)],
            )
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
        Unary::Bend {
            turns,
            lower,
            upper,
            falloff,
            ..
        } => {
            // ⛔⛔ **A CURVATURA EFECTIVA, e não a pedida** (2026-09-01). A árvore dobra por
            // `bend_curvature`, que **satura** contra a parede da peça — e uma caixa calculada
            // sobre a curvatura crua descreve um arco que ninguém desenhou. *Um bordo tem de
            // responder pela geometria que a árvore produz, não pela que o slider pede.*
            let k = crate::stack_bend::bend_curvature(turns, b);
            if k == 0.0 || !k.is_finite() {
                return b;
            }
            let rho = (1.0 / k).abs();
            let h = b.half();
            let alcance = f64::from(b.center[0].abs() + h[0]);
            // ⭐⭐⭐ **A meia-altura é a do EIXO DOBRADO, e não o raio** (2026-08-31). Ela entra num
            // `sin` multiplicado por `2(ρ + alcance)`, então o exagero da esfera compõe-se: numa
            // caixa `0,35 × 0,35 × 0,30` o raio é `0,579` contra os `0,30` do eixo — **1,93×** — e a
            // bola saía **3,6× maior do que a peça**. Isso ia direito ao `piso` da dobra e o divisor
            // dela ficava preso no tecto (`10`).
            //
            // ⭐ Medido: com a extensão axial, `[Bend]` passa de `72,2` para `27,0` passos por raio,
            // `[Bend, Twist]` de `233,1` para `68,4` e `[Bend, Twist, Taper]` de `1 543,6` para
            // `717,3` — e as **cinco** imagens contra a marcha honesta ficam idênticas.
            let meia_altura = f64::from(b.center[2].abs() + h[2]);
            // A corda máxima que um ponto descreve ao varrer o ângulo da peça inteira.
            let corda = 2.0 * (rho + alcance) * (k.abs() * meia_altura * 0.5).sin().abs();
            let pela_corda = f64::from(b.radius) + corda;
            let pelo_centro = rho + alcance;
            #[allow(clippy::cast_possible_truncation)]
            let raio = (pela_corda.min(pelo_centro) as f32).max(b.radius);
            // ⚠️ O centro fica onde estava: a bola é grande o suficiente para conter o arco, e
            // mover o centro exigiria a imagem dele — que é a conta que se está a evitar.
            //
            // ⭐⭐⭐ **E a CAIXA é a do arco, não o cubo do raio** (2026-09-01) — ver
            // [`crate::bounds_bend`]. Ela era `[raio, h₁, raio]`, e numa caixa `0,35³` a `0,12`
            // voltas isso é `0,957` contra os `0,376` que a peça dobrada de facto ocupa: **`2,5×`
            // em dois eixos**, herdados por toda a pilha a jusante.
            //
            // ⚠️ **A lei antiga fica como tecto**: a nova nunca é aceite se for mais folgada, e
            // quando o arco é pequeno demais para ser distinguível da identidade ela devolve a
            // pergunta. *Uma cerca nova entra por `min` com a que já defendia o sítio.*
            let largo = |e: usize| if e == 1 { h[1] } else { raio };
            let meia = crate::bounds_bend::bent_extent(
                b.center,
                h,
                k,
                f64::from(lower),
                f64::from(upper),
                f64::from(falloff),
            )
            .map_or([largo(0), largo(1), largo(2)], |(lo, hi)| {
                std::array::from_fn(|e| {
                    let c = f64::from(b.center[e]);
                    #[allow(clippy::cast_possible_truncation)]
                    let m = (hi[e] - c).abs().max((lo[e] - c).abs()) as f32;
                    m.min(largo(e))
                })
            });
            Ball::of(b.center, raio, meia)
        }
    }
}

/// A esfera vista do referencial do **pai** — e é aqui que a invariância à rotação se paga.
fn place(ball: Ball, xform: Xform) -> Ball {
    // ⛔⛔ **A CAIXA NÃO É INVARIANTE À ROTAÇÃO, e é essa a nota do topo deste ficheiro.** Uma caixa
    // rodada tem de ser re-envolvida, e cada re-envolvimento **cresce** — daí a esfera ser a moeda
    // da composição. ⇒ aqui a caixa é re-envolvida pela lei exacta (`|R|·h`, a matriz dos módulos),
    // e o `Ball::of` prende-a ao raio, que **não** cresce. *O pior que a rotação faz é devolver a
    // caixa ao cubo circunscrito, que é o que a esfera já dizia.*
    let r = ball.radius * xform.scale.abs();
    let h = ball.half();
    // ⚠️ **A base sai do próprio [`Xform::apply_dir`]**, que já leva a escala — e não de uma segunda
    // conta da rotação. *Duas leis para a mesma matriz é como as duas metades divergem.*
    let coluna = |j: usize| {
        let mut e = [0.0f32; 3];
        e[j] = 1.0;
        xform.apply_dir(e)
    };
    let (cx, cy, cz) = (coluna(0), coluna(1), coluna(2));
    let eixo = |e: usize| (cx[e].abs() * h[0] + cy[e].abs() * h[1] + cz[e].abs() * h[2]).min(r);
    Ball::of(xform.apply(ball.center), r, [eixo(0), eixo(1), eixo(2)])
}

#[cfg(test)]
#[path = "bounds_tests.rs"]
mod tests;
