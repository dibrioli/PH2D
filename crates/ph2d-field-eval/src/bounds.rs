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
    ///
    /// # ⛔⛔⛔ O atalho «uma contém a outra» DEITAVA FORA a caixa da outra (Enio, 2026-09-01, foto)
    ///
    /// Ele comparava **só as esferas** e devolvia a bola vencedora inteira — com o `half` dela. Três
    /// cilindros cruzados são o caso exacto em que isso morde: as três estão no **mesmo centro** com
    /// o **mesmo raio**, o teste `dist + r_outra ≤ r_esta` dá verdadeiro logo à primeira, e a união
    /// fica com a caixa de **um** cilindro (`0,18 × 0,18 × 0,60`). ⇒ o recorte da marcha corta os
    /// outros dois braços, e o report foi *«os 3 cilindros cruzados viraram isso»*: um bolo gordo
    /// com cunhas escuras, `754` de `2 576` pixels do interior com a normal a `172,7°` do oráculo.
    ///
    /// ⚠️ **O defeito nasceu quando o [`Ball::aabb`] passou a devolver a caixa**: enquanto o recorte
    /// era o **cubo do raio**, uma caixa herdada errada não cortava nada, porque o raio das três é o
    /// mesmo. *Uma resposta pode estar errada há semanas e só doer no dia em que alguém a lê.*
    ///
    /// ⇒ o atalho passa a escolher **apenas o centro e o raio**; a caixa é **sempre** computada a
    /// partir das duas, que é o que a linha de baixo já fazia no caso geral.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        let d = [
            other.center[0] - self.center[0],
            other.center[1] - self.center[1],
            other.center[2] - self.center[2],
        ];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        // ⭐ A esfera: uma contém a outra, ou a mínima que envolve as duas. ⚠️ **Só o centro e o
        // raio saem daqui** — ver o doc, e o que custou devolver a bola inteira.
        let (center, radius) = if dist + other.radius <= self.radius {
            (self.center, self.radius)
        } else if dist + self.radius <= other.radius {
            (other.center, other.radius)
        } else {
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
            (center, radius)
        };
        // ⭐ **A caixa fundida contém as DUAS**, medida do centro escolhido, e presa ao raio — que é
        // a invariante da estrutura. *Herdar a caixa de uma delas corta a outra.*
        let alcance = |b: &Self, e: usize| (b.center[e] - center[e]).abs() + b.half[e];
        Self {
            center,
            radius,
            half: std::array::from_fn(|e| alcance(&self, e).max(alcance(&other, e)).min(radius)),
        }
    }
}

pub(crate) use crate::bounds_mods::axis_shift_of;
pub use crate::bounds_mods::step_mod;

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
/// ⛔⛔⛔ **O RAIO DA MISTURA ENTRA — e esta linha afirmava o CONTRÁRIO** (2026-09-01). Ela dizia
/// *«um arredondamento enche o vinco côncavo, que fica dentro da união dos dois bordos; ele não
/// cresce a peça»*, e isso é falso: uma união suave **empurra a superfície para fora** no vinco. Na
/// união exacta o campo vale `r(1 − √2)` onde as duas superfícies se tocam — bem dentro da peça —,
/// logo a matéria passa do bordo das duas. Medido num par de cilindros com junta `0,10`: o eixo `Y`
/// chega a `0,2092` e a caixa dizia `0,1804`.
///
/// ⚠️ **A lei já estava escrita duas vezes neste ficheiro**, para a matriz e para a repetição radial
/// (`Joint::reach`, *«a costura entre as cópias enche o vinco»*) — e não foi aplicada ao `Combine`,
/// que é onde o artista de facto põe uma junta. *Uma lei escrita para dois leitores tem de ser
/// procurada nos outros.*
///
/// ⚠️ **O `radius` inteiro é conservador e DEMONSTRÁVEL**: as três misturas só mexem no campo dentro
/// de `radius` das duas superfícies, então a superfície não se pode mover mais do que isso
/// (`Exact` move `r(√2 − 1) ≈ 0,41 r`, `Chamfer` `r/√2 ≈ 0,71 r`, `Organic` `k/4 ≈ 0,29 r`).
/// Apertá-lo para o pior dos três exige medir os três — *e um bordo que erra para baixo corta a
/// peça e não diz nada*.
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
        let efectivo = ph2d_field::fold_verb(op, verb);
        let juntou = match efectivo {
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
        };
        // ⭐⭐⭐ **E a MISTURA empurra a superfície para fora** — ver o doc desta função, e o report
        // do Enio que a corrigiu. ⚠️ Vale para as três operações: o vinco de uma intersecção suave e
        // o de uma diferença suave também recebem matéria que a aresta viva não tinha.
        let raio = match efectivo {
            Op::Union(b) | Op::Intersection(b) | Op::Difference(b) => b.amount(),
        };
        acc = Some(if raio > 0.0 {
            juntou.expanded_by(raio)
        } else {
            // ⚠️ **Aresta viva devolve a bola INTACTA, ao bit** — uma engorda por zero passaria por
            // um `min` com o raio e podia mexer no `half`.
            juntou
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
