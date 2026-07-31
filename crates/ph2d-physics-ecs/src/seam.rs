//! **Onde dois corpos SE ENCONTRAM** — a emenda, e o pivô que toda rota de
//! criação de joint usa como ponto de partida.
//!
//! A regra antiga era o **ponto médio dos dois centros**, e o próprio doc dela
//! dizia o que ela estava aproximando: *"para um Pin entre dois corpos que se
//! TOCAM — um elo de corrente, o caso comum — o meio É o pivô certo"*. É verdade,
//! e só nesse caso: dois retângulos iguais encostados têm o meio exatamente na
//! junta. Uma cabeça pequena sobre um tronco grande **não**, e o pivô cai dentro
//! do tronco — a cabeça passa a girar em torno de um ponto no peito.
//!
//! Este módulo torna a regra EXATA em vez de aproximada. A emenda é o meio entre
//! os dois pontos onde as SILHUETAS cruzam a linha que liga os centros:
//!
//! ```text
//!        ra                    rb
//!   A ●------|===============|------● B
//!          borda de A      borda de B
//!                   ↑
//!              a emenda
//! ```
//!
//! ⚠️ **Ela REDUZ ao ponto médio quando as duas formas são iguais e se tocam**
//! (`ra == rb` e `|AB| == ra + rb` ⇒ as duas bordas são o mesmo ponto, que é o
//! meio) — o caso que o desenho antigo acertava continua acertado, e é isso que
//! torna a troca segura para a corrente que já shipa.
//!
//! ⚠️ **E não há geometria nova aqui.** `ShapeDesc::radial_fraction` — a régua
//! que o falloff de área (W-AreaFalloff) já usa — é uma **função-calibre**:
//! `f(t·d) = t·f(d)` e `f == 1` exatamente na borda. Então o alcance da silhueta
//! numa direção unitária é `1 / f(d)`, em forma fechada, para as três famílias de
//! forma **e** já com a escala do W6 dentro (o `scaled_shape` é quem a resolve).
//! Uma segunda resposta a *"onde esta forma termina?"* divergiria dela na
//! primeira vez que qualquer uma mudasse.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics::ShapeDesc;

use crate::Collider;
use crate::scale::{collider_offset, scaled_shape};

/// Um collider como o SOLVER o vê: a forma já resolvida pela escala, e o centro
/// dela em MUNDO (a pose do corpo mais o offset autorado, girado com ele).
#[derive(Copy, Clone, Debug)]
pub struct ColliderPose {
    /// O centro do COLLIDER em mundo — não o do corpo. Os dois só coincidem com
    /// offset zero, e um hitbox de pés é exatamente o caso em que não coincidem.
    pub centre: [f32; 2],
    /// A rotação do corpo, radianos CCW: é o frame em que a forma é medida.
    pub rotation: f32,
    /// A forma com a escala de mundo já aplicada (W6).
    pub shape: ShapeDesc,
}

impl ColliderPose {
    /// Resolve o collider de `col` sob a pose de **MUNDO** `t`.
    ///
    /// ⚠️ `t` tem de ser a pose de mundo (`ph2d_ecs::world_transform`), nunca o
    /// `Transform` cru: ele é LOCAL num corpo parenteado, e medir a emenda entre
    /// uma pose de mundo e um offset de pai é o defeito que a W-Rig achou na
    /// porta de criação.
    #[must_use]
    pub fn resolve(col: &Collider, t: &Transform) -> Self {
        let off = collider_offset(col, t.scale);
        let (s, c) = libm::sincosf(t.rotation);
        Self {
            centre: [
                t.translation.x + c * off[0] - s * off[1],
                t.translation.y + s * off[0] + c * off[1],
            ],
            rotation: t.rotation,
            shape: scaled_shape(col.shape, t.scale),
        }
    }

    /// Quão longe a silhueta chega a partir do centro, na direção UNITÁRIA de
    /// mundo `dir`.
    ///
    /// `0` para uma forma degenerada (meia-extensão zero): não há interior a
    /// medir, e a falha honesta de uma régua indefinida é não deslocar nada — a
    /// mesma política que a `radial_fraction` já adota.
    #[must_use]
    fn reach(&self, dir: [f32; 2]) -> f32 {
        // A direção no frame do collider. `libm`, não `f32::sin_cos`: este número
        // vira uma âncora, a âncora vira uma restrição, e a restrição alcança os
        // impulsos que o `physics_ecs_c9` compara entre os três OSes (lei 6).
        let (s, c) = libm::sincosf(self.rotation);
        let local = [c * dir[0] + s * dir[1], (-s) * dir[0] + c * dir[1]];
        let f = self.shape.radial_fraction(local);
        if f > 0.0 { 1.0 / f } else { 0.0 }
    }
}

/// **A emenda entre dois colliders** — o meio entre os dois pontos em que as
/// silhuetas cruzam a linha dos centros.
///
/// Encostados, é o ponto de contato. Sobrepostos ou afastados, é o meio da região
/// que os separa — a interpolação natural, e a única que continua contínua quando
/// o artista arrasta um dos dois.
///
/// Centros coincidentes não têm direção: a resposta é o próprio ponto, sem
/// inventar um eixo.
#[must_use]
pub fn seam_point(a: &ColliderPose, b: &ColliderPose) -> [f32; 2] {
    let d = [b.centre[0] - a.centre[0], b.centre[1] - a.centre[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    if len2 <= f32::EPSILON {
        return [
            (a.centre[0] + b.centre[0]) * 0.5,
            (a.centre[1] + b.centre[1]) * 0.5,
        ];
    }
    let inv = 1.0 / len2.sqrt();
    let dir = [d[0] * inv, d[1] * inv];
    let ra = a.reach(dir);
    let rb = b.reach([-dir[0], -dir[1]]);
    // As duas bordas, e o meio delas. Com formas iguais que se tocam as duas
    // caem no MESMO ponto, que é o ponto médio dos centros — o desenho antigo,
    // preservado exatamente onde ele estava certo.
    [
        (a.centre[0] + dir[0] * ra + b.centre[0] - dir[0] * rb) * 0.5,
        (a.centre[1] + dir[1] * ra + b.centre[1] - dir[1] * rb) * 0.5,
    ]
}

/// A emenda entre dois corpos, resolvida das componentes — a porta que a criação
/// de joint chama.
///
/// `None` quando um dos dois não tem collider: sem forma não há silhueta, e a
/// resposta honesta é deixar o chamador cair no ponto médio dos centros (que é o
/// que ele sabe sem geometria nenhuma).
#[must_use]
pub fn seam_between(
    a: Option<(&Collider, Transform)>,
    b: Option<(&Collider, Transform)>,
) -> Option<Vec2> {
    let (ca, ta) = a?;
    let (cb, tb) = b?;
    let p = seam_point(
        &ColliderPose::resolve(ca, &ta),
        &ColliderPose::resolve(cb, &tb),
    );
    Some(Vec2::new(p[0], p[1]))
}
