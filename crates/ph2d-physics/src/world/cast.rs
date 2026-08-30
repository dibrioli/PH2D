//! **Consultas de cena** (W1 do player de plataforma) — perguntar ao mundo *"o
//! que há naquela direção?"* sem simular nada.
//!
//! Até aqui este wrapper não tinha nenhuma: a explosão acha corpos **iterando o
//! body set** e medindo distância (`world::blast`), o empuxo recorta polígonos
//! que ele já tem em mãos, e o resto lê o narrow phase (que só sabe de quem já
//! está encostado). Nada disso responde *"a que distância está o chão?"*, que é
//! a pergunta de que uma **cápsula flutuante** vive — a perna dela é um raio.
//!
//! # ⚠️ Não há pipeline a manter, e isso é da versão
//!
//! A leitura inicial supunha um `QueryPipeline` persistente com um `update()`
//! que o chamador teria de lembrar de rodar — o modo de falha clássico (um cast
//! contra um pipeline velho responde sobre um mundo que não existe mais). **No
//! rapier 0.28 isso não existe:** a query é *derivada* do BVH do broad phase por
//! empréstimo ([`BroadPhaseBvh::as_query_pipeline`]), e o broad phase já é campo
//! desta struct, atualizado pelo `step`.
//!
//! O que sobra é uma pergunta **diferente** e mais estreita, medida no gate
//! [`tests::the_bvh_answers_for_the_world_the_last_step_left`]: o BVH descreve o
//! mundo que o **último `step`** deixou. Um corpo recém-spawnado, antes de
//! qualquer passo, **não está nele**. Isso não é bug — é o contrato —, e o
//! consumidor (a ponte do player) casta **dentro** do laço de ticks, depois de
//! pelo menos um `step`, onde a pergunta é sempre sobre um mundo já indexado.
//!
//! # As duas metades do filtro, e por que nenhuma é opcional
//!
//! - **O próprio corpo sai** (`exclude_rigid_body`). Sem isso um personagem
//!   encontra a si mesmo a distância zero e conclui que está no chão — sempre.
//! - **As CAMADAS valem** (`world::groups_for`, a mesma porta que o solver usa).
//!   Um player não pisa no que ele atravessa; se o cast ignorasse a matriz, o
//!   sensor de chão e o solver discordariam sobre o que é sólido, e essa é
//!   exatamente a forma de defeito que este módulo inteiro existe para não ter.

use crate::rmath::Vector;
use rapier2d::geometry::{ColliderHandle, Ray};
use rapier2d::parry::query::DefaultQueryDispatcher;
use rapier2d::pipeline::{QueryFilter, QueryFilterFlags, QueryPipeline};

use super::{PhysicsWorld, groups_for};
use rapier2d::dynamics::RigidBodyHandle;

/// **O que um cast encontrou.** Tudo em MUNDO (metros, a fronteira 1:1 do
/// ADR-0131 D4).
///
/// Os quatro campos existem porque a mola de flutuação precisa dos quatro, e
/// cada um por um motivo distinto: a `distance` é o erro que a mola corrige, o
/// `body` é de quem se lê a velocidade do chão (e em quem a reação da 3ª lei
/// será aplicada), o `point` é **onde** ela é aplicada (é o que produz torque, e
/// é o que faz uma jangada INCLINAR), e a `normal` é o que decide se aquilo é
/// chão ou parede.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CastHit {
    /// A forma atingida.
    pub collider: ColliderHandle,
    /// O corpo dono dela — `None` para um collider sem corpo (o rapier permite).
    pub body: Option<RigidBodyHandle>,
    /// Distância da origem ao ponto de impacto, ao longo da direção.
    ///
    /// ⚠️ **Zero é um valor legítimo e significa penetração** (a origem estava
    /// dentro da forma). O cast é `solid`, então esse caso é REPORTADO em vez de
    /// atravessado — um personagem afundado no chão precisa saber que está.
    pub distance: f32,
    /// O ponto de impacto, em mundo.
    pub point: [f32; 2],
    /// A normal da superfície no impacto.
    ///
    /// ⚠️ **Pode ser o vetor NULO quando `distance == 0`** (origem dentro da
    /// forma): não há superfície onde medir. Quem decide "isto é chão?" pela
    /// normal tem de tratar esse caso, e é por isso que ele está escrito aqui e
    /// não descoberto depois.
    pub normal: [f32; 2],
}

impl PhysicsWorld {
    /// **Lançar um raio** de `origin` na direção `dir`, até `max_dist`.
    ///
    /// `exclude_body` sai da consulta (o chamador típico é um corpo perguntando
    /// sobre o mundo à volta DELE); `layer` é a camada de quem pergunta, e a
    /// matriz do mundo decide o que ela enxerga.
    ///
    /// Devolve `None` quando não há nada no alcance — e também para entrada
    /// degenerada (direção nula ou `NaN`, `max_dist` negativo ou `NaN`).
    ///
    /// ⚠️ **O guard de degeneração é uma defesa em CAMADA, e a camada de fora
    /// não é observável — está MEDIDO, não suposto.** Removendo-o, as quatro
    /// entradas degeneradas continuam devolvendo `None`: o parry já as rejeita
    /// (a direção vira `NaN` e nada intersecta). Ele fica por três motivos, e
    /// nenhum deles é "senão daria NaN":
    ///
    /// 1. **recusar cedo é mais barato** que descer no BVH para descobrir;
    /// 2. a intenção fica **escrita** (`world::blast` recusa um corpo no olho da
    ///    explosão pela mesma razão, e lá a recusa É observável);
    /// 3. ele não depende de um detalhe do parry que pode mudar.
    ///
    /// A mutação que o remove **não sangra** nenhum gate, e isso está registrado
    /// aqui em vez de descoberto por quem tentar "limpar" o código depois
    /// ([[feedback_layered_defenses_need_per_layer_gates]]).
    ///
    /// ⚠️ `dir` **não** precisa vir normalizado — esta porta o normaliza, e é
    /// deliberado: o `max_toi` do rapier é medido em múltiplos da norma da
    /// direção, então um chamador que passasse `dir` não-unitário obteria um
    /// alcance diferente do que pediu, em silêncio.
    #[must_use]
    pub fn cast_ray(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        exclude_body: Option<RigidBodyHandle>,
        layer: u8,
    ) -> Option<CastHit> {
        self.cast_ray_skipping(origin, dir, max_dist, exclude_body, None, layer)
    }

    /// **O mesmo raio, ignorando UMA forma** (W12) — a rota que o sensor de chão
    /// usa enquanto o personagem atravessa uma plataforma jump-through.
    ///
    /// # ⚠️ Por que o sensor precisa disto, e não só o solver
    ///
    /// A descida é resolvida no `oneway`, que limpa os contatos — mas quem
    /// segura o personagem no ar não é o solver, é a **MOLA**, e ela age porque
    /// o raio achou chão. Sem esta porta, o solver deixaria passar e a perna
    /// seguraria em cima: o personagem ficaria pairando sobre exatamente aquilo
    /// que pediu para atravessar, e nada na tela diria por quê.
    ///
    /// ⚠️ **Uma porta, duas faces:** [`Self::cast_ray`] delega para cá com
    /// `None`, então não existe uma segunda implementação de *"lançar um raio"*
    /// para divergir desta — o precedente é o `denoise_ml` do módulo de áudio,
    /// que delega com callback vazio pelo mesmo motivo.
    #[must_use]
    pub fn cast_ray_skipping(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        exclude_body: Option<RigidBodyHandle>,
        exclude_collider: Option<ColliderHandle>,
        layer: u8,
    ) -> Option<CastHit> {
        let d = Vector::new(dir[0], dir[1]);
        let n = d.length();
        if !n.is_finite() || n <= f32::EPSILON || !max_dist.is_finite() || max_dist < 0.0 {
            return None;
        }
        let unit = d / n;
        // ⚠️ `Ray::new` recebe a ORIGEM (um lugar) e a DIREÇÃO (um deslocamento) —
        // eram `Point2` e `Vector2`, hoje são o MESMO tipo. Ver `crate::rmath`.
        let ray = Ray::new(Vector::new(origin[0], origin[1]), unit);

        let dispatcher = DefaultQueryDispatcher;
        let filter = QueryFilter {
            groups: Some(groups_for(layer as usize, self.layer_matrix)),
            exclude_rigid_body: exclude_body,
            exclude_collider,
            // ⚠️ **UM SENSOR NÃO É MATÉRIA, e esta porta pergunta por matéria.**
            //
            // O default do rapier é `QueryFilterFlags::empty()`, que o próprio
            // código dele documenta como *"no filter"* — ou seja, um SENSOR
            // responde ao raio como pedra. O preço estava medido e era grande:
            // um personagem sobre uma poça de empuxo assentava **de pé sobre a
            // superfície**, à `float_height` exacta acima dela, e imune à
            // correnteza (`x = 0,0000` em cinco segundos), porque a caminhada
            // freia a velocidade de quem está no chão. O mesmo raio serve o
            // teto, o headroom e a parede, então ele também batia a cabeça num
            // volume de gatilho (**0,17 m** de desvio lateral medido).
            //
            // O `buoyancy.rs` já escreve a frase do outro lado — *"um sensor não
            // desloca fluido: um sensor é um marcador, não matéria"*. Esta é a
            // mesma frase deste lado.
            //
            // ⚠️ **Por que na PORTA e não num parâmetro:** o `cast_ray` tem
            // exactamente cinco consumidores no repo inteiro — o sensor de chão,
            // os dois de teto, o de headroom e o de parede —, e os cinco querem
            // matéria. Um parâmetro seria uma escolha oferecida a ninguém, e o
            // dia em que alguém quiser detectar um sensor a resposta já existe e
            // é outra: o canal de TRIGGERS (W7), que é o grafo de interseção do
            // solver e não um raio.
            flags: QueryFilterFlags::EXCLUDE_SENSORS,
            ..QueryFilter::default()
        };
        let pipeline: QueryPipeline<'_> =
            self.broad_phase
                .as_query_pipeline(&dispatcher, &self.bodies, &self.colliders, filter);

        // `solid = true`: um raio que NASCE dentro de uma forma reporta impacto
        // em 0 em vez de sair pelo outro lado. Ver o aviso em `CastHit::distance`.
        let (collider, hit) = pipeline.cast_ray_and_get_normal(&ray, max_dist, true)?;
        let point = ray.point_at(hit.time_of_impact);
        Some(CastHit {
            collider,
            body: self.colliders.get(collider).and_then(|c| c.parent()),
            distance: hit.time_of_impact,
            point: [point.x, point.y],
            normal: [hit.normal.x, hit.normal.y],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O chão está onde o raio diz que está — e o corpo devolvido é o dono da
    /// forma, que é de quem a mola vai ler a velocidade.
    #[test]
    fn a_ray_finds_the_floor_below() {
        let mut w = PhysicsWorld::new();
        let (floor, _) = w.add_static_cuboid(0.0, 0.0, 10.0, 0.5); // topo em y = 0.5
        w.step();

        let hit = w
            .cast_ray([0.0, 3.0], [0.0, -1.0], 10.0, None, 0)
            .expect("o raio tem de achar o chao");
        assert_eq!(hit.body, Some(floor));
        assert!(
            (hit.distance - 2.5).abs() < 1.0e-3,
            "distancia ao topo do chao: {}",
            hit.distance
        );
        assert!(
            hit.normal[1] > 0.9,
            "a normal aponta para CIMA: {:?}",
            hit.normal
        );
    }

    /// ⚠️ **A metade sem a qual um personagem se acha no chão para sempre.**
    #[test]
    fn the_caster_can_exclude_itself() {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 10.0, 0.5);
        let (me, _) = w.add_dynamic_circle(0.0, 3.0, 0.5, 1.0);
        w.step();

        // Sem excluir, o raio que nasce no meu centro me acha primeiro.
        let self_hit = w.cast_ray([0.0, 3.0], [0.0, -1.0], 10.0, None, 0);
        assert_eq!(
            self_hit.map(|h| h.body),
            Some(Some(me)),
            "sem exclusao o caster acha a si mesmo"
        );

        // Excluindo, ele acha o chão.
        let hit = w
            .cast_ray([0.0, 3.0], [0.0, -1.0], 10.0, Some(me), 0)
            .expect("com exclusao tem de achar o chao");
        assert_ne!(hit.body, Some(me));
    }

    /// ⚠️ **As CAMADAS valem no cast** — a metade que impede o sensor de chão e o
    /// solver de discordarem sobre o que é sólido.
    ///
    /// Sem ela um personagem "pisa" numa plataforma que ele atravessa: a mola o
    /// segura no ar sobre algo com que ele não colide, e nada na tela explica
    /// por quê.
    #[test]
    fn a_layer_the_caster_does_not_collide_with_is_invisible_to_it() {
        use super::super::desc::BodyDesc;
        use super::super::layers::LayerMatrix;
        use super::super::shape::ShapeDesc;
        use rapier2d::dynamics::RigidBodyType;

        let wall = |layer: u8| BodyDesc {
            body_type: RigidBodyType::Fixed,
            x: 2.0,
            y: 0.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 10.0,
            },
            restitution: 0.0,
            friction: 0.5,
            layer,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        };

        // Controle: mesma parede, MESMA camada — o cast a vê.
        let mut same = PhysicsWorld::new();
        same.spawn_body(wall(0));
        same.step();
        assert!(
            same.cast_ray([0.0, 0.0], [1.0, 0.0], 10.0, None, 0)
                .is_some(),
            "controle: na mesma camada a parede e' visivel"
        );

        // Experimento: a parede na camada 1, e a matriz diz que 0 e 1 NÃO
        // colidem. Quem pergunta pela camada 0 não pode enxergá-la.
        let mut split = PhysicsWorld::new();
        let mut m = LayerMatrix::default();
        m.set(0, 1, false);
        split.set_layer_matrix(m);
        split.spawn_body(wall(1));
        split.step();
        assert!(
            split
                .cast_ray([0.0, 0.0], [1.0, 0.0], 10.0, None, 0)
                .is_none(),
            "uma camada que nao colide comigo e' invisivel ao meu cast"
        );
    }

    /// Fora do alcance é `None` — o que a mola lê como *"estou no ar"*.
    #[test]
    fn nothing_within_reach_is_none() {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 10.0, 0.5);
        w.step();
        assert!(w.cast_ray([0.0, 3.0], [0.0, -1.0], 1.0, None, 0).is_none());
    }

    /// **Entrada degenerada devolve `None`** — pelas duas camadas, e o gate
    /// afirma só isso.
    ///
    /// ⚠️ Ele **não prova o guard** da porta: medido, o parry sozinho já
    /// devolve `None` para as quatro entradas abaixo, então a mutação que
    /// remove o guard passa aqui. O gate honesto é sobre o CONTRATO da porta
    /// (*"o que sai de entrada degenerada?"*), que é o que os chamadores leem;
    /// o porquê do guard existir mesmo assim está no doc de `cast_ray`.
    #[test]
    fn degenerate_input_yields_nothing() {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 10.0, 0.5);
        w.step();
        assert!(w.cast_ray([0.0, 3.0], [0.0, 0.0], 10.0, None, 0).is_none());
        assert!(
            w.cast_ray([0.0, 3.0], [f32::NAN, -1.0], 10.0, None, 0)
                .is_none()
        );
        assert!(
            w.cast_ray([0.0, 3.0], [0.0, -1.0], f32::NAN, None, 0)
                .is_none()
        );
        assert!(w.cast_ray([0.0, 3.0], [0.0, -1.0], -5.0, None, 0).is_none());
    }

    /// A direção não precisa ser unitária: o alcance é em METROS, sempre.
    ///
    /// ⚠️ Sem a normalização desta porta, `dir = [0, -2]` com `max_dist = 2`
    /// alcançaria 4 m — o chamador pediria 2 e receberia o dobro, em silêncio.
    #[test]
    fn the_reach_is_in_metres_whatever_the_direction_length() {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 10.0, 0.5); // topo em 0.5
        w.step();
        // Origem em y=3 ⇒ o chão está a 2.5 m. Com alcance 2.0 não deve achar,
        // por mais longo que seja o vetor de direção.
        assert!(
            w.cast_ray([0.0, 3.0], [0.0, -10.0], 2.0, None, 0).is_none(),
            "um dir longo nao pode esticar o alcance"
        );
        assert!(w.cast_ray([0.0, 3.0], [0.0, -10.0], 3.0, None, 0).is_some());
    }

    /// **O CONTRATO da versão, medido em vez de suposto:** o BVH descreve o
    /// mundo que o último `step` deixou.
    ///
    /// Este gate não afirma que o comportamento é bom — afirma **qual é**, para
    /// que o consumidor saiba onde pode castar. Se um dia o rapier passar a
    /// indexar no insert, este teste falha e a nota do módulo é reescrita; é
    /// exatamente para isso que ele existe.
    #[test]
    fn the_bvh_answers_for_the_world_the_last_step_left() {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 10.0, 0.5);
        w.step();
        assert!(
            w.cast_ray([0.0, 3.0], [0.0, -1.0], 10.0, None, 0).is_some(),
            "depois de um step o chao esta' indexado"
        );

        // Uma parede NOVA, ainda sem step, no caminho de um raio horizontal.
        //
        // ⚠️ Os dois lados são MEDIDOS, não supostos: antes do step o cast
        // devolve `None`, depois devolve `Some(1.5)` (a face da parede está em
        // x = 1.5). É esse par que torna a nota do módulo verificável em vez de
        // folclore — e se um dia o rapier passar a indexar no `insert`, é aqui
        // que se descobre, em vez de num personagem que cai pelo chão.
        let mut w2 = PhysicsWorld::new();
        w2.add_static_cuboid(2.0, 0.0, 0.5, 10.0);
        assert!(
            w2.cast_ray([0.0, 0.0], [1.0, 0.0], 10.0, None, 0).is_none(),
            "ANTES do primeiro step o corpo novo nao esta' no BVH"
        );
        w2.step();
        let after = w2
            .cast_ray([0.0, 0.0], [1.0, 0.0], 10.0, None, 0)
            .expect("depois do step a parede tem de estar la'");
        assert!(
            (after.distance - 1.5).abs() < 1.0e-3,
            "a face da parede esta' em x = 1.5: {}",
            after.distance
        );
    }
}
