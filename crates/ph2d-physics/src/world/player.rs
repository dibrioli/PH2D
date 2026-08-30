//! **Aplicar o motor de um player** ao corpo (W2 do player de plataforma).
//!
//! A LEI mora na `ph2d-platformer` (uma folha pura, sem rapier); a ponte
//! (`ph2d-physics-ecs`) orquestra; e este módulo é a única coisa que **toca o
//! corpo**. A divisão é a mesma de todo o módulo, e existe por um motivo
//! verificável: a ponte declara-se *rapier-free* no próprio `Cargo.toml` (ela
//! só carrega os handles re-exportados), então quem mexe num `RigidBody` tem de
//! ser este wrapper.
//!
//! # As duas grandezas chegam por caminhos diferentes, e é o desenho
//!
//! - **`accel`** (m/s²) vira **IMPULSO** (`a · m · dt`). Multiplicar pela massa
//!   é o que faz o personagem acelerar igual seja qual for a massa dele — e ao
//!   mesmo tempo produz a **força real** que a reação da 3ª lei (W6) vai
//!   devolver ao chão. Um controlador que escrevesse velocidade não teria esse
//!   número para devolver.
//! - **`boost`** (m/s) é somado à velocidade **direto**. É o termo exato: matar
//!   a oscilação da mola, parar no lugar, herdar a velocidade de uma
//!   plataforma.
//!
//! ⚠️ **Impulso, nunca `add_force`.** A força do rapier é CONSTANTE até um
//! `reset_forces` que este pipeline nunca chama — o `world::drag` já pagou esse
//! bug (acumulou ~720× e os terminais saíram não-monotônicos). O primitivo certo
//! é `F·dt`, e é o que todo o resto deste módulo usa.
//!
//! ⚠️ **E o corpo é ACORDADO.** Um corpo adormecido não é integrado, e um player
//! parado adormece — sem isto ele acordaria só quando alguém encostasse nele, o
//! que é a assinatura exata de *"os controles pararam de funcionar"* (a mesma
//! lição, medida, do `move_grab`).

use crate::rmath::Vector;
use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};

use super::PhysicsWorld;

/// **Pagar as acelerações de player enfileiradas**, uma fatia por sub-passo.
///
/// `sub_dt` é o passo do INTEGRADOR ([`PhysicsWorld::substep_dt`]), não o do
/// tique: chamado uma vez por sub-passo, o impulso total ao fim do tique é
/// exatamente `a·m·dt` — o mesmo de antes desta wave. **O que muda é onde ele
/// cai dentro do tique**, e é só isso que o defeito era.
pub(super) fn apply_queued(
    bodies: &mut RigidBodySet,
    queued: &[(RigidBodyHandle, [f32; 2])],
    sub_dt: f32,
) {
    for &(handle, accel) in queued {
        let Some(body) = bodies.get_mut(handle) else {
            continue;
        };
        let mass = body.mass();
        if !mass.is_finite() || mass <= 0.0 {
            continue;
        }
        body.apply_impulse(
            Vector::new(accel[0] * mass * sub_dt, accel[1] * mass * sub_dt),
            true,
        );
    }
}

impl PhysicsWorld {
    /// A gravidade do mundo, em m/s².
    ///
    /// A porta simétrica do [`PhysicsWorld::set_gravity`]: a lei do player
    /// precisa dela para **compensá-la** enquanto a mola segura o personagem, e
    /// lê-la daqui é o que impede uma segunda cópia do número de existir.
    #[must_use]
    pub fn gravity(&self) -> [f32; 2] {
        [self.gravity.x, self.gravity.y]
    }

    /// **Deslocar um corpo sem tocar na velocidade dele** (W10) — a correção de
    /// quina, e o único lugar deste módulo que escreve POSIÇÃO.
    ///
    /// ⚠️ **Não é `set_body_pose`**, e a diferença é o bug inteiro: aquela porta
    /// **zera a velocidade** (ela existe para assentar um corpo na pose autorada,
    /// onde zerar é o certo). Usá-la aqui pararia o pulo no ar no exato tique em
    /// que a assistência tenta salvá-lo — a correção mataria o que ela corrige.
    ///
    /// ⚠️ E **acorda o corpo**, pela mesma razão que o motor: um corpo adormecido
    /// não é integrado, e mover a translação dele sem acordar deixa o BVH e o
    /// desenho a discordar até alguém encostar nele.
    ///
    /// No-op para deslocamento nulo ou não-finito — recusar cedo é mais barato
    /// que descobrir depois, e um `NaN` numa translação envenena a pose do corpo
    /// e o `physics_ecs_c9` junto.
    pub fn nudge_body(&mut self, handle: RigidBodyHandle, delta: [f32; 2]) {
        if !delta[0].is_finite() || !delta[1].is_finite() {
            return;
        }
        if delta[0] == 0.0 && delta[1] == 0.0 {
            return;
        }
        let Some(rb) = self.bodies.get_mut(handle) else {
            return;
        };
        // ⚠️ LUGAR + DESLOCAMENTO = lugar. Os dois lados são hoje o MESMO tipo
        // (`Vector`), então nada impede escrever `delta + delta` ou somar duas
        // translações — leia o que a expressão significa, não o que ela aceita.
        // (`translation()` também deixou de devolver referência: é por valor.)
        let next = rb.translation() + Vector::new(delta[0], delta[1]);
        rb.set_translation(next, true);
    }

    /// A velocidade linear de um corpo, em mundo. `None` se o handle morreu.
    #[must_use]
    pub fn body_velocity(&self, handle: RigidBodyHandle) -> Option<[f32; 2]> {
        let v = self.bodies.get(handle)?.linvel();
        Some([v.x, v.y])
    }

    /// **A velocidade de um ponto de um corpo** — a translação DELE mais o que a
    /// rotação acrescenta naquele ponto (`v + ω × r`).
    ///
    /// ⚠️ É esta, e não a do centro, que o sensor de chão precisa: um personagem
    /// sobre a ponta de uma plataforma que GIRA está sendo levado por um ponto
    /// que se move, mesmo que o centro dela esteja parado. Ler a do centro faria
    /// a mola lutar contra a rotação da plataforma.
    #[must_use]
    pub fn point_velocity(&self, handle: RigidBodyHandle, point: [f32; 2]) -> Option<[f32; 2]> {
        let b = self.bodies.get(handle)?;
        // ⚠️ `point` é um LUGAR em mundo, e o `velocity_at_point` o toma por
        // VALOR agora (era `&Point2`). O tipo já não distingue lugar de
        // deslocamento: passar aqui uma velocidade ou um `r` relativo compila e
        // devolve um número plausível — a conta que o rapier faz é
        // `v + ω × (point − centro_de_massa)`, logo o argumento tem de ser
        // ABSOLUTO.
        let v = b.velocity_at_point(Vector::new(point[0], point[1]));
        Some([v.x, v.y])
    }

    /// **Aplicar o motor** ao corpo: a aceleração como impulso, o boost como
    /// escrita de velocidade.
    ///
    /// No-op silencioso para handle morto — o chamador é um laço por-entidade e
    /// não deve ter de perguntar antes.
    pub fn apply_player_motor(
        &mut self,
        handle: RigidBodyHandle,
        accel: [f32; 2],
        boost: [f32; 2],
    ) {
        let dt = self.dt();
        let Some(body) = self.bodies.get_mut(handle) else {
            return;
        };
        // Ver o aviso do módulo: um player parado adormece, e um corpo dormindo
        // não é integrado.
        body.wake_up(true);
        let mass = body.mass();
        if accel != [0.0, 0.0] && mass.is_finite() && mass > 0.0 {
            body.apply_impulse(
                Vector::new(accel[0] * mass * dt, accel[1] * mass * dt),
                true,
            );
        }
        if boost != [0.0, 0.0] {
            // `linvel()` devolve por VALOR agora — o `*` que estava aqui era o
            // deref do `&Vector2` do nalgebra.
            let v = body.linvel();
            body.set_linvel(Vector::new(v.x + boost[0], v.y + boost[1]), true);
        }
    }

    /// **Enfileirar o cancelamento da gravidade da perna** (W11), para ser pago
    /// **por SUB-PASSO** dentro do [`PhysicsWorld::step`].
    ///
    /// ⚠️ **É a MESMA aceleração de sempre, entregue noutro instante** — o
    /// impulso total ao fim do tique é `a·m·dt`, byte a byte o de antes. O que
    /// muda é que ela deixa de ser um bloco no topo do tique e passa a ser
    /// integrada como a gravidade que ela cancela; o porquê, com o número, está
    /// em [`ph2d_platformer::PlayerStep::gravity_hold`].
    ///
    /// ⚠️ **Não confundir com o [`Self::apply_player_motor`]:** aquele é o
    /// caminho de tudo o que NÃO cancela gravidade (a mola, a caminhada, o
    /// pulo), e ele fica agrupado de propósito — é o ordenamento
    /// semi-implícito, e foi ele que manteve a mola estável quando a wave mediu
    /// o piso do amortecimento (`measure_substep.rs`: fatiar a mola também faz
    /// um `spring_damping = 0` LARGAR o personagem).
    pub fn queue_player_hold(&mut self, handle: RigidBodyHandle, accel: [f32; 2]) {
        if accel != [0.0, 0.0] {
            self.player_accels.push((handle, accel));
        }
    }

    /// Quantos cancelamentos de gravidade este tique tem enfileirados — o
    /// oráculo do gate de que a fila é DRENADA (uma fila que crescesse por tique
    /// seria um vazamento com todos os outros números certos).
    #[must_use]
    pub fn queued_player_holds(&self) -> usize {
        self.player_accels.len()
    }

    /// **O EMPURRÃO LATERAL do player** (§8.2) — a metade horizontal da 3ª lei,
    /// e ela é uma porta PRÓPRIA porque a pergunta é outra.
    ///
    /// # ⚠️ `apply_impulse`, no CENTRO — o oposto da irmã vertical
    ///
    /// A [`Self::apply_player_reaction`] empurra **num ponto** de propósito: é o
    /// `r × F` dela que faz a jangada INCLINAR sob o personagem (cena `=103`,
    /// aprovada). Aqui o mesmo ponto produzia um **rodopio**: o bloqueio inteiro
    /// do tique entra como uma martelada num ponto ALTO do caixote, e medido
    /// (`measure_push_spin`) um caixote pequeno dava **12 voltas em 3 s**
    /// (74,29 rad) contra 0,3175 do corpo dinâmico — o giro seguindo a ALAVANCA
    /// e a desaparecer quando o contato desce ao centro de massa.
    ///
    /// ⚠️ **E o rodopio não era só feio, ele REALIMENTAVA:** um caixote a girar
    /// apresenta faces diferentes ao sweep, é bloqueado mais, recebe mais
    /// empurrão e gira mais — o caixote pequeno viajava **21,36 m** contra os
    /// 7,13 do dinâmico. Com o impulso central ele viaja **7,26**, ou seja o
    /// empurrão LINEAR passou a bater com o dinâmico em todo tamanho, e não só
    /// no caso de rotação travada em que a `W-KinPush` o calibrou.
    ///
    /// ⛔ **Espalhar o impulso pelos sub-passos foi CONSTRUÍDO, MEDIDO e
    /// REVERTIDO — não refaça.** A hipótese era que o braço de alavanca encolhe
    /// enquanto o caixote tomba, então `N` fatias somariam menos torque que uma
    /// martelada. Em 16 ms o caixote quase não gira: medido, **77,51 rad** (pior
    /// que os 74,29 de origem). O que a tentativa deixou de bom foi esta porta —
    /// separar as duas metades é o que torna o impulso central barato, e era
    /// justamente a objeção que encarecia esta escolha.
    ///
    /// ⚠️ **O trade fica NOMEADO:** um caixote empurrado no peito **nunca
    /// tomba**, enquanto o dinâmico tomba um pouco (0,3175 rad no menor). É
    /// escolha de produto, não descuido.
    ///
    /// A massa é lida do PLAYER e o impulso vai no CHÃO, como na irmã; um corpo
    /// morto de qualquer lado é no-op silencioso.
    pub fn apply_player_push(
        &mut self,
        ground: RigidBodyHandle,
        player: RigidBodyHandle,
        boost: [f32; 2],
    ) {
        let Some(mass) = self
            .bodies
            .get(player)
            .map(rapier2d::dynamics::RigidBody::mass)
        else {
            return;
        };
        if !mass.is_finite() || mass <= 0.0 {
            return;
        }
        let Some(body) = self.bodies.get_mut(ground) else {
            return;
        };
        let (jx, jy) = (boost[0] * mass, boost[1] * mass);
        if jx == 0.0 && jy == 0.0 {
            return;
        }
        // O `true` é o despertar, pela mesma razão da irmã vertical.
        body.apply_impulse(Vector::new(jx, jy), true);
    }

    /// **A REAÇÃO da 3ª lei** (W6): devolver ao CHÃO o que a perna do player lhe
    /// tirou, **no ponto onde o raio o encontrou**.
    ///
    /// `accel`/`boost` chegam nas unidades do player (o que a lei devolveu,
    /// negado e escalado) e são convertidos com a massa **DELE** — é o `m·a` do
    /// lado de quem empurra. O que o chão faz com isso é do solver: uma jangada
    /// leve afunda, um continente não se move, e nenhum dos dois precisa de
    /// código.
    ///
    /// ⚠️ **`apply_impulse_at_point`, nunca `apply_impulse`** — é o ponto que
    /// produz o TORQUE (`r × F`), e o torque é a jangada **INCLINAR** quando o
    /// personagem anda para a borda. Aplicado no centro de massa, ela afundaria
    /// paralela a si mesma e a gangorra seria um elevador.
    ///
    /// ⚠️ **A massa é lida do PLAYER e o impulso vai no CHÃO**, então os dois
    /// handles são pedidos e nenhum dos dois pode ser inferido do outro. Um
    /// corpo morto de qualquer lado é no-op silencioso: o chamador é um laço
    /// por-entidade.
    ///
    /// ⚠️ **Estático e kinematic absorvem sem se mexer**, e é correto — massa
    /// infinita é o que "o chão do mundo" significa. Não há caso especial aqui.
    pub fn apply_player_reaction(
        &mut self,
        ground: RigidBodyHandle,
        player: RigidBodyHandle,
        accel: [f32; 2],
        boost: [f32; 2],
        at: [f32; 2],
    ) {
        let dt = self.dt();
        let Some(mass) = self
            .bodies
            .get(player)
            .map(rapier2d::dynamics::RigidBody::mass)
        else {
            return;
        };
        if !mass.is_finite() || mass <= 0.0 {
            return;
        }
        let Some(body) = self.bodies.get_mut(ground) else {
            return;
        };
        // A soma dos dois canais: a aceleração vira `a·m·dt` (o mesmo caminho do
        // motor) e o boost já É uma mudança de velocidade, logo `Δv·m`.
        let jx = accel[0] * mass * dt + boost[0] * mass;
        let jy = accel[1] * mass * dt + boost[1] * mass;
        if jx == 0.0 && jy == 0.0 {
            return;
        }
        // ⚠️ **O `true` final É o despertar**, e ele é load-bearing: uma jangada
        // parada DORME, um corpo dormindo não é integrado, e sem isto ela só
        // afundaria quando alguma outra coisa esbarrasse nela — a assinatura de
        // *"a física parou de funcionar"* que o `move_grab` já mediu.
        //
        // ⚠️ Eu tinha escrito um `body.wake_up(true)` explícito ao lado, e ele
        // era **redundante**: medido, a mutação que o remove não move um
        // milímetro, e só remover o `true` daqui é que deixa a jangada dormindo.
        // Duas portas para *"acorde"* é uma a mais.
        //
        // ⛔ **E os dois primeiros argumentos são hoje o MESMO tipo, lado a
        // lado:** o 1º é um IMPULSO (um deslocamento de quantidade de
        // movimento), o 2º é o LUGAR onde ele entra. No nalgebra um `Point2` na
        // 1ª posição não compilava; hoje trocá-los passa em silêncio e o torque
        // sai da coordenada de mundo do contato — quanto mais longe da origem a
        // jangada estiver, maior o empurrão fantasma. A ordem é
        // `(impulso, ponto, acordar)`.
        body.apply_impulse_at_point(Vector::new(jx, jy), Vector::new(at[0], at[1]), true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A aceleração é resistida pela MASSA na hora de virar impulso, então o
    /// resultado em velocidade é o mesmo para qualquer massa — que é o que um
    /// controlador quer, e é o que dá o número certo para a reação da W6.
    #[test]
    fn the_same_accel_moves_any_mass_the_same() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (light, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 1.0);
        let (heavy, _) = w.add_dynamic_circle(5.0, 0.0, 0.5, 50.0);

        w.apply_player_motor(light, [10.0, 0.0], [0.0, 0.0]);
        w.apply_player_motor(heavy, [10.0, 0.0], [0.0, 0.0]);

        let vl = w.body_velocity(light).unwrap();
        let vh = w.body_velocity(heavy).unwrap();
        assert!(
            (vl[0] - vh[0]).abs() < 1.0e-5,
            "a mesma aceleracao tem de dar a mesma velocidade: leve {} pesado {}",
            vl[0],
            vh[0]
        );
        assert!(vl[0] > 0.0, "e ela tem de mover: {}", vl[0]);
    }

    /// O boost é escrita DIRETA — soma na velocidade, sem passar pela massa.
    #[test]
    fn the_boost_is_written_straight_onto_the_velocity() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 7.0);
        w.apply_player_motor(b, [0.0, 0.0], [3.0, -2.0]);
        let v = w.body_velocity(b).unwrap();
        assert!(
            (v[0] - 3.0).abs() < 1.0e-6 && (v[1] + 2.0).abs() < 1.0e-6,
            "{v:?}"
        );
    }

    /// ⚠️ **O canal fatiado entrega o MESMO impulso total que o agrupado** — é
    /// esta linha que faz da W11 uma reescrita de integração e não uma mudança
    /// de lei.
    ///
    /// Sem gravidade e sem contato, a velocidade ao fim de um tique é o impulso
    /// total dividido pela massa, e ela não sabe por quantas fatias ele veio. O
    /// que a fatia muda é o DESLOCAMENTO — a metade que este gate NÃO afirma, e
    /// que o `measure_substep.rs` mede no produto.
    #[test]
    fn the_sliced_channel_delivers_the_same_total_impulse() {
        let mut lumped = PhysicsWorld::new();
        lumped.set_gravity(0.0, 0.0);
        let (a, _) = lumped.add_dynamic_circle(0.0, 0.0, 0.5, 3.0);
        lumped.apply_player_motor(a, [0.0, 9.81], [0.0, 0.0]);
        lumped.step();

        let mut sliced = PhysicsWorld::new();
        sliced.set_gravity(0.0, 0.0);
        let (b, _) = sliced.add_dynamic_circle(0.0, 0.0, 0.5, 3.0);
        sliced.queue_player_hold(b, [0.0, 9.81]);
        sliced.step();

        let va = lumped.body_velocity(a).unwrap();
        let vb = sliced.body_velocity(b).unwrap();
        assert!(
            (va[1] - vb[1]).abs() < 1.0e-5,
            "o impulso total tem de ser o mesmo: agrupado {} fatiado {}",
            va[1],
            vb[1]
        );
        assert!(va[1] > 0.0, "e ele tem de mover: {}", va[1]);
    }

    /// ⚠️ **A fila é DRENADA**, e um vazamento aqui seria invisível em tudo o
    /// resto: ela cresceria um item por tique por player e o personagem passaria
    /// a ser segurado N vezes.
    #[test]
    fn the_hold_queue_is_spent_by_the_tick_that_reads_it() {
        let mut w = PhysicsWorld::new();
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 1.0);
        w.queue_player_hold(b, [0.0, 9.81]);
        assert_eq!(w.queued_player_holds(), 1, "a fixture precisa de um item");
        w.step();
        assert_eq!(w.queued_player_holds(), 0, "o tique paga e limpa");
        // E dois tiques sem enfileirar nada não deixam resíduo.
        w.step();
        assert_eq!(w.queued_player_holds(), 0);
    }

    /// ⚠️ **Zero não entra na fila** — um item nulo custaria uma volta do laço
    /// de sub-passos por tique e por player, para não fazer nada.
    #[test]
    fn a_zero_hold_is_not_queued_at_all() {
        let mut w = PhysicsWorld::new();
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 1.0);
        w.queue_player_hold(b, [0.0, 0.0]);
        assert_eq!(w.queued_player_holds(), 0);
    }

    /// ⚠️ Um corpo ADORMECIDO volta a ser integrado — sem isto um player parado
    /// para de responder aos controles.
    #[test]
    fn the_motor_wakes_a_sleeping_body() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 1.0);
        for _ in 0..240 {
            w.step();
        }
        assert!(
            w.bodies().get(b).unwrap().is_sleeping(),
            "a fixture precisa de um corpo DORMINDO para o gate significar algo"
        );
        w.apply_player_motor(b, [10.0, 0.0], [0.0, 0.0]);
        assert!(!w.bodies().get(b).unwrap().is_sleeping());
    }

    /// A velocidade de um PONTO inclui o que a rotação acrescenta ali.
    #[test]
    fn a_spinning_platform_carries_its_edge() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 1.0, 1.0);
        w.bodies_mut().get_mut(b).unwrap().set_angvel(2.0, true);

        let centre = w.point_velocity(b, [0.0, 0.0]).unwrap();
        let edge = w.point_velocity(b, [1.0, 0.0]).unwrap();
        assert!(centre[1].abs() < 1.0e-6, "o centro nao anda: {centre:?}");
        assert!(
            (edge[1] - 2.0).abs() < 1.0e-4,
            "a borda a 1 m com omega=2 anda a 2 m/s: {edge:?}"
        );
    }
}
