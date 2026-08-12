//! **A PERNA** (`W-FootFan`) — como ela procura o chão, e o que ela aceita como
//! chão.
//!
//! Módulo irmão do `player.rs` por RESPONSABILIDADE: ali mora *o que o tique faz
//! com a resposta*, aqui mora *como a resposta é obtida*. O corte nasceu de um
//! teto de LOC, mas a linha é real — esta é a única pergunta do tique que tem
//! uma **lei** própria, e ela tem duas metades que se aprendem juntas.
//!
//! # ⚠️ A perna é um LEQUE, não um raio
//!
//! Um raio só no centro afunda **0,411 m — 46% do `float_height`** parado sobre
//! uma fenda de 10 cm que as bordas do corpo suportam fisicamente; a 40 cm ele
//! sai do mundo (113 m). É a mesma doença que o flanco teve na W13, e a mesma
//! cura.
//!
//! # ⚠️ E duas leis decidem o que conta como chão
//!
//! **A redução é o MAIS PRÓXIMO** — o chão é o degrau mais alto que qualquer pé
//! alcança —, e o `<` estrito faz o raio do MEIO ganhar todo empate (ele é o
//! índice 0 do `wall_offsets`), o que mantém um corpo sobre chão plano
//! **byte-idêntico** ao que era.
//!
//! **Mas um DEGRAU não é uma RAMPA.** Sem a segunda lei o leque transforma em
//! chão tudo o que o corpo alcança: medido, um personagem que empurrava um
//! caixote de 0,6 m passa a **SUBIR nele**, e o deslocamento do caixote cai de
//! **7,27 para −0,02 m**. Um pé de fora só conta se o chão dele for
//! **caminhável** a partir do pé do meio — e o número não é escolhido: é o
//! `max_slope` que o artista já autorou, a mesma pergunta que produz o
//! [`ph2d_platformer::Footing::Steep`].
//!
//! ⚠️ **E a terceira metade não é uma lei, é o `spread`:** um raio na borda
//! EXATA da cápsula é tangente a tudo o que o corpo encosta (ver
//! [`ph2d_platformer::RideConfig::spread`]).

use ph2d_physics::CastHit;
use ph2d_platformer::{MAX_WALL_SAMPLES, PlayerConfig};

use super::probes;

/// A folga que faz de uma rampa exatamente no `max_slope` uma rampa, e não um
/// degrau: um milímetro, a mesma ordem do `normalized_allowed_linear_error` com
/// que o solver assenta.
const STEP_EPS: f32 = 1.0e-3;

/// **Quanto um pé de fora pode estar ACIMA do pé do meio, por metro de
/// afastamento** — a tangente do limite de rampa.
///
/// ⚠️ **Por `max_slope_cos()` e uma raiz, nunca por uma tangente do `std`:** este
/// número entra no `physics_ecs_c9`, e a lei do módulo é que 1 ulp entre dois
/// sistemas operacionais é um bug. A porta do cosseno já passa por `libm`; o
/// resto é `sqrt`, que o IEEE-754 especifica exatamente.
///
/// `max_slope` de 90° dá cosseno 0 ⇒ tudo é caminhável ⇒ limite infinito, que é
/// o que aquele número significa.
fn step_limit(cfg: &PlayerConfig) -> f32 {
    let c = cfg.walk.max_slope_cos();
    if c <= 0.0 {
        return f32::INFINITY;
    }
    (1.0 - c * c).max(0.0).sqrt() / c
}

/// **O que a perna achou** — o hit que a lei consome e a resposta de CADA pé.
pub(super) struct LegRead {
    /// O vencedor da redução: o chão mais alto e caminhável.
    pub hit: Option<CastHit>,
    /// ⚠️ **As respostas POR RAIO, para o overlay** (o padrão do flanco,
    /// `WallProbe.hits`): quem desenha mostra o que a lei de facto viu, e não
    /// uma segunda consulta. Sem isto o desenho carimba o veredito reduzido no
    /// raio do MEIO, que nem sempre é o vencedor — o overlay diria que o pé do
    /// meio achou chão num tique em que quem achou foi o da borda.
    pub per_foot: [Option<f32>; MAX_WALL_SAMPLES],
}

/// **Casta a perna inteira** e aplica as duas leis (ver o doc do módulo).
pub(super) fn cast_leg(
    world: &ph2d_physics::PhysicsWorld,
    body: ph2d_physics::RigidBodyHandle,
    cfg: &PlayerConfig,
    origin: [f32; 2],
    reach: f32,
    passing: Option<ph2d_physics::ColliderHandle>,
    layer: u8,
) -> LegRead {
    let (legs, n) = probes::ground_rays(world, body, cfg, origin);
    let mut per_foot = [None; MAX_WALL_SAMPLES];
    let mut hit: Option<CastHit> = None;
    let step = step_limit(cfg);
    // A referência é o pé do MEIO, o índice 0 do `wall_offsets` — e é por isso
    // que ele é castado primeiro.
    let mut centre: Option<f32> = None;
    for (i, (leg, slot)) in legs.iter().zip(per_foot.iter_mut()).enumerate().take(n) {
        let Some(h) =
            world.cast_ray_skipping(leg.origin, leg.dir, reach, Some(body), passing, layer)
        else {
            continue;
        };
        // ⚠️ **O que o raio ACHOU viaja sempre** — inclusive quando a lei o
        // recusa: ver o pé da frente pousado no caixote e a perna a ignorá-lo É
        // o diagnóstico, e escondê-lo faria o desenho contar meia história.
        *slot = Some(h.distance);
        if i == 0 {
            centre = Some(h.distance);
        } else if let Some(c) = centre {
            // Só o degrau para CIMA importa: um pé sobre um buraco acha chão
            // mais LONGE, e mais longe nunca ganha a redução.
            let dx = (leg.origin[0] - origin[0]).abs();
            if c - h.distance > dx * step + STEP_EPS {
                continue;
            }
        }
        if hit
            .as_ref()
            .is_none_or(|best: &CastHit| h.distance < best.distance)
        {
            hit = Some(h);
        }
    }
    LegRead { hit, per_foot }
}
