//! **O objecto passa a OLHAR** (suplente #21) — um raio persistente, autorado.
//!
//! # ⭐⭐⭐ O buraco tem TRÊS nomes, e nenhum se compõe do que existia
//!
//! A sonda [`mede_o_que_a_composicao_ja_da_ao_raio`] correu antes da primeira linha (`CLAUDE.md`
//! §5.0), com a melhor composição que a casa tem — um colisor `is_sensor` fino deitado ao longo da
//! linha, que é exactamente como o **#3 `SensorZone`** fechou **por composição** dois dias antes:
//!
//! | a pergunta | a barra sensor | o [`cast_ray`](ph2d_physics::PhysicsWorld::cast_ray) |
//! |---|---|---|
//! | **ORDEM** | `triggered_sensors()` devolve `["Barra"]` — **um** elemento, com as duas paredes lá dentro | acerta na de `x = 2`, a mais perto |
//! | **MÉTRICA** | **`0` contactos de pé** (um sensor atravessa) ⇒ `point`/`normal`/`impulse` vazios | `d = 1,7500` · ponto `(1,75 ; 0)` · normal `(−1 ; 0)` |
//! | **DIRECÇÃO** | **não distingue**: uma FORMA é simétrica, e a barra apanha a parede de `x = −3` | a de trás está no alcance e **não** volta |
//!
//! ⇒ *o motor está pago e é rico; o que faltava era um componente autorável que lhe chegue.*
//!
//! # ⚠️⚠️ A nota da porta do motor, RECONFERIDA de novo (§0.0, dono 15/11, versão C)
//!
//! O [`cast_ray`](ph2d_physics::PhysicsWorld::cast_ray) põe `EXCLUDE_SENSORS` e `solid = true`
//! **dentro dele** — *«os cinco consumidores querem matéria»*. A primeira leitura deste componente
//! herdou a nota. **A versão C revoga-a para este consumidor só:** o artista pode querer um raio que
//! acerta gatilhos (`hit_triggers`) ou que nasce dentro da própria forma sem se achar
//! (`starts_inside`), e a porta dele é outra — [`cast_ray_query`](ph2d_physics::PhysicsWorld::cast_ray_query),
//! com um [`RayQuery`](ph2d_physics::RayQuery). Os cinco sensores do personagem não mudam (gate
//! `the_caster_can_exclude_itself` e os do `player_probes`). Os dois defaults (`false`) dão a resposta de
//! antes para os gatilhos; `starts_inside = false` é a de Godot (`hit_from_inside = false`).
//!
//! # ⚠️ É CONFIG, nunca estado vivo
//!
//! A lei do módulo: o que o raio VÊ muda a cada tique e **não** vive aqui — ele vive no mapa da
//! ponte, que é onde o canal de triggers (W7) já guarda o dele. Um campo «o que estou a ver» dentro
//! de um componente registado faria o `canonicalize` do undo ver **cada quadro como um passo**.

use bevy_ecs::prelude::Component;
use ph2d_core::Vec2;
use ph2d_ecs::SimComponent;
use serde::{Deserialize, Serialize};

/// **Um raio persistente: o objecto «Ray Sensor»** (GEF 07, dono 15/11 — objecto filho, como a forma do ADR-0199).
///
/// A origem é a POSIÇÃO do objecto e o rumo é `angle` rodado pela pose dele (e a do pai): dois raios
/// são dois objectos. ⚠️ `angle` é LOCAL e em GRAUS (a unidade autorada, `FieldKind::Angle`): anima-se
/// sem passar por `0,707`, e o motor faz o vetor (`dir_local`).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RaySensor {
    /// Para onde aponta, em graus locais: `0` = direita, `90` = cima, `-90` = baixo.
    pub angle: f32,
    /// Até onde ele enxerga, em metros.
    pub reach: f32,
    /// A camada de colisão pela qual ele pergunta — a matriz do mundo decide o que ele vê.
    pub layer: u8,
    /// Espelha com o lado para onde o personagem dono olha (`PlayerState::facing`): a olhar para a
    /// esquerda, o ângulo e o desvio do raio trocam de lado. Opção do artista (dono 15/11), desligada.
    pub turns_with_facing: bool,
    /// O corpo dono sai do raio («Ignore my body»). Falso = ele vê o próprio dono.
    pub ignore_body: bool,
    /// Os outros corpos da FAMÍLIA do dono também saem («Ignore my other parts») — ver
    /// [`topo_da_familia`](crate::topo_da_familia): a espada e o escudo com corpo próprio do herói.
    pub ignore_parts: bool,
    /// «Ignore up to»: o `StableId` do objecto do TOPO da família (um antepassado do raio); `0` =
    /// automático (o personagem, senão o corpo mais alto). Dono 15/11 (C8): campo + conta-gotas.
    pub ignore_up_to: u64,
    /// O que ele aceita ver, pelas tags do corpo acertado («Only see»).
    pub only_see: SeeFilter,
    /// A tag do `only_see` (`TagId.0`; `0` = por escolher).
    pub see_tag: u64,
    /// «Walls block»: o raio pára na primeira matéria e só VÊ se ela passa no `only_see` (uma parede
    /// sem a tag TAPA o alvo — linha de visão). Falso = ele atravessa quem não passa.
    pub walls_block: bool,
    /// «Hit triggers»: os sensores/gatilhos também respondem.
    pub hit_triggers: bool,
    /// «Starts inside»: a forma que contém a origem responde a `0`. Falso = ignorada.
    pub starts_inside: bool,
    /// «See all hits»: guarda todos os acertos ao longo do alcance, por distância.
    pub all_hits: bool,
    /// «Enabled» (RS-6): desligado, o raio não olha — sem marca, sem acertos; o que ele via sai
    /// com On Exit no tique em que se desliga (a regra do Unreal: desligar a meio fecha a aresta).
    pub enabled: bool,
    /// «Thickness» (RS-1), em metros: `> 0` varre um CÍRCULO deste diâmetro ao longo do raio (o
    /// ShapeCast2D do Godot); `0` = a linha.
    pub thickness: f32,
    /// «Cone» (RS-2), em graus: a abertura TOTAL do leque à volta do rumo; `0` = a linha. Vê cada
    /// objecto dentro do leque e não tapado, um acerto por objecto (como «See all hits»).
    pub cone: f32,
    /// «Aim at» (RS-3): o `StableId` do objecto para onde o raio aponta, na pose VIVA dele no mesmo
    /// tique; `0` = o `angle`. Com alvo, o `angle` e o «Turns with facing» não decidem.
    pub aim_at: u64,
    /// «Also ignore» (RS-12): os `StableId` dos objectos que ele não vê (com a família de cada um),
    /// escolhidos pelo conta-gotas — o «Actors to Ignore» do Unreal.
    pub also_ignore: Vec<u64>,
    /// «Shape» (dono 15/11): QUAL forma o raio lança — só o número da forma activa conta
    /// ([`RaySensor::largura_activa`], [`RaySensor::cone_activo`]); os outros guardam o valor.
    pub shape: RayShape,
}

/// **A forma do raio** («Shape», dono 15/11: *«thickness e cone são conflitantes»*): um dropdown de três.
///
/// Variantes SEM dados (como [`SeeFilter`]): o catálogo e o script leem-nas pelo nome serde.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RayShape {
    /// A linha fina: nem `thickness` nem `cone` contam.
    #[default]
    Line,
    /// Um CÍRCULO de diâmetro `thickness` varrido ao longo do raio.
    Thick,
    /// O leque de abertura `cone`, com linhas de vista FINAS (o cone não usa `thickness`).
    Cone,
}

/// **O filtro de tags do raio** («Only see»): o MODO; a tag é o campo [`RaySensor::see_tag`].
///
/// ⚠️ **Variantes SEM dados, e a tag num campo ao lado:** o catálogo (`FieldKind::Enum` + opções) e o
/// script leem uma variante pelo NOME serde de unidade; um `OnlyTag(u64)` seria um valor composto que
/// nenhum dos dois sabe ler. O id é o mesmo do [`SignalTagFilter`](crate::SignalTagFilter)
/// (`ph2d_tags::TagId.0`); `0` = por escolher, e passa todos (como lá).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeeFilter {
    /// Vê tudo.
    #[default]
    Everything,
    /// Só quem pertence à tag (com a subárvore). Uma tag apagada não deixa passar ninguém.
    OnlyTag,
    /// Todos menos quem pertence à tag.
    ExceptTag,
}

impl SeeFilter {
    /// `other` passa, com a tag `id`? `tags` = as tags dele (ou `None`).
    #[must_use]
    pub fn passes(
        self,
        id: u64,
        tree: &ph2d_tags::TagTree,
        tags: Option<&ph2d_ecs::tags::Tags>,
    ) -> bool {
        let so = match self {
            Self::Everything => return true,
            Self::OnlyTag => true,
            Self::ExceptTag => false,
        };
        if id == 0 {
            return true;
        }
        let q = ph2d_tags::TagId(id);
        let pertence =
            tree.get(q).is_some() && tags.is_some_and(|t| ph2d_ecs::tags::belongs(t, tree, q));
        pertence == so
    }
}

impl RaySensor {
    /// O rumo unitário local que `angle` diz.
    #[must_use]
    pub fn dir_local(&self) -> Vec2 {
        let r = self.angle.to_radians();
        Vec2::new(libm::cosf(r), libm::sinf(r))
    }

    /// O diâmetro do círculo varrido que CONTA: `thickness` só com [`RayShape::Thick`]; senão `0`.
    #[must_use]
    pub fn largura_activa(&self) -> f32 {
        match self.shape {
            RayShape::Thick if self.thickness.is_finite() => self.thickness.max(0.0),
            _ => 0.0,
        }
    }

    /// A abertura do leque que CONTA, em graus: `cone` só com [`RayShape::Cone`]; senão `0`.
    #[must_use]
    pub fn cone_activo(&self) -> f32 {
        match self.shape {
            RayShape::Cone if self.cone.is_finite() => self.cone.clamp(0.0, 360.0),
            _ => 0.0,
        }
    }
}

impl Default for RaySensor {
    /// O default OLHA PARA BAIXO: o sensor de chão é o primeiro raio de um jogo 2D.
    fn default() -> Self {
        Self {
            angle: -90.0,
            reach: 1.0,
            layer: 0,
            turns_with_facing: false,
            ignore_body: true,
            ignore_parts: true,
            ignore_up_to: 0,
            only_see: SeeFilter::Everything,
            see_tag: 0,
            walls_block: true,
            hit_triggers: false,
            starts_inside: false,
            all_hits: false,
            enabled: true,
            thickness: 0.0,
            cone: 0.0,
            aim_at: 0,
            also_ignore: Vec::new(),
            shape: RayShape::Line,
        }
    }
}

impl SimComponent for RaySensor {}

/// **O que um raio publica quando PASSA A ver e quando DEIXA de ver.**
///
/// ⚠️ **Dois nomes, porque são duas perguntas** — a regra que o [`SignalOnHit`](super::SignalOnHit)
/// já escreve: emitir o mesmo nome nos dois extremos torna o sinal ambíguo, e quem escuta casa numa
/// string. ⚠️ **Vazio = calado**, palavra por palavra a regra dele.
///
/// ⛔ **Componente à parte e não campos do [`RaySensor`]:** ele é serializado POSICIONALMENTE pelo
/// postcard, e um objecto que só quer *«há chão?»* não paga dois nomes que nunca usa. É o mesmo
/// trade que o `AreaEffector`/`AreaDrag` pagou.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaySignals {
    /// Publicado no tique em que o raio passa a ver alguma coisa.
    pub on_enter: String,
    /// Publicado no tique em que ele deixa de ver.
    pub on_exit: String,
}

impl RaySignals {
    /// O nome de entrada, ou `None` se estiver em branco.
    #[must_use]
    pub fn enter(&self) -> Option<&str> {
        let t = self.on_enter.trim();
        (!t.is_empty()).then_some(t)
    }

    /// O nome de saída, ou `None` se estiver em branco.
    #[must_use]
    pub fn exit(&self) -> Option<&str> {
        let t = self.on_exit.trim();
        (!t.is_empty()).then_some(t)
    }
}

impl SimComponent for RaySignals {}

/// **O que um raio viu neste tique** — a leitura que a ponte publica.
///
/// ⛔ Ela **não** é um componente: o que nasce numa corrida não é documento (a lei do #11 e do #20),
/// e a memória de um tique cabe no mapa da ponte — o precedente é o canal de triggers.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayHit {
    /// Quem ele acertou.
    pub body: ph2d_ecs::Entity,
    /// A FORMA acertada (a «Collision Shape» filha, ou o próprio corpo quando a forma mora nele) —
    /// as tags dela são o «material» do acerto (RS-4, dono 15/11: o Surface Type do Unreal).
    pub shape: ph2d_ecs::Entity,
    /// A que distância da origem, em metros.
    pub distance: f32,
    /// Onde, em mundo.
    pub point: [f32; 2],
    /// A normal da superfície ali — é ela que faz a REFLEXÃO do Construct ser exprimível.
    pub normal: [f32; 2],
}
