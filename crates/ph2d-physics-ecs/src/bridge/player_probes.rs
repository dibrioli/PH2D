//! **OS SENSORES do player** — o teto da quina, o headroom do agachar e a
//! parede.
//!
//! ⚠️ **Corte por RESPONSABILIDADE:** o `bridge::player` responde *"o que a lei
//! decidiu neste tique?"* — o cast do chão, a chamada da lei, o motor, a reação —
//! e estes três respondem a pergunta anterior: *"o que há à volta?"*. Cada um é
//! um LEQUE de raios com uma régua própria, e nenhum deles tem política: a
//! decisão de castar (`*_probe_wanted`) é da LEI, e é ela que os chama.
//!
//! Módulo FILHO por `#[path]`, então `super::*` continua a alcançar o que o pai
//! não exporta.

use super::*;

/// **O perfil do teto acima da cabeça** (W10) — a metade do sensor que este
/// módulo possui, e nada de política.
///
/// Os raios nascem no TOPO da caixa do corpo e medem `rel_up · dt ·
/// [`CORNER_LOOKAHEAD`]` — a distância que a cabeça sobe no PRÓXIMO tique. É
/// isso, e só isso, que torna a assistência preditiva: o que o perfil descreve é
/// um contato que ainda não aconteceu.
///
/// ⚠️ **A grade de deslocamentos vem da lei** ([`corner_offsets`]), nunca de uma
/// aritmética local. Duas cópias deslocariam o perfil de meia célula em relação
/// ao corpo, e o sintoma seria um personagem empurrado para dentro da quina de
/// que ele fugia.
///
/// ⚠️ **A folga lateral é medida do CENTRO e descontada a meia-largura**, porque
/// o raio nasce no centro (a origem tem de estar dentro do corpo para o
/// `exclude_body` fazer sentido) e o que a lei quer saber é quanto há de espaço
/// **além** da borda. Saturada no próprio alcance: para esta decisão, *livre até
/// onde eu poderia querer ir* e *livre até o infinito* são a mesma coisa.
pub(super) fn probe_ceiling(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    rel_up: f32,
    dt: f32,
) -> Option<CeilingProbe> {
    let sweep = rel_up * dt * CORNER_LOOKAHEAD;
    if !sweep.is_finite() || sweep <= 0.0 {
        return None;
    }
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let top = maxs[1];
    let mid = (maxs[1] + mins[1]) * 0.5;
    let reach = cfg.jump.corner_reach;

    let mut blocked = [false; CORNER_SAMPLES];
    for (slot, off) in blocked
        .iter_mut()
        .zip(corner_offsets(half_width, reach).iter())
    {
        *slot = world
            .cast_ray([cx + off, top], [0.0, 1.0], sweep, Some(handle), layer)
            .is_some();
    }

    let free = |dir: f32| {
        world
            .cast_ray(
                [cx, mid],
                [dir, 0.0],
                half_width + reach,
                Some(handle),
                layer,
            )
            .map_or(reach, |h| (h.distance - half_width).clamp(0.0, reach))
    };

    Some(CeilingProbe {
        half_width,
        blocked,
        side_clear: [free(-1.0), free(1.0)],
    })
}

/// **O que há sobre a cabeça** (W15) — os raios que decidem se ele pode
/// levantar-se de um agachar.
///
/// Eles nascem no TOPO da caixa do corpo e medem exactamente
/// [`ph2d_platformer::CrouchConfig::rise`] — *quanto o corpo SOBE ao
/// levantar-se*. Nem um milímetro além: perguntar mais longe recusaria o gesto
/// por causa de um teto que o personagem, de pé, não alcança.
///
/// ⚠️ **A grade vem da lei** ([`headroom_offsets`]), nunca de uma aritmética
/// local — a mesma regra do sensor de quina, e pelo mesmo motivo: duas
/// aritméticas deslocariam as amostras em relação ao corpo.
///
/// ⚠️ **A caixa envolvente é CONSERVADORA, e a direcção do erro é a certa** (ver
/// o doc de [`headroom_offsets`]): um teto que toque só a quina da caixa recusa
/// o levantar. Ficar agachado onde caberia é um incómodo; levantar-se para
/// dentro da pedra é o solver a resolver uma penetração que ninguém autorou.
///
/// ⚠️ **O QUE NÃO ESTÁ GATEADO, e porquê:** a GRADE tem gate na lei
/// (`the_headroom_grid_spans_the_body`) e o `blocked` tem gate no produto — mas
/// a metade *"o laço percorre os TRÊS deslocamentos"* só é observável sob um
/// teto **PARCIAL**, que cubra uma borda do corpo e não o centro. Uma fixture
/// dessas teria de calibrar a aresta da laje contra a posição MEDIDA de um
/// personagem a andar, dentro de uma janela de 0,2 m — um gate que falharia por
/// deriva de fixture em vez de por defeito. Fica NOMEADO: trocar o laço por um
/// raio central sobrevive à suíte.
pub(super) fn probe_headroom(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
) -> Option<Headroom> {
    let rise = cfg.crouch.rise(&cfg.ride);
    if !rise.is_finite() || rise <= 0.0 {
        return None;
    }
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let top = maxs[1];

    let mut blocked = [false; HEADROOM_SAMPLES];
    for (slot, off) in blocked.iter_mut().zip(headroom_offsets(half_width).iter()) {
        *slot = world
            .cast_ray([cx + off, top], [0.0, 1.0], rise, Some(handle), layer)
            .is_some();
    }
    Some(Headroom { blocked })
}

/// **A parede ao lado** (W13) — a metade do sensor que este módulo possui, e
/// nada de política.
///
/// Um raio, na direção em que o jogador empurra, do CENTRO do corpo até meia
/// largura mais o alcance autorado.
///
/// ⚠️ **Do centro, e o alcance desconta a meia-largura**, pela mesma razão do
/// sensor de quina: a origem tem de estar DENTRO do corpo para o `exclude_body`
/// significar alguma coisa, e o que a lei quer saber é quanto há de parede
/// **além** da borda.
///
/// ⚠️ **O FLANCO INTEIRO, não só a cintura** — a altura de cada raio vem de
/// [`wall_offsets`], que é a porta única (o `crouch` e a quina têm as suas, pela
/// mesma razão). A versão de um raio só media a meia-altura, e o preço disso
/// está MEDIDO no `measure_wall_flank`: uma parede com fresta de 0,75 m, num
/// corpo de 1,0 m, **recusava o pulo de parede por inteiro** com 12,5 cm de pé e
/// de ombro ainda encostados.
///
/// ⚠️ **Fica o hit mais PRÓXIMO, e o meio desempata** (é o primeiro da lista):
/// numa parede plana os três raios medem a mesma distância, então a resposta é a
/// de sempre — o que muda é só onde o meio sozinho não via nada. E *mais
/// próximo* é o que um corpo empurrado de lado de facto encosta primeiro.
///
/// ⚠️ **Uma normal DEGENERADA não é candidata.** `distance == 0` significa que a
/// ⚠️ **Este módulo NÃO decide se aquilo é parede, NEM qual das amostras vale.**
/// Ele entrega o array inteiro e a lei escolhe (`ph2d_platformer::cling`) — o
/// padrão exato dos outros dois sensores multi-amostra desta ponte
/// ([`ph2d_platformer::Headroom`], [`ph2d_platformer::CeilingProbe`]). Reduzir
/// aqui seria uma segunda régua ao lado da que a perna já usa, e as duas
/// divergiriam no dia em que o `max_slope` autorado se movesse.
pub(super) fn probe_wall(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    drive: f32,
) -> Option<WallProbe> {
    let side = if drive > 0.0 { 1.0 } else { -1.0 };
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    let half_height = (maxs[1] - mins[1]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 || !half_height.is_finite() {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let cy = (maxs[1] + mins[1]) * 0.5;
    let reach = half_width + cfg.wall.reach.max(0.0);
    let mut hits = [None; WALL_SAMPLES];
    for (slot, off) in hits
        .iter_mut()
        .zip(wall_offsets(half_height.max(0.0)).iter())
    {
        *slot = world
            .cast_ray([cx, cy + off], [side, 0.0], reach, Some(handle), layer)
            .map(|hit| WallHit {
                distance: hit.distance,
                normal: hit.normal,
            });
    }
    Some(WallProbe { side, hits })
}
