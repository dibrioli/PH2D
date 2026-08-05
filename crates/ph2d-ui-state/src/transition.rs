//! **A TRANSIÇÃO** — o casamento pago uma vez, e o passo pago por frame.

use crate::pose::{ObjectPose, UiState};
use ph2d_vec_blend::Plan;

/// Meia volta, em radianos — a fronteira do arco mais curto.
const HALF_TURN: f64 = std::f64::consts::PI;
/// Uma volta.
const FULL_TURN: f64 = std::f64::consts::TAU;

/// O que acontece com UM objeto ao longo desta transição.
///
/// ⚠️ `Moving` é muito maior que os outros dois (ele guarda as DUAS poses), e a caixa fica nele em
/// vez de na variante: um `Box<Step>` custaria uma indireção por objeto no laço que roda POR
/// FRAME, para poupar bytes num vetor que tem dezenas de elementos e é construído uma vez.
#[allow(clippy::large_enum_variant)]
enum Step {
    /// Existe nos dois estados e alguma coisa difere.
    Moving {
        from: ObjectPose,
        to: ObjectPose,
        /// O casamento de forma, **só quando as geometrias diferem**. `None` é o caso comum e o
        /// barato — ver o doc da crate.
        shape: Option<Box<Plan>>,
    },
    /// Só no estado de origem: **sai**.
    Leaving(ObjectPose),
    /// Só no estado de destino: **entra**.
    Entering(ObjectPose),
}

/// O casamento entre dois estados, computado UMA vez.
///
/// ⚠️ A forma desta API não é gosto: `ph2d_vec_blend::Plan::new` custa **13 079×** um passo, então
/// um `smart_animate(from, to, t)` que casasse a cada frame seria inutilizável. É o mesmo
/// `new`/`at` que o `Plan` já usa, e pelo mesmo motivo.
pub struct Transition {
    steps: Vec<Step>,
    plans_built: usize,
}

impl Transition {
    /// Casa os dois estados **por id** e prepara o que for preciso para andar.
    ///
    /// ⚠️ Objetos **idênticos nos dois estados não entram** na transição. Não é otimização: é a
    /// afirmação de que *não animar* e *animar de x para x* são coisas diferentes — a segunda
    /// custaria um `Plan` e produziria trabalho por frame para não mover nada.
    #[must_use]
    pub fn new(from: &UiState, to: &UiState) -> Self {
        let mut steps = Vec::new();
        let mut plans_built = 0;

        for a in &from.objects {
            match to.pose(a.id) {
                Some(b) if a.is_same_as(b) => {}
                Some(b) => {
                    // O `Plan` é construído **só** quando as geometrias de facto diferem. Formas
                    // iguais (ou ausentes) atravessam sem pagar a busca de fase.
                    let shape = match (&a.geometry, &b.geometry) {
                        (Some(ga), Some(gb)) if ga.verts != gb.verts || ga.closed != gb.closed => {
                            let p = Plan::new(ga, gb).map(Box::new);
                            if p.is_some() {
                                plans_built += 1;
                            }
                            p
                        }
                        _ => None,
                    };
                    steps.push(Step::Moving {
                        from: a.clone(),
                        to: b.clone(),
                        shape,
                    });
                }
                None => steps.push(Step::Leaving(a.clone())),
            }
        }
        for b in &to.objects {
            if from.pose(b.id).is_none() {
                steps.push(Step::Entering(b.clone()));
            }
        }

        Self { steps, plans_built }
    }

    /// **Quantos casamentos de forma este par custou.** Existe para o gate de custo: um par
    /// só-de-cor tem de responder `0`, e é esse zero que vale 12,79 ms numa cena de vinte objetos.
    #[must_use]
    pub fn plans_built(&self) -> usize {
        self.plans_built
    }

    /// Quantos objetos esta transição de facto move.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// A pose de cada objeto em movimento, no ponto `t` do caminho.
    ///
    /// `t` é clampado a `[0, 1]`: uma curva com *overshoot* (um ease `Back` ou `Elastic`) encosta
    /// na ponta e para, em vez de quebrar a forma — a mesma escolha que o `VecMorph` já faz.
    ///
    /// ⚠️ **A ROTAÇÃO vai pelo ARCO MAIS CURTO.** De 350° para 10° ela anda +20°, não −340°, que é
    /// o que qualquer ferramenta de UI faz e o que o artista espera de um par de estados. **A
    /// consequência é nomeada em vez de descoberta:** uma volta inteira autorada (0 → 360°) é o
    /// mesmo ângulo, logo ela **não gira** — girar N voltas é animação com percurso, e percurso é
    /// keyframe, não estado.
    #[must_use]
    pub fn at(&self, t: f64) -> Vec<ObjectPose> {
        let t = t.clamp(0.0, 1.0);
        self.steps
            .iter()
            .map(|s| match s {
                Step::Moving { from, to, shape } => {
                    let mut p = ObjectPose {
                        id: from.id,
                        translation: [
                            lerp(from.translation[0], to.translation[0], t),
                            lerp(from.translation[1], to.translation[1], t),
                        ],
                        rotation: lerp_angle(from.rotation, to.rotation, t),
                        scale: [
                            lerp(from.scale[0], to.scale[0], t),
                            lerp(from.scale[1], to.scale[1], t),
                        ],
                        opacity: lerp_f32(from.opacity, to.opacity, t),
                        // A TINTA vai SEMPRE pela porta do Blend, com forma ou sem ela — uma
                        // resposta só para *"como duas tintas interpolam neste app"*.
                        fill: ph2d_vec_blend::mix_paint(from.fill.as_ref(), to.fill.as_ref(), t),
                        stroke: ph2d_vec_blend::mix_stroke(from.stroke, to.stroke, t),
                        geometry: None,
                    };
                    // ⚠️ E a forma que sai do `Plan` recebe a tinta da POSE, não a que o `Plan`
                    // interpolou por conta: se o objeto sai auto-consistente daqui, ninguém a
                    // jusante tem de decidir qual das duas vale.
                    p.geometry = shape.as_ref().map(|plan| {
                        let mut g = plan.at(t);
                        g.fill.clone_from(&p.fill);
                        g.stroke = p.stroke;
                        g
                    });
                    p
                }
                // Quem SAI fica onde estava e desvanece; quem ENTRA já está no lugar de destino e
                // aparece. ⚠️ Nenhum dos dois se move: mover algo que só existe de um lado seria
                // inventar a outra ponta do caminho.
                Step::Leaving(p) => ObjectPose {
                    opacity: lerp_f32(p.opacity, 0.0, t),
                    ..p.clone()
                },
                Step::Entering(p) => ObjectPose {
                    opacity: lerp_f32(0.0, p.opacity, t),
                    ..p.clone()
                },
            })
            .collect()
    }
}

#[inline]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
fn lerp_f32(a: f32, b: f32, t: f64) -> f32 {
    a + (b - a) * t as f32
}

/// O arco mais curto entre dois ângulos.
///
/// ⚠️ O `rem_euclid` traz a diferença para `[0, 2π)` **antes** de escolher o lado, e é isso que
/// torna a função correta para ângulos fora de uma volta (um objeto girado 3 voltas pelo artista).
#[inline]
fn lerp_angle(a: f64, b: f64, t: f64) -> f64 {
    let mut d = (b - a).rem_euclid(FULL_TURN);
    if d > HALF_TURN {
        d -= FULL_TURN;
    }
    a + d * t
}
