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
use ph2d_ecs::{SimWorld, Transform};

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
    /// **Este contato é a CAUDA da rota** — o retorno de um eixo de Weston, o
    /// último nó que a corda toca (W-Weston).
    ///
    /// Entra na chave de ordenação ANTES do `order` autorado, então ele vai para o
    /// fim da corda por construção. Um sentinela em `u16::MAX` faria o mesmo até o
    /// dia em que alguém autorasse uma roldana ali — e então o desempate por nome
    /// escolheria a rota em silêncio.
    pub(super) tail: bool,
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
        self.wheels_to_seed.clear();
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
            // **A MONTAGEM (W3):** em que corpo o eixo desta roldana está.
            //
            // ⚠️ **Semeia o local UMA vez**, do `Transform` autorado contra a pose
            // de REPOUSO do corpo — a MESMA conversão que a âncora de joint faz, e
            // pela mesma razão que ela a faz uma vez só: re-derivar contra a pose
            // VIVA todo reconcile é o que fazia o pino DESLIZAR pelo corpo
            // (W-AnchorFollow, medido em 2 m). Depois de semeado, mover o corpo lê
            // o local inalterado e o eixo o acompanha.
            //
            // Corpo que não resolve (apagado, renomeado, ainda não spawnado) deixa
            // a roldana no CENÁRIO — inerte e não quebrada, a mesma cura que a
            // corda órfã e as bindings da timeline recebem.
            let mount = self
                .names
                .get(&wheel.body)
                .and_then(|e| self.bodies.get(e))
                .map(|b| {
                    let rest = [b.rest.x, b.rest.y, b.rest.rotation];
                    let local = if wheel.mounted {
                        wheel.local
                    } else {
                        let l = ph2d_physics::PhysicsWorld::local_anchor_at_pose(
                            rest,
                            [t.translation.x, t.translation.y],
                        );
                        self.wheels_to_seed.push((we, l));
                        l
                    };
                    // ⚠️ **O centro de uma roldana montada é DERIVADO**, mesmo em
                    // repouso: `corpo · local`. Ler o `Transform` dela aqui faria
                    // o eixo ficar onde o artista o largou enquanto o bloco anda —
                    // e é este número que o `sync_mounted_wheels` devolve ao
                    // `Transform` para o dot seguir o bloco. Em play o
                    // `refresh_mounts` reescreve o MESMO campo da pose VIVA: uma
                    // pergunta, uma fórmula, duas poses.
                    let centre = ph2d_physics::PhysicsWorld::world_from_local_at_pose(rest, local);
                    (b.handle, local, centre)
                });
            let name_id = world
                .get::<Name>(we)
                .map_or(0, |n| stable_name_id(n.as_str()));
            // **A talha de WESTON (W-Weston):** este eixo é atravessado DUAS vezes,
            // com o que houver no meio abraçado pelos dois contatos.
            //
            // ⚠️ **O marcador sozinho não faz um par** — sem segundo diâmetro não há
            // o que retornar por, e o `radius_out` de um eixo comum é `0`. Perguntar
            // as duas coisas aqui é o que impede uma roldana com o marcador e sem
            // diâmetro de emitir um contato de raio zero na cauda da rota.
            let weston = world.get::<crate::WestonAxle>(we).is_some() && wheel.radius_out > 0.0;
            self.rope_wheels.push(RopeWheelRow {
                rope: wheel.rope,
                key: (wheel.order, name_id),
                tail: false,
                entity: we,
                wheel: RopeWheel {
                    // Sem montagem o `Transform` da roldana É o centro — ela é um
                    // ponto pregado no cenário, e não há de onde derivar nada.
                    centre: mount.map_or([t.translation.x, t.translation.y], |(_, _, c)| c),
                    body: mount.map(|(h, _, _)| h),
                    local: mount.map_or([0.0, 0.0], |(_, l, _)| l),
                    radius: wheel.radius,
                    // W4: o segundo diâmetro do tambor diferencial. O `0 = roldana
                    // comum` do componente vira o `None` da geometria numa
                    // conversão só — as duas pontas dizem a mesma coisa em
                    // vocabulários próprios (um número que a row edita · uma
                    // ausência que a rota entende).
                    //
                    // ⚠️ **Numa WESTON o segundo diâmetro não é um `radius_out`, é o
                    // OUTRO CONTATO** (a row abaixo): os dois contatos de um par são
                    // cada um de raio único, e o que os torna um eixo composto é o
                    // `axle` compartilhado. Deixar o `radius_out` aqui seria a mesma
                    // máquina dita de duas formas, e a rota teria de escolher.
                    radius_out: (!weston && wheel.radius_out > 0.0).then_some(wheel.radius_out),
                    // Um par de Weston compartilha o EIXO — é ele que faz a rotação
                    // ser UMA, e é dessa unicidade que o peso `R/(R−r)` sai.
                    axle: if weston { name_id } else { 0 },
                    // A roldana é apontada pelo NOME dela, a mesma chave por que
                    // a corda aponta os corpos — bits de entidade mudam a cada
                    // undo, e o eixo partido migraria para a vizinha.
                    id: name_id,
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
            if weston {
                // **O contato de RETORNO**, pelo diâmetro pequeno e no MESMO centro:
                // duas circunferências concêntricas são o que um eixo composto É, e a
                // rota as aceita porque os dois contatos nunca são consecutivos —
                // entre eles está o que a corda abraça.
                let mut ret = self.rope_wheels[self.rope_wheels.len() - 1];
                ret.wheel.radius = wheel.radius_out;
                // ⚠️ **O retorno NÃO recolhe.** Um eixo tem UMA rotação, logo um
                // termo de recolhimento — o do contato de entrada, `ω·R`, que é
                // exatamente o que um sarilho diferencial paga do lado do esforço.
                // Somar o segundo contaria a mesma volta duas vezes.
                ret.reel_rate = 0.0;
                // A cauda da rota: é o último nó que a corda toca, porque é ali que
                // ela **termina enrolada**. O que vier depois é o ramo SOLTO, e a
                // rota lhe dá peso zero.
                ret.tail = true;
                self.rope_wheels.push(ret);
            }
        }
        self.wheel_query = Some(wq);
        // ⚠️ **A cauda entra na CHAVE, não num sentinela de `order`.** Pôr o retorno
        // em `u16::MAX` empataria com uma roldana que o artista autorasse ali, e o
        // desempate por nome escolheria a rota em silêncio; um campo diz o que ele é.
        self.rope_wheels
            .sort_unstable_by_key(|r| (r.rope, r.tail, r.key.0, r.key.1));
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
    // Sítio de PRODUTO: nomeia os seis. Com `..default()` o campo que a próxima
    // wave acrescentar nasceria neutro aqui em silêncio, e uma polia recém-criada
    // é exatamente onde isso não pode acontecer (o §0 do plano tem a foto).
    let wheel = |c: [f32; 2]| RopeWheel {
        centre: c,
        // Uma polia recém-montada nasce com as duas roldanas no CENÁRIO — a
        // cadernal móvel do W3 é um segundo gesto, não um default.
        body: None,
        local: [0.0, 0.0],
        radius: lift * WHEEL_RADIUS_FRACTION,
        // Uma polia recém-montada nasce com roldanas COMUNS: o tambor
        // diferencial do W4 é um segundo gesto, como a cadernal móvel do W3.
        radius_out: None,
        // …e com eixo PRÓPRIO: a talha de Weston do W-Weston é um terceiro gesto,
        // e ela nem é expressável sem o segundo diâmetro que a linha acima recusa.
        axle: 0,
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
            let count = r.desc.wheel_count as usize;
            for i in start..start + count {
                let (Some(w), Some(&e)) = (
                    self.world.pulley_wheels().get(i),
                    self.wheel_entities.get(i),
                ) else {
                    continue;
                };
                if w.radius <= 0.0 {
                    continue;
                }
                // ⚠️ **Um eixo, UMA rotação** (W-Weston): o contato de RETORNO de uma
                // Weston compartilha a entidade com o de entrada, então integrá-lo
                // também avançaria o mesmo ângulo duas vezes — e com o raio errado, o
                // que desenharia dois anéis concêntricos girando em velocidades
                // diferentes. Ele COPIA o ângulo que a entrada acabou de escrever (a
                // arena põe a cauda depois, então a ordem já é essa), e é isso que
                // mantém os dois anéis rígidos.
                let d = if rope_route::is_axle_return(
                    &self.world.pulley_wheels()[start..start + count],
                    i - start,
                ) {
                    0.0
                } else {
                    speed * f32::from(w.side) / w.radius * dt
                };
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

impl PhysicsBridge {
    /// **O centro de DESENHO de uma roldana montada** — o `Transform.translation`
    /// dela — levado para onde o eixo autorado agora está (`corpo_repouso ·
    /// local`).
    ///
    /// É isto que faz o dot de centro e a §2 Position **seguirem o bloco**: mover
    /// o corpo não muda o `local` guardado, então o centro derivado — e o dot
    /// desenhado ali — anda com ele. Irmão exato do `sync_joint_pivots`, e a
    /// prosa dele vale palavra por palavra.
    ///
    /// Rest-only (o chamador gateia em `!playing`): em play a ARENA carrega o
    /// centro vivo, refrescado por sub-passo, e é dela que o desenho lê.
    ///
    /// A escrita é condicional para não fabricar diff: o `post_frame_undo`
    /// registra por DIFF, e um `Transform` reescrito com o mesmo número todo
    /// frame seria um passo de undo por frame.
    pub(super) fn sync_mounted_wheels(&mut self, sim: &mut SimWorld) {
        if self.rope_wheels.is_empty() {
            return;
        }
        let mut scratch = std::mem::take(&mut self.wheels_to_seed);
        scratch.clear();
        for row in &self.rope_wheels {
            // A roldana de CENÁRIO não tem de onde derivar centro nenhum: o
            // `Transform` dela É o centro, e reescrevê-lo seria a segunda porta.
            if row.wheel.body.is_none() {
                continue;
            }
            scratch.push((row.entity, row.wheel.centre));
        }
        for &(e, centre) in scratch.iter() {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e)
                && (t.translation.x != centre[0] || t.translation.y != centre[1])
            {
                t.translation.x = centre[0];
                t.translation.y = centre[1];
            }
        }
        scratch.clear();
        self.wheels_to_seed = scratch;
    }
}

impl PhysicsBridge {
    /// **Que corda está sob este ponto do mundo?** — o alvo do eyedropper de corda
    /// da §13 (W-Pulley W1).
    ///
    /// Devolve a polia cuja ROTA passa mais perto de `p`, dentro de `tol` metros, e
    /// `None` quando nenhuma passa.
    ///
    /// ⚠️ **Isto NÃO pode ser o irmão do eyedropper da §12, e a diferença foi
    /// medida** (`measure_rope_pick`): o da §12 resolve o alvo com
    /// `pick_sprites_at_world`, que exige um **sprite** sob o cursor. Um corpo tem
    /// um; uma corda é uma **LINHA** e a entidade dela não tem nenhum — copiar
    /// aquele gesto daria `None` sobre a corda **para sempre**, um botão que arma e
    /// nunca acerta. A nota do plano que prometia *"o irmão exato"* está corrigida.
    ///
    /// ⚠️ **A geometria vem da MESMA `rope_route::route` que DESENHA a corda.** Um
    /// segundo caminho — a reta entre as âncoras, digamos — faria o artista clicar
    /// no traço que ele vê e acertar uma linha que só existe no código; e ninguém
    /// confere geometria numa screenshot. É a lei que o `physics_overlay_pulley` já
    /// afirma em voz alta sobre o desenho.
    ///
    /// Uma corda **degenerada** (rota que não resolve) é apontável pela RETA entre
    /// as âncoras, que é exatamente o que o overlay desenha nesse caso — as duas
    /// respostas continuam saindo da mesma pergunta (*"a rota resolve?"*), então o
    /// alvo é sempre o traço que está na tela.
    ///
    /// Medido: sobre a rota dá **0,00000 m**; afastar `d` pela normal dá `d` ao
    /// quinto decimal; e entre duas cordas paralelas a mais próxima ganha em TODA
    /// separação (a escolha nunca é ambígua, ela só fica fina).
    #[must_use]
    pub fn rope_at_world(&self, p: [f32; 2], tol: f32) -> Option<Entity> {
        let mut best: Option<(Entity, f32)> = None;
        let mut wheels: Vec<RopeWheel> = Vec::new();
        // ⚠️ **As DUAS metades da geometria vêm das portas do DESENHO** — as
        // âncoras de `joint_views` e as rodas de `rope_wheels`, o par exato que o
        // `physics_overlay_pulley` lê. Re-derivá-las aqui (das poses, dos
        // componentes) seria a segunda opinião que faz o clique acertar uma corda
        // que não é a desenhada.
        for v in self
            .joint_views()
            .filter(|v| v.kind == crate::JointKind::Pulley)
        {
            wheels.clear();
            wheels.extend(self.rope_wheels(v.entity).map(|(_, w)| w));
            let d = route_distance(v.anchor_a, v.anchor_b, &wheels, p);
            if d <= tol && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((v.entity, d));
            }
        }
        best.map(|(e, _)| e)
    }
}

/// A distância de `p` à polilinha que a corda DESENHA.
///
/// Rota que não resolve cai na reta âncora-a-âncora, o mesmo traço que o overlay
/// desenha para uma corda degenerada — uma pergunta, dois desenhos, o mesmo alvo.
fn route_distance(a: [f32; 2], b: [f32; 2], wheels: &[RopeWheel], p: [f32; 2]) -> f32 {
    let mut segs = Vec::new();
    let mut best = f32::INFINITY;
    let mut prev = a;
    if rope_route::route(a, b, wheels, &mut segs).is_some() {
        for t in &segs {
            best = best.min(point_to_segment(p, prev, t.from));
            best = best.min(point_to_segment(p, t.from, t.to));
            prev = t.to;
        }
    }
    best.min(point_to_segment(p, prev, b))
}

/// Distância de um ponto a um segmento — a aritmética que um hit-test de linha é.
///
/// ⚠️ Sem transcendental e sem `hypot` no caminho de decisão: o `hypot` é a libm da
/// plataforma, e embora este número **não** alcance o `physics_ecs_c9` (é uma
/// consulta de UI, não um passo de sim), manter a mesma disciplina custa nada e
/// evita a próxima chamada nascer num lugar onde ela alcança.
fn point_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ax, ay) = (b[0] - a[0], b[1] - a[1]);
    let len2 = ax * ax + ay * ay;
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (((p[0] - a[0]) * ax + (p[1] - a[1]) * ay) / len2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (p[0] - (a[0] + ax * t), p[1] - (a[1] + ay * t));
    (dx * dx + dy * dy).sqrt()
}
