//! **O gesto do modo OSSO** (estudo 42 item 5, doc 47 §2.6) — arrastar faz um osso.
//!
//! ```text
//! press  → caiu na PONTA de um osso?  sim → o novo é FILHO dele, e parte dessa ponta
//!                                     não → o novo é uma RAIZ, e parte de onde a mão pousou
//! release→ nasce o osso: origem no press, comprimento e ângulo no arrasto
//! ```
//!
//! O parentesco é um SÍTIO na tela (a ponta do pai), não a selecção (dono 2026-09-09). A pose é
//! LOCAL do pai, derivada levando os dois pontos ao espaço dele. O que o dedo aponta mora em
//! [`crate::bone_pick`]; o [`press`] pergunta ao `hover` de lá.

use crate::bone_pick::{BONE_HIT_PX, hover, tip_of};
use ph2d_ecs::{ChildOf, Entity, RootOrder, SimWorld};
use ph2d_skeleton_ecs::Bone;

/// **O que o press do modo Osso DECIDE** — a porta única, observável de um teste.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BonePress {
    /// **Transformar:** acertou um osso — selecciona-o e ARMA o verbo que a `part` diz.
    Grab {
        bone: u64,
        part: ph2d_skeleton_render::BonePart,
    },
    /// **Criar:** arma o osso que vai nascer e diz que FORMA estava sob o cursor (para o *Bind*).
    Start {
        birth: BoneBirth,
        pick: Option<ph2d_vec_scene::VecPathId>,
    },
    /// **Transformar:** não acertou osso — aponta a forma sob o cursor e mais nada. ⛔ Nenhum osso
    /// nasce neste verbo, nem por arrasto longo.
    Pick {
        path: Option<ph2d_vec_scene::VecPathId>,
    },
    /// **Peso:** o traço pinta a influência do osso aceso sobre a arte presa `alvo` (`None` ⇒ o
    /// gesto não começa). Não toca na selecção.
    Weight { alvo: Option<u64> },
}

/// **O pincel de peso** — viaja como argumento para o press ser observável de um teste.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pincel {
    /// O `pixels_per_meter` do projecto — só a mídia IMAGEM o lê.
    pub ppm: f32,
    /// O raio, em unidades de MUNDO. É também a tolerância do dedo: fora dele, não há arte.
    pub raio: f64,
}

impl Pincel {
    /// O pincel de quem NÃO está a pintar peso — raio nulo ⇒ nenhuma arte é apanhada.
    pub const INERTE: Self = Self {
        ppm: 1.0,
        raio: 0.0,
    };
}

/// **O osso que está a nascer** — origem e pai, decididos JUNTOS no press e lidos no release.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneBirth {
    /// A origem, em MUNDO — a ponta do pai, ou o ponto onde a mão pousou.
    pub origin: [f64; 2],
    /// O pai, em bits — `None` faz uma RAIZ.
    pub parent: Option<u64>,
}

/// A decisão do press, sem tocar em nada; o `action` resolve a ambiguidade do ponteiro.
#[expect(
    clippy::too_many_arguments,
    reason = "os oito sao o que a DECISAO do press precisa, e a alternativa e' a decisao viver \
              dentro do dispatch - que e' exactamente o defeito que esta porta curou"
)]
pub fn press(
    sim: &SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    world: [f64; 2],
    px_to_world: f64,
    selected: Option<u64>,
    action: ph2d_tool_bone::BoneAction,
    pincel: Pincel,
) -> BonePress {
    use ph2d_tool_bone::BoneAction;
    // O peso não pergunta nada ao rig: o sujeito é a arte presa.
    if action == BoneAction::Weight {
        return BonePress::Weight {
            alvo: ph2d_skeleton_live::peso_a_mao::pele_sob_o_cursor(
                sim,
                pincel.ppm,
                world,
                pincel.raio,
            )
            .map(ph2d_ecs::Entity::to_bits),
        };
    }
    // O clique pergunta ao realce: o que está aceso é o que se pega.
    let foco = selected_bone(sim, selected);
    let alvo = hover(sim, world, px_to_world, foco, action);
    if action == BoneAction::Transform {
        return match alvo {
            Some(h) => BonePress::Grab {
                bone: h.bone,
                part: h.part,
            },
            // Em *Transformar* um press no vazio só aponta a forma.
            None => BonePress::Pick {
                path: pen.path_at(scene, world, BONE_HIT_PX * px_to_world),
            },
        };
    }
    // Em *Criar* todo press arma um osso; a ponta sob o dedo (o `hover` devolve `Tip`) é o pai.
    let parent = alvo.map(|h| h.bone);
    let origin = parent
        .and_then(|b| tip_of(sim, b))
        // Sem pai, a origem é o ponto cru (começar sobre o corpo de outro osso é legítimo).
        .unwrap_or(world);
    BonePress::Start {
        birth: BoneBirth { origin, parent },
        pick: pen.path_at(scene, world, BONE_HIT_PX * px_to_world),
    }
}

/// **A mira** — a rotação LOCAL que faz o osso apontar a `world` (porta do [`pose`] e do
/// [`crate::goal`]). `None` com pai singular ou `world` sobre a origem do osso.
#[must_use]
pub fn aim_rotation(sim: &SimWorld, bone: Entity, world: [f64; 2]) -> Option<f64> {
    ph2d_skeleton_live::bone::aim_rotation(sim, bone, world)
}

/// **A selecção do gizmo é um OSSO?** — o primeiro osso da selecção INTEIRA (não só o primário).
pub fn selected_bone(sim: &SimWorld, selection: impl IntoIterator<Item = u64>) -> Option<u64> {
    selection
        .into_iter()
        .find(|&b| sim.world().get::<Bone>(Entity::from_bits(b)).is_some())
}

/// **O segmento de MUNDO de um osso** — fixtura partilhada; `test-support` porque a shell a lê.
#[cfg(any(test, feature = "test-support"))]
pub fn test_segment(sim: &SimWorld, bits: u64) -> ([f64; 2], [f64; 2]) {
    ph2d_skeleton_live::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .map(|(_, a, b)| (a, b))
        .expect("o osso")
}

/// Uma corrente de `n` ossos de 10 unidades, deitada sobre o eixo X. Devolve `[raiz, .., ponta]`.
#[cfg(test)]
pub fn test_chain(sim: &mut SimWorld, n: usize) -> Vec<u64> {
    let mut out = Vec::new();
    let mut pai = None;
    for i in 0..n {
        let x = f64::from(u16::try_from(i).unwrap_or(0)) * 10.0;
        let b = create(sim, pai, [x, 0.0], [x + 10.0, 0.0]).expect("osso");
        pai = Some(Entity::from_bits(b));
        out.push(b);
    }
    out
}

#[cfg(test)]
#[path = "bone_gesture_tests.rs"]
mod tests;

/// **A corrente alcança `goal`** — IK sobre a cadeia que acaba em `tip`, escrita pela mesma porta
/// da rotação à mão (`pose(.., Body)`), do pai para o filho (cada `pose` lê o mundo do pai).
pub fn reach_chain(sim: &mut SimWorld, tip: Entity, goal: [f64; 2]) -> bool {
    let cadeia = ph2d_skeleton_live::skin_live::chain_to(sim, tip.to_bits());
    let segs = ph2d_skeleton_live::skin_live::bone_segments(sim);
    let mut juntas: Vec<[f64; 2]> = Vec::with_capacity(cadeia.len() + 1);
    let mut comps: Vec<f64> = Vec::with_capacity(cadeia.len());
    for &e in &cadeia {
        let Some((_, a, b)) = segs.iter().copied().find(|(x, _, _)| *x == e.to_bits()) else {
            return false;
        };
        juntas.push(a);
        comps.push((b[0] - a[0]).hypot(b[1] - a[1]));
        if e == tip {
            juntas.push(b);
        }
    }
    if comps.is_empty() || juntas.len() != comps.len() + 1 {
        return false;
    }
    ph2d_skeleton::reach(&mut juntas, &comps, goal, ph2d_skeleton::Reach::default());
    let mut mexeu = false;
    for (i, &e) in cadeia.iter().enumerate() {
        mexeu |=
            crate::bone_pose::pose(sim, e, juntas[i + 1], ph2d_skeleton_render::BonePart::Body);
    }
    mexeu
}

/// **Este arrasto chega a fazer um osso?** — porta do `Up` e da pré-visualização; mais curto que o
/// raio das alças (em píxeis de tela) não faz osso.
#[must_use]
pub fn drag_makes_a_bone(origin: [f64; 2], tip: [f64; 2], px_to_world: f64) -> bool {
    (tip[0] - origin[0]).hypot(tip[1] - origin[1]) >= BONE_HIT_PX * px_to_world
}

/// **O que este arrasto significa agora** — lido pela pré-visualização e pelo release.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneDragNow {
    /// A corrente solta que o release vai ADOPTAR, e a base dela em mundo. `None` = sem emenda.
    pub splice: Option<(u64, [f64; 2])>,
    /// A ponta do osso: a **base do alvo** quando há emenda, senão o ponto onde a mão está.
    pub tip: [f64; 2],
    /// Este arrasto chega a fazer um osso? — medido sobre a ponta **já encaixada**.
    pub armed: bool,
}

/// [`BoneDragNow`] para este instante do arrasto — a porta única dos dois consumidores.
#[must_use]
pub fn drag_now(
    sim: &SimWorld,
    birth: BoneBirth,
    pointer: [f64; 2],
    px_to_world: f64,
) -> BoneDragNow {
    let splice = splice_target(sim, pointer, px_to_world, birth);
    let tip = splice.map_or(pointer, |(_, base)| base);
    BoneDragNow {
        splice,
        tip,
        // Sobre a ponta ENCAIXADA: é esse o osso que nasce.
        armed: drag_makes_a_bone(birth.origin, tip, px_to_world),
    }
}

/// **Este arrasto junta duas correntes?** — a porta única da emenda (dono 2026-09-09): o osso a
/// ADOPTAR e a base dele em mundo. A recusa do ciclo vive aqui para a pré-visualização e o `Up`
/// recusarem o mesmo gesto.
#[must_use]
pub fn splice_target(
    sim: &SimWorld,
    world: [f64; 2],
    px_to_world: f64,
    birth: BoneBirth,
) -> Option<(u64, [f64; 2])> {
    let (alvo, base) = crate::bone_pick::free_root_at(sim, world, px_to_world)?;
    (!adopting_would_cycle(sim, alvo, birth.parent)).then_some((alvo, base))
}

/// **Adoptar `alvo` sob um osso novo filho de `novo_pai` fecharia um laço?** (sobe a árvore inteira)
#[must_use]
fn adopting_would_cycle(sim: &SimWorld, alvo: u64, novo_pai: Option<u64>) -> bool {
    let Some(alvo) = Entity::try_from_bits(alvo) else {
        return true;
    };
    let mut actual = novo_pai.and_then(Entity::try_from_bits);
    while let Some(e) = actual {
        if e == alvo {
            return true;
        }
        actual = sim.world().get::<ChildOf>(e).map(ChildOf::parent);
    }
    false
}

/// **Pendura `child` em `parent` sem o mover** ([`ph2d_vec_entities::transform::reparent_keeping_world`])
/// e tira o `RootOrder` (um filho não o leva). O ciclo é recusado no [`splice_target`].
pub fn connect(sim: &mut SimWorld, child: u64, parent: u64) -> bool {
    let (Some(c), Some(p)) = (Entity::try_from_bits(child), Entity::try_from_bits(parent)) else {
        return false;
    };
    if !ph2d_vec_entities::transform::reparent_keeping_world(sim, c, p) {
        return false;
    }
    if let Ok(mut e) = sim.world_mut().get_entity_mut(c) {
        e.remove::<RootOrder>();
    }
    true
}

/// **Faz um osso** de `origin` a `tip` (mundo), filho de `parent` — delega a [`ph2d_skeleton_live::bone::create`].
pub fn create(
    sim: &mut SimWorld,
    parent: Option<Entity>,
    origin: [f64; 2],
    tip: [f64; 2],
) -> Option<u64> {
    ph2d_skeleton_live::bone::create(sim, parent, origin, tip)
}
