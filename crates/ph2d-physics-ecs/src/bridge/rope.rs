//! **A CORDA de uma polia**: onde as roldanas dela estão, de que lado ela passa
//! em cada uma, e como uma polia recém-criada nasce montada.
//!
//! Irmão de [`super::joints`], e o corte é o assunto: lá mora *o que um JOINT é*
//! (as âncoras, o descritor, o ciclo de vida no solver); aqui, *o que uma
//! RODA é para a corda* — a colheita das entidades, o ponto fixo do lado, e a
//! geometria de montagem. Nasceu do cap de 700 LOC quando a roldana virou
//! entidade (W-Pulley W1), e a próxima wave da polia (motor, ruptura no centro)
//! chega aqui, não lá.

use ph2d_ecs::{Entity, Name, stable_name_id};
use ph2d_physics::world::rope_route::{self, RopeWheel};

use super::PhysicsBridge;
use crate::joint::PhysicsJoint;
use bevy_ecs::query::QueryState;
use ph2d_ecs::Transform;

/// As roldanas — entidades com [`crate::PulleyWheel`] e uma pose (W-Pulley W1).
///
/// Query própria, separada da dos joints, porque uma roldana é uma ENTIDADE e a
/// corda a alcança pelo NOME.
pub(super) type WheelQuery = QueryState<(Entity, &'static crate::PulleyWheel, &'static Transform)>;

/// Uma roldana colhida do mundo, com a chave por que ela é ordenada.
///
/// A ordem é `(corda, order autorado, desempate)` e o desempate é o
/// `stable_name_id` do NOME — nunca os bits da entidade, que são id de ALOCAÇÃO e
/// mudam a cada undo. Duas roldanas com o mesmo `order` na mesma corda são um
/// empate que o artista criou, e resolvê-lo por algo que o undo embaralha faria a
/// corda trocar de rota ao desfazer.
#[derive(Copy, Clone, Debug)]
pub(super) struct RopeWheelRow {
    /// `stable_name_id` da corda a que ela pertence.
    pub(super) rope: u64,
    /// A chave de ordenação dentro da corda.
    pub(super) key: (u16, u64),
    /// A entidade — o que o desenho e as alças precisam para saber QUEM é.
    pub(super) entity: Entity,
    /// A roda como a rota a consome.
    pub(super) wheel: RopeWheel,
    /// O que o artista escolheu sobre o lado.
    pub(super) wrap: crate::WrapSide,
    /// **Quanto de corda este tambor recolhe por segundo** — `ω·r`, já
    /// convertido, porque é essa a grandeza que a corda entende (W2).
    ///
    /// A conversão mora aqui e não no kernel porque o RAIO é da roldana e o
    /// kernel só conhece a corda: somar `ω·r` na colheita é o que torna *"as
    /// taxas somam"* uma soma de metros por segundo, e não de radianos por
    /// segundo de rodas de tamanhos diferentes.
    pub(super) reel_rate: f32,
}

impl PhysicsBridge {
    /// **Colher as roldanas do mundo** para a lista que o reconcile consome.
    ///
    /// Um passe próprio, antes do laço dos joints, porque uma roldana é uma
    /// ENTIDADE (W-Pulley W1) e a corda a alcança por NOME — não há como
    /// descobri-las de dentro do laço sem uma query aninhada por corda.
    ///
    /// A ordem é `(corda, order autorado, nome)`, e a chave inteira é estável
    /// através do undo: os bits de entidade são id de ALOCAÇÃO e o undo respawna
    /// tudo, então ordenar por eles faria a corda trocar de rota ao desfazer.
    pub(super) fn harvest_rope_wheels(&mut self, world: &ph2d_ecs::World) {
        self.rope_wheels.clear();
        let mut wq = self.wheel_query.take().expect("query built in dispatch");
        for (we, wheel, _local) in wq.iter(world) {
            // `rope: 0` não casa com nome nenhum: uma roldana órfã é inerte, e
            // nunca uma roldana que se prende à primeira corda que aparecer.
            if wheel.rope == 0 {
                continue;
            }
            // A pose de MUNDO, como a âncora do joint: uma roldana pode ser
            // parenteada como qualquer entidade, e ler o local como mundo é
            // exatamente o bug que o W5 pagou nos cinco sítios do corpo.
            let Some(t) = super::space::world_transform(world, we, &mut self.chain) else {
                continue;
            };
            let wheel = wheel.clamped();
            self.rope_wheels.push(RopeWheelRow {
                rope: wheel.rope,
                key: (
                    wheel.order,
                    world
                        .get::<Name>(we)
                        .map_or(0, |n| stable_name_id(n.as_str())),
                ),
                entity: we,
                wheel: RopeWheel {
                    centre: [t.translation.x, t.translation.y],
                    radius: wheel.radius,
                    // A roldana é apontada pelo NOME dela, a mesma chave por que
                    // a corda aponta os corpos — bits de entidade mudam a cada
                    // undo, e o eixo partido migraria para a vizinha.
                    id: world
                        .get::<Name>(we)
                        .map_or(0, |n| stable_name_id(n.as_str())),
                    break_force: if wheel.break_enabled {
                        wheel.break_force
                    } else {
                        f32::INFINITY
                    },
                    // Substituído pela resolução de lado, que precisa da corda
                    // inteira para responder. Este é só o valor de partida.
                    side: 1,
                },
                wrap: wheel.wrap,
                // ⚠️ Raio ZERO não recolhe, e não é um caso especial a lembrar:
                // uma roldana-PONTO não tem superfície de que a corda se agarre,
                // e `ω·0` já é zero. A aritmética responde sozinha.
                reel_rate: wheel.motor_speed * wheel.radius,
            });
        }
        self.wheel_query = Some(wq);
        self.rope_wheels
            .sort_unstable_by_key(|r| (r.rope, r.key.0, r.key.1));
    }
}

/// **Onde nascem as roldanas de uma polia recém-montada, e que comprimento a
/// corda tem.**
///
/// As roldanas ficam **diretamente acima** de cada corpo — que é o que uma
/// roldana faz — e a altura é METADE da distância entre os dois, derivada da
/// geometria que o artista já montou em vez de uma constante que não escala com
/// a cena. `PhysicsJoint::MIN_WHEEL_LIFT` é só o piso do caso degenerado (dois
/// corpos quase no mesmo lugar), onde os ramos nasceriam sem direção.
///
/// **O RAIO sai da mesma geometria** — uma fração da altura —, pela mesma razão:
/// uma roldana de raio constante seria invisível numa cena de 50 m e maior que a
/// carga numa de 1 m. O artista a redimensiona pela alça do aro.
///
/// O comprimento é o da ROTA que a montagem tem AGORA (trechos + arcos), então a
/// corda nasce exatamente esticada e o primeiro frame não dá um puxão.
///
/// ⚠️ **As roldanas saem SEMPRE; o comprimento, só quando o gesto sabe onde a
/// corda se amarra.** As duas rotas de criação diferem exatamente nisso — a do
/// canvas aponta as âncoras (e nasce `anchored: true`, então ninguém mais vai
/// semear), a da SELEÇÃO deixa o reconcile derivá-las. Devolver o par separado é
/// o que impede o `None` de um significar *sem roldanas* no outro: a polia da
/// seleção nascia sem roda nenhuma, e o gate das duas rotas pegou.
///
/// `None` no comprimento também cobre a rota degenerada — um corpo em cima de uma
/// roldana, a mesma recusa que o passe de impulso faz, perguntada pela mesma
/// porta. Sem ela o comprimento nasceria `NaN`.
///
/// ## Por que ela é `pub`: DOIS chamadores, uma resposta
///
/// O semeio do reconcile (logo abaixo) é gateado em `anchored`, e esse sentinela
/// responde *"as âncoras estão autoradas?"*. O gesto de criação pelo CANVAS
/// (press no corpo A -> arrasta -> solta no B) **sabe** onde as âncoras vão e por
/// isso nasce `anchored: true` — deliberadamente, senão a política de semeio
/// jogaria fora o ponto que o artista apontou.
///
/// ⚠️ **E aí um sentinela respondia DUAS perguntas.** A rota que aprendeu a
/// responder a primeira pulava a segunda em silêncio: uma polia criada pelo
/// canvas ficava com as duas roldanas em `[0, 0]`, ou seja **na origem do
/// mundo**, com a corda saindo de cada corpo até lá. Foi o que o artista
/// fotografou. A cura não é um segundo sentinela: é o gesto de criação
/// **estabelecer a geometria autorada INTEIRA**, chamando esta mesma função — e
/// hoje isso quer dizer SPAWNAR as duas roldanas, porque cada uma é uma entidade.
#[must_use]
pub fn pulley_rig(
    rest_a: [f32; 2],
    rest_b: [f32; 2],
    attach: Option<([f32; 2], [f32; 2])>,
) -> ([RopeWheel; 2], Option<f32>) {
    let dx = rest_a[0] - rest_b[0];
    let dy = rest_a[1] - rest_b[1];
    let lift = (0.5 * (dx * dx + dy * dy).sqrt()).max(PhysicsJoint::MIN_WHEEL_LIFT);
    let wheel = |c: [f32; 2]| RopeWheel {
        centre: c,
        radius: lift * WHEEL_RADIUS_FRACTION,
        side: 1,
        id: 0,
        break_force: f32::INFINITY,
    };
    let mut wheels = [
        wheel([rest_a[0], rest_a[1] + lift]),
        wheel([rest_b[0], rest_b[1] + lift]),
    ];
    let Some((attach_a, attach_b)) = attach else {
        return (wheels, None);
    };
    let mut scratch = Vec::new();
    rope_route::resolve_sides(attach_a, attach_b, &mut wheels, &mut scratch);
    let length = rope_route::route(attach_a, attach_b, &wheels, &mut scratch).map(|r| r.length);
    (wheels, length)
}

/// **Que fração da altura das roldanas o RAIO delas mede** ao nascer.
///
/// Não é um teto nem um limite físico: é o tamanho com que uma roldana nasce
/// para ser VISÍVEL e proporcional à cena que o artista acabou de montar — no
/// rig do smoke (corpos a 3 m, altura 1,5 m) dá **0,375 m**, um pouco maior que
/// os corpos de 0,25 m, que é como uma polia se parece. O artista muda pela alça
/// do aro, e a fração some da história dali em diante.
const WHEEL_RADIUS_FRACTION: f32 = 0.25;

/// **De que lado a corda passa nesta roldana, honrando a escolha do artista.**
///
/// `Auto` devolve o que a geometria achou (o ponto fixo do `resolve_sides`, que é
/// quem chamou aqui). `Over`/`Under` são ditos como um humano olha para a tela, e
/// por isso são resolvidos contra a CORDA como um todo: a roldana desvia a corda
/// para o lado dela, então *a corda passa por CIMA* quer dizer que a roldana está
/// ACIMA da reta que liga as duas pontas.
///
/// ⚠️ **A régua é a CORDA (âncora → âncora), não o eixo Y do mundo.** Uma corda
/// que corre na vertical não tem "cima" e "baixo" — o que ela tem são dois lados,
/// e é isso que estas duas palavras nomeiam para o artista. Medir contra o mundo
/// deixaria as duas opções indistinguíveis exatamente na montagem em que ele
/// precisa delas.
pub(super) fn wrap_side(
    wrap: crate::WrapSide,
    auto: i8,
    centre: [f32; 2],
    a: [f32; 2],
    b: [f32; 2],
) -> i8 {
    let wanted_above = match wrap {
        crate::WrapSide::Auto => return auto,
        crate::WrapSide::Over => true,
        crate::WrapSide::Under => false,
    };
    // De que lado da corda a roldana está. `cross > 0` é à ESQUERDA do sentido
    // A → B; com a corda correndo para a direita, esquerda é para CIMA.
    let d = [b[0] - a[0], b[1] - a[1]];
    let r = [centre[0] - a[0], centre[1] - a[1]];
    let cross = d[0] * r[1] - d[1] * r[0];
    // Colinear: nem um lado nem outro. Fica com o que a geometria achou, pelo
    // mesmo motivo que o `sign_or` da rota fica com o anterior.
    if cross == 0.0 {
        return auto;
    }
    // A roldana ACIMA da corda a desvia para cima, e a corda a abraça por cima:
    // o centro fica ABAIXO do trecho, que é `side = −1` (o centro à direita do
    // sentido de percurso). A tabela inteira é essa frase e a sua negação.
    let above = if d[0] >= 0.0 {
        cross > 0.0
    } else {
        cross < 0.0
    };
    if above == wanted_above { -1 } else { 1 }
}

impl PhysicsBridge {
    /// **Girar as roldanas** — um tick de `ω = s·lado/r`.
    ///
    /// O pedido (3) do artista tinha duas metades: *"não temos o diâmetro da
    /// roldana na simulação nem a representação dela e sua ROTAÇÃO"*. O diâmetro é
    /// o raio; isto é a rotação, e sem ela uma roda grande e uma pequena giram
    /// igual — que é dizer que o diâmetro não faz nada.
    ///
    /// ⚠️ **Uma velocidade por CORDA, não por roda:** a corda é inextensível,
    /// então ela corre na mesma taxa por todas as roldanas dela, e é o RAIO de cada
    /// uma que decide quanto ela gira (`ω = s/r`). **A roda grande gira mais
    /// devagar, e é isso que se vê.**
    ///
    /// ⚠️ **Por TICK, nunca por frame** — o ângulo é a integral de uma taxa, e
    /// integrá-lo no frame faria a roda girar mais rápido numa máquina mais
    /// rápida. É a mesma lei que o `drag` segue uma porta acima.
    ///
    /// Raio zero não gira: uma roldana-PONTO não tem superfície para a corda
    /// arrastar, e `s/0` seria o infinito que envenena a pose de desenho.
    pub(super) fn spin_rope_wheels(&mut self) {
        if self.pulley_records.is_empty() {
            return;
        }
        let dt = self.world.substep_dt() * self.world.substeps() as f32;
        for r in &self.pulley_records {
            let Some(speed) = self.world.pulley_rope_speed(&r.desc) else {
                continue;
            };
            let start = r.desc.wheel_start as usize;
            for i in start..start + r.desc.wheel_count as usize {
                let (Some(w), Some(&e)) = (
                    self.world.pulley_wheels().get(i),
                    self.wheel_entities.get(i),
                ) else {
                    continue;
                };
                if w.radius <= 0.0 {
                    continue;
                }
                let d = speed * f32::from(w.side) / w.radius * dt;
                let a = self.wheel_spin_by_entity.entry(e).or_insert(0.0);
                // Enrolado em ±π: um ângulo que cresce sem parar perde precisão de
                // `f32` num take longo, e o desenho só quer a direção do raio-guia.
                *a = wrap_pi(*a + d);
                if let Some(slot) = self.wheel_spin.get_mut(i) {
                    *slot = *a;
                }
            }
        }
    }

    /// O ângulo de cada roldana da arena, na MESMA ordem — o que o desenho gira o
    /// raio-guia por.
    #[must_use]
    pub fn pulley_wheel_spins(&self) -> &[f32] {
        &self.wheel_spin
    }
}

/// `a` trazido para `(−π, π]`.
///
/// Sem transcendental: o ângulo de giro é estado de DESENHO e não alcança o hash,
/// mas a lei 6 vale para o módulo inteiro e um `%` com `TAU` é exato o bastante.
fn wrap_pi(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut x = a % tau;
    if x > std::f32::consts::PI {
        x -= tau;
    } else if x <= -std::f32::consts::PI {
        x += tau;
    }
    x
}
