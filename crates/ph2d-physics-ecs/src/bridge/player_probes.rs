//! **OS SENSORES do player** — o teto da quina, o headroom do agachar e a
//! parede.
//!
//! ⚠️ **Corte por RESPONSABILIDADE:** o `bridge::player` responde *"o que a lei
//! decidiu neste tique?"* — o cast do chão, a chamada da lei, o motor, a reação —
//! e estes três respondem a pergunta anterior: *"o que há à volta?"*. Nenhum
//! deles tem política: a decisão de castar (`*_probe_wanted`) é da LEI, e é ela
//! que os chama.
//!
//! ⚠️ **Dois leem por RAIOS e um VARRE o corpo**, e a diferença não é estilo:
//!
//! - o da **quina** é um PERFIL (65 amostras que dizem *onde* há teto, para a
//!   lei escolher para que lado escapar) — uma varredura devolve um contacto e
//!   não sabe responder isso;
//! - o da **parede** entrega o flanco inteiro e a lei reduz com a régua da perna
//!   (`max_slope`), porque *qual superfície* e *se é parede* são a mesma
//!   pergunta;
//! - o do **agachar** só alguma vez foi perguntado *"cabe?"*, e essa é
//!   literalmente a pergunta que uma varredura responde. Ele foi convertido na
//!   `W-ShapeCast`; os outros dois não, com o motivo escrito em cada um.
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

/// **O que há sobre a cabeça** (W15) — a varredura que decide se ele pode
/// levantar-se de um agachar.
///
/// O corpo é varrido para CIMA por exactamente
/// [`ph2d_platformer::CrouchConfig::rise`] — *quanto ele SOBE ao levantar-se*.
/// Nem um milímetro além: perguntar mais longe recusaria o gesto por causa de um
/// teto que o personagem, de pé, não alcança.
///
/// # ⚠️ Eram TRÊS RAIOS, e a `W-ShapeCast` trocou-os por UMA varredura
///
/// A pergunta é *"o corpo cabe se subir `rise`?"*, e uma grade de linhas só sabe
/// respondê-la por amostragem. O preço estava medido
/// (`measure_the_gap_between_rays`): um pilar de 8 cm entre duas amostras é
/// invisível, a cabeça chega a **1,267** contra pedra em **1,25** e o solver
/// segura-a lá dentro.
///
/// ⚠️ **E some com ele a caixa envolvente:** a varredura usa a forma REAL do
/// collider, então a ressalva *"um teto que toque só a quina da caixa recusa o
/// levantar"* deixa de existir — não é uma tolerância que alguém afrouxou, é uma
/// pergunta que passou a ser feita sobre o corpo em vez de sobre a caixa dele.
///
/// ⚠️ **A dívida de gate que isto PAGA:** o doc anterior nomeava uma metade sem
/// gate (*"o laço percorre os TRÊS deslocamentos" só é observável sob um teto
/// parcial … trocar o laço por um raio central sobrevive à suíte*). Não há laço
/// a percorrer: a varredura não tem amostras para alguém esquecer.
///
/// ⚠️ **`sweep_body` já exclui o próprio corpo**, então não há aqui a aritmética
/// do "nasce no centro e desconta meia-largura" que os raios precisavam.
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
    Some(Headroom {
        blocked: world.sweep_body(handle, [0.0, 1.0], rise, layer).is_some(),
    })
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
